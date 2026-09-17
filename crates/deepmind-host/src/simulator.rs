//! The other end of the conversation, as a port you can choose.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use deepmind_midi::ids::{DeviceId, ProtocolVersion, Slot};
use deepmind_midi::param::{ParamId, Shape};
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::sim::{Library, Synth};
use deepmind_midi::syx::File;
use deepmind_midi::transport::Port;

use crate::error::{Closed, PortError};

/// What the simulated synthesizer has in its memory.
///
/// A real unit always has something in every slot; this one holds what it was
/// given and answers nothing for the slots it was not, which is more useful to
/// test against than an invented program. [`Pack::empty`] is a unit whose memory
/// is not part of the test, and [`Pack::from_syx`] makes a preset pack the
/// contents of one, so "read a bank" is testable without hardware.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pack {
    programs: Vec<(Slot, Program)>,
}

impl Pack {
    /// A unit with nothing stored.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            programs: Vec::new(),
        }
    }

    /// Reads the stored programs out of a `.syx` file's bytes.
    ///
    /// Frames the library cannot read are skipped rather than refused: a file is
    /// untrusted input, and one bad dump in a pack is one missing slot. An edit
    /// buffer dump names no slot and is not stored anywhere, so it is skipped
    /// too. A slot that appears twice keeps the last one, as loading a pack into
    /// a unit would.
    #[must_use]
    pub fn from_syx(bytes: &[u8]) -> Self {
        let mut pack = Self::empty();
        for entry in File::new(bytes).programs().flatten() {
            if let Some(slot) = entry.slot {
                pack.insert(slot, entry.program);
            }
        }
        pack
    }

    /// Puts a program in a slot, replacing whatever was there.
    pub fn insert(&mut self, slot: Slot, program: Program) {
        if let Some(held) = self
            .programs
            .iter_mut()
            .find(|(stored, _)| *stored == slot)
            .map(|(_, program)| program)
        {
            *held = program;
        } else {
            self.programs.push((slot, program));
        }
    }

    /// Returns how many slots hold something.
    #[must_use]
    pub fn len(&self) -> usize {
        self.programs.len()
    }

    /// Returns whether nothing is stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }
}

impl Library for Pack {
    fn program(&self, slot: Slot) -> Option<Program> {
        self.programs
            .iter()
            .find(|(stored, _)| *stored == slot)
            .map(|(_, program)| program.clone())
    }

    fn contains(&self, slot: Slot) -> bool {
        self.programs.iter().any(|(stored, _)| *stored == slot)
    }
}

/// The simulated synthesizer's front panel.
///
/// The instrument is the other editor. A real unit has a player standing in
/// front of it, turning knobs the host never asked about, and this is that
/// player: [`turn`](Panel::turn) moves a parameter at the unit, which then sends
/// the NRPN a panel move sends. It is the one thing a real port offers that a
/// simulated one would not, and without it "the view follows the instrument" is
/// only testable with a cable.
///
/// Handed out by [`open_simulator`](crate::open_simulator), and useful for as
/// long as that port is open.
#[derive(Debug, Clone)]
pub struct Panel {
    turns: Sender<(ParamId, u16)>,
}

impl Panel {
    /// Moves a parameter at the unit, as a hand on the panel would.
    ///
    /// Takes effect on the next turn of the device loop. A value the parameter
    /// does not accept moves nothing, because a knob cannot be turned past its
    /// end either.
    ///
    /// # Errors
    ///
    /// Returns [`Closed`] when the port this panel belongs to has been closed.
    pub fn turn(&self, parameter: ParamId, value: u16) -> Result<(), Closed> {
        self.turns.send((parameter, value)).map_err(|_| Closed)
    }
}

/// The library's simulated synthesizer, wired up as a [`Port`].
///
/// One reply per [`receive`](Port::receive), which is what makes a bank read out
/// of it look like a bank read: 128 dumps arrive one at a time, through the same
/// loop, past the same progress reporting, rather than all at once.
///
/// There is no clock in it. It answers as soon as it is read, so this port is as
/// fast as the loop around it, and the twelve seconds a real transfer takes are
/// not simulated.
#[derive(Debug)]
pub struct SimPort {
    synth: Synth<Pack>,
    partial: Vec<u8>,
    turns: Receiver<(ParamId, u16)>,
}

impl SimPort {
    /// Builds a unit holding `memory`, and the panel that stands in front of it.
    #[must_use]
    pub fn new(memory: Pack) -> (Self, Panel) {
        let mut sound = blank();
        if let Ok(name) = ProgramName::new("Simulator") {
            sound.set_name(name);
        }
        Self::holding(memory, sound)
    }

    /// The same, for a unit that powers up with `sound` in its edit buffer.
    ///
    /// What [`new`](Self::new) does with the blank sound, for a caller that has
    /// one of its own. A sound reached through [`Panel`] is the same sound by a
    /// longer road, two hundred and forty-two knobs turned one at a time and
    /// each one reported back, and some things want the unit to be *holding*
    /// something rather than to have been played: a picture of the
    /// editor with a sound in it, or a test that starts from one.
    #[must_use]
    pub fn holding(memory: Pack, sound: Program) -> (Self, Panel) {
        let (turns, pending) = mpsc::channel();
        let port = Self {
            synth: Synth::with_library(DeviceId::Unit(0), sound, memory),
            partial: Vec::new(),
            turns: pending,
        };
        (port, Panel { turns })
    }

    /// Turns whatever the panel has been asked to turn.
    ///
    /// What that produces is queued for sending like any other reply, so a knob
    /// moved at the unit reaches the host through the same path a dump does.
    fn take_panel_moves(&mut self) {
        loop {
            match self.turns.try_recv() {
                Ok((parameter, value)) => {
                    // A value the parameter does not accept moves nothing.
                    let _ = self.synth.turn(parameter, value);
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return,
            }
        }
    }
}

/// The sound the simulated unit powers up holding.
///
/// `Program::new` puts every parameter at the floor of its own range, which for
/// the 197 that count up from one is the right answer and for the other 45 is
/// not: a value read about a centre has its zero in the middle, so a program
/// built out of minimums is one with every modulation depth at full negative,
/// every detune at the bottom of its swing and every pan hard left. That is not
/// a blank sound, it is a particular and rather strange one, and eight
/// modulation depths reading `-128` in a window that has just opened look like
/// the window's own default rather than the instrument's answer.
///
/// So the bipolar ones are put where their own zero is. The library publishes
/// that centre for each of them, so nothing here is invented: this is the
/// library's own arithmetic applied to the library's own list.
fn blank() -> Program {
    let mut sound = Program::new(ProtocolVersion::V7);
    for parameter in ParamId::ALL.iter().copied() {
        if let Shape::Bipolar { centre } = parameter.shape() {
            sound.set_clamped(parameter, u8::try_from(centre).unwrap_or_default());
        }
    }
    sound
}

/// Handed to the simulator's drain to stop it after one reply.
struct Enough;

impl Port for SimPort {
    type Error = PortError;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.synth.feed(bytes);
        // Nothing reads what the unit made of it, and a full queue would start
        // dropping observations rather than replies.
        while self.synth.poll_heard().is_some() {}
        Ok(())
    }

    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
        self.take_panel_moves();
        if self.partial.is_empty() {
            let mut taken: Option<Vec<u8>> = None;
            // A reply the closure refuses stays queued, so this takes exactly
            // one and leaves the rest for the next call.
            let _: Result<usize, Enough> = self.synth.drain_tx(|bytes| {
                if taken.is_some() {
                    return Err(Enough);
                }
                taken = Some(bytes.to_vec());
                Ok(())
            });
            match taken {
                Some(reply) => self.partial = reply,
                None => return Ok(0),
            }
        }
        let taken = self.partial.len().min(into.len());
        let (Some(source), Some(target)) = (self.partial.get(..taken), into.get_mut(..taken))
        else {
            return Ok(0);
        };
        target.copy_from_slice(source);
        self.partial.drain(..taken);
        Ok(taken)
    }
}

#[cfg(test)]
#[expect(clippy::panic, reason = "a failed expectation is the test failure")]
mod tests {
    use deepmind_midi::param::{ParamId, Shape};

    use super::blank;

    #[test]
    fn the_simulated_unit_powers_up_with_its_bipolar_values_at_zero() {
        // A modulation depth of `-128` is full negative modulation, and eight
        // of them is what a program built out of minimums says the instrument
        // is doing. The 197 that count up from one are untouched: their floor
        // is where they read from.
        let depth = ParamId::Mod1Depth;
        let Shape::Bipolar { centre } = depth.shape() else {
            panic!("a depth is read about its centre");
        };
        let sound = blank();

        assert_eq!(u16::from(sound.get(depth)), centre);
        assert_eq!(
            u16::from(sound.get(ParamId::Lfo1Rate)),
            ParamId::Lfo1Rate.min(),
            "a value that counts up from one is left where it counts from"
        );
    }
}

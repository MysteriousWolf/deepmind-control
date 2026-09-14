//! What the device thread reports back.

use core::fmt;

use deepmind_midi::device::Event as DeviceEvent;
use deepmind_midi::ids::Bank;
use deepmind_midi::param::ParamId;
use deepmind_midi::sysex::inquiry::Identity;
use deepmind_midi::wire::Channel;

use crate::command::CommandKind;
use crate::error::PortError;

/// Something that happened on the port, or to the thread that owns it.
///
/// The device thread holds the truth and the interface holds a copy it builds
/// out of these. There is no lock between them: two copies of the state, one
/// direction of flow, and the copy the user sees is the one their own events
/// built.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "the library's event carries a program inline rather than pointing at               a copy a bank transfer overwrites 128 times, and this passes it               through unchanged. Boxing it would buy an allocation per dump with               a memcpy of 242 bytes"
)]
pub enum Event {
    /// The port is open and the thread is running.
    ///
    /// Always first. A device inquiry goes out immediately after it, so the next
    /// thing is [`Identified`](Event::Identified) or [`Silent`](Event::Silent).
    Opened,
    /// A synthesizer answered the inquiry, and this is who it is.
    ///
    /// Everything the interface needs to address the unit settles here: the
    /// device ID the dumps carry, the MIDI channel the edits go out on, and the
    /// firmware the value tables are read for. Firmware 1.1 renumbered three of
    /// those tables, so until this arrives the interface is reading the
    /// library's default firmware and should say so.
    Identified {
        /// What the inquiry reported.
        identity: Identity,
        /// The channel NRPN now goes out on.
        ///
        /// The manual's global settings table says the device ID is also the
        /// global MIDI channel. Whether the instrument agrees is question 5 of
        /// the plan, and is answered with a cable.
        channel: Channel,
    },
    /// Nothing answered the inquiry within the timeout.
    ///
    /// Which is how a port is judged: a port that answers has a `DeepMind` on
    /// it, and a port that does not is something else with a similar name. The
    /// thread stays open and stays answerable, because the synthesizer may
    /// simply not be switched on yet, and another
    /// [`Identify`](crate::Command::Identify) is all that asking again costs.
    Silent,
    /// Something the synthesizer said, as the library read it.
    ///
    /// Passed through rather than re-described. Nothing in this crate re-decodes
    /// a frame or re-tabulates a parameter.
    ///
    /// Every library event but one arrives this way. A parameter report is
    /// published as [`Parameter`](Event::Parameter) instead, because on its own
    /// the library's event does not say how much of the value arrived.
    Device(DeviceEvent),
    /// One parameter moved at the synthesizer, by NRPN or by its controller.
    ///
    /// The instrument is the other editor: a hand on the front panel raises this
    /// and the view follows it. Last writer wins, and nothing here echoes the
    /// value back, because the synthesizer already has it and the echo is a
    /// loop.
    Parameter {
        /// The parameter the synthesizer named.
        parameter: ParamId,
        /// The value it now holds, as the library read the message.
        value: u16,
        /// Whether the sound this thread tracks is still confirmed with it.
        ///
        /// An NRPN carries the whole value and a control change carries seven
        /// bits of it, and the library says which arrived by what it does to the
        /// program it tracks: an exact report leaves a confirmed program
        /// confirmed, and a coarse one leaves it assumed. That answer is only
        /// readable at the moment the report lands, so it is read here and
        /// published with it.
        ///
        /// `false` is also what a program nobody has read yet says, and what a
        /// program the host has edited since the last dump says. Both are the
        /// same claim honestly made: nothing has confirmed this sound, so
        /// nothing in it is confirmed.
        confirmed: bool,
    },
    /// How far a bank read has got.
    ///
    /// Raised as each dump lands, so twelve seconds of transfer is something to
    /// draw rather than something to wait through.
    Progress {
        /// Bank being read.
        bank: Bank,
        /// Dumps that have arrived.
        received: u16,
        /// Dumps the run asked for.
        expected: u16,
    },
    /// A bank read ended, and how.
    Finished {
        /// Bank that was being read.
        bank: Bank,
        /// Dumps that arrived.
        received: u16,
        /// Dumps the run asked for.
        expected: u16,
        /// Whether it ran out, was called off, or went quiet.
        outcome: Outcome,
    },
    /// A command could not be carried out, and what the library said about it.
    ///
    /// The usual one is editing before anything has been read: an edit is a
    /// difference, and there is nothing to take a difference against until a
    /// dump has arrived.
    Refused {
        /// What was asked for.
        command: CommandKind,
        /// Why it was not done.
        reason: deepmind_midi::Error,
    },
    /// The port failed, and the thread is stopping.
    ///
    /// The last event but [`Closed`](Event::Closed).
    Failed(PortError),
    /// The thread has stopped and the port is closed.
    ///
    /// Always last.
    Closed,
}

/// How a bank read ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Every dump the run asked for arrived.
    Complete,
    /// The interface called it off.
    ///
    /// Dumps already on their way still arrive, as
    /// [`Device`](Event::Device) events outside any transfer. Nothing in the
    /// protocol stops a unit that has started sending.
    Cancelled,
    /// The gap between dumps outlasted the timeout.
    ///
    /// The library times a run by the gap between its dumps rather than by the
    /// length of the whole transfer, so this is a unit that stopped partway and
    /// not a transfer that took too long.
    TimedOut,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => f.write_str("complete"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::TimedOut => f.write_str("timed out"),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opened => f.write_str("port open"),
            Self::Identified { identity, channel } => write!(
                f,
                "firmware {} on {}, channel {}",
                identity.firmware,
                identity.device,
                channel.number()
            ),
            Self::Silent => f.write_str("nothing answered the inquiry"),
            Self::Device(event) => write!(f, "{event}"),
            Self::Parameter {
                parameter,
                value,
                confirmed,
            } => {
                let claim = if *confirmed { "confirmed" } else { "assumed" };
                write!(f, "{parameter} = {value}, {claim}")
            }
            Self::Progress {
                bank,
                received,
                expected,
            } => write!(f, "bank {bank}: {received} of {expected}"),
            Self::Finished {
                bank,
                received,
                expected,
                outcome,
            } => write!(f, "bank {bank}: {received} of {expected}, {outcome}"),
            Self::Refused { command, reason } => write!(f, "{command} refused: {reason}"),
            Self::Failed(error) => write!(f, "{error}"),
            Self::Closed => f.write_str("port closed"),
        }
    }
}

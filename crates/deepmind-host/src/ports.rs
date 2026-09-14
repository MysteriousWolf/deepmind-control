//! What the machine offers, and the port opened on top of it.

use std::sync::mpsc::{self, Receiver, TryRecvError};

use deepmind_midi::transport::Port;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use crate::error::{Direction, OpenError, PortError};

/// What this application calls itself when it asks the backend for a port.
const CLIENT: &str = "deepmind-control";

/// The name of the simulated synthesizer in the port list.
const SIMULATOR: &str = "Simulated DeepMind";

/// A port that could be opened, as the port picker shows it.
///
/// A name and nothing else. The backend's own port handles go stale when a
/// device is unplugged and are looked up again by name when a port is opened, so
/// this is what survives sitting in a list while somebody decides.
///
/// Two ports with the same name are indistinguishable here, and the first is the
/// one that opens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortRef {
    name: String,
    simulated: bool,
}

impl PortRef {
    /// The simulated synthesizer, which is a port like any other.
    ///
    /// It is in the list so that the interface can be built without a
    /// synthesizer on the desk, and so that a bug report that arrives without
    /// hardware still has something to reproduce against. What it cannot do is
    /// find out that the manual is wrong: both ends are generated from the same
    /// specification.
    #[must_use]
    pub fn simulator() -> Self {
        Self {
            name: SIMULATOR.to_owned(),
            simulated: true,
        }
    }

    /// Returns the name the backend gave the port.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether this is the simulated synthesizer rather than a cable.
    #[must_use]
    pub const fn is_simulator(&self) -> bool {
        self.simulated
    }
}

impl core::fmt::Display for PortRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.name)
    }
}

/// Lists the ports a conversation could be held over.
///
/// Only the duplex ones: a synthesizer that is told things and never answers is
/// a synthesizer this application cannot edit, so a name that exists in one
/// direction only is left out. The simulator is first when this crate was built
/// with it.
///
/// Nothing here filters by name. Which of these has a `DeepMind` on it is
/// settled by opening one and seeing whether the device inquiry is answered,
/// which is the only thing that reports firmware anyway.
///
/// # Errors
///
/// Returns [`OpenError::Backend`] when the MIDI backend will not start, which is
/// a machine with no MIDI support rather than a machine with no synthesizer.
pub fn ports() -> Result<Vec<PortRef>, OpenError> {
    let mut found = Vec::new();
    if cfg!(feature = "sim") {
        found.push(PortRef::simulator());
    }

    let input = MidiInput::new(CLIENT).map_err(OpenError::Backend)?;
    let output = MidiOutput::new(CLIENT).map_err(OpenError::Backend)?;

    let sendable: Vec<String> = output
        .ports()
        .iter()
        .filter_map(|port| output.port_name(port).ok())
        .collect();

    for port in input.ports() {
        let Ok(name) = input.port_name(&port) else {
            continue;
        };
        if sendable.contains(&name) && !found.iter().any(|listed| listed.name == name) {
            found.push(PortRef {
                name,
                simulated: false,
            });
        }
    }
    Ok(found)
}

/// A MIDI port, opened both ways.
///
/// The input connection is never touched after it is made and is held for as
/// long as the port is open: dropping it is what closes the callback the backend
/// delivers on.
pub struct MidiPort {
    #[expect(dead_code, reason = "dropping this closes the input")]
    input: MidiInputConnection<()>,
    output: MidiOutputConnection,
    inbound: Receiver<Vec<u8>>,
    partial: Vec<u8>,
}

// The backend's connection types carry no `Debug`, and what is worth printing
// about a port is which one it is rather than what midir is holding for it.
impl core::fmt::Debug for MidiPort {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MidiPort")
            .field("waiting", &self.partial.len())
            .finish_non_exhaustive()
    }
}

impl MidiPort {
    /// Opens both halves of the port named `name`.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Backend`] when the backend will not start,
    /// [`OpenError::Missing`] when nothing carries that name any more, and
    /// [`OpenError::Connect`] when the backend will not hand the port over.
    pub fn open(name: &str) -> Result<Self, OpenError> {
        let mut input = MidiInput::new(CLIENT).map_err(OpenError::Backend)?;
        // The whole conversation is SysEx, and midir drops it by default.
        input.ignore(Ignore::None);
        let output = MidiOutput::new(CLIENT).map_err(OpenError::Backend)?;

        let source = input
            .ports()
            .into_iter()
            .find(|port| input.port_name(port).is_ok_and(|found| found == name))
            .ok_or_else(|| OpenError::Missing {
                name: name.to_owned(),
                direction: Direction::Input,
            })?;
        let sink = output
            .ports()
            .into_iter()
            .find(|port| output.port_name(port).is_ok_and(|found| found == name))
            .ok_or_else(|| OpenError::Missing {
                name: name.to_owned(),
                direction: Direction::Output,
            })?;

        // The backend delivers on a thread of its own, so what it delivers is
        // queued and read back by `receive` on the thread that owns the device.
        let (sender, inbound) = mpsc::channel();
        let input = input
            .connect(
                &source,
                CLIENT,
                move |_timestamp, message, ()| {
                    let _ = sender.send(message.to_vec());
                },
                (),
            )
            .map_err(|error| OpenError::Connect {
                name: name.to_owned(),
                direction: Direction::Input,
                reason: error.to_string(),
            })?;
        let output = output
            .connect(&sink, CLIENT)
            .map_err(|error| OpenError::Connect {
                name: name.to_owned(),
                direction: Direction::Output,
                reason: error.to_string(),
            })?;

        Ok(Self {
            input,
            output,
            inbound,
            partial: Vec::new(),
        })
    }
}

impl Port for MidiPort {
    type Error = PortError;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.output
            .send(bytes)
            .map_err(|error| PortError::Send(error.to_string()))
    }

    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
        if self.partial.is_empty() {
            match self.inbound.try_recv() {
                Ok(message) => self.partial = message,
                // A quiet port is the normal case and is not an ending.
                Err(TryRecvError::Empty) => return Ok(0),
                Err(TryRecvError::Disconnected) => return Err(PortError::Gone),
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

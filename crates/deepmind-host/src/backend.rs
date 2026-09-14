//! The port the device thread owns, whichever kind it is.

use deepmind_midi::transport::Port;

use crate::error::PortError;
use crate::ports::MidiPort;
#[cfg(feature = "sim")]
use crate::simulator::SimPort;

/// What the device loop reads and writes.
///
/// An enum rather than a boxed trait object: there are two kinds of port in this
/// application and there is never going to be a third, so the dispatch is a
/// match and the thread owns its port outright.
#[derive(Debug)]
pub(crate) enum Backend {
    /// A MIDI port on the machine.
    Midi(MidiPort),
    /// The library's simulated synthesizer.
    #[cfg(feature = "sim")]
    Simulated(Box<SimPort>),
    /// No port at all.
    ///
    /// Held for the instant a [`Device`](deepmind_midi::Device) is rebuilt
    /// around a real port, and never pumped. A transport owns its port, so
    /// taking the port out of one to put it into another needs something to
    /// leave behind.
    Spent,
}

impl Port for Backend {
    type Error = PortError;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        match self {
            Self::Midi(port) => port.send(bytes),
            #[cfg(feature = "sim")]
            Self::Simulated(port) => port.send(bytes),
            Self::Spent => Err(PortError::Gone),
        }
    }

    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
        match self {
            Self::Midi(port) => port.receive(into),
            #[cfg(feature = "sim")]
            Self::Simulated(port) => port.receive(into),
            Self::Spent => Err(PortError::Gone),
        }
    }
}

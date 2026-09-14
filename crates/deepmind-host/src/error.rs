//! What opening a port, and running one, can fail with.

use core::fmt;

/// Which half of a MIDI connection something is about.
///
/// A conversation with a synthesizer needs both, and a machine can offer one
/// without the other, so a failure says which one it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// From the synthesizer to this application.
    Input,
    /// From this application to the synthesizer.
    Output,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => f.write_str("input"),
            Self::Output => f.write_str("output"),
        }
    }
}

/// Why a port could not be opened.
#[derive(Debug)]
#[non_exhaustive]
pub enum OpenError {
    /// The MIDI backend would not start at all.
    Backend(midir::InitError),
    /// Nothing on the system carries that name in that direction any more.
    ///
    /// Ports are named when they are listed and looked up again by name when
    /// they are opened, so a synthesizer unplugged in between arrives here.
    Missing {
        /// Name the port was listed under.
        name: String,
        /// The half that was not found.
        direction: Direction,
    },
    /// The backend found the port and would not connect to it.
    Connect {
        /// Name of the port.
        name: String,
        /// The half that refused.
        direction: Direction,
        /// What the backend said about it.
        reason: String,
    },
    /// A simulated synthesizer was asked for in a build compiled without one.
    NoSimulator,
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(error) => write!(f, "the MIDI backend would not start: {error}"),
            Self::Missing { name, direction } => {
                write!(f, "no {direction} port named {name:?}")
            }
            Self::Connect {
                name,
                direction,
                reason,
            } => write!(f, "could not open the {direction} of {name:?}: {reason}"),
            Self::NoSimulator => {
                f.write_str("this build has no simulator: the `sim` feature is off")
            }
        }
    }
}

impl core::error::Error for OpenError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Backend(error) => Some(error),
            _ => None,
        }
    }
}

/// What an open port failed with while it was being used.
///
/// The backend's own error types carry the connection they failed on and are
/// not worth moving across a channel, so what crosses it is what the backend
/// said. A failure here ends the device thread: a port that has gone away does
/// not come back without being opened again.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PortError {
    /// A message could not be written to the port.
    Send(String),
    /// The port stopped delivering: unplugged, or closed underneath us.
    Gone,
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(reason) => write!(f, "the port would not take a message: {reason}"),
            Self::Gone => f.write_str("the port has gone away"),
        }
    }
}

impl core::error::Error for PortError {}

/// The device thread is no longer there to take commands.
///
/// Either it was told to stop, or the port under it failed and it reported that
/// as its last event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Closed;

impl fmt::Display for Closed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the device thread has stopped")
    }
}

impl core::error::Error for Closed {}

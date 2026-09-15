//! Port ownership and the device loop for Behringer `DeepMind` synthesizers.
//!
//! [`deepmind-midi`](deepmind_midi) owns the protocol and refuses to own
//! anything else: it never opens a port, never spawns a thread and never blocks.
//! This crate is the other half. It opens the port, drives the clock, owns the
//! thread, and publishes what happened.
//!
//! ```text
//! DeepMind <--MIDI--> port <--bytes--> deepmind-midi <--events--> the interface
//!                      ^
//!               midir, or the simulator
//! ```
//!
//! # The shape of it
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use deepmind_host::{Command, Event, open, ports};
//!
//! let ports = ports()?;                              // what midir sees, and the simulator
//! let link = open(ports.first().ok_or("no ports")?)?; // one thread, two channels
//! link.send(Command::ReadEditBuffer)?;
//!
//! while let Some(event) = link.wait(Duration::from_secs(1)) {
//!     if let Event::Device(event) = event {
//!         println!("{event}");
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Events cross the boundary, state does not
//!
//! The device thread holds the truth. Whoever holds the [`Link`] holds a copy
//! built out of [`Event`]s, and there is no lock between them. Two copies of the
//! state, one direction of flow, and the copy the user sees is the one their own
//! events built.
//!
//! The thread uses [`Transport::pump`](deepmind_midi::Transport::pump) and none
//! of the blocking helpers. A bank is one request and 128 answers, about twelve
//! seconds at MIDI speed, and a thread that waits for them cannot report
//! progress or be cancelled. Requests go out, answers come back as events, and
//! the twelve seconds are a progress bar.
//!
//! # What it does on its own
//!
//! Two things, and no more:
//!
//! - It sends a device inquiry as soon as the port is open, because that is the
//!   only message that reports firmware, and firmware 1.1 renumbered three value
//!   tables. The reply also carries the unit's device ID, which the manual says
//!   is its global MIDI channel, so one message settles who to address and on
//!   which channel. A port that does not answer is not a `DeepMind`, and that is
//!   how a port list is filtered rather than by matching names.
//! - It coalesces parameter edits, because the plugin and the desktop build have
//!   the same wire and the views should not know that a wire has a speed.
//!
//! Everything else is a [`Command`]. It never polls the edit buffer on a timer:
//! a poll that costs 290 bytes of a 3 kB/s link is a poll that competes with the
//! edits it is checking.
//!
//! # Nothing here writes a program into the synthesizer
//!
//! The manual describes no message that stores a program into a unit. The
//! library refuses to invent one, and so does this. The application edits the
//! edit buffer live, and the librarian's output is a file.

mod backend;
mod command;
mod driver;
mod edits;
mod error;
mod event;
mod link;
mod ports;
#[cfg(feature = "sim")]
mod simulator;

pub use command::{Command, CommandKind};
pub use error::{Closed, Direction, OpenError, PortError};
pub use event::{Event, Outcome};
pub use link::Link;
pub use ports::{PortRef, ports};
#[cfg(feature = "sim")]
pub use simulator::{Pack, Panel};

use backend::Backend;
use ports::MidiPort;

/// How patient the loop is, and how fast it sends.
///
/// The defaults are arithmetic and not measurement. What a `DeepMind` actually
/// does with edits sent back to back is question 2 of the plan, and is answered
/// with a cable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Options {
    /// How long a request waits for its answer, in milliseconds.
    ///
    /// A bank run is timed by the gap between its dumps rather than by the
    /// length of the whole transfer, so this does not have to cover twelve
    /// seconds.
    pub timeout_ms: u64,
    /// How long the loop sleeps when there is nothing to do, in milliseconds.
    ///
    /// This bounds how late an edit can be, so it is small. Zero spins.
    pub poll_ms: u64,
    /// How long between one coalesced parameter edit and the next, in
    /// milliseconds.
    ///
    /// One NRPN is twelve bytes, about four milliseconds of a MIDI cable. This
    /// is a little slower than that, because a synthesizer that drops edits is
    /// worse than a knob that lags.
    pub edit_interval_ms: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            timeout_ms: 2_000,
            poll_ms: 1,
            edit_interval_ms: 5,
        }
    }
}

/// Opens a port and starts the device thread on it.
///
/// # Errors
///
/// Returns [`OpenError`] when the backend will not start, when nothing carries
/// that name any more, or when the port will not open. A port that opens and
/// holds no synthesizer is not an error here: it is an [`Event::Silent`], which
/// arrives once the inquiry has gone unanswered for as long as the timeout
/// allows.
pub fn open(port: &PortRef) -> Result<Link, OpenError> {
    open_with(port, Options::default())
}

/// Opens a port with something other than the default [`Options`].
///
/// # Errors
///
/// The same as [`open`].
pub fn open_with(port: &PortRef, options: Options) -> Result<Link, OpenError> {
    let backend = if port.is_simulator() {
        #[cfg(feature = "sim")]
        {
            // Nobody is standing at this one: a port picked out of the list is
            // the unit, and the panel that comes with it belongs to whoever
            // built the unit. `open_simulator` is where that is somebody.
            let (port, _panel) = simulator::SimPort::new(Pack::empty());
            Backend::Simulated(Box::new(port))
        }
        #[cfg(not(feature = "sim"))]
        {
            return Err(OpenError::NoSimulator);
        }
    } else {
        Backend::Midi(MidiPort::open(port.name())?)
    };
    Ok(Link::spawn(port.name().to_owned(), backend, options))
}

/// Starts the device thread on a simulated synthesizer holding `memory`, and
/// hands back the unit's front panel with it.
///
/// The simulator is a port you can choose, and this is the one thing a real port
/// cannot offer: a unit whose memory is a `.syx` pack, so that reading a bank is
/// testable against a preset pack with nothing plugged in, and a
/// [`Panel`] to turn its knobs from, so that "the view follows the instrument"
/// is testable without a hand on the hardware.
///
/// What it cannot do is find out that the manual is wrong. Both ends are
/// generated from the same specification.
#[cfg(feature = "sim")]
#[must_use]
pub fn open_simulator(memory: Pack, options: Options) -> (Link, Panel) {
    let (port, panel) = simulator::SimPort::new(memory);
    let link = Link::spawn(
        PortRef::simulator().name().to_owned(),
        Backend::Simulated(Box::new(port)),
        options,
    );
    (link, panel)
}

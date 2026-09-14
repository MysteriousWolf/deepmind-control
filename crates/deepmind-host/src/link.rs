//! The two channels, and the thread between them.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::Options;
use crate::backend::Backend;
use crate::command::Command;
use crate::driver::Driver;
use crate::error::Closed;
use crate::event::Event;

/// An open port, and the thread that owns it.
///
/// Commands go in, events come out, and the `Device` stays on the other side.
/// Sharing an `Arc<Mutex<Device>>` instead would be shorter to write and would
/// put a lock in a view, which is a dropped frame waiting for a bank transfer to
/// let go.
///
/// Dropping this stops the thread and closes the port.
///
/// ```no_run
/// use deepmind_host::{Command, ports, open};
///
/// let ports = ports()?;
/// let link = open(ports.first().ok_or("no MIDI ports")?)?;
/// link.send(Command::ReadEditBuffer)?;
///
/// // In a window, this is the update function; the events are the state.
/// for event in link.drain() {
///     println!("{event}");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct Link {
    name: String,
    commands: Sender<Command>,
    events: Receiver<Event>,
    thread: Option<JoinHandle<()>>,
}

impl Link {
    /// Starts the device thread on a port that is already open.
    pub(crate) fn spawn(name: String, backend: Backend, options: Options) -> Self {
        let (commands, inbox) = mpsc::channel();
        let (outbox, events) = mpsc::channel();
        let thread = thread::Builder::new()
            .name(format!("deepmind-host {name}"))
            .spawn(move || Driver::new(backend, options, inbox, outbox).run())
            .ok();
        Self {
            name,
            commands,
            events,
            thread,
        }
    }

    /// Returns the name of the port this is open on.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Asks the device thread for something.
    ///
    /// Returns as soon as the command is queued. What happened to it arrives as
    /// an [`Event`].
    ///
    /// # Errors
    ///
    /// Returns [`Closed`] when the thread has stopped, which is a port that
    /// failed or a [`Stop`](Command::Stop) that has already been sent.
    pub fn send(&self, command: Command) -> Result<(), Closed> {
        self.commands.send(command).map_err(|_| Closed)
    }

    /// Takes everything that has happened since the last call.
    ///
    /// Never blocks: this is what a view's update function calls, and a view
    /// that waits is a view that is not being drawn.
    pub fn drain(&self) -> impl Iterator<Item = Event> + '_ {
        self.events.try_iter()
    }

    /// Waits for one event, for as long as `limit`.
    ///
    /// The exception to the rule above, for a caller with nothing else to do:
    /// a test, or a command-line tool. A window drains instead.
    #[must_use]
    pub fn wait(&self, limit: Duration) -> Option<Event> {
        self.events.recv_timeout(limit).ok()
    }

    /// Returns whether the device thread is still running.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }

    /// Stops the thread and closes the port, and waits for it to happen.
    ///
    /// The same as dropping it, written down where a reader can see it.
    pub fn close(self) {
        drop(self);
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

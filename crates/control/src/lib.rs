//! The `DeepMind` editor and librarian, on the desktop.
//!
//! The window, the port picker and the file dialogs. Everything it draws comes
//! from [`control_ui`] and everything it says to a synthesizer goes through
//! [`deepmind_host`]; what is here is the wiring between them.
//!
//! # The shape of a turn
//!
//! The device thread holds the truth and this holds a copy built out of the
//! events it publishes. A message arrives, the state changes, whatever the
//! thread has said since the last message is folded in, and the window is drawn
//! from what that left behind. There is no lock anywhere in it: a view that
//! waits on a bank transfer is a dropped frame, and a bank transfer is twelve
//! seconds long.
//!
//! ```no_run
//! use control::{App, Message};
//! use deepmind_host::PortRef;
//!
//! let mut app = App::new();
//! app.update(Message::Choose(PortRef::simulator()));
//! app.update(Message::Connect);
//! app.update(Message::Tick);          // whatever the thread has said by now
//! ```
//!
//! The library's simulated synthesizer is in the port list like any other
//! choice, so all of that works with nothing plugged in.

mod app;
mod window;

pub use app::{App, Message};
pub use window::run;

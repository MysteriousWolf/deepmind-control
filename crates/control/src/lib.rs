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
//!
//! # Two surfaces, one window
//!
//! [`View::Editor`] is the sound in front of somebody and [`View::Library`] is
//! the sounds they keep. The second is the librarian: a [`Shelf`] filled by
//! opening a `.syx` file or by reading a bank off the instrument, browsed in
//! slot order, and loaded into the edit buffer one program at a time. It is
//! written here rather than in `control-ui` because bank and librarian
//! operations are desktop only; the editing surface above it is the crate both
//! builds share.

mod app;
mod files;
mod librarian;
mod shelf;
mod window;

pub use app::{App, Message, View};
pub use shelf::{Held, Shelf, Source, Transfer, patch_to_syx};
pub use window::run;

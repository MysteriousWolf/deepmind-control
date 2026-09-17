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
//! # Three surfaces, one window
//!
//! [`View::Panel`] is the instrument's own front: the handful of controls it
//! puts a fader under, and on every section the press it calls `EDIT`. It is
//! where the window opens and where the other two are reached from.
//!
//! [`View::Editor`] is one of the fourteen sections, which is what an `EDIT`
//! opens, and [`View::Library`] is the sounds somebody keeps. The second is the librarian: a [`Shelf`] filled by
//! opening a `.syx` file or by reading a bank off the instrument, browsed in
//! slot order, and loaded into the edit buffer one program at a time. It is
//! written here rather than in `control-ui` because bank and librarian
//! operations are desktop only; the editing surface above it is the crate both
//! builds share.

mod app;
mod files;
mod librarian;
#[cfg(feature = "previews")]
pub mod preview;
mod shelf;
mod window;

pub use app::{App, Message, View};
pub use shelf::{Held, Shelf, Source, Transfer, patch_to_syx};
pub use window::run;

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
//! # Two surfaces and a sheet
//!
//! [`View::Panel`] is the instrument's own front: the handful of controls it
//! puts a fader under, and on every section the press it calls `EDIT`. It is
//! where the window opens.
//!
//! [`View::Library`] is the other one, and it is the librarian: a [`Shelf`]
//! filled by opening a `.syx` file or by reading a bank off the instrument,
//! browsed in slot order, and loaded into the edit buffer one program at a
//! time. It is written here rather than in `control-ui` because bank and
//! librarian operations are desktop only; the editing surface is the crate both
//! builds share.
//!
//! # And a shelf somebody else filled
//!
//! [`catalogue`] is the third thing, and it is groundwork rather than a
//! surface: the index a repository of shared presets would carry, read the way
//! this application reads everything somebody else wrote — leniently, saying
//! what it could not read, and trusting nothing in it with a path. Nothing
//! draws it yet. See [the plan](../../../docs/presets.md) for what it is for and
//! what has to be decided before any of it fetches anything.
//!
//! There is no third surface. One of the fourteen sections is what an `EDIT` press
//! opens, and it opens *over* the panel rather than instead of it: a sheet with
//! the instrument still underneath, which is what the hardware does when a
//! section button is pressed and its own front stays where it is. Which one is
//! open is [`App::editing`], and escape, the mark on the sheet and a press on
//! the window around it all put it away.

mod app;
pub mod catalogue;
mod files;
mod librarian;
#[cfg(feature = "previews")]
pub mod preview;
mod shelf;
mod window;

pub use app::{App, Message, View};
pub use shelf::{Held, ORDERS, Order, Shelf, Source, Transfer, patch_to_syx};
pub use window::run;

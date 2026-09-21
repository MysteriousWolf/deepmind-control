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
//! [`catalogue`] is the librarian's second shelf, and it is a surface now:
//! [`deepmind-patches`](https://github.com/MysteriousWolf/deepmind-patches) is
//! a real repository, one `.syx` and one `.toml` per sound, published as a
//! release on every merge. It is reached either by `git clone` and a folder
//! dialog or by fetching the newest release on a thread, and pressing anything
//! on it puts that sound on the same [`Shelf`] every other route already fills.
//! Which of the two shelves is showing is [`Browsing`].
//!
//! Almost none of the format is read here. The library publishes the
//! `deepmind-patches` crate, which is the same split `deepmind-midi` is on the
//! protocol's side, and this application links it rather than keeping a reader
//! of its own. See [the plan](../../../docs/presets.md) for the reasoning and
//! [the specification](../../../docs/patches-repo.md) for what the repository
//! is.
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
mod publish;
mod sharing;
mod shelf;
mod sounds;
mod window;

pub use app::{App, Browsing, Message, Publishing, View};
pub use shelf::{Held, ORDERS, Order, Shelf, Source, Transfer, patch_to_syx};
pub use window::run;

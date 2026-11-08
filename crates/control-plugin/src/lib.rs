//! The `DeepMind` editor as a plugin.
//!
//! Stage 6 of [the plan]. This is a CLAP: the editor's front panel and the
//! fourteen sections behind it, in a window the host owns, with the instrument's
//! own edits as its MIDI output. Route that output to the synthesizer and the
//! window is the front of it.
//!
//! What it does:
//!
//! - draws the same views as the desktop build, because they are the same
//!   [`control_ui`];
//! - turns every edit into the NRPN the instrument takes, one at a time and no
//!   faster than a cable could carry it;
//! - keeps the sound it is showing in the session, and says it again when the
//!   session comes back.
//!
//! What it does not do yet: read the instrument, which needs the host to route
//! the synthesizer's output *into* the plugin, and select programs on their own
//! in the plan's *simple* mode. Bank and librarian work is desktop only and
//! will stay that way: a preset pack is thirty-five kilobytes of `SysEx`, and a
//! plugin event buffer is sized for a few notes.
//!
//! The AU and the VST3 are wrapped around this CLAP and are not built here.
//!
//! [the plan]: https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/plan.md

mod editor;
mod export;
mod outbox;
mod plugin;
mod wire;

pub use outbox::Outbox;
pub use plugin::{ControlParams, DeepMindControl};

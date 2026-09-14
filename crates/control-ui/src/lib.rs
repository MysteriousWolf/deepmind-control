//! Views and widgets for the `DeepMind` editor.
//!
//! This crate is the pixels and nothing else. It carries no runtime, opens no
//! window and owns no port: it depends on `iced_core` and `iced_widget` rather
//! than on `iced`, so the same views compile into the desktop application and
//! into the plugin's baseview window.
//!
//! # What it is given
//!
//! A [`Patch`]: the interface's copy of the sound, built out of the events the
//! host crate publishes and carrying, for every parameter, how it came to hold
//! the value it holds. A value the host put there and a value the synthesizer
//! reported are not the same claim, and this is the layer where that finally
//! means something to a person: [`Confidence`] is what the views draw
//! differently.
//!
//! # What it produces
//!
//! [`Message`], which says a parameter should move and nothing else. What that
//! costs on a wire, when it is sent, and what it is sent behind is the host
//! crate's problem: the views do not know that a wire has a speed.
//!
//! ```
//! use control_ui::{Confidence, Patch};
//! use deepmind_midi::param::ParamId;
//! use deepmind_midi::program::Program;
//! use deepmind_midi::ids::ProtocolVersion;
//!
//! let mut patch = Patch::new();
//! patch.confirm(Program::new(ProtocolVersion::V7));      // a dump arrived
//! assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Confirmed);
//!
//! patch.edit(ParamId::VcfFrequency, 200);                // somebody dragged it
//! assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Assumed);
//! assert_eq!(patch.claim(ParamId::VcfResonance), Confidence::Confirmed);
//! ```
//!
//! # One palette, in one file
//!
//! [`deepmind`] is the theme both builds are drawn in, and `style.rs` is the
//! only file in this repository that writes down a colour. It is dark because
//! the instrument is: a `DeepMind` is a dark panel between wooden end cheeks
//! with metal fader caps on it, and the palette is read off the mark that draws
//! exactly those materials.
//!
//! # The theme is shared and the renderer is not
//!
//! Views are generic over the renderer, because the desktop build and the plugin
//! do not have to agree on one, and concrete in the theme, because
//! [`iced_core::Theme`] is what both of them have.

mod confidence;
mod panel;
mod patch;
mod style;

pub use confidence::Confidence;
pub use panel::{Message, group, legend};
pub use patch::Patch;
pub use style::{deepmind, tint};

/// A piece of interface, produced by the views in this crate.
///
/// The message type is [`Message`] throughout: a view here asks for a parameter
/// to move, and the application it is embedded in maps that into whatever its
/// own message type is.
pub type Element<'a, Renderer> = iced_core::Element<'a, Message, iced_core::Theme, Renderer>;

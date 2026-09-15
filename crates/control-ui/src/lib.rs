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
//! [`Message`], which says a parameter should move, or which section should be
//! on the screen, and nothing else. What an edit costs on a wire, when it is
//! sent, and what it is sent behind is the host crate's problem: the views do
//! not know that a wire has a speed.
//!
//! # The panel, and then the fourteen behind it
//!
//! [`panel`] is the front of the instrument: two rows of section plates with a
//! screen between them, the twenty-odd controls Behringer put a fader under,
//! and on every plate the press the hardware calls `EDIT`. It is where a window
//! opens, because it is where a player looks first, and everything else is
//! behind one of those presses.
//!
//! # Fourteen panels, one at a time
//!
//! [`group`] draws one section as the instrument lays it out, and
//! [`section_bar`] draws the bar that chooses which. Every parameter the
//! synthesizer has is reachable through the two of them, drawn from the
//! library's own table: what a control looks like is what the library says the
//! parameter is, so a panel nobody has laid out by hand is complete before it
//! is beautiful.
//!
//! Nothing here re-tabulates the library, the order of the panels included:
//! [`sections`] is read off the parameter table's own offsets rather than
//! written down, so a group a later library adds arrives in its right place
//! with nothing here to edit.
//!
//! Laying a panel out by hand changes the arrangement and never what a control
//! is. The program's name is the first one and the only exception to even that:
//! [`name_characters`] are the seventeen parameters the instrument stores a name
//! in, and they are drawn as the one display it shows them on, because
//! seventeen faders are not a name.
//!
//! The effects are the panel that needs the library most. `FX 1 Param 3` is
//! `Size` on a Room Reverb and `Depth` on a Phaser, and which of the two it is
//! depends on a byte somewhere else in the same program: the panel reads the
//! algorithm each engine is running and names its twelve slots from the table
//! `deepmind-midi` 26.2 publishes, for the firmware that answered the inquiry.
//! The control is still the parameter table's own, because what the display
//! reads there and what the byte is are two different claims and only one of
//! them is published.
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
//! The same file holds the three faces anything is set in — [`printed`],
//! [`wordmark`] and [`reading`] — and what the parts of a window that are not
//! parameters are drawn as: [`ground`] is the panel gradient the whole window
//! stands on, and [`chrome`], [`selector`], [`shortlist`] and [`bay`] are a
//! button, a picker, the list it opens and a panel of words in the instrument's
//! own materials. A port picker is not a parameter and is still on the
//! instrument.
//!
//! # The theme is shared and the renderer is not
//!
//! Views are generic over the renderer, because the desktop build and the plugin
//! do not have to agree on one, and concrete in the theme, because
//! [`iced_core::Theme`] is what both of them have.

mod confidence;
mod effect;
mod envelope;
mod fader;
mod home;
mod matrix;
mod name;
mod panel;
mod patch;
mod section;
mod sequencer;
mod style;

pub use confidence::Confidence;
pub use fader::{Axis, Fader, fader};
pub use home::{panel, panelled};
pub use name::characters as name_characters;
pub use panel::{Message, group, legend};
pub use patch::Patch;
pub use section::{first_section, section_bar, sections};
pub use style::{
    Materials, bay, chrome, deepmind, ground, materials, printed, reading, selector, shortlist,
    tint, wordmark,
};

/// A piece of interface, produced by the views in this crate.
///
/// The message type is [`Message`] throughout: a view here asks for a parameter
/// to move, and the application it is embedded in maps that into whatever its
/// own message type is.
pub type Element<'a, Renderer> = iced_core::Element<'a, Message, iced_core::Theme, Renderer>;

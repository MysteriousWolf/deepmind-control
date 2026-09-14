//! Views and widgets for the `DeepMind` editor.
//!
//! This crate is the pixels and nothing else. It carries no runtime, opens no
//! window and owns no port: it depends on `iced_core` and `iced_widget` rather
//! than on `iced`, so the same views compile into the desktop application and
//! into the plugin's baseview window.
//!
//! It takes [`Known<Program>`](deepmind_midi::Known) and not `Program`, because
//! a value the host sent and a value the synthesizer reported are not the same
//! claim, and this is the layer where that distinction finally means something
//! to a person.
//!
//! The views themselves are stage 2 and stage 3 of [the plan]; what is here now
//! is the crate they arrive into.
//!
//! [the plan]: https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/plan.md

// The view layer is pinned to what the baseview adapter supports, and the pin
// is only a pin while something in the workspace resolves it.
use iced_core as _;
use iced_widget as _;

use deepmind_midi::Known;
use deepmind_midi::program::Program;

/// How a value is drawn depends on whether anything confirmed it.
///
/// The one piece of the view layer that exists before the views do, because it
/// is the rule the rest of them are written against: a knob showing where you
/// put it and a knob showing where the instrument says it is are different
/// drawings of the same number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Confidence {
    /// Nothing has been read back yet. Nothing is known about the sound.
    Unknown,
    /// The host put the value there and the synthesizer has not said so.
    Assumed,
    /// The synthesizer reported it.
    Confirmed,
}

impl Confidence {
    /// Reads the confidence of whatever the device is tracking.
    #[must_use]
    pub const fn of(program: &Known<Program>) -> Self {
        match program {
            Known::Unknown => Self::Unknown,
            Known::Assumed { .. } => Self::Assumed,
            Known::Confirmed { .. } => Self::Confirmed,
        }
    }

    /// Returns whether a synthesizer reported this value.
    #[must_use]
    pub const fn is_confirmed(self) -> bool {
        matches!(self, Self::Confirmed)
    }
}

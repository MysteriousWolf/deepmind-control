//! What backs a value on the screen.

use deepmind_midi::Known;
use deepmind_midi::program::Program;

/// How a value is drawn depends on whether anything confirmed it.
///
/// The rule the rest of the views are written against: a knob showing where you
/// put it and a knob showing where the instrument says it is are different
/// drawings of the same number, and an editor that draws them the same way is an
/// editor that lies about the sound in the room.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Confidence {
    /// Nothing has been read back yet. Nothing is known about the sound.
    #[default]
    Unknown,
    /// The host put the value there and the synthesizer has not said so.
    Assumed,
    /// The synthesizer reported it.
    Confirmed,
}

impl Confidence {
    /// Reads the confidence of whatever the device is tracking.
    ///
    /// The bridge from the library's own claim, for a caller that holds one.
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

    /// Returns whether there is a value at all.
    #[must_use]
    pub const fn is_known(self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// Returns the word for it, for a status line or a legend.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unread",
            Self::Assumed => "assumed",
            Self::Confirmed => "confirmed",
        }
    }
}

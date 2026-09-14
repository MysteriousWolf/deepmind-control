//! How assumed and confirmed are drawn.
//!
//! One colour per claim, taken from the theme rather than written down here, so
//! that a light window and a dark one are both legible and neither is this
//! crate's business.

use iced_core::{Color, Theme};

use crate::Confidence;

/// The colour a value is written in, given what backs it.
///
/// Green for what the synthesizer reported, amber for what this window put
/// there and nothing has confirmed, and the theme's own muted text for a sound
/// nobody has read. The amber is the point: it is the colour of a claim, and it
/// goes away when a dump comes back.
#[must_use]
pub fn tint(theme: &Theme, confidence: Confidence) -> Color {
    let palette = theme.extended_palette();
    match confidence {
        Confidence::Unknown => palette.background.strong.color,
        Confidence::Assumed => palette.warning.base.color,
        Confidence::Confirmed => palette.success.base.color,
    }
}

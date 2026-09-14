//! The one place a colour is chosen.
//!
//! Every colour in either build comes from here: the palette below, and the
//! [`tint`] a claim is drawn in. Nothing else in this repository writes down a
//! colour, so restyling the editor is editing this file and nothing near it.

use std::sync::LazyLock;

use iced_core::theme::Palette;
use iced_core::{Color, Theme, color};

use crate::Confidence;

/// The instrument's own colours, read off the mark this repository draws with.
///
/// A `DeepMind` is a dark panel between wooden end cheeks, with metal fader caps
/// on it, and `docs/logo.svg` is drawn from exactly those four materials. The
/// window is the same object seen from the front, so it is dark by default and
/// its accents are the panel's: steel for what can be touched, and the copper of
/// the wood for what is only claimed.
///
/// | | |
/// | --- | --- |
/// | `#15181e` | the panel, which is the ground everything sits on |
/// | `#e6e9ef` | the legend printed on it |
/// | `#8e939f` | the metal of a fader cap: anything that can be touched |
/// | `#c26436` | the copper edge of the wooden cheeks: a claim, not a fact |
///
/// Green and red are not on the instrument and are chosen to sit with it: a
/// confirmed value has to be told apart from an assumed one at a glance, and
/// two greys would not do it.
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x00c2_6436` is not one"
)]
const PALETTE: Palette = Palette {
    background: color!(0x15181e),
    text: color!(0xe6e9ef),
    primary: color!(0x8e939f),
    success: color!(0x5fa87f),
    warning: color!(0xc26436),
    danger: color!(0xb4483c),
};

/// What a drawn control is made of.
///
/// The panel comes from whatever theme is passed, so a window someone has
/// themed differently still holds together. The rest are the instrument's own
/// materials, read off the same mark as the palette: a recess is cut into the
/// panel and lit along its lower wall, and metal is what a hand touches.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Materials {
    /// The panel a control is cut into.
    pub panel: Color,
    /// The body of a recess.
    pub recess: Color,
    /// The near edge of a recess.
    pub recess_edge: Color,
    /// The lower wall of a recess, which is the side the light reaches.
    pub lit: Color,
    /// A scale printed on the panel.
    pub scale: Color,
    /// The face of a metal part.
    pub metal: Color,
    /// Its lit edge.
    pub metal_high: Color,
    /// Its shaded edge.
    pub metal_low: Color,
}

/// Returns the materials a drawn control is made of, in `theme`.
#[must_use]
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x0011_141a` is not one"
)]
pub fn materials(theme: &Theme) -> Materials {
    let palette = theme.extended_palette();
    Materials {
        panel: palette.background.base.color,
        recess: color!(0x11141a),
        recess_edge: color!(0x242932),
        lit: color!(0x7d838f),
        scale: palette.background.strong.color,
        metal: color!(0xc9cdd6),
        metal_high: color!(0xf4f5f8),
        metal_low: color!(0x8e939f),
    }
}

/// The theme both builds are drawn in.
///
/// Built once. Cloning it is cloning a pointer, which is what the window does
/// every time it draws.
#[must_use]
pub fn deepmind() -> Theme {
    static THEME: LazyLock<Theme> = LazyLock::new(|| Theme::custom("DeepMind".to_owned(), PALETTE));
    THEME.clone()
}

/// The colour a value is written in, given what backs it.
///
/// Copper for what this window put there and nothing has confirmed, green for
/// what the synthesizer reported, and the panel's own muted grey for a sound
/// nobody has read. The copper is the point: it is the colour of a claim, and it
/// goes away when a dump comes back.
///
/// It is never the only thing doing the work. A drawn control carries its claim
/// in the fill of its moving part — filled for a fact, an outline for a claim,
/// nothing at all for a value nobody has read — and this colour agrees with
/// that fill rather than replacing it, so the difference survives greyscale, a
/// projector, and the readers who would not see the pair.
///
/// Taken from whatever theme is passed rather than from this crate's own palette,
/// so a window someone has themed differently is still legible.
#[must_use]
pub fn tint(theme: &Theme, confidence: Confidence) -> Color {
    let palette = theme.extended_palette();
    match confidence {
        Confidence::Unknown => palette.background.strong.color,
        Confidence::Assumed => palette.warning.base.color,
        Confidence::Confirmed => palette.success.base.color,
    }
}

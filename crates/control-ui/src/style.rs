//! The one place a colour is chosen, and a face to set it in.
//!
//! Every colour in either build comes from here: the palette below, and the
//! [`tint`] a claim is drawn in. Nothing else in this repository writes down a
//! colour, so restyling the editor is editing this file and nothing near it.
//!
//! The same rule now covers type and the chrome. [`printed`], [`wordmark`] and
//! [`reading`] are the three faces anything in either build is set in, and
//! [`ground`], [`chrome`], [`selector`], [`shortlist`] and [`bay`] are what the parts of the
//! window that are not parameters are drawn as: a port picker cut into the
//! panel like a fader's track, and a button with a metal rim rather than a
//! toolkit's own grey. The panel is for parameters and the toolkit is correct
//! everywhere else, but everywhere else is still on the instrument.

use std::sync::LazyLock;

use iced_core::gradient::Linear;
use iced_core::theme::Palette;
use iced_core::{Background, Border, Color, Font, Gradient, Radians, Shadow, Theme, border, color};
use iced_widget::overlay::menu;
use iced_widget::{button, container, pick_list};

use crate::Confidence;

/// The family the instrument's legends are set in.
///
/// The mark's own. `docs/banner.svg` outlines the project's name from Liberation
/// Sans Bold so that it renders identically wherever the file is shown, and a
/// window cannot outline anything: it asks for a family and takes what the
/// machine has. So it asks for the mark's family where that is the usual name
/// for it, and for the metrically compatible face every other platform ships
/// under a different name — Liberation Sans is an Arial clone, which is what
/// makes the substitution a spelling rather than a second typeface.
///
/// A machine with neither falls back to the system's own sans, which is the
/// state a window is in today rather than a failure: the letters are readable
/// and the proportions are somebody else's. Carrying the file in the binary is
/// what would make both builds identical on every machine, and that is a
/// decision about a licence and a megabyte rather than a line here.
const FAMILY: &str = if cfg!(target_os = "linux") {
    "Liberation Sans"
} else {
    "Arial"
};

/// The face a title, a label and the rest of the application are set in.
///
/// What is printed on the panel, as against what the display
/// [reads](reading()): the two faces the instrument has, and the whole of the
/// typography either build chooses.
#[must_use]
pub const fn printed() -> Font {
    Font::with_name(FAMILY)
}

/// The same face, bold, which is what the mark sets the name in.
#[must_use]
pub const fn wordmark() -> Font {
    Font {
        weight: iced_core::font::Weight::Bold,
        ..printed()
    }
}

/// The face anything the synthesizer's own display would show is set in.
///
/// Mono, so that a column of readings is a column and a value running through
/// its range does not jiggle the digits beside it. Asked for by kind rather
/// than by name: what a machine has is what a machine has, and the only thing
/// this needs of it is that every glyph is the same width.
#[must_use]
pub const fn reading() -> Font {
    Font::MONOSPACE
}

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

/// The one saturated colour, and it only ever means that something other than
/// a hand can move a control: the modulation matrix is pointed at it.
///
/// Not on the instrument, and chosen to sit with it. Everything else on the
/// panel is the panel, the metal or the ink, so a mark in this is the only
/// thing on a rack of forty that is not one of those.
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x00ff_b95c` is not one"
)]
pub const LAMP: Color = color!(0xffb95c);

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
    /// The face plate a group's slots sit on, which is raised off the panel.
    pub plate: Color,
    /// The body of a recess.
    pub recess: Color,
    /// The face of a display, which is the deepest recess on the panel.
    ///
    /// Read against [`recess`](Materials::recess) rather than instead of it: a
    /// display is a pane of glass over a lit case, so it is the recess with the
    /// light that reaches the top of it, and a dot lit on it is the only thing
    /// in this window that is neither panel, metal nor ink.
    pub glass: Color,
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
        plate: color!(0x1b1f26),
        recess: color!(0x080a0e),
        glass: color!(0x101620),
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

/// The ground the whole window is drawn on.
///
/// The panel of the mark, which is not a flat colour: `#282c36` at the top
/// falling to `#15181e`, the gradient `logo.svg` and `banner.svg` both fill the
/// case with. A window that used the darker end alone would be the bottom of
/// the panel stretched over the whole of it, which is the one part of the
/// instrument that is not lit.
#[must_use]
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x0028_2c36` is not one"
)]
pub fn ground(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            // Down the panel: the light end at the top, where the light is.
            Linear::new(Radians(std::f32::consts::PI))
                .add_stop(0.0, color!(0x282c36))
                .add_stop(1.0, color!(0x15181e)),
        ))),
        ..container::Style::default()
    }
}

/// What a button that is not a parameter is drawn as.
///
/// The panel with a metal rim, rather than the toolkit's filled slab: a row of
/// those is the brightest thing on a dark panel, and the brightest thing on
/// this panel has to be a fader cap. Pressing lights the rim the way a section
/// button on the instrument lights, and a button with nothing to do keeps the
/// panel and loses the metal, so that it reads as unavailable rather than as
/// a different kind of button.
#[must_use]
pub fn chrome(theme: &Theme, status: button::Status) -> button::Style {
    let material = materials(theme);
    let (rim, ink) = match status {
        button::Status::Active => (material.recess_edge, material.metal),
        button::Status::Hovered => (material.metal_low, material.metal_high),
        button::Status::Pressed => (material.lit, material.metal_high),
        button::Status::Disabled => (material.recess_edge, material.metal_low),
    };
    button::Style {
        background: Some(Background::Color(match status {
            button::Status::Pressed => material.plate,
            _ => material.panel,
        })),
        text_color: ink,
        border: border::rounded(3).width(1.0).color(rim),
        ..button::Style::default()
    }
}

/// What something chosen from a list is drawn as.
///
/// A recess cut into the panel, which is what every other place a value is
/// chosen in this window already is: the track of a fader, the field a name is
/// typed in, the list a modulation destination is picked from. A port is not a
/// parameter, and it is still chosen on the same panel.
#[must_use]
pub fn selector(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let material = materials(theme);
    let rim = match status {
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => material.metal_low,
        pick_list::Status::Active => material.recess_edge,
    };
    pick_list::Style {
        text_color: material.metal_high,
        placeholder_color: material.metal_low,
        handle_color: material.metal,
        background: Background::Color(material.recess),
        border: Border {
            color: rim,
            width: 1.0,
            radius: 2.into(),
        },
    }
}

/// What the list a [`selector`] opens is drawn as.
///
/// The same recess, continued: a picker that is a recess closed and a pale
/// slab open is two controls wearing one name. What is under the pointer is
/// lit the way a pressed section button is lit, which is the window's one idea
/// of "this one".
#[must_use]
pub fn shortlist(theme: &Theme) -> menu::Style {
    let material = materials(theme);
    menu::Style {
        background: Background::Color(material.recess),
        border: Border {
            color: material.recess_edge,
            width: 1.0,
            radius: 2.into(),
        },
        text_color: material.metal,
        selected_text_color: material.metal_high,
        selected_background: Background::Color(material.plate),
        shadow: Shadow::default(),
    }
}

/// What a panel that holds words rather than controls is drawn as.
///
/// The face plate a rack of slots sits on, so that what the window is saying
/// about the instrument sits on the instrument rather than in a dialog.
#[must_use]
pub fn bay(theme: &Theme) -> container::Style {
    let material = materials(theme);
    container::Style {
        background: Some(Background::Color(material.plate)),
        border: Border {
            color: material.recess_edge,
            width: 1.0,
            radius: 3.into(),
        },
        ..container::Style::default()
    }
}

//! The one place a colour is chosen, and a face to set it in.
//!
//! Every colour in either build comes from here: the palette below, and the
//! [`tint`] a claim is drawn in. Nothing else in this repository writes down a
//! colour, so restyling the editor is editing this file and nothing near it.
//!
//! The same rule now covers type and the chrome. [`printed`] and [`reading`]
//! are the two faces anything in either build is set in, and
//! [`ground`], [`chrome`], [`selector`], [`shortlist`], [`bay`] and [`unlit`] are what the
//! parts of the window that are not parameters are drawn as: a port picker cut
//! into the panel like a fader's track, a button with a metal rim rather than a
//! toolkit's own grey, and the panel with the light off it while a sheet is
//! lying over it. The panel is for parameters and the toolkit is correct
//! everywhere else, but everywhere else is still on the instrument.

use std::sync::LazyLock;

use iced_core::gradient::Linear;
use iced_core::theme::{Base, Palette};
use iced_core::{Background, Border, Color, Font, Gradient, Radians, Shadow, Theme, border, color};
use iced_widget::overlay::menu;
use iced_widget::{button, container, pick_list, text_input};

use deepmind_midi::effect::Colour;

use crate::Confidence;

/// The family the instrument's legends are set in.
///
/// The mark's own. `docs/banner.svg` outlines the project's name from Liberation
/// Sans Bold so that it renders identically wherever the file is shown, and a
/// window cannot outline anything: it asks for a family and takes what the
/// machine has. So it asks for the mark's family where that is the usual name
/// for it, and for the metrically compatible face every other platform ships
/// under a different name. Liberation Sans is an Arial clone, which is what
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

/// The colours of the light coming through the instrument's buttons.
///
/// A `DeepMind`'s front panel is dark, and two things on it are not: the
/// [banners](BANNER) its section names are printed on, and the lamps behind its
/// buttons. These are the lamps — amber on every `EDIT` and on the presses that
/// change what the display is showing, cyan on `MOD`, `CHORD` and `CURVES`,
/// white on a plain switch. This window takes the hardware's own three rather
/// than inventing any:
///
/// | | |
/// | --- | --- |
/// | [`WAY_IN`] | a press that opens a section, which is what the hardware's amber `EDIT` does |
/// | [`MODULATION`] | something other than a hand can move this control, and the hardware's cyan button is the one named `MOD` |
/// | [`PLAIN`] | a switch that turns something on and changes nothing about what the other controls mean |
///
/// Everything else on the panel is the panel, the metal or the ink, so a mark
/// in one of these is one of the few things on a rack of forty that is not.
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x00ff_be3d` is not one"
)]
pub const WAY_IN: Color = color!(0xffbe3d);

/// The cyan the instrument lights `MOD` in: a control something else can move.
///
/// See [`WAY_IN`] for why these two and no others.
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x0040_d0e6` is not one"
)]
pub const MODULATION: Color = color!(0x40d0e6);

/// The same cyan, as a cap lit in it actually reads.
///
/// [`MODULATION`] is the lamp, and a lamp behind a rubber cap comes out paler
/// than the colour it was: what the window draws *about* modulation — a routing
/// outlined in the matrix, the dot beside a control something else moves, the
/// legend in the footer — is drawn in this so that it is the same teal as the
/// `MOD` cap it refers to, rather than a harder one beside it.
///
/// A function and not a constant because the mixing is [`crate::cap`]'s, which
/// is where a lamp is turned into what a hand sees, and there is nothing to be
/// gained by writing the answer down twice.
#[must_use]
pub fn modulated() -> Color {
    crate::cap::glow(MODULATION)
}

/// The white the instrument lights a plain switch in.
///
/// The third lamp, and the one that is not a colour. `SYNC`, `BOOST`, `2 POLE`,
/// `INVERT` and the waveform pair are lit white on a `DeepMind`: a switch that
/// turns something on and does not change what anything else means gets no hue,
/// which is what makes the amber and the cyan mean something when they appear.
///
/// A shade off white, because it is an LED behind a translucent cap and not a
/// pixel: the cap's own diffuser carries it the rest of the way at the middle.
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x00e4_e9f3` is not one"
)]
pub const PLAIN: Color = color!(0xe4e9f3);

/// Returns a colour the library measured, as a colour this window can draw.
///
/// Three components rather than a parse, because three components are what the
/// library publishes and what a window wants; the one thing this does is put
/// them on the scale `iced` uses.
///
/// Every measured colour in this window comes through here: an effect's four
/// chassis colours, and the three a section's banner is printed on. None of
/// them is written down in this file, which is the point — a colour read off a
/// photograph of the instrument is the library's fact, and what this crate
/// supplies is the drawing.
#[must_use]
pub fn measured(colour: Colour) -> Color {
    let [red, green, blue] = colour.to_rgb();
    Color::from_rgb8(red, green, blue)
}

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
    /// The lit face of a display.
    ///
    /// The one bright surface on the instrument. A `DeepMind`'s screen is a
    /// backlit positive display, a pale green-white ground with the dots printed
    /// dark on it, so it is the only part of this panel that gives off light
    /// rather than catching it, and everything drawn on it is darker than it is.
    pub glass: Color,
    /// The far end of the backlight, which is the bottom of the glass.
    ///
    /// A backlight is a lamp behind a panel and never a flat fill: it is
    /// brightest where the light enters and falls away across the glass. Two
    /// stops is all the difference a screen this size shows, and a display
    /// drawn in one flat tone is the one part of the window that would look
    /// printed rather than lit.
    pub glass_low: Color,
    /// The darkest a dot on the glass is printed.
    ///
    /// Where [`written`](crate::written) mixes a claim to, so that the three claims are
    /// three depths of one ink rather than three colours on a pale ground.
    pub ink: Color,
    /// The near edge of a recess.
    pub recess_edge: Color,
    /// The lower wall of a recess, which is the side the light reaches.
    pub lit: Color,
    /// A scale printed on the panel.
    pub scale: Color,
    /// The end cheek the panel is bolted between.
    ///
    /// A `DeepMind` is a panel between two pieces of wood, and it is the one
    /// material on the instrument that is not metal, ink or the panel itself.
    /// The mark and the banner are both built out of it (see `docs/logo.svg`),
    /// so the window that carries the same mark carries the same two colours
    /// rather than a third pair chosen to look like them.
    pub wood: Color,
    /// The far side of it, which is where the light has stopped reaching.
    pub wood_low: Color,
    /// A line of grain along it, lighter than the cheek.
    pub grain: Color,
    /// The face of a metal part.
    pub metal: Color,
    /// Its lit edge.
    pub metal_high: Color,
    /// Its shaded edge.
    pub metal_low: Color,
}

/// Returns the glass, the far end of its backlight, and the ink on it, for a
/// display of the polarity `negative` asks for.
///
/// The same two greens either way round: the lit one is what a backlight puts
/// through the panel and the dark one is the panel with nothing driving it, so
/// turning them over is the display turned over rather than a second palette.
///
/// Apart from [`materials`] because of the press that turns them over: that
/// press shows a display of the polarity it is about to give you, which is the
/// one place in this window that has to draw a display the theme is not wearing.
#[must_use]
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x00d6_e7cd` is not one"
)]
pub fn glazing(negative: bool) -> (Color, Color, Color) {
    if negative {
        (color!(0x101a12), color!(0x18231b), color!(0xd6e7cd))
    } else {
        (color!(0xd6e7cd), color!(0xbfd3b6), color!(0x101a12))
    }
}

/// Returns the materials a drawn control is made of, in `theme`.
#[must_use]
#[expect(
    clippy::unreadable_literal,
    reason = "a colour is read as a colour, and `0x0011_141a` is not one"
)]
pub fn materials(theme: &Theme) -> Materials {
    let palette = theme.extended_palette();
    // The glass driven the other way, where the window is wearing that livery.
    // The same two greens: the lit one is what a backlight puts through the
    // panel and the dark one is the panel with nothing driving it, so turning
    // them over is the display turned over rather than a second palette.
    let (glass, glass_low, ink) = glazing(is_negative(theme));
    Materials {
        panel: palette.background.base.color,
        plate: color!(0x1b1f26),
        recess: color!(0x080a0e),
        glass,
        glass_low,
        ink,
        recess_edge: color!(0x242932),
        lit: color!(0x7d838f),
        scale: palette.background.strong.color,
        wood: color!(0xa9552c),
        wood_low: color!(0x8a3f1f),
        grain: color!(0xc26436),
        metal: color!(0xc9cdd6),
        metal_high: color!(0xf4f5f8),
        metal_low: color!(0x8e939f),
    }
}

/// What the theme of the window with a negative display is called.
///
/// The livery is carried on the theme's own name rather than in a flag beside
/// it, because it is not a flag: it is which of two ways this window is
/// painted, and every drawing in it already asks the theme what it is made of.
/// A second channel saying the same thing is a second channel to thread through
/// nine files and keep in step with the first.
const NEGATIVE: &str = "DeepMind Negative";

/// The theme both builds are drawn in.
///
/// Built once. Cloning it is cloning a pointer, which is what the window does
/// every time it draws.
#[must_use]
pub fn deepmind() -> Theme {
    static THEME: LazyLock<Theme> = LazyLock::new(|| Theme::custom("DeepMind".to_owned(), PALETTE));
    THEME.clone()
}

/// The same window with the display the other way up.
///
/// Every `DeepMind` ships with a positive display, dark dots printed on lit
/// glass, and the panel around it is dark, so the screen is the one thing on
/// the instrument that is brighter than its surroundings. That is what
/// [`deepmind`] draws. A negative display is the same glass driven the other
/// way: the ground goes dark and the dots light up, which is what most dot
/// matrix screens on dark equipment look like and is what somebody reading this
/// window in a dark room is likely to want.
///
/// Nothing else in the window changes. The panel, the metal, the recesses and
/// both claims stay exactly what they are, and the claim on the glass stays
/// three depths of one ink rather than three colours. See [`written`], which
/// turns the ordering over with the glass so that the strongest reading is
/// still the one that stands out most.
#[must_use]
pub fn negative() -> Theme {
    static THEME: LazyLock<Theme> = LazyLock::new(|| Theme::custom(NEGATIVE.to_owned(), PALETTE));
    THEME.clone()
}

/// Returns whether `theme` draws its displays the other way up.
#[must_use]
pub fn is_negative(theme: &Theme) -> bool {
    theme.name() == NEGATIVE
}

/// The colour a value is written in, given what backs it.
///
/// Copper for what this window put there and nothing has confirmed, green for
/// what the synthesizer reported, and the panel's own muted grey for a sound
/// nobody has read. The copper is the point: it is the colour of a claim, and it
/// goes away when a dump comes back.
///
/// It is never the only thing doing the work. A drawn control carries its claim
/// in the fill of its moving part, filled for a fact, an outline for a claim and
/// nothing at all for a value nobody has read, and this colour agrees with that
/// fill rather than replacing it, so the difference survives greyscale, a
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

/// The colour something written on the glass is written in, given what backs it.
///
/// The display's answer to [`tint`], and it has to be a different one: a lit
/// screen is the one pale surface in this window, and the copper and the green
/// that read as a claim and a fact against a dark panel are both lighter than
/// the ground they would be printed on here. So a claim is not a colour on the
/// glass but a depth of ink: the fact is printed hard, the claim is printed in
/// the copper mixed most of the way to the same black, and what nobody has read
/// is barely printed at all.
///
/// That ordering is the point. It is the one way of carrying a claim that
/// survives being photographed, projected, or read by somebody who does not see
/// the copper, because the three are three lightnesses before they are three
/// hues.
///
/// It turns over with the glass and needs no second rule to do it. Every one of
/// the three is mixed towards the material it is printed on or the material it
/// is printed in, and [`negative`] swaps those two: on lit glass the strongest
/// reading is the one mixed furthest towards the dark ink, and on dark glass
/// the same arithmetic mixes it furthest towards the light one. Strongest is
/// still furthest from the ground either way, which is the whole of what the
/// ordering has to mean.
///
/// Mixed from the theme's own colours rather than written down, for the reason
/// [`tint`] takes them from the theme: a window somebody has themed differently
/// gets a readable display out of it rather than this file's idea of green.
#[must_use]
pub fn written(theme: &Theme, confidence: Confidence) -> Color {
    let palette = theme.extended_palette();
    let material = materials(theme);
    match confidence {
        // Not drawn at all, wherever a drawing can refuse; a washed-out grey
        // where one cannot, which is what an unwritten dot looks like through
        // the glass.
        Confidence::Unknown => mix(palette.background.strong.color, material.glass, 0.55),
        Confidence::Assumed => mix(palette.warning.base.color, material.ink, 0.55),
        Confidence::Confirmed => mix(palette.success.base.color, material.ink, 0.72),
    }
}

/// Mixes `amount` of `into` through `colour`, channel by channel.
///
/// Straight linear interpolation in whatever space the channels are already in,
/// which for darkening one colour towards another dark one is close enough that
/// a colour space would be arithmetic nobody could see the result of.
///
/// Public because a control that lights mixes its own lamp through the panel: an
/// unlit cap is the panel with a little metal in it, and a claimed one is the
/// claim's colour a fifth of the way up. A caller doing that with its own
/// arithmetic is a second mixer to keep in step with this one.
#[must_use]
pub fn mix(colour: Color, into: Color, amount: f32) -> Color {
    let across = amount.clamp(0.0, 1.0);
    let channel = |from: f32, to: f32| from + (to - from) * across;
    Color {
        r: channel(colour.r, into.r),
        g: channel(colour.g, into.g),
        b: channel(colour.b, into.b),
        a: colour.a,
    }
}

/// How much lighter one colour is than another, on the scale contrast is
/// measured in.
///
/// The ratio the web has used to decide whether text can be read since anybody
/// measured it: both colours' relative luminance, lightened by a twentieth so
/// that black against black is 1 rather than a division by zero, larger over
/// smaller. 1 is two colours nobody can tell apart and 21 is black on white.
///
/// Relative luminance and not a mean of the channels: a saturated green is
/// bright and a saturated blue of the same numbers is not, and an average puts
/// dark ink on the second one.
#[must_use]
pub fn contrast(one: Color, other: Color) -> f32 {
    let light = |colour: Color| {
        let channel = |value: f32| {
            if value <= 0.040_45 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        channel(colour.r).mul_add(0.2126, channel(colour.g) * 0.7152) + channel(colour.b) * 0.0722
    };
    let (high, low) = {
        let (one, other) = (light(one), light(other));
        (one.max(other), one.min(other))
    };
    (high + 0.05) / (low + 0.05)
}

/// How much contrast a word has to have with what it is printed on.
///
/// The threshold the accessibility guidelines set for text at the sizes this
/// window sets it in. Written down once rather than judged per panel, because
/// the whole point of measuring is that nobody has to squint at a screenshot
/// and decide.
pub const READABLE: f32 = 4.5;

/// Returns the ink to print on `surface` so that it can be read.
///
/// The instrument's own two, chosen by which one the surface is further from:
/// a recess is what a legend is knocked out of on a pale part, and a metal
/// highlight is what one is printed in on a dark one. Neither is a new colour,
/// because the whole point of this file is that there are none.
///
/// By distance rather than by a threshold, because a measured chassis half way
/// between the instrument's ink and its metal is exactly where a threshold
/// flips on a rounding error, and the larger of two differences never does.
#[must_use]
pub fn ink_on(surface: Color, theme: &Theme) -> Color {
    let material = materials(theme);
    if contrast(surface, material.recess) > contrast(surface, material.metal_high) {
        material.recess
    } else {
        material.metal_high
    }
}

/// Returns `ink`, moved as far as it has to go to be read on `surface`.
///
/// A colour in this window is chosen for what it means, a claim amber or green
/// and a legend the panel's own grey, and what it is printed on is not always
/// chosen at all: an effect plate wears a chassis somebody measured off
/// a photograph of a rack unit, and half of the 35 are pale. So the meaning
/// picks the colour and this makes sure it arrives: nothing, where the ink
/// already clears [`READABLE`], and otherwise the same ink carried towards
/// whichever of the instrument's two the surface is further from, by the least
/// that gets it there.
///
/// Towards an ink rather than replaced by one, so an amber claim stays amber
/// and a green one stays green: what moves is how light it is, which is what
/// contrast is made of, and not which colour it is, which is what it means.
///
/// This is the one function that has to be called on every word the window
/// prints on a surface it did not choose. A colour that is right in a palette
/// and unreadable on a panel is not right.
#[must_use]
pub fn legible(ink: Color, surface: Color, theme: &Theme) -> Color {
    if contrast(ink, surface) >= READABLE {
        return ink;
    }
    let toward = ink_on(surface, theme);
    // Sixteen steps of the whole distance, which resolves finer than an eye
    // does and costs sixteen multiplications. The last step is `toward` itself,
    // so a surface that nothing can be read on still gets the best there is
    // rather than the ink it started with.
    (1..=STEPS)
        .map(|step| mix(ink, toward, f32::from(step) / f32::from(STEPS)))
        .find(|lifted| contrast(*lifted, surface) >= READABLE)
        .unwrap_or(toward)
}

/// How finely [`legible`] looks for the least it can move an ink.
const STEPS: u8 = 16;

/// The face of a button's cap, moulded rather than filled.
///
/// The body of a rubber cap, down its own height: the room's light lands on the
/// crown, the face is the material's own colour, and the foot sits in the
/// shadow the cap casts into its own slot. So a cap is a gradient and never one
/// flat colour, which is the same reason [`ground`] is a gradient rather than
/// the dark end of one.
///
/// Four stops rather than three, and the reason is what a moulded cap actually
/// is: the crown is nearly flat, the sides turn away quickly, and the last of
/// it is in shadow. A gradient that ran evenly from top to bottom is a bevel,
/// which is a hard thing with a chamfer on it.
///
/// `pressed` turns it over. A cap pushed into the panel catches the light along
/// its foot instead, which is the same face seen from the other side of the
/// press and is why nothing here needs a second colour to say it is held down.
///
/// What this does not draw is the sheen, the lamp and the slot: see
/// [`cap`](crate::cap), which lays those on in quads because there is no radial
/// gradient to light a cap from its middle with.
#[must_use]
pub fn moulded(face: Color, pressed: bool) -> Background {
    let (crown, shoulder, foot) = if pressed {
        (shade(face, -0.26), shade(face, -0.1), shade(face, 0.16))
    } else {
        (shade(face, 0.2), shade(face, 0.05), shade(face, -0.34))
    };
    Background::Gradient(Gradient::Linear(
        // Down the cap, like the panel behind it: the light end at the top,
        // where the light is.
        Linear::new(Radians(std::f32::consts::PI))
            .add_stop(0.0, crown)
            // The crown holds its tone most of the way down, because the top of
            // a moulded cap is a plateau and not the peak of a curve.
            .add_stop(0.34, shoulder)
            .add_stop(0.66, face)
            .add_stop(1.0, foot),
    ))
}

/// `colour` lifted towards the light, or dropped into shadow.
///
/// One knob rather than two calls to [`mix`], because a cap is lit and shaded
/// by the same light and the two ends of it should be written the same way.
fn shade(colour: Color, amount: f32) -> Color {
    if amount >= 0.0 {
        mix(colour, Color::WHITE, amount)
    } else {
        mix(colour, Color::BLACK, -amount)
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

/// What a press whose whole face is a mark is drawn as.
///
/// Nothing at all, until a hand comes near it. A press with a word in it needs
/// a rim to say where the word stops being a label and starts being a button;
/// a press whose face is a nine-dot mark does not, because the mark is already
/// a shape on the panel and a rounded rectangle round it is a second shape
/// saying the same thing. A rim drawn to the height of the *words* beside it
/// leaves a twenty-two point mark floating in a twenty-nine point box, with four
/// points of panel above and below it.
///
/// So the mark stands on the panel the way the numbers on a rack unit's case
/// stand on the case, and what a hand gets back is light rather than a frame:
/// the panel lifts to the plate under the pointer and to the recess while it
/// is held, which is the same pair of surfaces every other press here moves
/// between. What the press *does* is said in the footer while the pointer is
/// on it, where this window already says what is under the pointer.
#[must_use]
pub fn marked(theme: &Theme, status: button::Status) -> button::Style {
    let material = materials(theme);
    button::Style {
        background: match status {
            button::Status::Hovered => Some(Background::Color(material.plate)),
            button::Status::Pressed => Some(Background::Color(material.recess)),
            button::Status::Active | button::Status::Disabled => None,
        },
        text_color: material.metal,
        border: border::rounded(3),
        ..button::Style::default()
    }
}

/// The ink something printed on a cap is stencilled in.
///
/// Dark, whichever cap it is. Every cap in this window is a pale thing now —
/// lit, it is a lamp behind amber rubber; unlit, it is the off-white the rubber
/// itself is moulded from — so both are brighter than anything the panel prints
/// its legends in, and both take the ink the instrument silkscreens *on* a pale
/// surface rather than the metal it silkscreens on a dark one.
///
/// That was not true while an unlit cap was drawn at the panel's own colour,
/// which is why this used to be two inks. It is one because the cap changed.
///
/// Mixed off the theme's own materials rather than written down as a colour, so
/// a window someone has themed differently prints on its caps in its own ink.
#[must_use]
pub fn on_cap(theme: &Theme) -> Color {
    let material = materials(theme);
    mix(material.recess, material.panel, PRINTED)
}

/// How far the printing on a cap is carried from the dark towards the panel.
///
/// Barely at all: it is ink on something lit.
const PRINTED: f32 = 0.2;

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

/// What a field somebody types into is drawn as.
///
/// The same recess a [`selector`] is cut into, because it is the same control
/// wearing a caret: the modulation matrix's destinations are a list of 133 and
/// the only way through them at speed is to type, so the picker on that row is
/// a list that can be typed into rather than a list beside a search box.
///
/// What is typed is metal and what has not been typed yet is the dim metal a
/// legend is printed in, which is the same pair every unlit thing in this
/// window uses. The edge lights the way a section button lights while the
/// caret is in it, so that a field being typed into is as obvious as a section
/// being looked at.
#[must_use]
pub fn field(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let material = materials(theme);
    let rim = match status {
        text_input::Status::Focused { .. } => material.lit,
        text_input::Status::Hovered => material.metal_low,
        text_input::Status::Active | text_input::Status::Disabled => material.recess_edge,
    };
    text_input::Style {
        background: Background::Color(material.recess),
        border: Border {
            color: rim,
            width: 1.0,
            radius: 2.into(),
        },
        icon: material.metal_low,
        placeholder: material.metal_low,
        value: material.metal_high,
        selection: material.plate,
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

/// What the window is drawn as while a sheet is lying over it.
///
/// The panel in the shadow the sheet casts, which is one thing rather than two:
/// a modal that dimmed the window by painting grey over it would be a grey
/// window, and what is behind a sheet is the instrument with the light off it.
/// So it is the recess, the darkest material this panel has, at the alpha that
/// leaves the plates behind still legible as plates.
///
/// Translucent rather than opaque because the thing it is covering is the thing
/// the sheet was opened *from*. A `VCF` opened off the panel's own `VCF` plate
/// should still have that plate under it: an editor whose detail view replaced
/// the instrument would be back to being an application that happens to control
/// a synthesizer.
#[must_use]
pub fn unlit(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: SHADED,
            ..materials(theme).recess
        })),
        ..container::Style::default()
    }
}

/// How much of the shadow is shadow.
///
/// Three fifths. Less and the panel behind competes with the sheet for the eye,
/// which is the failure a modal exists to avoid; more and it is a black window
/// with a panel on it, which loses the one thing the translucency is for, that
/// what is behind the sheet is where the sheet came from. At this much the
/// plates behind a sheet are still plates and nothing on them can be read,
/// which is the pair of things wanted.
const SHADED: f32 = 0.6;

/// What a sheet lying over the window is drawn as.
///
/// The panel, lifted off the window and lit along its edge, which is the one
/// edge in this window that is a thing standing *above* another rather than a
/// recess cut into one: the light catches the near edge of a raised plate and
/// the far wall of a cut one.
///
/// The panel and not [`bay`], which is the face plate. What goes on a sheet is a
/// rack, and a rack draws its own face plate: a sheet in the same material would
/// be a plate on a plate, with the rack's own border the only thing saying where
/// one ended. On the panel it sits exactly as it sat on the window, which is
/// what a sheet is meant to be — the same rack, brought forward.
#[must_use]
pub fn lifted(theme: &Theme) -> container::Style {
    let material = materials(theme);
    container::Style {
        background: Some(Background::Color(material.panel)),
        border: Border {
            color: material.lit,
            width: 1.0,
            radius: 4.into(),
        },
        ..container::Style::default()
    }
}

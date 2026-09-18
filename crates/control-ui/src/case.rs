//! The surface an effect's case is made of.
//!
//! Four engines stand on this page and the library says what kind of thing each
//! one is: `Algorithm::characters` is *vintage*, *modelled*, *lo-fi*, *stereo*,
//! *dual*, *multiband*, *combined*, *modulated* and *dynamic*, read off the
//! specification rather than matched on a name
//! ([deepmind-midi#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31)
//! published the marks, 26.5 the characters). Two of those are facts about the
//! *unit* rather than about the signal, and they are the two a surface can
//! carry: a Tel-Ray delay is a box from before effects were digital, and a
//! decimator is a thing that degrades what goes through it on purpose.
//!
//! So a case is finished the way the thing it is a picture of would be. Nothing
//! here is a claim about the sound, and nothing is invented about the
//! instrument: the colours are the library's measurements, and this is the
//! *material* they are laid on, which is the window's own business, the same
//! way the weight of a stroke and the pitch of a display's dots are.
//!
//! # Made of quads, because that is what a renderer here has
//!
//! The same constraint the [marks](crate::mark) are drawn under: this crate is
//! generic over the renderer, and what every renderer behind
//! [`iced_core::Renderer`] can do is fill a rounded rectangle. A texture is
//! therefore a few hundred of them, hairlines for a brushed face, blotches and
//! scratches for a worn one, a broken diagonal for a degraded one, laid
//! down from a
//! sequence that is the same on every frame, because a surface that reshuffled
//! itself sixty times a second would be a surface nobody could look at.
//!
//! The sequence is a counter run through a hash rather than a random number
//! generator: the same algorithm in the same engine gets the same scuffs every
//! time the page is drawn, and two engines running the same algorithm get the
//! same case, which is what two of the same unit in a rack look like.

use deepmind_midi::effect::{Algorithm, Character};
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Color, Element, Length, Rectangle, Size, Theme, Widget, mouse, renderer,
};

use crate::style;

/// How a case's face is finished.
///
/// One per engine, chosen from what the library says the algorithm *is*. A
/// character that says nothing about the box, such as stereo, dual or dynamic,
/// leaves the face alone, because a stereo chorus is a rack unit like any other
/// and a
/// surface that changed for every tag would be a page of four different
/// materials that mean nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Finish {
    /// A modern unit: a brushed face, and little else.
    ///
    /// Still not flat. A rack unit's face is brushed metal or a painted steel
    /// panel, and four large blocks of one colour on a page is the one way a
    /// case measured off a photograph gives itself away.
    #[default]
    Brushed,
    /// A unit from before effects were digital: worn, and rubbed where hands
    /// go.
    ///
    /// Coarser than brushed and unevenly lit: broad blotches where the light
    /// has not fallen evenly, short scratches over them where things have gone
    /// past, a few long rubs where a hand goes, and corners darker than the
    /// middle the way a panel that has been handled for thirty years is.
    Worn,
    /// A unit that degrades what goes through it: a fine diagonal grain, with
    /// blocks missing out of it.
    ///
    /// The one face on the page that does not run the way everything standing
    /// on it does, which is what keeps a grain this fine underneath a legend
    /// rather than behind it.
    Gritty,
}

impl Finish {
    /// Returns how the case of the engine running `algorithm` is finished.
    ///
    /// Vintage first, because a Tel-Ray delay is a vintage unit *and* a lo-fi
    /// one and the box is the older fact. A window that layered the two would
    /// be making a third material up.
    pub(crate) fn of(algorithm: &'static Algorithm) -> Self {
        if algorithm.has(Character::Vintage) {
            Self::Worn
        } else if algorithm.has(Character::LoFi) {
            Self::Gritty
        } else {
            Self::Brushed
        }
    }

    /// How hard this finish's pattern is laid down, as a share of the way from
    /// the case towards the light on it or the shadow in it.
    ///
    /// Enough to feel and not enough to read as a pattern: the test is whether
    /// a legend printed over it is any harder to read, which is the same test
    /// the [hero mark](crate::mark::hero) is under.
    const fn ink(self) -> f32 {
        match self {
            Self::Brushed => 0.055,
            Self::Worn => 0.09,
            Self::Gritty => 0.075,
        }
    }
}

/// Returns the number a case's scuffs are laid out from, for the algorithm of
/// this name.
///
/// The name rather than a value byte, because a `FX n Type` byte is a fact about
/// a firmware, since 1.0 and 1.1 number the 35 differently, and a unit that
/// changed its scuffs when the synthesizer reported a different firmware would
/// be a unit that changed because the cable did.
pub(crate) fn seed_of(name: &str) -> u32 {
    name.bytes().fold(0x811C_9DC5_u32, |word, byte| {
        (word ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

/// Draws the face of a case, in `chassis`, finished the way `finish` says.
///
/// `seed` is what makes two of the same algorithm two of the same unit: the
/// scuffs are the same drawing for the same number, so the page does not
/// reshuffle itself under a hand that is trying to read it.
pub(crate) fn surface<'a, Renderer, Ink>(
    finish: Finish,
    seed: u32,
    chassis: Ink,
) -> Element<'a, crate::Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Ink: Fn(&Theme) -> Color + 'a,
{
    Element::new(Face {
        finish,
        seed,
        chassis,
    })
}

/// The face, and what it is made of.
#[derive(Debug)]
struct Face<Ink> {
    finish: Finish,
    seed: u32,
    /// The case's own colour, asked of the theme because the window can be
    /// turned over while it is open.
    chassis: Ink,
}

/// Returns a number between nothing and one, from `seed` and `step`.
///
/// A counter through a hash rather than a generator with state: the same pair
/// is the same number on every frame and in every process, which is what makes
/// a face a property of the algorithm rather than of when the page was drawn.
fn hash(seed: u32, step: u32) -> f32 {
    let mut word = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(step.wrapping_mul(0x85EB_CA6B));
    word ^= word >> 15;
    word = word.wrapping_mul(0x2545_F491);
    word ^= word >> 13;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a hash laid onto nothing-to-one, where the low bits are the point"
    )]
    let fraction = (word % 100_003) as f32 / 100_003.0;
    fraction
}

/// How far apart the lines of a brushed face run, in points.
///
/// Close enough that the face reads as drawn in one direction and far enough
/// that the lines are lines: a brushed panel under a lamp is a few dozen of
/// them across a rack unit, not a hatch.
const BRUSH: f32 = 3.0;

/// How many scuffs a worn face carries, per thousand square points of it.
///
/// Density rather than a count, so that a wide case and a narrow one are the
/// same material rather than the same number of marks stretched over different
/// room.
const SCUFFS: f32 = 6.0;

/// How long a scuff is, as a share of the face's width: the shortest, and how
/// much longer the longest is.
///
/// Short. What wears a panel is a hand going past it, and what that leaves is a
/// streak an inch long rather than a line across the unit, which is what the
/// [rubs](rubbed) are and there are five of those.
const SCUFF: (f32, f32) = (0.03, 0.13);

/// How many blotches are laid under them, per thousand square points.
///
/// Far fewer than scuffs and far larger: the two together are what a surface
/// that has been in a room for thirty years looks like, which is unevenly lit
/// at arm's length and scratched at reading distance. Either alone is a
/// texture; both is a material.
const BLOTCHES: f32 = 1.4;

/// How large a blotch is, across and down, as a share of the shorter side.
const BLOTCH: (f32, f32) = (0.18, 0.55);

/// How hard a blotch is laid down, against a scuff.
const SOFTLY: f32 = 0.27;

/// How many rings a blotch is laid down in.
///
/// A quad has an edge, and a patch of uneven light does not. Three rectangles
/// inside each other at a third of the weight each is a falloff rather than a
/// rectangle: the edge is where one of the three stops, which at this weight is
/// a step nobody can find, and the middle is where all three are.
const FEATHER: u32 = 3;

/// The same count, to divide by.
const RINGS: f32 = 3.0;

/// How far apart the diagonals of a degraded face run, in points.
///
/// A hatch rather than a field of dots. What a lattice of dots draws is a screen
/// door: the eye finds the grid, and then the grid is the loudest thing on a
/// plate whose controls are supposed to be. A broken diagonal has no grid
/// in it to find: it reads as the surface being *made of* something, which is
/// what lo-fi is a picture of.
const HATCH: f32 = 6.0;

/// How long one step of a diagonal is, in points.
const DASH: f32 = 2.0;

/// How much of a diagonal is actually laid down.
const DRAWN: f32 = 0.55;

/// How many dropouts a degraded face carries, per thousand square points.
///
/// The other half of what a converter running out of bits does: most of it is
/// grain, and now and then a whole block of it goes.
const DROPOUTS: f32 = 0.8;

/// How large a dropout is, across and down, in points.
const DROPOUT: (f32, f32) = (7.0, 3.0);

/// How long the rubs across a worn face are, as a share of its width.
const RUB: f32 = 0.45;

/// How many of them there are.
const RUBS: u32 = 5;

/// How dark the corners of a worn face are, against its middle.
const HANDLED: f32 = 0.16;

/// How far in from an edge that darkening reaches, as a share of the shorter
/// side.
const REACH: f32 = 0.22;

/// How many bands the darkening is drawn in.
///
/// Enough that it reads as a surface falling away rather than as a frame drawn
/// round one, and few enough that a page of four cases is not a page of
/// gradients.
const BANDS: i32 = 7;

/// The same count, to divide by.
const EVERY: f32 = 7.0;

/// How many marks are laid on a face at the very most.
///
/// A ceiling rather than a number anybody chose: the counts above are
/// densities, and a window dragged to the size of a wall would otherwise be a
/// window laying down quads until it stopped answering.
const AT_MOST: u32 = 400;

/// Returns how many marks a face of these bounds takes, at `per` marks to the
/// thousand square points.
fn many(bounds: Rectangle, per: f32) -> u32 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of marks on a plate"
    )]
    let count = (bounds.width * bounds.height / 1000.0 * per)
        .round()
        .max(0.0) as u32;
    count.min(AT_MOST)
}

impl<Message, Renderer, Ink> Widget<Message, Theme, Renderer> for Face<Ink>
where
    Renderer: iced_core::Renderer,
    Ink: Fn(&Theme) -> Color,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }
        // Light and shadow rather than ink: a finish is what the room does to a
        // surface, and it does the same thing to a cream panel as to a black
        // one. Every other mark on this page is mixed towards an *ink*, which
        // is the right rule for something printed and the wrong one for
        // something worn.
        let case = (self.chassis)(theme);
        let lit = style::mix(case, Color::WHITE, self.finish.ink());
        let dark = style::mix(case, Color::BLACK, self.finish.ink());
        // Everything is cut to the case, because a mark laid across the plate
        // beside it is a mark on somebody else's unit — and to the room the
        // case was *given*, which is not the same thing. `with_layer` starts a
        // clip rather than narrowing the one already in force, so a layer at
        // the case's own bounds is a case that paints outside whatever is
        // holding it: on a page of four engines that scrolls, the two below the
        // fold were drawn over the footer.
        let Some(cut) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(cut, |renderer| {
            fill(renderer, bounds, case);
            match self.finish {
                Finish::Brushed => brushed(renderer, bounds, self.seed, lit, dark),
                Finish::Worn => {
                    worn(renderer, bounds, self.seed, lit, dark);
                    rubbed(renderer, bounds, self.seed, lit, dark);
                    handled(renderer, bounds, Color::BLACK);
                }
                Finish::Gritty => gritty(renderer, bounds, self.seed, lit, dark),
            }
        });
    }
}

/// Draws a worn face's grain: blotches under scuffs, neither on a lattice.
///
/// Nothing here has a pitch. What a lattice draws, however hard its cells are
/// jittered, is a *weave*, because the eye finds the row and the column and
/// then cannot stop finding them; and a weave is a claim about the unit that
/// the library never made. What wears a box is a room and a pair of hands, and
/// neither works to a grid: the blotches are where the light has not fallen
/// evenly for thirty years, and the scuffs are where something went past.
fn worn<Renderer>(renderer: &mut Renderer, bounds: Rectangle, seed: u32, lit: Color, dark: Color)
where
    Renderer: iced_core::Renderer,
{
    let side = bounds.width.min(bounds.height);
    let (least, more) = BLOTCH;
    for blotch in 0..many(bounds, BLOTCHES) {
        let across = hash(seed, blotch * 5);
        let down = hash(seed, blotch * 5 + 1);
        let wide = side * (least + more * hash(seed, blotch * 5 + 2));
        let tall = side * (least + more * hash(seed, blotch * 5 + 3)) * 0.5;
        let light = hash(seed, blotch * 5 + 4);
        let mark = Rectangle {
            x: bounds.x + (bounds.width - wide).max(0.0) * across,
            y: bounds.y + (bounds.height - tall).max(0.0) * down,
            width: wide,
            height: tall,
        };
        let shade = if light > 0.5 { lit } else { dark };
        for ring in 0..FEATHER {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a handful of rings, counted out"
            )]
            let step = ring as f32 / RINGS / 2.0;
            fill(
                renderer,
                Rectangle {
                    x: mark.x + wide * step,
                    y: mark.y + tall * step,
                    width: (wide - wide * step * 2.0).max(0.0),
                    height: (tall - tall * step * 2.0).max(0.0),
                },
                Color {
                    a: SOFTLY / RINGS,
                    ..shade
                },
            );
        }
    }
    let (least, more) = SCUFF;
    for scuff in 0..many(bounds, SCUFFS) {
        let across = hash(seed, 104_729 + scuff * 4);
        let down = hash(seed, 104_729 + scuff * 4 + 1);
        let along = hash(seed, 104_729 + scuff * 4 + 2);
        let light = hash(seed, 104_729 + scuff * 4 + 3);
        let width = bounds.width * (least + more * along);
        let mark = Rectangle {
            x: bounds.x + (bounds.width - width).max(0.0) * across,
            y: bounds.y + (bounds.height - 1.0).max(0.0) * down,
            width,
            height: 1.0,
        };
        let shade = if light > 0.55 { lit } else { dark };
        // Most of them are barely there. A face whose every scratch caught the
        // light is a face that has been drawn on rather than used.
        fill(
            renderer,
            mark,
            Color {
                a: 0.25 + light * 0.75,
                ..shade
            },
        );
    }
}

/// Draws a brushed face: lines the long way, at one pitch, at many weights.
///
/// What a brush leaves is a direction. The pitch is even, because a face whose
/// lines were scattered has been sanded rather than brushed, and what varies is
/// how hard each one is laid down and how far along the face it runs,
/// which is what keeps a few dozen parallel lines from reading as a hatch.
fn brushed<Renderer>(renderer: &mut Renderer, bounds: Rectangle, seed: u32, lit: Color, dark: Color)
where
    Renderer: iced_core::Renderer,
{
    for (line, y) in steps(bounds.height, BRUSH) {
        let weight = hash(seed, line * 3);
        let from = hash(seed, line * 3 + 1);
        let along = hash(seed, line * 3 + 2);
        // A line that is barely there is most of them: the ones that catch are
        // what the eye reads, and a face of equal lines is a grating.
        if weight < 0.35 {
            continue;
        }
        let width = bounds.width * (0.35 + along * 0.65);
        let mark = Rectangle {
            x: bounds.x + (bounds.width - width) * from,
            y: bounds.y + y,
            width,
            height: 1.0,
        };
        fill(renderer, mark, if weight > 0.68 { lit } else { dark });
    }
}

/// Draws the long rubs across a worn face, where a hand has been.
fn rubbed<Renderer>(renderer: &mut Renderer, bounds: Rectangle, seed: u32, lit: Color, dark: Color)
where
    Renderer: iced_core::Renderer,
{
    for rub in 0..RUBS {
        let down = hash(seed, 7919 + rub * 2);
        let along = hash(seed, 7919 + rub * 2 + 1);
        let width = bounds.width * RUB;
        let mark = Rectangle {
            x: bounds.x + (bounds.width - width) * along,
            y: bounds.y + bounds.height * down,
            width,
            height: 1.0,
        };
        if !inside(bounds, mark) {
            continue;
        }
        fill(renderer, mark, if down > 0.5 { lit } else { dark });
    }
}

/// Draws a degraded face: a broken diagonal grain, and blocks where it drops
/// out altogether.
///
/// Diagonal because neither of the other two faces is, and because it is the
/// one direction nothing else on this page runs in: the legends are across, the
/// brushing is across, the columns are down. A surface that runs the other way
/// to everything standing on it stays underneath them.
fn gritty<Renderer>(renderer: &mut Renderer, bounds: Rectangle, seed: u32, lit: Color, dark: Color)
where
    Renderer: iced_core::Renderer,
{
    let reach = bounds.width + bounds.height;
    for (line, from) in steps(reach, HATCH) {
        let catches = hash(seed, line * 2) > 0.62;
        for (step, along) in steps(bounds.height, DASH) {
            if hash(seed, line * 8191 + step) > DRAWN {
                continue;
            }
            let mark = Rectangle {
                x: bounds.x + from - bounds.height + along,
                y: bounds.y + along,
                width: DASH,
                height: 1.0,
            };
            if !inside(bounds, mark) {
                continue;
            }
            fill(renderer, mark, if catches { lit } else { dark });
        }
    }
    let (wide, tall) = DROPOUT;
    for dropout in 0..many(bounds, DROPOUTS) {
        let across = hash(seed, 65_537 + dropout * 3);
        let down = hash(seed, 65_537 + dropout * 3 + 1);
        let light = hash(seed, 65_537 + dropout * 3 + 2);
        let mark = Rectangle {
            x: bounds.x + (bounds.width - wide).max(0.0) * across,
            y: bounds.y + (bounds.height - tall).max(0.0) * down,
            width: wide,
            height: tall,
        };
        fill(
            renderer,
            mark,
            Color {
                a: SOFTLY,
                ..if light > 0.5 { lit } else { dark }
            },
        );
    }
}

/// Returns the cells of one axis at `pitch`, counted out with their positions.
fn steps(length: f32, pitch: f32) -> impl Iterator<Item = (u32, f32)> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of cells across a plate"
    )]
    let count = (length / pitch).floor().max(0.0) as u32;
    (0..count).map(move |cell| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a count of cells across a plate"
        )]
        let at = cell as f32 * pitch;
        (cell, at)
    })
}

/// Whether a mark falls wholly inside the face it is on.
fn inside(bounds: Rectangle, mark: Rectangle) -> bool {
    mark.x >= bounds.x
        && mark.y >= bounds.y
        && mark.x + mark.width <= bounds.x + bounds.width
        && mark.y + mark.height <= bounds.y + bounds.height
}

/// Darkens the edges of a face, the way a panel that has been handled darkens.
fn handled<Renderer>(renderer: &mut Renderer, bounds: Rectangle, shade: Color)
where
    Renderer: iced_core::Renderer,
{
    let reach = bounds.width.min(bounds.height) * REACH;
    for band in 0..BANDS {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a handful of bands, counted out"
        )]
        let step = (band as f32 + 1.0) / EVERY;
        let inset = reach * step;
        let shade = Color {
            a: HANDLED / EVERY,
            ..shade
        };
        let band = Rectangle {
            x: bounds.x + inset,
            y: bounds.y + inset,
            width: (bounds.width - inset * 2.0).max(0.0),
            height: (bounds.height - inset * 2.0).max(0.0),
        };
        // The band is drawn as four edges rather than as a filled rectangle,
        // because a stack of filled rectangles is a stack of one colour and
        // what is wanted is the light falling away towards the frame.
        for edge in edges(bounds, band) {
            fill(renderer, edge, shade);
        }
    }
}

/// Returns the four strips of `outer` that `inner` does not cover.
fn edges(outer: Rectangle, inner: Rectangle) -> [Rectangle; 4] {
    [
        Rectangle {
            height: inner.y - outer.y,
            ..outer
        },
        Rectangle {
            y: inner.y + inner.height,
            height: (outer.y + outer.height) - (inner.y + inner.height),
            ..outer
        },
        Rectangle {
            width: inner.x - outer.x,
            ..outer
        },
        Rectangle {
            x: inner.x + inner.width,
            width: (outer.x + outer.width) - (inner.x + inner.width),
            ..outer
        },
    ]
}

/// Lays one quad down.
fn fill<Renderer>(renderer: &mut Renderer, bounds: Rectangle, colour: Color)
where
    Renderer: iced_core::Renderer,
{
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            ..renderer::Quad::default()
        },
        Background::Color(colour),
    );
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::effect::Algorithm;

    use super::{Finish, hash};

    #[test]
    fn the_same_algorithm_is_the_same_unit_every_time() {
        // A surface that reshuffled itself between frames would be a surface
        // nobody could look at, and one that differed between two engines
        // running the same algorithm would be two units that are supposed to
        // be one.
        for step in 0..64 {
            assert!((hash(7, step) - hash(7, step)).abs() < f32::EPSILON);
            assert!((0.0..1.0).contains(&hash(7, step)), "{}", hash(7, step));
        }
        assert!(
            (0..64).any(|step| (hash(7, step) - hash(8, step)).abs() > 0.01),
            "two seeds are the same unit"
        );
    }

    #[test]
    fn the_box_a_unit_came_in_decides_the_finish() {
        let tel_ray = Algorithm::by_name("T-RayDelay").expect("a Tel-Ray delay");
        let decimator = Algorithm::by_name("DecimDelay").expect("a decimator delay");
        let hall = Algorithm::by_name("HallRev").expect("a hall reverb");

        // Vintage first: the Tel-Ray is a lo-fi unit as well, and the box it
        // came in is the older fact about it.
        assert_eq!(Finish::of(tel_ray), Finish::Worn);
        assert_eq!(Finish::of(decimator), Finish::Gritty);
        assert_eq!(Finish::of(hall), Finish::Brushed);
    }
}

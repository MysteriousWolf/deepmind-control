//! The surface an effect's case is made of.
//!
//! Four engines stand on this page and the library says what kind of thing each
//! one is: `Algorithm::characters` is *vintage*, *modelled*, *lo-fi*, *stereo*,
//! *dual*, *multiband*, *combined*, *modulated*, *dynamic* — read off the
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
//! *material* they are laid on — which is the window's own business, the same
//! way the weight of a stroke and the pitch of a display's dots are.
//!
//! # Made of quads, because that is what a renderer here has
//!
//! The same constraint the [marks](crate::mark) are drawn under: this crate is
//! generic over the renderer, and what every renderer behind
//! [`iced_core::Renderer`] can do is fill a rounded rectangle. A texture is
//! therefore a few hundred of them — hairlines for a brushed face, a coarse
//! stipple for a worn one, a fine one for a degraded one — laid down from a
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
/// character that says nothing about the box — stereo, dual, dynamic — leaves
/// the face alone, because a stereo chorus is a rack unit like any other and a
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
    /// Coarser than brushed, and unevenly lit: the grain is a stipple rather
    /// than a set of lines, the corners are darker than the middle the way a
    /// panel that has been handled for thirty years is, and two long creases
    /// run across it.
    Worn,
    /// A unit that degrades what goes through it: finer, noisier, harder.
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

    /// How many marks are laid down across the face.
    const fn marks(self) -> u32 {
        match self {
            Self::Brushed => 90,
            Self::Worn => 260,
            Self::Gritty => 420,
        }
    }

    /// How hard a mark is laid down, as a share of the way from the case
    /// towards the light on it or the shadow in it.
    ///
    /// Enough to feel and not enough to read as a pattern: the test is whether
    /// a legend printed over it is any harder to read, which is the same test
    /// the [hero mark](crate::mark::hero) is under.
    const fn ink(self) -> f32 {
        match self {
            Self::Brushed => 0.055,
            Self::Worn => 0.13,
            Self::Gritty => 0.10,
        }
    }
}

/// Returns the number a case's scuffs are laid out from, for the algorithm of
/// this name.
///
/// The name rather than a value byte, because a `FX n Type` byte is a fact
/// about a firmware — 1.0 and 1.1 number the 35 differently — and a unit that
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

/// One mark on a face: where it is, how big, and how far towards the ink.
#[derive(Debug, Clone, Copy)]
struct Scuff {
    /// Across the face, as a share of its width.
    x: f32,
    /// Down it, the same way.
    y: f32,
    /// How wide, as a share of the width.
    wide: f32,
    /// How tall, in points, because a hairline is a hairline at any size.
    tall: f32,
    /// How far towards the ink, and which way: positive lights the case and
    /// negative darkens it.
    lift: f32,
}

impl<Ink> Face<Ink> {
    /// Returns the marks this face carries.
    ///
    /// Counted out rather than stored: a few hundred numbers hashed from a
    /// counter cost less than the table that would hold them, and the table
    /// would have to be regenerated whenever a number in it changed.
    fn scuffs(&self) -> impl Iterator<Item = Scuff> {
        let finish = self.finish;
        let seed = self.seed;
        (0..finish.marks()).map(move |index| {
            let one = hash(seed, index * 4);
            let two = hash(seed, index * 4 + 1);
            let three = hash(seed, index * 4 + 2);
            let four = hash(seed, index * 4 + 3);
            let sign = if four < 0.45 { -1.0 } else { 1.0 };
            match finish {
                // Hairlines the long way, as a brush leaves them.
                Finish::Brushed => Scuff {
                    x: one * 0.9,
                    y: two,
                    wide: 0.06 + three * 0.22,
                    tall: 1.0,
                    lift: sign * (0.5 + four * 0.5),
                },
                // A coarse, uneven grain — the pebbling of a covered box —
                // with the odd long rub across it where a hand has been.
                Finish::Worn => Scuff {
                    x: one,
                    y: two,
                    wide: if three > 0.90 {
                        0.06 + three * 0.12
                    } else {
                        0.004 + three * 0.016
                    },
                    tall: if three > 0.90 { 1.0 } else { 1.0 + three * 3.0 },
                    lift: sign * (0.35 + four * 0.65),
                },
                // Fine, dense and hard: dust on a converter.
                Finish::Gritty => Scuff {
                    x: one,
                    y: two,
                    wide: 0.002 + three * 0.004,
                    tall: 1.0,
                    lift: sign * (0.6 + four * 0.4),
                },
            }
        })
    }
}

/// Returns a number between nothing and one, from `seed` and `step`.
///
/// A counter through a hash rather than a generator with state: the same pair
/// is the same number on every frame and in every process, which is what makes
/// a scuff a property of the algorithm rather than of when the page was drawn.
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
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }
        // Light and shadow rather than ink: a scuff is what the room does to a
        // surface, and it does the same thing to a cream panel as to a black
        // one. Every other mark on this page is mixed towards an *ink*, which
        // is the right rule for something printed and the wrong one for
        // something worn.
        let case = (self.chassis)(theme);
        let lit = style::mix(case, Color::WHITE, self.finish.ink());
        let dark = style::mix(case, Color::BLACK, self.finish.ink());
        // Everything is cut to the case, because a scuff laid across the plate
        // beside it is a scuff on somebody else's unit.
        renderer.with_layer(bounds, |renderer| {
            fill(renderer, bounds, case);
            if self.finish == Finish::Worn {
                handled(renderer, bounds, Color::BLACK);
            }
            for scuff in self.scuffs() {
                let shade = if scuff.lift < 0.0 { dark } else { lit };
                let mark = Rectangle {
                    x: bounds.x + scuff.x * bounds.width,
                    y: bounds.y + scuff.y * bounds.height,
                    width: (scuff.wide * bounds.width).max(1.0),
                    height: scuff.tall,
                };
                if mark.x + mark.width > bounds.x + bounds.width
                    || mark.y + mark.height > bounds.y + bounds.height
                {
                    continue;
                }
                fill(renderer, mark, shade);
            }
        });
    }
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

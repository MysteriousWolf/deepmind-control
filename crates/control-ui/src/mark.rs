//! The mark an effect wears, drawn in the window's own materials.
//!
//! `deepmind-midi` 26.4 publishes one per family
//! ([deepmind-midi#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31)):
//! nine of them across the 35 algorithms. 26.5 publishes a finer one where an
//! effect's kind is something a symbol can carry, a plate reverb as a plate with
//! wavefronts leaving it and a hall as wavefronts far from their source, and
//! `Algorithm::mark` hands back whichever applies, so a window that asks
//! for a mark got the better one with nothing here to change. They are the same
//! language either way, which is what keeps the four engines reading as one
//! set whichever marks they land on.
//!
//! What is published is the strokes and not the picture, a polyline in a unit
//! box, an arc, a sine and a filled disc, for the same reason the effect panels
//! are published as data: this window and the plugin want the same mark at two
//! sizes, and neither can theme an image it did not lay out.
//!
//! So this is the laying out. The host provides the size, the stroke width and
//! the ink, which is the whole of what the library says a host provides.
//!
//! # Drawn in quads, because that is what a renderer here has
//!
//! This crate is generic over the renderer, because the desktop build and the
//! plugin do not have to agree on one, and what every renderer behind
//! [`iced_core::Renderer`] can do is fill a rounded rectangle. A stroke is
//! therefore a run of round-capped quads along its own path, the way the
//! [envelope](crate::envelope) drawing is a run of them along its outline, and a
//! curve is sampled into that run at a rate the size it is drawn at decides.
//! Nothing here holds a path, a mesh or a shader.

use deepmind_midi::effect::{Mark, Point, Stroke};
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Color, Element, Length, Rectangle, Size, Theme, Widget, mouse, renderer,
};

/// How thick a stroke is, as a share of the mark's own side.
///
/// A mark is a line drawing at the weight the window's own legends are
/// silkscreened at, so it thickens with the size rather than staying a hairline
/// on a mark drawn twice as large.
const WEIGHT: f32 = 0.085;

/// The thinnest a stroke is drawn, whatever the size says.
const HAIRLINE: f32 = 1.0;

/// How far apart two quads of one stroke stand, as a share of their width.
///
/// Closer than half, so a run of them is a line rather than a row of beads at
/// the shallowest angle a mark contains.
const STEP: f32 = 0.4;

/// How finely a curve is sampled, in quads per unit of length.
///
/// Read off the size it is drawn at rather than written into the library, which
/// is why the library publishes an arc and a sine as themselves: a polyline
/// through a sine is a row of corners, and how many corners is a question about
/// this window's pixels.
fn steps(length: f32, width: f32) -> usize {
    let apart = (width * STEP).max(0.5);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of quads along a stroke a window has room for"
    )]
    let count = (length / apart).ceil() as usize;
    count.max(1)
}

/// Draws `mark` at `side` points square, in whatever ink the surface takes.
///
/// The ink is a closure rather than a colour because every word and line on the
/// effects page names the surface it lands on and asks which of the
/// instrument's two can be read there: a mark on a cream chassis and the same
/// How large a hero mark is drawn, as a share of the shorter side of the case.
///
/// Most of the case's depth, so what is left on the plate is about a third of
/// its width. Larger than this and it stops being a watermark in a corner and
/// becomes a drawing the controls are standing on: a case is half again as wide
/// as it is deep, so a mark taken from the width sweeps the whole plate.
const HERO: f32 = 0.85;

/// How far a hero mark hangs off the corner it is anchored to, as a share of
/// its own side.
///
/// A third of it, which is what makes it read as a mark on a case rather than a
/// picture placed on one. A drawing wholly inside its corner is a badge that
/// grew.
const HANGS: f32 = 0.35;

/// How much of its usual weight a hero's strokes are laid down at.
///
/// Much less. Weight is a share of the side, so the same rule that keeps a
/// badge's lines visible at sixteen points gives a hero lines a quarter of an
/// inch thick, which is not faint at any colour.
const HERO_WEIGHT: f32 = 0.38;

/// Draws `mark` large and faint across the case it is the mark of.
///
/// The other way to put a family's mark on an engine, and the one that leaves
/// the strip alone: a badge on the strip is sixteen points of line drawing
/// beside a name that says the same thing in words, at a size where a shallow
/// mark and a round one are hard to place against each other. This is the same
/// nine strokes at ten times the area, in the case's own ink taken almost all
/// the way back to the case, anchored into a corner and running off it.
///
/// It says which family without being read, which is what a mark is for, and it
/// never competes with a word because it is barely there.
/// # Stamped, raised, or neither
///
/// [`Relief`] is how the mark meets the case it is on, and it is read off what
/// kind of unit the case is: a mark on a worn panel is *stamped into* it, one
/// on a modern face is *raised off* it, and one on a case that is neither is
/// printed flat. A stamping is two edges, the light that catches on one side and
/// the shadow that falls on the other, so a relieved hero is the same nine
/// strokes laid down three times, a dot apart, and which of the two edges comes
/// first is the whole difference between an indent and a boss.
#[must_use]
pub(crate) fn hero<'a, Renderer, Ink>(
    mark: &'static Mark,
    relief: Relief,
    ink: Ink,
) -> Element<'a, crate::Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Ink: Fn(&Theme) -> Inks + 'a,
{
    Element::new(Drawing {
        mark,
        side: 0.0,
        ink,
        fits: Fits::of(mark),
        anchored: true,
        relief,
    })
}

/// The three inks a relieved mark is laid down in: the light on its edge, the
/// shadow off it, and the body of the mark itself.
pub(crate) type Inks = (Color, Color, Color);

/// How a mark meets the surface it is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Relief {
    /// Printed on it: one pass, in the ink.
    #[default]
    Flat,
    /// Standing off it: lit along its top-left edge and casting below-right,
    /// which is what a boss does under a light from the top left, the same light
    /// every cap, plate and display in this window is drawn under.
    Raised,
    /// Stamped into it: the same two edges the other way round.
    Sunk,
}

impl Relief {
    /// Returns the two edges this relief draws, as (offset, which ink), in the
    /// order they are laid down.
    ///
    /// Behind the body of the mark in both cases, so the ink is what a reader
    /// sees and the edges are what they feel.
    const fn edges(self) -> &'static [(f32, f32)] {
        match self {
            // (across and down, and which way the ink goes: -1 lit, 1 shadow)
            Self::Flat => &[],
            Self::Raised => &[(-1.0, -1.0), (1.0, 1.0)],
            Self::Sunk => &[(-1.0, 1.0), (1.0, -1.0)],
        }
    }
}

/// How far an edge of a relieved mark stands from the mark itself, as a share
/// of the shorter side of the room it is drawn in.
///
/// Small: a stamping in a panel is a hundredth of an inch deep, and what says
/// so at this size is a line of light a few dots off the line it belongs to.
/// Any further and the three passes read as three marks rather than as one with
/// a depth.
const RELIEF: f32 = 0.013;

/// The least that is, in points, because a hundredth of nothing is nothing.
const RELIEF_LEAST: f32 = 1.0;

/// The mark itself: its strokes, the room they are drawn in, and the ink.
#[derive(Debug)]
struct Drawing<Ink> {
    mark: &'static Mark,
    side: f32,
    ink: Ink,
    /// Whether this is a hero rather than a badge.
    ///
    /// A badge is a square of its own and fills it. A hero takes whatever room
    /// it is put in, draws itself larger than that room's shorter side, and
    /// hangs off the far corner, so `at` is answering a different question and
    /// `size` is a different answer.
    anchored: bool,
    /// How the mark meets the surface it is on.
    relief: Relief,
    /// What of the library's unit box this mark's own strokes reach.
    ///
    /// Measured once, when the mark is built, because it is a property of the
    /// nine drawings rather than of the frame: it cannot change between frames
    /// and sampling nine curves on every one of them would be arithmetic done
    /// sixty times a second for an answer that is already known.
    fits: Fits,
}

/// How much of the unit box a mark's strokes actually use.
///
/// The library publishes the strokes in a box and does not claim they fill it,
/// and they do not: the reverb's wavefronts leave a third of the width empty on
/// one side, the imaging mark uses less than half the height, and the delay's
/// bars use nearly all of it. Drawn straight onto the room they are given, nine
/// marks side by side are nine different sizes hanging at nine different
/// heights, which is what a row of engine strips showed.
///
/// So the window measures what each one reaches and lays *that* into the room:
/// centred, and as large as fits in the direction it is longer, keeping its
/// aspect. The library's box is a coordinate system and not a claim about
/// relative sizes, since it never says a rotary mark is larger than an imaging
/// one, and drawn at that box's own scale the nine come out between seven and
/// thirteen points tall in a sixteen point space. Where the strokes sit inside
/// the box is the library's business and how big the drawing is on a strip is
/// this window's, which is the same division the effect panels are laid out
/// under.
#[derive(Debug, Clone, Copy)]
struct Fits {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Fits {
    /// How finely a curve is walked to find where it reaches.
    ///
    /// Generous, because this is measured once and a curve whose extreme falls
    /// between two samples is a mark that hangs a little low for ever.
    const SAMPLES: usize = 64;

    /// Measures what `mark`'s strokes reach.
    fn of(mark: &Mark) -> Self {
        let (mut left, mut top) = (f32::MAX, f32::MAX);
        let (mut right, mut bottom) = (f32::MIN, f32::MIN);
        let mut reached = |x: f32, y: f32| {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        };
        for stroke in mark.strokes() {
            match *stroke {
                Stroke::Line { points } => {
                    for point in points {
                        reached(point.x(), point.y());
                    }
                }
                Stroke::Dot { centre, radius } => {
                    reached(centre.x() - radius, centre.y() - radius);
                    reached(centre.x() + radius, centre.y() + radius);
                }
                Stroke::Arc { .. } | Stroke::Wave { .. } => {
                    for step in 0..=Self::SAMPLES {
                        let point = along(*stroke, fraction(step, Self::SAMPLES));
                        reached(point.x(), point.y());
                    }
                }
                // A stroke this window cannot draw reaches nowhere, which keeps
                // the measurement and the drawing agreeing about what is there.
                _ => {}
            }
        }
        if left > right || top > bottom {
            // A mark made of nothing this window can draw. The unit box is as
            // good an answer as any, and nothing is drawn into it.
            return Self {
                left: 0.0,
                top: 0.0,
                width: 1.0,
                height: 1.0,
            };
        }
        Self {
            left,
            top,
            width: right - left,
            height: bottom - top,
        }
    }
}

/// Returns `step` of `count` as a fraction of the way along.
#[expect(
    clippy::cast_precision_loss,
    reason = "a step of a curve, counted in tens"
)]
fn fraction(step: usize, count: usize) -> f32 {
    step as f32 / count as f32
}

/// Returns where a curved stroke has reached, a fraction of the way along it.
///
/// The same two walks the drawing makes, written once so that what is measured
/// and what is drawn cannot disagree about where a curve goes.
fn along(stroke: Stroke, through: f32) -> Point {
    match stroke {
        Stroke::Arc {
            centre,
            radius,
            start,
            sweep,
        } => {
            let turn = (start + sweep * through) * core::f32::consts::TAU;
            Point::new(
                centre.x() + radius * turn.cos(),
                centre.y() + radius * turn.sin(),
            )
        }
        Stroke::Wave {
            start,
            end,
            amplitude,
            cycles,
        } => {
            let (run, rise) = (end.x() - start.x(), end.y() - start.y());
            // The perpendicular is the start-to-end direction turned a quarter
            // turn anticlockwise, which the library documents and which points
            // up the screen in a box whose y increases downward.
            let swing = amplitude * (core::f32::consts::TAU * cycles * through).sin()
                / run.hypot(rise).max(f32::EPSILON);
            Point::new(
                start.x() + run * through + rise * swing,
                start.y() + rise * through - run * swing,
            )
        }
        _ => Point::new(0.5, 0.5),
    }
}

impl<Ink> Drawing<Ink> {
    /// Returns where a point of the library's unit box lands in `bounds`.
    ///
    /// Through what the strokes actually reach rather than through the box they
    /// were published in: the mark is scaled to fill the room in whichever
    /// direction it is longer, keeping its proportions, and centred in the
    /// other. See [`Fits`].
    fn at(&self, bounds: Rectangle, point: Point) -> (f32, f32) {
        let scale = self.scale(bounds);
        let (across, down) = (self.fits.width * scale, self.fits.height * scale);
        if self.anchored {
            // Into the bottom right and off it, so the case keeps the corner
            // the eye starts at and the drawing runs out of the one it does not.
            return (
                bounds.x + bounds.width - across * (1.0 - HANGS)
                    + (point.x() - self.fits.left) * scale,
                bounds.y + bounds.height - down * (1.0 - HANGS)
                    + (point.y() - self.fits.top) * scale,
            );
        }
        (
            bounds.x + (bounds.width - across) / 2.0 + (point.x() - self.fits.left) * scale,
            bounds.y + (bounds.height - down) / 2.0 + (point.y() - self.fits.top) * scale,
        )
    }

    /// How many points of the room one unit of the library's box covers.
    ///
    /// Whatever makes this mark's own extent fill the room in the direction it
    /// is longer, so the nine arrive at one optical size.
    ///
    /// This has been both ways and the reason it is this way is worth keeping.
    /// The library's box is a coordinate system rather than a claim about
    /// relative sizes: it never says a rotary mark is larger than an imaging
    /// one. Drawn at that box's own scale the nine come out between seven and
    /// thirteen points tall in a sixteen point space, so a row of engine strips
    /// is some marks the size of the words beside them and some half that,
    /// which reads as a drawing that has slipped rather than as a set.
    ///
    /// The aspect is kept, so nothing is distorted: the imaging mark stays wide
    /// and flat, it is simply as wide as the rotary mark is round. That is what
    /// every set of icons does and it is what makes them read as one.
    ///
    /// Everything measured in the unit box goes through this, a disc's radius
    /// included, or a mark would be placed at one size and drawn at another.
    fn scale(&self, bounds: Rectangle) -> f32 {
        let room = if self.anchored {
            // Larger than the room, which is the whole of what a hero is.
            let side = bounds.width.min(bounds.height) * HERO;
            Rectangle {
                width: side,
                height: side,
                ..bounds
            }
        } else {
            bounds
        };
        (room.width / self.fits.width.max(f32::EPSILON))
            .min(room.height / self.fits.height.max(f32::EPSILON))
    }
}

impl<Message, Renderer, Ink> Widget<Message, Theme, Renderer> for Drawing<Ink>
where
    Renderer: iced_core::Renderer,
    Ink: Fn(&Theme) -> Inks,
{
    fn size(&self) -> Size<Length> {
        if self.anchored {
            return Size::new(Length::Fill, Length::Fill);
        }
        Size::new(Length::Fixed(self.side), Length::Fixed(self.side))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if self.anchored {
            return layout::atomic(limits, Length::Fill, Length::Fill);
        }
        layout::atomic(limits, Length::Fixed(self.side), Length::Fixed(self.side))
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
        let (lit, shadow, ink) = (self.ink)(theme);
        let off = (bounds.width.min(bounds.height) * RELIEF).max(RELIEF_LEAST);
        // A hero runs off two edges of the case, so what falls outside is cut
        // rather than drawn over the plate beside it — and cut to the room the
        // mark was *given* as well, because `with_layer` starts a clip rather
        // than narrowing the one already in force. A layer at the mark's own
        // bounds is a mark that paints outside whatever is holding it.
        let Some(cut) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(cut, |renderer| {
            // The two edges first and the mark over them, so that what a
            // reader sees is the mark and what they feel is the light on it.
            for (step, which) in self.relief.edges().iter().copied() {
                let edge = Rectangle {
                    x: bounds.x + step * off,
                    y: bounds.y + step * off,
                    ..bounds
                };
                self.strokes(renderer, edge, if which < 0.0 { lit } else { shadow });
            }
            self.strokes(renderer, bounds, ink);
        });
    }
}

impl<Ink> Drawing<Ink> {
    /// Lays every stroke of the mark down in `bounds`.
    fn strokes<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle, ink: Color)
    where
        Renderer: iced_core::Renderer,
    {
        let weight = if self.anchored {
            WEIGHT * HERO_WEIGHT
        } else {
            WEIGHT
        };
        let width = (bounds.width.min(bounds.height) * weight).max(HAIRLINE);
        for stroke in self.mark.strokes() {
            match *stroke {
                Stroke::Line { points } => {
                    for pair in points.windows(2) {
                        let (Some(from), Some(to)) = (pair.first(), pair.last()) else {
                            continue;
                        };
                        segment(
                            renderer,
                            self.at(bounds, *from),
                            self.at(bounds, *to),
                            width,
                            ink,
                        );
                    }
                }
                Stroke::Arc { radius, sweep, .. } => {
                    let run = (sweep.abs() * core::f32::consts::TAU * radius * bounds.width)
                        .max(bounds.width * 0.1);
                    let count = steps(run, width);
                    let mut last = None;
                    for step in 0..=count {
                        let at = self.at(bounds, along(*stroke, fraction(step, count)));
                        if let Some(before) = last {
                            segment(renderer, before, at, width, ink);
                        }
                        last = Some(at);
                    }
                }
                Stroke::Dot { centre, radius } => {
                    let at = self.at(bounds, centre);
                    let across = (radius * 2.0 * self.scale(bounds)).max(width);
                    quad(renderer, at, across, ink);
                }
                Stroke::Wave {
                    start, end, cycles, ..
                } => {
                    let (run, rise) = (end.x() - start.x(), end.y() - start.y());
                    let length = (run.hypot(rise) * bounds.width).max(bounds.width * 0.1);
                    // A quarter of a cycle is the shortest run a sine has that
                    // is not a straight line, so it decides the sampling as much
                    // as the length does.
                    let count = steps(length, width).max(steps(cycles.abs() * 8.0, 1.0));
                    let mut last = None;
                    for step in 0..=count {
                        let at = self.at(bounds, along(*stroke, fraction(step, count)));
                        if let Some(before) = last {
                            segment(renderer, before, at, width, ink);
                        }
                        last = Some(at);
                    }
                }
                // A stroke a later library adds is one this window has no way to
                // draw, and a mark with it left out is better than a mark with
                // something invented in its place.
                _ => {}
            }
        }
    }
}

/// Lays a straight run of quads from one point to another.
fn segment<Renderer>(
    renderer: &mut Renderer,
    from: (f32, f32),
    to: (f32, f32),
    width: f32,
    ink: Color,
) where
    Renderer: iced_core::Renderer,
{
    let (run, rise) = (to.0 - from.0, to.1 - from.1);
    let length = run.hypot(rise);
    let count = steps(length, width);
    for step in 0..=count {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a step of a stroke this window has room for"
        )]
        let through = step as f32 / count as f32;
        quad(
            renderer,
            (from.0 + run * through, from.1 + rise * through),
            width,
            ink,
        );
    }
}

/// Fills one round quad, centred where it is put.
fn quad<Renderer>(renderer: &mut Renderer, at: (f32, f32), across: f32, ink: Color)
where
    Renderer: iced_core::Renderer,
{
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: at.0 - across / 2.0,
                y: at.1 - across / 2.0,
                width: across,
                height: across,
            },
            border: iced_core::Border {
                radius: (across / 2.0).into(),
                ..iced_core::Border::default()
            },
            ..renderer::Quad::default()
        },
        Background::Color(ink),
    );
}

#[cfg(test)]
mod tests {
    use super::{Drawing, Fits};
    use deepmind_midi::effect::{Algorithm, Point};
    use iced_core::{Color, Rectangle, Theme};

    /// The room a mark is measured in, away from the origin so that a mapping
    /// that forgot to add the offset fails rather than passing at zero.
    const ROOM: Rectangle = Rectangle {
        x: 40.0,
        y: 70.0,
        width: 20.0,
        height: 20.0,
    };

    /// One mark, ready to be asked where its points land.
    fn drawn(mark: &'static deepmind_midi::effect::Mark) -> Drawing<fn(&Theme) -> super::Inks> {
        Drawing {
            mark,
            side: ROOM.width,
            ink: (|_: &Theme| (Color::BLACK, Color::BLACK, Color::BLACK))
                as fn(&Theme) -> super::Inks,
            fits: Fits::of(mark),
            anchored: false,
            relief: super::Relief::Flat,
        }
    }

    #[test]
    fn every_mark_is_centred_in_the_room_it_is_given() {
        // Nine drawings published in one unit box, and not one of them fills
        // it: the reverb's wavefronts leave a third of the width empty on one
        // side. Drawn straight onto the room, they hang at nine different
        // heights, which is what a row of engine strips showed.
        for algorithm in Algorithm::all() {
            let drawing = drawn(algorithm.mark());
            let fits = drawing.fits;
            let (left, top) = drawing.at(ROOM, Point::new(fits.left, fits.top));
            let (right, bottom) = drawing.at(
                ROOM,
                Point::new(fits.left + fits.width, fits.top + fits.height),
            );
            let family = algorithm.family();

            let across = (left - ROOM.x) - (ROOM.x + ROOM.width - right);
            let down = (top - ROOM.y) - (ROOM.y + ROOM.height - bottom);
            assert!(
                across.abs() < 0.01,
                "{family:?} sits {across} off centre across the room"
            );
            assert!(
                down.abs() < 0.01,
                "{family:?} sits {down} off centre down the room"
            );
        }
    }

    #[test]
    fn every_mark_comes_out_at_one_optical_size() {
        // The other half of placing one: centred and tiny would be centred. A
        // mark reaches both edges of the room in the direction it is longer, so
        // the nine are one size beside each other rather than between seven and
        // thirteen points tall in a sixteen point space, which is what drawing
        // them at the library's own box scale gave.
        for algorithm in Algorithm::all() {
            let drawing = drawn(algorithm.mark());
            let fits = drawing.fits;
            let (left, top) = drawing.at(ROOM, Point::new(fits.left, fits.top));
            let (right, bottom) = drawing.at(
                ROOM,
                Point::new(fits.left + fits.width, fits.top + fits.height),
            );
            let family = algorithm.family();

            let filled = (right - left - ROOM.width).abs() < 0.01
                || (bottom - top - ROOM.height).abs() < 0.01;
            assert!(filled, "{family:?} is drawn smaller than the room it has");
            assert!(
                right - left <= ROOM.width + 0.01 && bottom - top <= ROOM.height + 0.01,
                "{family:?} is drawn past the room it has"
            );
            // Its own shape, not the room's: a wide mark stays wide.
            let drawn_aspect = (right - left) / (bottom - top);
            let own_aspect = fits.width / fits.height;
            assert!(
                (drawn_aspect - own_aspect).abs() < 0.01,
                "{family:?} is drawn at {drawn_aspect} where the library drew {own_aspect}"
            );
        }
    }
}

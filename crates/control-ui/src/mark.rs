//! The mark an effect wears, drawn in the window's own materials.
//!
//! `deepmind-midi` 26.4 publishes one per family
//! ([deepmind-midi#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31)):
//! nine of them across the 35 algorithms, because the difference between a Hall
//! Reverb and a Plate Reverb is not something a symbol carries and a drawing
//! that implied it would be inventing one. What it publishes is the strokes and
//! not the picture — a polyline in a unit box, an arc, a sine, a filled disc —
//! for the same reason it publishes the effect panels as data: this window and
//! the plugin want the same mark at two sizes, and neither can theme an image it
//! did not lay out.
//!
//! So this is the laying out. The host provides the size, the stroke width and
//! the ink, which is the whole of what the library says a host provides.
//!
//! # Drawn in quads, because that is what a renderer here has
//!
//! This crate is generic over the renderer — the desktop build and the plugin
//! do not have to agree on one — and what every renderer behind
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
/// mark on the window's own plate are not the same colour.
pub(crate) fn mark<'a, Renderer, Ink>(
    mark: &'static Mark,
    side: f32,
    ink: Ink,
) -> Element<'a, crate::Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Ink: Fn(&Theme) -> Color + 'a,
{
    Element::new(Drawing { mark, side, ink })
}

/// The mark itself: its strokes, the room they are drawn in, and the ink.
#[derive(Debug)]
struct Drawing<Ink> {
    mark: &'static Mark,
    side: f32,
    ink: Ink,
}

impl<Ink> Drawing<Ink> {
    /// Returns where a point of the library's unit box lands in `bounds`.
    fn at(bounds: Rectangle, point: Point) -> (f32, f32) {
        (
            bounds.x + point.x() * bounds.width,
            bounds.y + point.y() * bounds.height,
        )
    }
}

impl<Message, Renderer, Ink> Widget<Message, Theme, Renderer> for Drawing<Ink>
where
    Renderer: iced_core::Renderer,
    Ink: Fn(&Theme) -> Color,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.side), Length::Fixed(self.side))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
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
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let ink = (self.ink)(theme);
        let width = (bounds.width.min(bounds.height) * WEIGHT).max(HAIRLINE);
        for stroke in self.mark.strokes() {
            match *stroke {
                Stroke::Line { points } => {
                    for pair in points.windows(2) {
                        let (Some(from), Some(to)) = (pair.first(), pair.last()) else {
                            continue;
                        };
                        segment(
                            renderer,
                            Self::at(bounds, *from),
                            Self::at(bounds, *to),
                            width,
                            ink,
                        );
                    }
                }
                Stroke::Arc {
                    centre,
                    radius,
                    start,
                    sweep,
                } => {
                    let along = (sweep.abs() * core::f32::consts::TAU * radius * bounds.width)
                        .max(bounds.width * 0.1);
                    let count = steps(along, width);
                    let mut last = None;
                    for step in 0..=count {
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "a step of a curve this window has room for"
                        )]
                        let through = step as f32 / count as f32;
                        let turn = (start + sweep * through) * core::f32::consts::TAU;
                        let point = Point::new(
                            centre.x() + radius * turn.cos(),
                            centre.y() + radius * turn.sin(),
                        );
                        let at = Self::at(bounds, point);
                        if let Some(before) = last {
                            segment(renderer, before, at, width, ink);
                        }
                        last = Some(at);
                    }
                }
                Stroke::Dot { centre, radius } => {
                    let at = Self::at(bounds, centre);
                    let across = (radius * 2.0 * bounds.width).max(width);
                    quad(renderer, at, across, ink);
                }
                Stroke::Wave {
                    start,
                    end,
                    amplitude,
                    cycles,
                } => {
                    let (run, rise) = (end.x() - start.x(), end.y() - start.y());
                    let along = (run.hypot(rise) * bounds.width).max(bounds.width * 0.1);
                    // A quarter of a cycle is the shortest run a sine has that
                    // is not a straight line, so it decides the sampling as much
                    // as the length does.
                    let count = steps(along, width).max(steps(cycles.abs() * 8.0, 1.0));
                    let mut last = None;
                    for step in 0..=count {
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "a step of a curve this window has room for"
                        )]
                        let through = step as f32 / count as f32;
                        // The perpendicular is the start-to-end direction turned
                        // a quarter turn anticlockwise, which the library
                        // documents and which points up the screen in a box
                        // whose y increases downward.
                        let swing = amplitude * (core::f32::consts::TAU * cycles * through).sin()
                            / run.hypot(rise).max(f32::EPSILON);
                        let point = Point::new(
                            start.x() + run * through + rise * swing,
                            start.y() + rise * through - run * swing,
                        );
                        let at = Self::at(bounds, point);
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

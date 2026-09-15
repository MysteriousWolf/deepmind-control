//! The fader, drawn rather than borrowed.
//!
//! The instrument is covered in these, so the editor is too. A toolkit slider is
//! a different object: a thin rail with a dot on it, sized for a settings page,
//! with no scale and no cap. This is a track cut into the panel, a scale either
//! side of it, and a metal cap riding it, which is what a player is looking at
//! when they look down.
//!
//! # A press never jumps
//!
//! A toolkit slider moves its value to wherever the pointer landed. Nothing on
//! an instrument teleports, and a stray click that throws `VCF Frequency` from
//! 20 to 200 is audible, sent, and not undone by letting go. Pressing anywhere
//! on a fader takes hold of the cap where it already is, and the drag is
//! relative from there.
//!
//! # It runs down the panel, or across it
//!
//! Down, everywhere the rack draws one, because that is the way every fader on
//! the instrument runs. [`Axis::Across`] turns the same fader onto its side and
//! changes nothing else about it: the same recessed track, the same scale in
//! pairs, the same metal cap, the same relative grab, and the drag follows the
//! axis the cap does.
//!
//! That exists for one reason. A panel laid out by hand as rows — eight
//! modulation routings read across, source to destination to depth — cannot
//! give each row a column 128 points tall, and a value drawn as a number
//! because it would not fit is a value nobody can compare with the seven
//! above it. Turning the fader is the arrangement changing. What the control is
//! is untouched, which is the line hand layout does not cross.
//!
//! # What the claim changes
//!
//! Everything about the cap, and nothing about the track. A confirmed value is
//! the metal, filled; an assumed one is the metal as a stroke around the panel;
//! a value nobody has read has no cap at all, and the control is inert, because
//! the host crate refuses an edit before anything is known and a control that
//! cannot be moved must not look like one that can.

use core::ops::RangeInclusive;

use iced_core::layout::{self, Layout};
use iced_core::widget::{Tree, tree};
use iced_core::{
    Background, Border, Clipboard, Element, Event, Length, Point, Rectangle, Shell, Size, Theme,
    Widget, keyboard, mouse, renderer, touch,
};

use crate::Confidence;
use crate::style::{materials, tint};

/// Width of a fader, including the room its scale needs.
///
/// Across the travel rather than left to right: it is the height of one that
/// runs across the panel.
pub const WIDTH: f32 = 44.0;

/// Height of a fader, which is the travel plus the cap.
///
/// Along the travel, and the length one is given unless a caller asks for
/// another.
pub const HEIGHT: f32 = 128.0;

/// Width of the track cut into the panel.
const TRACK: f32 = 7.0;

/// The cap across its travel.
const CAP_WIDTH: f32 = 38.0;

/// The cap along its travel. Travel is the track less this, so the ends of the
/// range put the cap flush with the ends of the track.
const CAP_HEIGHT: f32 = 15.0;

/// Length of one arm of a scale tick.
const TICK: f32 = 8.0;

/// Where the scale is ticked, as a fraction of the travel.
const TICKS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// How much of the range a fine drag covers: an eighth.
const FINE: f32 = 0.125;

/// Which way a fader runs.
///
/// The way the cap travels, the way a drag is measured, and the way the scale
/// is ticked. Nothing else about the control depends on it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Down the panel, with the top of the travel the top of the range.
    ///
    /// Which is every fader the instrument has, and every fader the rack draws.
    #[default]
    Down,
    /// Across it, with the right of the travel the top of the range.
    Across,
}

/// A fader over one parameter's range.
#[expect(
    missing_debug_implementations,
    reason = "the callback is a closure, and a widget is not inspected"
)]
pub struct Fader<'a, Message> {
    value: u8,
    range: RangeInclusive<u8>,
    claim: Confidence,
    axis: Axis,
    length: f32,
    thickness: f32,
    on_change: Box<dyn Fn(u8) -> Message + 'a>,
}

/// Builds a fader over `range`, sitting at `value`.
///
/// `claim` is what backs the value, and it decides whether there is a cap to
/// take hold of at all. It runs down the panel until [`Fader::across`] says
/// otherwise.
pub fn fader<'a, Message>(
    range: RangeInclusive<u8>,
    value: u8,
    claim: Confidence,
    on_change: impl Fn(u8) -> Message + 'a,
) -> Fader<'a, Message> {
    Fader {
        value,
        range,
        claim,
        axis: Axis::Down,
        length: HEIGHT,
        thickness: WIDTH,
        on_change: Box::new(on_change),
    }
}

impl<Message> Fader<'_, Message> {
    /// Narrows the fader across its travel, to `thickness` points.
    ///
    /// For a panel whose hand layout is a row of many: thirty-two sequencer
    /// steps at a rack slot's width are eight feet of window. The track keeps
    /// its width, because a recess that thin is a line; the cap narrows with
    /// the fader, and the scale goes when there is no longer room for an arm of
    /// it either side. What the control is does not change, and neither does
    /// its travel: a step is still dragged the way every other value is.
    #[must_use]
    pub fn narrow(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Turns the fader onto its side, `length` points of travel long.
    ///
    /// For a panel whose hand layout is rows. The control is the same one; the
    /// row is what changed.
    #[must_use]
    pub fn across(mut self, length: f32) -> Self {
        self.axis = Axis::Across;
        self.length = length;
        self
    }
}

/// A drag in progress.
///
/// The value is carried as a float and moved by each pointer step rather than
/// measured from where the press landed, so that changing to a fine drag half
/// way through bends the rate without moving the value.
#[derive(Debug, Clone, Copy)]
struct Grab {
    value: f32,
    /// Where the pointer was along the fader's own axis.
    last: f32,
}

/// What a fader remembers between events.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    grab: Option<Grab>,
    fine: bool,
}

impl<Message> Fader<'_, Message> {
    /// Returns how long the fader is, and how wide, in that order.
    fn size_of(&self) -> (Length, Length) {
        match self.axis {
            Axis::Down => (Length::Fixed(self.thickness), Length::Fixed(self.length)),
            Axis::Across => (Length::Fixed(self.length), Length::Fixed(self.thickness)),
        }
    }

    /// Returns how far the cap can travel.
    fn travel(&self, bounds: Rectangle) -> f32 {
        let along = match self.axis {
            Axis::Down => bounds.height,
            Axis::Across => bounds.width,
        };
        (along - CAP_HEIGHT).max(1.0)
    }

    /// Returns where a pointer is along the travel.
    fn along(&self, position: Point) -> f32 {
        match self.axis {
            Axis::Down => position.y,
            Axis::Across => position.x,
        }
    }

    /// Returns how much of the range the pointer covered going `from` to `to`.
    ///
    /// Up is more on a fader that runs down the panel, and right is more on one
    /// that runs across it, which is the only place the axis reaches the
    /// arithmetic.
    fn advance(&self, from: f32, to: f32, bounds: Rectangle) -> f32 {
        let moved = match self.axis {
            Axis::Down => from - to,
            Axis::Across => to - from,
        };
        moved / self.travel(bounds) * self.span()
    }

    /// Returns the range as a span of values, never zero.
    fn span(&self) -> f32 {
        let low = f32::from(*self.range.start());
        let high = f32::from(*self.range.end());
        (high - low).max(1.0)
    }

    /// Returns how far along its travel the cap sits, as a fraction.
    fn fraction(&self) -> f32 {
        let low = f32::from(*self.range.start());
        ((f32::from(self.value) - low) / self.span()).clamp(0.0, 1.0)
    }

    /// Returns the track cut into the panel.
    fn track(&self, bounds: Rectangle) -> Rectangle {
        match self.axis {
            Axis::Down => Rectangle {
                x: bounds.x + (bounds.width - TRACK) / 2.0,
                y: bounds.y,
                width: TRACK,
                height: bounds.height,
            },
            Axis::Across => Rectangle {
                x: bounds.x,
                y: bounds.y + (bounds.height - TRACK) / 2.0,
                width: bounds.width,
                height: TRACK,
            },
        }
    }

    /// Returns how wide the cap is across the travel.
    ///
    /// The full cap unless the fader is narrower than one, and never so narrow
    /// that there is nothing to take hold of.
    fn cap_across(&self) -> f32 {
        CAP_WIDTH.min((self.thickness - 6.0).max(8.0))
    }

    /// Returns where the cap is.
    fn cap(&self, bounds: Rectangle) -> Rectangle {
        let travelled = self.fraction() * self.travel(bounds);
        let across = self.cap_across();
        match self.axis {
            Axis::Down => Rectangle {
                x: bounds.x + (bounds.width - across) / 2.0,
                y: bounds.y + self.travel(bounds) - travelled,
                width: across,
                height: CAP_HEIGHT,
            },
            Axis::Across => Rectangle {
                x: bounds.x + travelled,
                y: bounds.y + (bounds.height - across) / 2.0,
                width: CAP_HEIGHT,
                height: across,
            },
        }
    }

    /// Returns the line a cap's value is read against.
    fn indicator(&self, cap: Rectangle) -> Rectangle {
        match self.axis {
            Axis::Down => {
                let inset = (cap.width / 6.0).min(6.0);
                Rectangle {
                    x: cap.x + inset,
                    y: cap.y + cap.height / 2.0 - 1.0,
                    width: cap.width - inset * 2.0,
                    height: 2.0,
                }
            }
            Axis::Across => {
                let inset = (cap.height / 6.0).min(6.0);
                Rectangle {
                    x: cap.x + cap.width / 2.0 - 1.0,
                    y: cap.y + inset,
                    width: 2.0,
                    height: cap.height - inset * 2.0,
                }
            }
        }
    }

    /// Returns the two arms of the scale tick at `fraction` of the travel.
    fn tick(&self, bounds: Rectangle, fraction: f32) -> [Rectangle; 2] {
        let travelled = fraction * self.travel(bounds);
        if self.thickness < TRACK + (TICK + 2.0) * 2.0 {
            // Nothing to print a scale on. An arm overlapping the track reads
            // as a mark on the fader rather than a scale beside it.
            return [Rectangle::default(); 2];
        }
        match self.axis {
            Axis::Down => {
                let y = bounds.y + CAP_HEIGHT / 2.0 + self.travel(bounds) - travelled;
                [bounds.x + 2.0, bounds.x + bounds.width - 2.0 - TICK].map(|x| Rectangle {
                    x,
                    y,
                    width: TICK,
                    height: 1.0,
                })
            }
            Axis::Across => {
                let x = bounds.x + CAP_HEIGHT / 2.0 + travelled;
                [bounds.y + 2.0, bounds.y + bounds.height - 2.0 - TICK].map(|y| Rectangle {
                    x,
                    y,
                    width: 1.0,
                    height: TICK,
                })
            }
        }
    }

    /// Returns whether there is anything here to take hold of.
    fn is_live(&self) -> bool {
        !matches!(self.claim, Confidence::Unknown)
    }

    /// Rounds a value a drag has accumulated into the byte a parameter holds.
    fn byte(&self, value: f32) -> u8 {
        let clamped = value.clamp(f32::from(*self.range.start()), f32::from(*self.range.end()));
        // The float is what a drag accumulates; the parameter is a byte.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to the range, which is a u8 either end"
        )]
        let byte = clamped.round() as u8;
        byte
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Fader<'_, Message>
where
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        let (width, height) = self.size_of();
        Size::new(width, height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let (width, height) = self.size_of();
        layout::atomic(limits, width, height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.fine = modifiers.shift();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if !self.is_live() {
                    return;
                }
                if let Some(position) = cursor.position_over(bounds) {
                    // Take hold of the cap where it is. Nothing jumps.
                    state.grab = Some(Grab {
                        value: f32::from(self.value),
                        last: self.along(position),
                    });
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                state.grab = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { position, .. })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                let Some(grab) = state.grab else {
                    return;
                };
                let rate = if state.fine { FINE } else { 1.0 };
                let now = self.along(*position);
                let value = grab.value + self.advance(grab.last, now, bounds) * rate;
                let byte = self.byte(value);
                state.grab = Some(Grab {
                    value: value
                        .clamp(f32::from(*self.range.start()), f32::from(*self.range.end())),
                    last: now,
                });
                if byte != self.value {
                    self.value = byte;
                    shell.publish((self.on_change)(byte));
                }
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !self.is_live() || cursor.position_over(bounds).is_none() {
                    return;
                }
                let lines = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 16.0,
                };
                let step = if state.fine { 8.0 } else { 1.0 };
                let moved = (lines * step).round();
                if moved.abs() < f32::EPSILON {
                    return;
                }
                let byte = self.byte(f32::from(self.value) + moved);
                if byte != self.value {
                    self.value = byte;
                    shell.publish((self.on_change)(byte));
                }
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.grab.is_some() {
            return mouse::Interaction::Grabbing;
        }
        if !self.is_live() || cursor.position_over(layout.bounds()).is_none() {
            return mouse::Interaction::default();
        }
        mouse::Interaction::Grab
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
        let material = materials(theme);
        let track = self.track(bounds);

        // The track: a recess cut into the panel, lit along its lower wall, so a
        // fader at the bottom of its travel still reads as a fader.
        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: (TRACK / 2.0).into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(material.recess),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: track.x + 1.0,
                    y: track.y + track.height - 3.0,
                    width: track.width - 2.0,
                    height: 2.0,
                },
                border: Border::default().rounded(1.0),
                ..renderer::Quad::default()
            },
            Background::Color(material.lit),
        );

        // The scale, in pairs either side of the track.
        for fraction in TICKS {
            for arm in self.tick(bounds, fraction) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: arm,
                        ..renderer::Quad::default()
                    },
                    Background::Color(material.scale),
                );
            }
        }

        // No cap for a value nobody has read: there is nothing to draw one at.
        if !self.is_live() {
            return;
        }

        let cap = self.cap(bounds);
        let confirmed = self.claim.is_confirmed();
        renderer.fill_quad(
            renderer::Quad {
                bounds: cap,
                border: Border {
                    // Filled metal for what the synthesizer reported, the metal
                    // as a stroke for what this window claims. The fill is what
                    // carries it; the colour only agrees with the fill.
                    color: if confirmed {
                        material.metal_low
                    } else {
                        tint(theme, self.claim)
                    },
                    width: if confirmed { 1.0 } else { 1.5 },
                    radius: 2.5.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(if confirmed {
                material.metal
            } else {
                material.panel
            }),
        );
        // The line a cap's value is read against.
        renderer.fill_quad(
            renderer::Quad {
                bounds: self.indicator(cap),
                border: Border::default().rounded(1.0),
                ..renderer::Quad::default()
            },
            Background::Color(if confirmed {
                material.panel
            } else {
                tint(theme, self.claim)
            }),
        );
    }
}

impl<'a, Message, Renderer> From<Fader<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(fader: Fader<'a, Message>) -> Self {
        Self::new(fader)
    }
}

#[cfg(test)]
mod tests {
    use super::{Axis, CAP_HEIGHT, Fader, HEIGHT, WIDTH, fader};
    use crate::Confidence;
    use iced_core::{Point, Rectangle};

    /// A fader over a whole byte, sitting at `value`.
    fn at(value: u8) -> Fader<'static, ()> {
        fader(u8::MIN..=u8::MAX, value, Confidence::Confirmed, |_| ())
    }

    /// The bounds a fader running down the panel is laid out in.
    const DOWN: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: WIDTH,
        height: HEIGHT,
    };

    /// The bounds the same fader is laid out in on its side.
    const ACROSS: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: HEIGHT,
        height: WIDTH,
    };

    #[test]
    fn both_ends_of_the_range_put_the_cap_flush_with_the_track() {
        // A control whose extremes are unreachable is a bug people file, and
        // the arithmetic that reaches them is the same either way round.
        let bottom = at(u8::MIN).cap(DOWN);
        assert!((bottom.y + bottom.height - DOWN.height).abs() < f32::EPSILON);
        assert!((at(u8::MAX).cap(DOWN).y - DOWN.y).abs() < f32::EPSILON);

        let left = at(u8::MIN).across(HEIGHT).cap(ACROSS);
        assert!((left.x - ACROSS.x).abs() < f32::EPSILON);
        let right = at(u8::MAX).across(HEIGHT).cap(ACROSS);
        assert!((right.x + right.width - ACROSS.width).abs() < f32::EPSILON);
    }

    #[test]
    fn the_cap_rides_the_travel_and_never_the_whole_length() {
        let travel = at(0).travel(DOWN);

        assert!((travel - (HEIGHT - CAP_HEIGHT)).abs() < f32::EPSILON);
        assert!((at(0).across(HEIGHT).travel(ACROSS) - travel).abs() < f32::EPSILON);
    }

    #[test]
    fn up_is_more_down_the_panel_and_right_is_more_across_it() {
        let down = at(128);
        let across = at(128).across(HEIGHT);

        // The pointer goes up: from a larger y to a smaller one.
        assert!(down.advance(100.0, 40.0, DOWN) > 0.0);
        assert!(down.advance(40.0, 100.0, DOWN) < 0.0);
        // And to the right: from a smaller x to a larger one.
        assert!(across.advance(40.0, 100.0, ACROSS) > 0.0);
        assert!(across.advance(100.0, 40.0, ACROSS) < 0.0);
    }

    #[test]
    fn a_drag_the_length_of_the_travel_covers_the_whole_range() {
        let down = at(0);
        let travel = down.travel(DOWN);

        let covered = down.advance(travel, 0.0, DOWN);
        assert!((covered - down.span()).abs() < 0.01, "covered {covered}");

        let across = at(0).across(HEIGHT);
        let covered = across.advance(0.0, travel, ACROSS);
        assert!((covered - across.span()).abs() < 0.01, "covered {covered}");
    }

    #[test]
    fn a_pointer_is_read_along_the_axis_the_cap_travels() {
        let position = Point::new(30.0, 90.0);

        assert!((at(0).along(position) - position.y).abs() < f32::EPSILON);
        assert!((at(0).across(HEIGHT).along(position) - position.x).abs() < f32::EPSILON);
    }

    #[test]
    fn a_fader_runs_down_the_panel_until_it_is_asked_not_to() {
        assert_eq!(at(0).axis, Axis::Down);
        assert_eq!(at(0).across(HEIGHT).axis, Axis::Across);
    }
}

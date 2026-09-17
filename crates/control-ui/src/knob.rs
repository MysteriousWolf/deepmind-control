//! The knob, drawn rather than borrowed.
//!
//! A `DeepMind` has no knobs on it. Its effects have: the manual prints a
//! figure of a rack unit beside each of the 35 algorithms, and 29 of those
//! figures are rotary knobs on a dark chassis. `deepmind-midi` 26.3 publishes
//! which shape each algorithm's panel uses, which is
//! [deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22),
//! so this is the shape that was missing.
//!
//! # It is the fader, turned
//!
//! Everything about the control is the fader's, because the two are the same
//! control and the arrangement is what differs: the same relative grab, so a
//! press never jumps; the same shift for a fine drag and the same wheel; the
//! same claim in the fill of the thing that moves — filled metal for what the
//! synthesizer reported, the metal as a stroke for what this window claims, and
//! no knob at all for a value nobody has read, because the host crate refuses
//! an edit before anything is known.
//!
//! A drag is up and down whatever the shape is. Turning a knob by dragging
//! round it is a gesture nobody performs twice, and every plug-in that has ever
//! tried it has quietly gone back to this one.
//!
//! # A sweep is 270 degrees, and it is not a measurement
//!
//! The pointer leaves the bottom left of the body at nothing and arrives at the
//! bottom right at the top of the range, which is where a hardware knob's own
//! ends are and what the manual's figures draw. The instrument publishes no
//! angle for a parameter, so the sweep is the shape of the control and never a
//! reading: what the byte is worth is the reading under it, the same as under
//! every fader in this window.

use core::f32::consts::PI;
use core::ops::RangeInclusive;

use iced_core::layout::{self, Layout};
use iced_core::widget::{Tree, tree};
use iced_core::{
    Background, Border, Clipboard, Element, Event, Length, Point, Rectangle, Shell, Size, Theme,
    Widget, keyboard, mouse, renderer, touch,
};

use crate::Confidence;
use crate::style::{materials, tint};

/// How wide across a knob is drawn, and how tall it stands.
///
/// A slot's own width less the room a name needs either side of it, which is
/// the same room the fader in the slot beside it is given.
pub const SIZE: f32 = 44.0;

/// How far a drag runs to cover the whole range.
///
/// A rack fader's travel, so that a knob and a fader move at the same rate
/// under the same hand. A knob whose range was the height of its own body
/// would be a control that went from nothing to everything in two
/// centimetres.
const SWEEP: f32 = 128.0;

/// How much of the range a fine drag covers: an eighth.
const FINE: f32 = 0.125;

/// How far round the body the pointer travels, in radians.
///
/// Three quarters of a turn, centred on the top: the ends are at the bottom
/// left and the bottom right, which is where a hardware knob stops.
const TURN: f32 = PI * 1.5;

/// Where the scale is ticked, as a fraction of the turn.
const TICKS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// How far a tick stands off the body.
const TICK: f32 = 3.0;

/// A knob over one parameter's range.
#[expect(
    missing_debug_implementations,
    reason = "the callback is a closure, and a widget is not inspected"
)]
pub struct Knob<'a, Message> {
    value: u8,
    range: RangeInclusive<u8>,
    claim: Confidence,
    size: f32,
    live: bool,
    on_change: Box<dyn Fn(u8) -> Message + 'a>,
}

/// Builds a knob over `range`, sitting at `value`.
///
/// `claim` is what backs the value, and it decides whether there is a body to
/// take hold of at all, exactly as it does for a fader.
pub fn knob<'a, Message>(
    range: RangeInclusive<u8>,
    value: u8,
    claim: Confidence,
    on_change: impl Fn(u8) -> Message + 'a,
) -> Knob<'a, Message> {
    Knob {
        value,
        range,
        claim,
        size: SIZE,
        live: true,
        on_change: Box::new(on_change),
    }
}

impl<Message> Knob<'_, Message> {
    /// Draws the knob `size` points across.
    ///
    /// For a panel laying a row of them out in the room it has, which is the
    /// only thing hand layout is allowed to change about one.
    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(12.0);
        self
    }

    /// Says whether this one can be taken hold of at all.
    ///
    /// The claim already answers that for a value nobody has read, and this is
    /// the other reason: the modulation matrix pointed at the window makes a
    /// control the matrix cannot reach something to look past rather than
    /// something to move. The drawing is untouched — the cap is where the value
    /// is, in the colour the claim is worth — and what goes is the grab.
    #[must_use]
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
}

/// A drag in progress.
///
/// The fader's, for the reason the fader carries one: the value is moved by
/// each pointer step rather than measured from where the press landed, so that
/// taking hold of shift half way through bends the rate without moving the
/// value.
#[derive(Debug, Clone, Copy)]
struct Grab {
    value: f32,
    /// Where the pointer was down the panel.
    last: f32,
}

/// What a knob remembers between events.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    grab: Option<Grab>,
    fine: bool,
}

impl<Message> Knob<'_, Message> {
    /// Returns the range as a span of values, never zero.
    fn span(&self) -> f32 {
        let low = f32::from(*self.range.start());
        let high = f32::from(*self.range.end());
        (high - low).max(1.0)
    }

    /// Returns how far round the turn the pointer sits, as a fraction.
    fn fraction(&self) -> f32 {
        let low = f32::from(*self.range.start());
        ((f32::from(self.value) - low) / self.span()).clamp(0.0, 1.0)
    }

    /// Returns how much of the range a drag from `from` to `to` covered.
    ///
    /// Up is more, which is the fader's own rule and the one a hand expects
    /// from anything it takes hold of on a panel.
    fn advance(&self, from: f32, to: f32) -> f32 {
        (from - to) / SWEEP * self.span()
    }

    /// Returns the body of the knob inside `bounds`.
    fn body(&self, bounds: Rectangle) -> Rectangle {
        let across = self.size.min(bounds.width).min(bounds.height);
        Rectangle {
            x: bounds.x + (bounds.width - across) / 2.0,
            y: bounds.y + (bounds.height - across) / 2.0,
            width: across,
            height: across,
        }
    }

    /// Returns where a fraction of the turn falls, `reach` out from the centre.
    ///
    /// Measured from the bottom left round the top, which is the way a knob
    /// turns and the opposite of the way an angle is usually written down.
    fn at(body: Rectangle, fraction: f32, reach: f32) -> Point {
        let turned = -TURN / 2.0 + fraction.clamp(0.0, 1.0) * TURN;
        Point::new(
            body.center_x() + turned.sin() * reach,
            body.center_y() - turned.cos() * reach,
        )
    }

    /// Returns whether there is anything here to take hold of.
    fn is_live(&self) -> bool {
        self.live && !matches!(self.claim, Confidence::Unknown)
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

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Knob<'_, Message>
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
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(self.size), Length::Fixed(self.size))
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
                    // Take hold of it where it is. Nothing jumps.
                    state.grab = Some(Grab {
                        value: f32::from(self.value),
                        last: position.y,
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
                let value = grab.value + self.advance(grab.last, position.y) * rate;
                let byte = self.byte(value);
                state.grab = Some(Grab {
                    value: value
                        .clamp(f32::from(*self.range.start()), f32::from(*self.range.end())),
                    last: position.y,
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
        let body = self.body(bounds);

        // The recess the knob is seated in, which is the track's answer to the
        // same question: a control has to be cut into the panel, or it is
        // floating on it.
        let seat = Rectangle {
            x: body.x - 2.0,
            y: body.y - 2.0,
            width: body.width + 4.0,
            height: body.height + 4.0,
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: seat,
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: (seat.width / 2.0).into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(material.recess),
        );

        // The scale, round the outside of the body, where a hardware panel
        // prints it.
        for fraction in TICKS {
            let at = Self::at(body, fraction, body.width / 2.0 + TICK + 1.0);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: at.x - 1.0,
                        y: at.y - 1.0,
                        width: 2.0,
                        height: 2.0,
                    },
                    border: Border::default().rounded(1.0),
                    ..renderer::Quad::default()
                },
                Background::Color(material.scale),
            );
        }

        // No body for a value nobody has read: there is nothing to turn it to.
        if !self.is_live() {
            return;
        }

        let confirmed = self.claim.is_confirmed();
        renderer.fill_quad(
            renderer::Quad {
                bounds: body,
                border: Border {
                    // Filled metal for what the synthesizer reported, the metal
                    // as a stroke for what this window claims. The fader's rule,
                    // and the fill is what carries it.
                    color: if confirmed {
                        material.metal_low
                    } else {
                        tint(theme, self.claim)
                    },
                    width: if confirmed { 1.0 } else { 1.5 },
                    radius: (body.width / 2.0).into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(if confirmed {
                material.metal
            } else {
                material.panel
            }),
        );

        // The pointer, from the middle of the body out to its rim. Laid down as
        // a run of marks rather than as one turned rectangle: a quad has no
        // angle, and a run of them is a line at this size.
        let ink = if confirmed {
            material.panel
        } else {
            tint(theme, self.claim)
        };
        let reach = body.width / 2.0;
        let mut step = reach * 0.25;
        while step <= reach - 3.0 {
            let at = Self::at(body, self.fraction(), step);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: at.x - 1.5,
                        y: at.y - 1.5,
                        width: 3.0,
                        height: 3.0,
                    },
                    border: Border::default().rounded(1.5),
                    ..renderer::Quad::default()
                },
                Background::Color(ink),
            );
            step += 1.5;
        }
    }
}

impl<'a, Message, Renderer> From<Knob<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(knob: Knob<'a, Message>) -> Self {
        Self::new(knob)
    }
}

#[cfg(test)]
mod tests {
    use super::{Knob, SIZE, TURN, knob};
    use crate::Confidence;
    use iced_core::{Point, Rectangle};

    /// A knob over the whole of a byte, sitting at `value`.
    fn over(value: u8) -> Knob<'static, ()> {
        knob(0..=255, value, Confidence::Confirmed, |_| ())
    }

    /// The body a knob of the written size draws.
    fn body() -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: SIZE,
            height: SIZE,
        }
    }

    #[test]
    fn the_ends_of_the_turn_are_the_bottom_corners() {
        // Where a hardware knob stops, and what makes the middle of the range
        // point straight up: a sweep that ran the whole way round would have
        // nothing to say at the top and two answers at the bottom.
        let body = body();
        let low = Knob::<()>::at(body, 0.0, 10.0);
        let high = Knob::<()>::at(body, 1.0, 10.0);
        let middle = Knob::<()>::at(body, 0.5, 10.0);

        assert!(low.x < body.center_x(), "nothing is not on the left");
        assert!(low.y > body.center_y(), "nothing is not at the bottom");
        assert!(
            high.x > body.center_x(),
            "the top of the range is not right"
        );
        assert!(high.y > body.center_y(), "the top of the range is not down");
        assert!(
            (middle.x - body.center_x()).abs() < 0.01 && middle.y < body.center_y(),
            "the middle of the range is not straight up"
        );
    }

    #[test]
    fn the_turn_is_three_quarters_of_a_circle() {
        // Not a measurement of anything the instrument publishes: it is the
        // shape of the control, and this is what says so out loud.
        assert!(
            (TURN - core::f32::consts::PI * 1.5).abs() < f32::EPSILON,
            "a knob that turns some other amount is a knob nobody has seen"
        );
    }

    #[test]
    fn a_drag_up_is_more_and_the_rate_is_the_faders() {
        // The whole range in one travel of a rack fader, so that a hand moving
        // between the two controls of one panel does not have to learn a second
        // rate.
        let knob = over(0);
        let moved = knob.advance(200.0, 200.0 - super::SWEEP);

        assert!((moved - 255.0).abs() < 0.01, "a whole travel moved {moved}");
        assert!(
            knob.advance(200.0, 210.0) < 0.0,
            "dragging down is not less"
        );
    }

    #[test]
    fn a_press_takes_hold_where_the_pointer_landed_and_nothing_jumps() {
        // The fader's rule, and the reason for it: a stray click that threw a
        // value across its range would be audible, sent, and not undone by
        // letting go.
        let knob = over(100);
        let landed = Point::new(20.0, 30.0);
        let value = f32::from(knob.value) + knob.advance(landed.y, landed.y);

        assert!((value - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_value_nobody_has_read_has_nothing_to_take_hold_of() {
        assert!(!knob(0..=255, 0, Confidence::Unknown, |_| ()).is_live());
        assert!(knob(0..=255, 0, Confidence::Assumed, |_| ()).is_live());
    }

    #[test]
    fn the_body_stays_round_in_a_room_that_is_not_square() {
        // A knob drawn as an ellipse is a knob drawn by somebody who let a
        // layout decide what a control is.
        let knob = over(0);
        let body = knob.body(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 90.0,
            height: SIZE,
        });

        assert!((body.width - body.height).abs() < f32::EPSILON);
        assert!((body.center_x() - 45.0).abs() < f32::EPSILON, "not centred");
    }
}

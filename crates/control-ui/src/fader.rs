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
    Background, Border, Clipboard, Element, Event, Length, Rectangle, Shell, Size, Theme, Widget,
    keyboard, mouse, renderer, touch,
};

use crate::Confidence;
use crate::style::{materials, tint};

/// Width of a fader, including the room its scale needs.
pub const WIDTH: f32 = 44.0;

/// Height of a fader, which is the travel plus the cap.
pub const HEIGHT: f32 = 128.0;

/// Width of the track cut into the panel.
const TRACK: f32 = 7.0;

/// Width of the cap.
const CAP_WIDTH: f32 = 38.0;

/// Height of the cap. Travel is the track less this, so the ends of the range
/// put the cap flush with the ends of the track.
const CAP_HEIGHT: f32 = 15.0;

/// Length of one arm of a scale tick.
const TICK: f32 = 8.0;

/// Where the scale is ticked, as a fraction of the travel.
const TICKS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// How much of the range a fine drag covers: an eighth.
const FINE: f32 = 0.125;

/// A vertical fader over one parameter's range.
#[expect(
    missing_debug_implementations,
    reason = "the callback is a closure, and a widget is not inspected"
)]
pub struct Fader<'a, Message> {
    value: u8,
    range: RangeInclusive<u8>,
    claim: Confidence,
    on_change: Box<dyn Fn(u8) -> Message + 'a>,
}

/// Builds a fader over `range`, sitting at `value`.
///
/// `claim` is what backs the value, and it decides whether there is a cap to
/// take hold of at all.
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
        on_change: Box::new(on_change),
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
    last_y: f32,
}

/// What a fader remembers between events.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    grab: Option<Grab>,
    fine: bool,
}

impl<Message> Fader<'_, Message> {
    /// Returns how far the cap can travel.
    fn travel(bounds: Rectangle) -> f32 {
        (bounds.height - CAP_HEIGHT).max(1.0)
    }

    /// Returns the range as a span of values, never zero.
    fn span(&self) -> f32 {
        let low = f32::from(*self.range.start());
        let high = f32::from(*self.range.end());
        (high - low).max(1.0)
    }

    /// Returns where the top of the cap sits for `value`.
    fn cap_top(&self, bounds: Rectangle) -> f32 {
        let low = f32::from(*self.range.start());
        let fraction = (f32::from(self.value) - low) / self.span();
        bounds.y + (1.0 - fraction.clamp(0.0, 1.0)) * Self::travel(bounds)
    }

    /// Returns whether there is anything here to take hold of.
    fn is_live(&self) -> bool {
        !matches!(self.claim, Confidence::Unknown)
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
        Size::new(Length::Fixed(WIDTH), Length::Fixed(HEIGHT))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(WIDTH), Length::Fixed(HEIGHT))
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
                        last_y: position.y,
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
                let Some(grab) = state.grab.as_mut() else {
                    return;
                };
                let rate = if state.fine { FINE } else { 1.0 };
                let moved = (grab.last_y - position.y) / Self::travel(bounds) * self.span() * rate;
                grab.last_y = position.y;
                grab.value = (grab.value + moved)
                    .clamp(f32::from(*self.range.start()), f32::from(*self.range.end()));

                // The float is what a drag accumulates; the parameter is a byte.
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "clamped to the range, which is a u8 either end"
                )]
                let value = grab.value.round() as u8;
                if value != self.value {
                    self.value = value;
                    shell.publish((self.on_change)(value));
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
                let next = (f32::from(self.value) + moved)
                    .clamp(f32::from(*self.range.start()), f32::from(*self.range.end()));
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "clamped to the range, which is a u8 either end"
                )]
                let value = next as u8;
                if value != self.value {
                    self.value = value;
                    shell.publish((self.on_change)(value));
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
        let centre = bounds.x + bounds.width / 2.0;

        // The track: a recess cut into the panel, lit along its lower wall, so a
        // fader at the bottom of its travel still reads as a fader.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: centre - TRACK / 2.0,
                    y: bounds.y,
                    width: TRACK,
                    height: bounds.height,
                },
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
                    x: centre - TRACK / 2.0 + 1.0,
                    y: bounds.y + bounds.height - 3.0,
                    width: TRACK - 2.0,
                    height: 2.0,
                },
                border: Border::default().rounded(1.0),
                ..renderer::Quad::default()
            },
            Background::Color(material.lit),
        );

        // The scale, in pairs either side of the track.
        let travel = Self::travel(bounds);
        for fraction in TICKS {
            let y = bounds.y + CAP_HEIGHT / 2.0 + (1.0 - fraction) * travel;
            for x in [bounds.x + 2.0, bounds.x + bounds.width - 2.0 - TICK] {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x,
                            y,
                            width: TICK,
                            height: 1.0,
                        },
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

        let cap = Rectangle {
            x: centre - CAP_WIDTH / 2.0,
            y: self.cap_top(bounds),
            width: CAP_WIDTH,
            height: CAP_HEIGHT,
        };
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
                bounds: Rectangle {
                    x: cap.x + 6.0,
                    y: cap.y + CAP_HEIGHT / 2.0 - 1.0,
                    width: CAP_WIDTH - 12.0,
                    height: 2.0,
                },
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

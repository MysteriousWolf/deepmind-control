//! The envelopes: four parameters, and the shape they make.
//!
//! Three groups on this instrument are the same nine parameters with a
//! different destination: an attack time, a decay time, a sustain level, a
//! release time, four curves and a trigger mode. Four of those are a picture
//! everybody who has touched a synthesizer can read at a glance, and four
//! numbers that do not draw it are four numbers.
//!
//! So the rack keeps all nine slots, exactly as the specification gives them,
//! and the shape is drawn above it. Hand layout changes the arrangement and
//! never what a control is: every one of those faders is still the fader the
//! generated panel had, and moving one redraws the picture because the picture
//! is only ever read from the patch.
//!
//! # What the drawing does not claim
//!
//! The horizontal axis is proportion and not time. The manual publishes the two
//! ends of each range and almost never the curve between them, so `Attack Time`
//! at 128 is drawn half as wide as at 255 and nothing here says how many
//! milliseconds either of those is.
//!
//! The four curve parameters do not bend the segments. What a curve byte does
//! to the shape is not published either, and a drawing that guessed would be
//! wrong in a way nobody could see. They are faders in the rack like the rest,
//! and the day the library measures them is the day the segments bend.

use deepmind_midi::param::{Group, ParamId};
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Border, Color, Element, Length, Rectangle, Size, Theme, Widget, mouse, renderer,
};

use crate::style::{materials, tint};
use crate::{Confidence, Patch};

/// Height of the drawing.
const HEIGHT: f32 = 92.0;

/// Width of the drawing, which is three slots of the rack it sits over.
const WIDTH: f32 = crate::panel::SLOT * 3.0;

/// How much of the width the sustain plateau takes.
///
/// Sustain is a level and not a time: the instrument holds it for as long as a
/// key is down, which is a length this drawing cannot know. A fixed quarter of
/// the axis says "and then it holds here" without pretending to time it.
const PLATEAU: f32 = 0.25;

/// The four parameters that make the shape.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Envelope {
    attack: ParamId,
    decay: ParamId,
    sustain: ParamId,
    release: ParamId,
}

impl Envelope {
    /// The four, in the order the shape reads them.
    pub(crate) fn parameters(self) -> [ParamId; 4] {
        [self.attack, self.decay, self.sustain, self.release]
    }
}

/// Returns the four parameters of `group`, when it is an envelope.
///
/// Found by what the library calls them rather than by offset, so the three
/// envelopes are one piece of code and a library that renumbers them moves this
/// with it.
pub(crate) fn of(group: Group) -> Option<Envelope> {
    let find = |suffix: &str| {
        group
            .parameters()
            .find(|parameter| parameter.name().ends_with(suffix))
    };
    Some(Envelope {
        attack: find("Attack Time")?,
        decay: find("Decay Time")?,
        sustain: find("Sustain Level")?,
        release: find("Release Time")?,
    })
}

/// Draws the shape `group` makes, when it is an envelope.
pub(crate) fn shape<'a, Renderer>(
    patch: &Patch,
    group: Group,
) -> Option<Element<'a, crate::Message, Theme, Renderer>>
where
    Renderer: iced_core::Renderer + 'a,
{
    let envelope = of(group)?;
    let parameters = envelope.parameters();
    // The weakest claim of the four, because one unread value is a shape nobody
    // can vouch for, and one claimed value makes the whole outline a claim. It
    // is the rule a name is drawn under, and the same call makes it.
    let claim = patch.claim_across(parameters);
    let values = parameters.map(|parameter| patch.value(parameter));
    let [attack, decay, sustain, release] = values;
    let known = match (attack, decay, sustain, release) {
        (Some(attack), Some(decay), Some(sustain), Some(release)) => {
            Some(Corners::new(attack, decay, sustain, release))
        }
        _ => None,
    };
    Some(Element::new(Drawing {
        corners: known,
        claim,
    }))
}

/// Returns the shape `group` makes, when it is an envelope all four of whose
/// values have been read.
///
/// What the drawing above the rack is read from, and what a display drawing
/// three envelopes at once on one screen is read from as well: the shape is
/// the four bytes and nothing else, so there is one place that works out what
/// they draw and two things that draw it.
pub(crate) fn corners(patch: &Patch, group: Group) -> Option<Corners> {
    let envelope = of(group)?;
    let [attack, decay, sustain, release] = envelope.parameters();
    Some(Corners::new(
        patch.value(attack)?,
        patch.value(decay)?,
        patch.value(sustain)?,
        patch.value(release)?,
    ))
}

/// The shape, as the four points a line through it turns at.
///
/// Widths are fractions of the drawing and heights are fractions of its height,
/// so the same numbers draw at any size the layout gives.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Corners {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl Corners {
    /// Works the four raw bytes into the fractions the drawing uses.
    fn new(attack: u8, decay: u8, sustain: u8, release: u8) -> Self {
        let times = f32::from(attack) + f32::from(decay) + f32::from(release);
        // Three instant segments are a spike, and a spike is what gets drawn:
        // the plateau, and no width either side of it.
        let scale = if times > 0.0 {
            (1.0 - PLATEAU) / times
        } else {
            0.0
        };
        Self {
            attack: f32::from(attack) * scale,
            decay: f32::from(decay) * scale,
            sustain: f32::from(sustain) / f32::from(u8::MAX),
            release: f32::from(release) * scale,
        }
    }

    /// Returns the height of the shape at `x`, both as fractions.
    ///
    /// It starts at nothing and ends at nothing, whatever the four values are.
    /// A release of nothing is the case that needs saying: its plateau runs to
    /// the right edge, and the drop has only the end of the axis to happen in.
    /// Drawn as a plateau that reaches the edge and stops, a gate would read as
    /// a sound that never ends.
    pub(crate) fn height_at(self, x: f32) -> f32 {
        let decay_ends = self.attack + self.decay;
        let plateau_ends = decay_ends + PLATEAU;
        if x <= self.attack {
            // Rising to the peak. An instant attack is already there.
            if self.attack <= 0.0 {
                1.0
            } else {
                x / self.attack
            }
        } else if x <= decay_ends {
            if self.decay <= 0.0 {
                self.sustain
            } else {
                let through = (x - self.attack) / self.decay;
                1.0 - through * (1.0 - self.sustain)
            }
        } else if x < plateau_ends {
            self.sustain
        } else if self.release <= 0.0 {
            0.0
        } else {
            let through = (x - plateau_ends) / self.release;
            self.sustain * (1.0 - through.min(1.0))
        }
    }
}

/// The drawing itself: a baseline, and the shape over it.
struct Drawing {
    corners: Option<Corners>,
    claim: Confidence,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Drawing
where
    Renderer: iced_core::Renderer,
{
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

        // The ground the shape stands on, cut into the plate like a track is.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 3.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(material.recess),
        );

        let Some(corners) = self.corners else {
            // Nothing has been read: the frame, and no shape in it, which is
            // the same answer a fader with no cap gives.
            return;
        };
        let filled = self.claim.is_confirmed();
        let line = if filled {
            material.metal
        } else {
            tint(theme, self.claim)
        };
        // The body of the shape is the line's own colour, dimmed, so that it
        // reads against the recess it is cut into. The plate would be the
        // colour of everything around the drawing, which is no fill at all.
        let body = Color { a: 0.22, ..line };

        let inset = 6.0;
        let width = (bounds.width - inset * 2.0).max(1.0);
        let height = (bounds.height - inset * 2.0).max(1.0);
        let base = bounds.y + bounds.height - inset;

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a column count from a width that layout has already bounded"
        )]
        let columns = width as u16;
        // The height the column before this one ended at, so that the line can
        // be drawn between the two rather than as a mark at each.
        let mut previous: Option<f32> = None;
        // One sample past the columns, taken at the end of the axis itself
        // rather than at the last whole pixel before it: an instant release has
        // nowhere else to say that the sound stopped.
        for column in 0..=columns {
            let offset = if column == columns {
                width
            } else {
                f32::from(column)
            };
            let top = base - corners.height_at(offset / width) * height;
            let x = bounds.x + inset + offset;
            if filled {
                // Filled under the line for a shape the synthesizer described,
                // the line alone for one this window is claiming.
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x,
                            y: top,
                            width: 1.0,
                            height: base - top,
                        },
                        ..renderer::Quad::default()
                    },
                    Background::Color(body),
                );
            }
            // Drawn from where the column before it ended, so that a segment
            // steeper than the line is thick is a line rather than a row of
            // marks with the shape missing between them. A decay of nothing
            // falls the whole height of the drawing between two columns, and
            // with nothing filled under it — which is how a value this window
            // is claiming is drawn — the fall was not drawn at all.
            let last = previous.unwrap_or(top);
            let from = last.min(top) - 1.0;
            let to = last.max(top) + 1.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: from,
                        width: 1.0,
                        height: to - from,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(line),
            );
            previous = Some(top);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{Corners, PLATEAU, of};
    use crate::{Confidence, Patch};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::Group;
    use deepmind_midi::program::Program;

    #[test]
    fn the_three_envelopes_are_found_and_nothing_else_is() {
        // Asked of every group the instrument has, rather than of a handful:
        // the three are found by what the library calls their parameters, and
        // a fourth group that grew an `Attack Time` would be found too.
        let found: Vec<Group> = Group::ALL
            .iter()
            .copied()
            .filter(|group| of(*group).is_some())
            .collect();

        assert_eq!(found.len(), 3, "found {found:?}");
        for group in [Group::VcaEnvelope, Group::VcfEnvelope, Group::ModEnvelope] {
            assert!(found.contains(&group), "{group:?} is an envelope");
        }
    }

    #[test]
    fn the_shape_starts_at_nothing_and_ends_at_nothing() {
        let corners = Corners::new(128, 128, 128, 128);
        assert!(corners.height_at(0.0).abs() < f32::EPSILON);
        assert!(corners.height_at(1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_peak_is_the_top_and_the_plateau_is_the_sustain() {
        let corners = Corners::new(100, 100, 200, 100);
        assert!((corners.height_at(corners.attack) - 1.0).abs() < 0.01);
        let plateau = corners.attack + corners.decay + PLATEAU / 2.0;
        assert!((corners.height_at(plateau) - corners.sustain).abs() < 0.01);
    }

    #[test]
    fn an_instant_envelope_is_a_plateau_and_not_a_panic() {
        let corners = Corners::new(0, 0, 255, 0);
        assert!((corners.height_at(0.0) - 1.0).abs() < 0.01);
        assert!((corners.height_at(PLATEAU / 2.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn a_release_of_nothing_still_ends_at_nothing() {
        // A gate: up, held, and gone the instant the key is. The drop has only
        // the end of the axis to happen in, and a plateau that reaches the
        // right edge and stops would read as a sound that never ends.
        let corners = Corners::new(255, 0, 255, 0);

        assert!((corners.height_at(0.99) - 1.0).abs() < 0.01);
        assert!(corners.height_at(1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn one_unread_value_makes_the_whole_shape_unread() {
        let envelope = of(Group::VcaEnvelope).expect("an envelope");
        let mut patch = Patch::new();

        // A sound nobody has read is a shape nobody can vouch for.
        assert_eq!(
            patch.claim_across(envelope.parameters()),
            Confidence::Unknown
        );

        patch.confirm(Program::new(ProtocolVersion::V7));
        assert_eq!(
            patch.claim_across(envelope.parameters()),
            Confidence::Confirmed
        );

        // One of the four moved in this window, and the whole outline is a
        // claim: three values the synthesizer described do not vouch for a
        // shape drawn through a fourth it has not.
        assert!(patch.edit(envelope.decay, 90));
        assert_eq!(
            patch.claim_across(envelope.parameters()),
            Confidence::Assumed
        );
    }
}

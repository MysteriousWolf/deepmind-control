//! The envelopes: eight parameters, and the shape they make.
//!
//! Three groups on this instrument are the same nine parameters with a
//! different destination: an attack time, a decay time, a sustain level, a
//! release time, four curves and a trigger mode. Eight of those are a picture
//! everybody who has touched a synthesizer can read at a glance, and eight
//! numbers that do not draw it are eight numbers.
//!
//! So the rack keeps all nine slots, exactly as the specification gives them,
//! and the shape is drawn above it. Hand layout changes the arrangement and
//! never what a control is: every one of those faders is still the fader the
//! generated panel had, and moving one redraws the picture because the picture
//! is only ever read from the patch.
//!
//! # The shape is the library's, not this window's
//!
//! [`generator::envelope`] is what draws it, and this file holds none of the
//! arithmetic it used to. Where the segments share the width, what a curve byte
//! bends, which way a sustain slope sags: all of it is a fact about the
//! instrument, published in 26.4
//! ([deepmind-midi#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32)),
//! and a window is the wrong place to keep any of it right. What this file does
//! is find the eight parameters in a group, ask which envelope that group is,
//! and sample the answer into the room the layout gave it.
//!
//! # What the drawing does not claim
//!
//! The horizontal axis is proportion and not time. It is
//! [`Scale::Normalised`](deepmind_midi::generator::Scale::Normalised), which is
//! the library saying outright that the axis is an ordering: the manual
//! publishes the two ends of each range and never the curve between them, so
//! `Attack Time` at 128 is drawn half as wide as at 255 and nothing here says
//! how many milliseconds either of those is.
//!
//! The four curves do bend the segments now, which is the one thing that
//! changed. The law is the library's and it is marked there as a reading of the
//! five responses the manual prints rather than as an equation the manual
//! gives.

use deepmind_midi::generator::{self, EnvelopeId, Generator};
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

/// What the eight parameters of an envelope are called, in the order the shape
/// reads them.
///
/// The four the library's own `EnvelopeId` reads first and then the four
/// curves, which is the order it documents them in. Matched against the end of
/// a parameter's name, the way this file has always found them: a library that
/// renamed one fails to find an envelope here rather than drawing three
/// quarters of one.
const NAMED: [&str; 8] = [
    "Attack Time",
    "Decay Time",
    "Sustain Level",
    "Release Time",
    "Attack Curve",
    "Decay Curve",
    "Sustain Curve",
    "Release Curve",
];

/// The eight parameters that make the shape, and which envelope they are.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Envelope {
    /// Which of the three, as the library names them.
    which: EnvelopeId,
    /// The eight, in [`NAMED`] order.
    parameters: [ParamId; 8],
}

impl Envelope {
    /// The eight, in the order the shape reads them.
    pub(crate) fn parameters(self) -> [ParamId; 8] {
        self.parameters
    }

    /// Returns the shape these eight make, out of a sound that has been read.
    pub(crate) fn shape(self, patch: &Patch) -> Option<Generator> {
        Some(generator::envelope(patch.program()?, self.which))
    }
}

/// Returns the group one of the library's three envelopes lives in.
///
/// The one bridge between the two, and it is a match over the library's own
/// enum rather than over the parameter table: a fourth envelope is a fourth
/// variant, and a fourth variant fails to compile here instead of quietly
/// drawing one of the three.
const fn group_of(which: EnvelopeId) -> Group {
    match which {
        EnvelopeId::Vca => Group::VcaEnvelope,
        EnvelopeId::Vcf => Group::VcfEnvelope,
        EnvelopeId::Mod => Group::ModEnvelope,
    }
}

/// Returns the eight parameters of `group`, when it is an envelope.
///
/// Found by what the library calls them rather than by offset, so the three
/// envelopes are one piece of code and a library that renumbers them moves this
/// with it. A group whose eight are all there and which is not one of the
/// library's three is not an envelope this window can draw, because there would
/// be no generator to ask for its shape.
pub(crate) fn of(group: Group) -> Option<Envelope> {
    let which = EnvelopeId::ALL
        .into_iter()
        .find(|which| group_of(*which) == group)?;
    let mut parameters = [ParamId::VcaEnvelopeAttackTime; 8];
    for (slot, suffix) in parameters.iter_mut().zip(NAMED) {
        *slot = group
            .parameters()
            .find(|parameter| parameter.name().ends_with(suffix))?;
    }
    Some(Envelope { which, parameters })
}

/// Returns the shape `group` makes, when it is an envelope and its sound has
/// been read.
///
/// What the drawing above the rack is read from, and what the display on the
/// envelope's own plate is read from as well: the shape is the eight bytes and
/// nothing else, so there is one place that asks the library what they draw and
/// two things that draw it.
pub(crate) fn shape_of(patch: &Patch, group: Group) -> Option<Generator> {
    of(group)?.shape(patch)
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
    // The weakest claim of the eight, because one unread value is a shape
    // nobody can vouch for, and one claimed value makes the whole outline a
    // claim. It is the rule a name is drawn under, and the same call makes it.
    let claim = patch.claim_across(envelope.parameters());
    Some(Element::new(Drawing {
        shape: envelope.shape(patch),
        claim,
    }))
}

/// The drawing itself: a baseline, and the shape over it.
struct Drawing {
    shape: Option<Generator>,
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

        let Some(shape) = self.shape else {
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
            let top = base - shape.at(offset / width) * height;
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
            // with nothing filled under it, which is how a value this window is
            // claiming is drawn, the fall was not drawn at all.
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
    use super::{NAMED, group_of, of, shape_of};
    use crate::{Confidence, Patch};
    use deepmind_midi::generator::EnvelopeId;
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::Group;
    use deepmind_midi::program::Program;

    /// A sound the synthesizer has described.
    fn read() -> Patch {
        let mut patch = Patch::new();
        patch.confirm(Program::new(ProtocolVersion::V7));
        patch
    }

    #[test]
    fn the_three_envelopes_are_found_and_nothing_else_is() {
        // Asked of every group the instrument has, rather than of a handful:
        // the three are the three the library publishes a generator for, and a
        // group that grew an `Attack Time` without one is not an envelope this
        // window can draw.
        let found: Vec<Group> = Group::ALL
            .iter()
            .copied()
            .filter(|group| of(*group).is_some())
            .collect();

        assert_eq!(found.len(), EnvelopeId::ALL.len(), "found {found:?}");
        for which in EnvelopeId::ALL {
            assert!(found.contains(&group_of(which)), "{which:?} is an envelope");
        }
    }

    #[test]
    fn an_envelope_reads_eight_parameters_of_its_own_group() {
        for which in EnvelopeId::ALL {
            let group = group_of(which);
            let envelope = of(group).expect("an envelope");
            let parameters = envelope.parameters();

            assert_eq!(parameters.len(), NAMED.len());
            for parameter in parameters {
                assert_eq!(
                    parameter.group(),
                    group,
                    "{parameter} is drawn on another envelope's plate"
                );
            }
            // Eight distinct parameters, so a suffix that matched twice would
            // fail here rather than drawing one byte as two segments.
            let mut offsets: Vec<u8> = parameters.iter().map(|it| it.offset()).collect();
            offsets.sort_unstable();
            offsets.dedup();
            assert_eq!(offsets.len(), NAMED.len(), "{group} reads a byte twice");
        }
    }

    #[test]
    fn each_group_asks_the_library_for_its_own_envelope() {
        // The bridge between a group and the library's own name for it, held to
        // by what the answer draws rather than by inspection: an envelope held
        // at the top is a sustain of 255 at a level curve, and it is drawn on
        // the plate of the group whose bytes were moved and on no other.
        for which in EnvelopeId::ALL {
            let group = group_of(which);
            let envelope = of(group).expect("an envelope");
            let [_, _, sustain, _, _, _, curve, _] = envelope.parameters();
            let mut patch = read();
            assert!(patch.edit(sustain, 255));
            assert!(patch.edit(curve, 128));

            let held = shape_of(&patch, group).expect("a shape");
            assert!(
                (held.at(0.5) - 1.0).abs() < 0.01,
                "{group} does not draw its own sustain"
            );
            for other in EnvelopeId::ALL.map(group_of) {
                if other == group {
                    continue;
                }
                let quiet = shape_of(&patch, other).expect("a shape");
                assert!(
                    quiet.at(0.5) < 0.01,
                    "{other} draws the bytes {group} holds"
                );
            }
        }
    }

    #[test]
    fn nothing_is_drawn_from_a_sound_nobody_has_read() {
        assert!(shape_of(&Patch::new(), Group::VcaEnvelope).is_none());
        assert!(shape_of(&read(), Group::VcaEnvelope).is_some());
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

        // One of the eight moved in this window, and the whole outline is a
        // claim: seven values the synthesizer described do not vouch for a
        // shape drawn through an eighth it has not. A curve is one of the
        // eight now, which is what changed when the segments started bending.
        let [_, _, _, _, _, curve, _, _] = envelope.parameters();
        assert!(patch.edit(curve, 90));
        assert_eq!(
            patch.claim_across(envelope.parameters()),
            Confidence::Assumed
        );
    }
}

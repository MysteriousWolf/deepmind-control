//! The control sequencer: thirty-two steps, which is a sequencer and not a rack.
//!
//! Thirty-eight parameters, and thirty-two of them are one control. A rack
//! wraps `Seq Step Value 1` through `32` into four lines of slots, each 88
//! points wide under a three-line name that is the same six words thirty-two
//! times, and a sequence read four lines at a time is not a sequence anybody
//! can read. They are one strip here, in the order the instrument plays them,
//! with the step's number under it and nothing else.
//!
//! The six settings that are not steps — enable, clock divider, length, swing,
//! sync and slew — stay slots in the rack beneath, because that is what they
//! are.
//!
//! # A step is the same control it was
//!
//! Every one of the thirty-two is the fader the generated panel gave it, drawn
//! [narrow](crate::fader::Fader::narrow) because thirty-two at a slot's width
//! is eight feet of window. It is dragged the same way, it carries its claim
//! the same way, and it sends the same NRPN. Hand layout changes the
//! arrangement and never what a control is.
//!
//! # What the strip does not say yet
//!
//! Two things the manual records and the library does not yet publish, so
//! nothing here acts on them:
//!
//! - A step is **bipolar**: the specification's note on offset 123 reads
//!   `-127 (1) to +127 (255)`, and `0` means *skip this step*. The strip draws
//!   each byte from the bottom, which is what the library's `min` and `max`
//!   describe, so a step at the centre of its range draws half height rather
//!   than at a centre line, and a skipped step draws as the zero it is.
//! - `Sequence Length` is `1 (0) to 32 (31)` steps, so the steps past it are
//!   not played. The strip draws all thirty-two alike.
//!
//! Both are notes in `spec/parameters.toml` rather than anything `ParamId`
//! answers, and transcribing them here is the second copy of a generated table
//! this repository refuses to keep. The strip gains a centre line, a skip mark
//! and a dimmed tail the day the library says so.

use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Horizontal;
use iced_core::{Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{column, container, row, text};

use crate::panel::{Room, control};
use crate::{Confidence, Element, Patch, tint};

/// What the library calls a step, before its number.
const STEP: &str = "Seq Step Value ";

/// How wide one step stands.
///
/// Thirty-two of these and the gaps between them are the width of the rack
/// they sit over, which is what makes the strip read as one control rather
/// than as thirty-two.
const WIDTH: f32 = 20.0;

/// Returns the steps of `group`, in the order the instrument plays them.
///
/// Found by what the library calls them: a parameter named `Seq Step Value`
/// and a number is a step. `None` for every group with no such parameter in it,
/// which is every group but one, so a library that adds a second sequencer
/// gets a second strip and one that renames the steps gets none rather than a
/// wrong one.
pub(crate) fn steps(group: Group) -> Option<Vec<ParamId>> {
    let mut steps: Vec<ParamId> = group
        .parameters()
        .filter(|parameter| parameter.name().starts_with(STEP))
        .collect();
    if steps.is_empty() {
        return None;
    }
    // Offset order is the order they are played, and the order a dump stores
    // them. Sorting says so rather than trusting the table's own order.
    steps.sort_unstable_by_key(|parameter| parameter.offset());
    Some(steps)
}

/// Returns the parameters the strip draws, which the rack then leaves alone.
///
/// Empty for a group that is not a sequencer, so a panel can ask without
/// knowing whether it has one.
pub(crate) fn stepped(group: Group) -> Vec<ParamId> {
    steps(group).unwrap_or_default()
}

/// Draws the steps of `group` as the strip they are, when it has any.
pub(crate) fn strip<'a, Renderer>(
    patch: &Patch,
    group: Group,
    firmware: Version,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let steps = steps(group)?;
    let lanes = steps.iter().enumerate().map(|(index, parameter)| {
        let parameter = *parameter;
        let claim = patch.claim(parameter);
        column![
            control(
                parameter,
                patch.value(parameter),
                claim,
                firmware,
                Room::step(WIDTH),
            ),
            // The step's number, and not its name: `Seq Step Value 17` under
            // the seventeenth of thirty-two is the panel saying its own name
            // thirty-two times.
            text(number(index))
                .size(10)
                .font(Font::MONOSPACE)
                .style(move |theme: &Theme| text::Style {
                    color: Some(tint(theme, Confidence::Unknown)),
                }),
        ]
        .spacing(4)
        .align_x(Horizontal::Center)
        .width(Length::Fixed(WIDTH))
        .into()
    });
    Some(
        container(row(lanes).spacing(2))
            .width(Length::Shrink)
            .into(),
    )
}

/// The number printed under a step, which counts from one as the panel does.
///
/// Every fourth one, because thirty-two numbers under thirty-two twenty-point
/// lanes is a line of digits nobody reads, and a sequencer is counted in fours.
fn number(index: usize) -> String {
    let step = index + 1;
    if step == 1 || step.is_multiple_of(4) {
        step.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{number, stepped, steps};
    use deepmind_midi::param::{Group, ParamId};

    #[test]
    fn one_group_is_a_sequencer_and_the_rest_are_racks() {
        assert!(steps(Group::ControlSequencer).is_some());
        for group in Group::ALL.iter().copied() {
            if group != Group::ControlSequencer {
                assert!(steps(group).is_none(), "{group:?} has no steps");
            }
        }
    }

    #[test]
    fn the_sequencer_is_thirty_two_steps_in_the_order_it_plays_them() {
        let steps = steps(Group::ControlSequencer).expect("the sequencer is one");
        assert_eq!(steps.len(), 32);
        let offsets: Vec<u8> = steps.iter().copied().map(ParamId::offset).collect();
        let mut sorted = offsets.clone();
        sorted.sort_unstable();
        assert_eq!(
            offsets, sorted,
            "the steps are in the order they are played"
        );
        assert!(
            offsets.windows(2).all(|pair| match pair {
                [first, second] => *second == first + 1,
                _ => false,
            }),
            "and they are one run of addresses"
        );
    }

    #[test]
    fn the_settings_that_are_not_steps_stay_in_the_rack() {
        let stepped = stepped(Group::ControlSequencer);
        let rack: Vec<ParamId> = Group::ControlSequencer
            .parameters()
            .filter(|parameter| !stepped.contains(parameter))
            .collect();
        // Enable, clock divider, length, swing, sync and slew.
        assert_eq!(rack.len(), 6);
        assert!(
            rack.iter()
                .all(|parameter| !parameter.name().starts_with("Seq Step Value")),
            "nothing the strip drew is left in the rack"
        );
    }

    #[test]
    fn every_parameter_is_either_a_step_or_a_slot() {
        let stepped = stepped(Group::ControlSequencer);
        let total = Group::ControlSequencer.parameters().count();
        let rack = Group::ControlSequencer
            .parameters()
            .filter(|parameter| !stepped.contains(parameter))
            .count();
        assert_eq!(stepped.len() + rack, total, "nothing falls between the two");
    }

    #[test]
    fn a_step_is_a_sweep_however_the_table_describes_it() {
        use deepmind_midi::param::Kind;

        // Two of the thirty-two say `Kind::Switch` while accepting 256 values,
        // from a `kind = "switch"` in the library's spec that their own range
        // and their own note disagree with. Every step is a sweep on the
        // instrument, so a strip that drew two of them as lamps would offer a
        // control that cannot reach 254 of the values it accepts.
        let steps = steps(Group::ControlSequencer).expect("the sequencer is one");
        for step in steps {
            assert_eq!(step.min(), 0, "{} starts at nothing", step.name());
            assert_eq!(step.max(), 255, "{} is a byte wide", step.name());
            if step.kind() == Kind::Switch {
                assert!(
                    step.max() > 1,
                    "{} is the contradiction this guards, not a real switch",
                    step.name()
                );
            }
        }
    }

    #[test]
    fn a_sequencer_is_counted_in_fours() {
        assert_eq!(number(0), "1");
        assert_eq!(number(3), "4");
        assert_eq!(number(31), "32");
        assert_eq!(number(1), "");
        assert_eq!(number(30), "");
    }
}

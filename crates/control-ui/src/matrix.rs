//! The modulation matrix: eight routings, read across.
//!
//! Twenty-four parameters that are really eight sentences. `Mod 3 Source` is
//! `LFO 2`, `Mod 3 Destination` is `VCF Frequency` and `Mod 3 Depth` is how
//! much, and a rack reading `Mod 1 Source`, `Mod 1 Destination`, `Mod 1 Depth`,
//! `Mod 2 Source` in one wrapping line puts the three words of a sentence in
//! three different places and the next sentence between them.
//!
//! So the panel is a table: one row per routing, source to destination to
//! depth, with the parameters' own names in the column headings rather than
//! repeated eight times over. The same rule the rack follows, where a slot's
//! title is its parameter's name with the group's taken off the front.
//!
//! # What it changes, and what it does not
//!
//! The arrangement. Every control in the table is the control the generated
//! rack drew: a list where the library names every value a parameter accepts,
//! a fader where it does not, and the same claim under both. What the library
//! says a parameter is is still the only thing that decides that — a routing
//! whose depth a later library gives a value table arrives as a list here
//! without a line of this file changing.
//!
//! The depth fader is turned onto its side, which is the arrangement too: eight
//! rows cannot each be 128 points tall, and eight depths in a column of bars
//! are comparable at a glance in a way eight numbers are not.
//!
//! # Nothing is written down here either
//!
//! The eight are found by what the library calls them: a parameter whose name
//! ends in `Source` with a `Destination` and a `Depth` sharing its prefix is a
//! routing, and three parameters that do not make one stay in the rack. A
//! library that grows a ninth routing draws a ninth row; a group with a lone
//! `Source` in it — the oscillators have one — is not a matrix and is not
//! drawn as one.

use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Vertical;
use iced_core::{Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{Space, column, container, row, text};

use crate::panel::{Room, control, readout};
use crate::{Confidence, Element, Patch, tint};

/// How much room the number of a routing is given.
const LABEL: f32 = 52.0;

/// How much room a source is chosen in.
const SOURCE: f32 = 150.0;

/// How much room a destination is chosen in.
///
/// Wider than a source, because there are 133 of them and their names are the
/// long ones: `VCF Envelope Attack` has to be readable to be chosen.
const DESTINATION: f32 = 200.0;

/// How long the depth fader's travel is.
const DEPTH: f32 = 260.0;

/// How much room the arrow between a source and its destination takes.
const ARROW: f32 = 18.0;

/// One routing: where a modulation comes from, where it goes, and how much.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Routing {
    /// What the library calls this routing, which is the three parameters'
    /// shared prefix: `Mod 1`.
    label: &'static str,
    source: ParamId,
    destination: ParamId,
    depth: ParamId,
}

impl Routing {
    /// The three, in the order the row reads them.
    fn parameters(self) -> [ParamId; 3] {
        [self.source, self.destination, self.depth]
    }

    /// Where the routing lives, as the run of addresses it occupies.
    ///
    /// Once for the row rather than once above each of its three controls,
    /// which is what a slot does and what a row of three has no room to: the
    /// name field already prints its seventeen characters as the run they are.
    /// A run only where the three are one — a library that scattered them says
    /// so by printing all three.
    fn addresses(self) -> String {
        let mut offsets = self.parameters().map(ParamId::offset);
        offsets.sort_unstable();
        let [first, second, last] = offsets;
        if second == first + 1 && last == second + 1 {
            format!("{first}\u{2013}{last}")
        } else {
            format!("{first}, {second}, {last}")
        }
    }
}

/// Returns the routings `group` is made of, when it is a matrix.
///
/// Found by what the library calls them rather than by offset: a `Source` with
/// a `Destination` and a `Depth` sharing its prefix is a routing. `None` for
/// every group that has no such triple in it, which is every group but one.
pub(crate) fn of(group: Group) -> Option<Vec<Routing>> {
    let matching = |prefix: &'static str, suffix: &'static str| {
        group
            .parameters()
            .find(move |parameter| parameter.name().strip_suffix(suffix) == Some(prefix))
    };
    let routings: Vec<Routing> = group
        .parameters()
        .filter_map(|source| {
            let prefix = source.name().strip_suffix(" Source")?;
            Some(Routing {
                label: prefix,
                source,
                destination: matching(prefix, " Destination")?,
                depth: matching(prefix, " Depth")?,
            })
        })
        .collect();
    (!routings.is_empty()).then_some(routings)
}

/// Returns the parameters `group` draws as routings rather than as slots.
///
/// What the rack leaves out, so that a group the table is drawn for keeps
/// whatever else the library puts in it: a parameter no routing claimed is
/// still a slot, and nothing disappears because a panel was laid out by hand.
pub(crate) fn routed(group: Group) -> Vec<ParamId> {
    of(group)
        .into_iter()
        .flatten()
        .flat_map(Routing::parameters)
        .collect()
}

/// Draws the table `group` makes, when it is a matrix.
pub(crate) fn table<'a, Renderer>(
    patch: &Patch,
    group: Group,
    firmware: Version,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let routings = of(group)?;
    let heading = |parameter: ParamId, width: f32| -> Element<'a, Renderer> {
        // The heading is the parameter's own name with what the row already
        // says taken off the front, which is the rule a slot's title follows
        // against its group: `Mod 1 Destination` in the `Mod 1` row is
        // `Destination`.
        let name = parameter.name();
        let title = name.rsplit(' ').next().unwrap_or(name);
        text(title).size(11).width(Length::Fixed(width)).into()
    };
    let first = routings.first().copied()?;
    let header = row![
        Space::new().width(Length::Fixed(LABEL)),
        heading(first.source, SOURCE),
        Space::new().width(Length::Fixed(ARROW)),
        heading(first.destination, DESTINATION),
        heading(first.depth, DEPTH),
    ]
    .spacing(10);
    let rows = routings.into_iter().map(|routing| {
        row![
            // The routing's own name and where it lives, which the rack would
            // print above and below each of its three parameters. Against the
            // whole row rather than its bottom, because a row is one sentence
            // and its first word does not sit under the others.
            container(
                column![
                    text(routing.label).size(12),
                    text(routing.addresses())
                        .size(10)
                        .font(Font::MONOSPACE)
                        .style(move |theme: &Theme| text::Style {
                            color: Some(tint(theme, Confidence::Unknown)),
                        }),
                ]
                .spacing(2)
            )
            .width(Length::Fixed(LABEL))
            .height(Length::Fill)
            .align_y(Vertical::Center),
            cell(patch, routing.source, firmware, Room::listed(SOURCE)),
            arrow(),
            cell(
                patch,
                routing.destination,
                firmware,
                Room::listed(DESTINATION)
            ),
            cell(patch, routing.depth, firmware, Room::across(DEPTH)),
        ]
        .spacing(10)
        .align_y(Vertical::Bottom)
        .into()
    });
    Some(column![header].extend(rows).spacing(8).into())
}

/// Draws one parameter of a routing, as the row has room for it.
///
/// A slot with what the row already says taken out of it: the control and the
/// reading under it, without the title the column heading carries once and
/// without the address the row carries once.
fn cell<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
    room: Room,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        control(parameter, value, claim, firmware, room),
        readout(parameter, value, claim, firmware),
    ]
    .spacing(3)
    .width(Length::Fixed(room.width()))
    .into()
}

/// The arrow that makes a row a sentence.
fn arrow<'a, Renderer>() -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text("\u{2192}")
            .size(13)
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, Confidence::Unknown)),
            }),
    )
    .width(Length::Fixed(ARROW))
    .height(Length::Fill)
    .align_y(Vertical::Center)
    .into()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};

    use super::{of, routed};

    #[test]
    fn one_group_is_a_matrix_and_the_rest_are_racks() {
        // Asked of every group rather than of a handful: the oscillators have
        // an `OSC 2 Tone Mod Source` and the filter a `VCF LFO Select`, and a
        // lone source with nothing to route is not a routing.
        let found: Vec<Group> = Group::ALL
            .iter()
            .copied()
            .filter(|group| of(*group).is_some())
            .collect();

        assert_eq!(found, vec![Group::ModMatrix], "found {found:?}");
    }

    #[test]
    fn the_matrix_is_eight_routings_of_three_parameters() {
        let routings = of(Group::ModMatrix).expect("the matrix");

        assert_eq!(routings.len(), 8);
        for routing in &routings {
            for parameter in routing.parameters() {
                assert!(
                    parameter.name().starts_with(routing.label),
                    "{parameter} is not part of {}",
                    routing.label
                );
                assert_eq!(parameter.group(), Group::ModMatrix);
            }
        }
    }

    #[test]
    fn the_rows_are_in_the_order_the_instrument_stores_them() {
        let routings = of(Group::ModMatrix).expect("the matrix");

        let offsets: Vec<u8> = routings
            .iter()
            .flat_map(|routing| routing.parameters())
            .map(ParamId::offset)
            .collect();
        let mut sorted = offsets.clone();
        sorted.sort_unstable();

        assert_eq!(offsets, sorted, "the table is not in the table's own order");
    }

    #[test]
    fn every_parameter_of_the_matrix_is_on_a_row() {
        // What the table leaves out the rack still draws, so this is about
        // whether anything is drawn twice rather than about anything being
        // lost. Today the eight routings are the whole group.
        let on_a_row = routed(Group::ModMatrix);

        assert_eq!(on_a_row.len(), Group::ModMatrix.parameters().count());
        for parameter in Group::ModMatrix.parameters() {
            assert!(on_a_row.contains(&parameter), "{parameter} is not on a row");
        }
    }

    #[test]
    fn a_row_says_where_its_three_parameters_live() {
        let routings = of(Group::ModMatrix).expect("the matrix");
        let first = routings.first().copied().expect("a first routing");

        // The three are one run in the instrument's memory, and the row prints
        // it as one, the way the name field prints its seventeen characters.
        let run = format!("{}\u{2013}{}", first.source.offset(), first.depth.offset());
        assert_eq!(first.addresses(), run);
        assert_eq!(first.depth.offset(), first.source.offset() + 2);
    }

    #[test]
    fn a_group_that_is_not_a_matrix_leaves_the_rack_alone() {
        assert!(routed(Group::Oscillators).is_empty());
        assert!(routed(Group::Vcf).is_empty());
    }

    #[test]
    fn a_routing_reads_source_destination_depth() {
        let routings = of(Group::ModMatrix).expect("the matrix");
        let first = routings.first().copied().expect("a first routing");

        assert_eq!(first.source, ParamId::Mod1Source);
        assert_eq!(first.destination, ParamId::Mod1Destination);
        assert_eq!(first.depth, ParamId::Mod1Depth);
        assert_eq!(first.label, "Mod 1");
    }

    #[test]
    fn the_rows_are_named_after_what_the_library_calls_them() {
        let routings = of(Group::ModMatrix).expect("the matrix");

        let labels: Vec<&str> = routings.iter().map(|routing| routing.label).collect();
        assert_eq!(
            labels,
            vec![
                "Mod 1", "Mod 2", "Mod 3", "Mod 4", "Mod 5", "Mod 6", "Mod 7", "Mod 8"
            ]
        );
    }
}

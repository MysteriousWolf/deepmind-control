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
//! says a parameter is is still the only thing that decides that, so a routing
//! whose depth a later library gives a value table arrives as a list here
//! without a line of this file changing.
//!
//! The depth fader is turned onto its side, which is the arrangement too: eight
//! rows cannot each be 128 points tall, and eight depths in a column of bars
//! are comparable at a glance in a way eight numbers are not.
//!
//! # Two ways to say where a routing goes
//!
//! A destination is one of 133 names and a depth is a number nobody can pick
//! without having heard it, and those are the two things that make this page
//! hard. Each has an answer here and they are answers to opposite problems.
//!
//! **Typing**, for somebody who knows the name. The destination is chosen from
//! a list that can be typed into rather than one that has to be scrolled: three
//! letters and `VCF Envelope Attack` is the only one left. The names are the
//! parameter's own value table, asked of the library for the firmware that
//! answered, which is the same call the rack makes when it draws that
//! parameter. Where the library has no complete table, the row keeps whatever
//! control the library says the parameter is, with nothing here to decide.
//!
//! **Mapping**, for somebody who knows the *control*. `map` sends the
//! routing out into the window: every control the matrix can reach lights up on
//! all three surfaces, everything else is passed over, and taking hold of one
//! is the answer: a click chooses it, and a drag sets the depth as well, from
//! how far the drag would have moved it. See [`Mapper`](crate::Mapper), which is
//! also where the one assumption on this page is written down.
//!
//! Mapping is the one an editor has and a front panel does not, and it is the
//! answer to the real difficulty of this column: a destination is an
//! abbreviation the instrument's display prints, and knowing which abbreviation
//! stands over the fader you have in mind is harder than knowing the fader.
//!
//! # Nothing is written down here either
//!
//! The eight are found by what the library calls them: a parameter whose name
//! ends in `Source` with a `Destination` and a `Depth` sharing its prefix is a
//! routing, and three parameters that do not make one stay in the rack. A
//! library that grows a ninth routing draws a ninth row; a group with a lone
//! `Source` in it, as the oscillators have, is not a matrix and is not drawn as
//! one.
//!
//! The same rule covers where a routing may be mapped. Which controls light up
//! is `ValueEntry::parameters` read backwards, the destinations that name the
//! parameter under the pointer, so a firmware that moves a destination lights a
//! different set of controls with nothing in this file to edit.

use deepmind_midi::param::{Group, Kind, ParamId};
use deepmind_midi::pixels::Pixels;
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Font, Length, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{Space, button, column, combo_box, container, mouse_area, row, text};

use crate::glyphs;
use crate::lcd::{self, Band, Ink, Screen, Size};
use crate::mapping::{Mapper, Mapping, Reach, Sent};
use crate::panel::{Choice, Message, Room, choices, control, sits_at};
use crate::style::{self, field, materials, shortlist};
use crate::{Confidence, Element, Patch, tint};

/// How far the numeral beside a row is carried from the plate towards the metal.
///
/// Most of the way. It is a mark on a panel rather than a reading, since the row
/// says what the routing does and this says which of the eight is saying it, but
/// it is also what the two presses either side of it move, and a label on a
/// control has to be as legible as the control.
const NUMERAL: f32 = 0.78;

/// How much room the routing's number and the two presses that move it take.
///
/// The number alone. Printing it over the three addresses the routing occupies
/// spends fifty points of every row on a prefix the heading already says and
/// three offsets nobody edits a program by. The number is what the glass beside
/// the rows prints against every source and destination it touches, and the
/// addresses are in that glass's heading, once.
const LABEL: f32 = 20.0;

/// What share of the room the two lists have between them a source takes.
///
/// A share rather than a width, because the table is drawn out to whatever the
/// window gives it: the eight rows are a table of two lists, and a table that
/// stopped two thirds of the way across the page with bare panel beside it is
/// a table that decided how wide a window should be. The glass beside it is a
/// fixed count of dots, so what the window has spare goes here.
const SOURCE: u16 = 116;

/// The same for a destination.
///
/// Half again, because there are 133 of them and their names are the long ones:
/// `VCF Envelope Attack` has to be readable to be chosen, where `LFO 1` is five
/// characters.
const DESTINATION: u16 = 168;

/// What share of that room a picture's box takes.
///
/// Its own size, in the same units as the two shares beside it, so that the
/// boxes stay the size they are drawn at while the lists grow around them.
const PICTURE_SHARE: u16 = 18;

/// And the card between a picture and the list beside it.
const BESIDE_SHARE: u16 = 6;

/// How big the depth's dial is drawn, which is all the room it takes.
///
/// A knob's own size, taken down enough to stand inside a row's card with the
/// card still showing either side of it. How far a *drag* on it runs is not a
/// number this file has: a knob's sweep is the rack fader's travel, written
/// down once in [`knob`](crate::knob) so that every control in this window
/// moves at one rate under one hand.
const DIAL: f32 = 38.0;

/// How much card there is between that picture and the list beside it.
const BESIDE_PICTURE: f32 = 6.0;

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
    /// Returns the parameter that says where this routing comes from.
    ///
    /// Asked for the same reason [`destination`](Self::destination) is: both
    /// ends of a routing are chosen from a searchable list built once for the
    /// firmware that answered, and the list is built out here.
    pub(crate) const fn source(self) -> ParamId {
        self.source
    }

    /// Returns the parameter that says where this routing goes.
    ///
    /// The one of the three anything outside this file asks about: it is what
    /// a searchable list is built for and what a routing mapped onto the window
    /// writes into.
    pub(crate) const fn destination(self) -> ParamId {
        self.destination
    }

    /// Which of the group's routings this is, counting from one.
    ///
    /// The digits off the end of what the library calls it: `Mod 3` is the
    /// third. Read rather than counted, so a library that numbered them
    /// differently, or named them something other than `Mod`, is what the row
    /// prints.
    fn number(self) -> &'static str {
        self.label
            .rsplit(' ')
            .next()
            .filter(|last| last.chars().all(|digit| digit.is_ascii_digit()))
            .unwrap_or(self.label)
    }

    /// The three, in the order the row reads them.
    fn parameters(self) -> [ParamId; 3] {
        [self.source, self.destination, self.depth]
    }

    /// Where the whole group lives, as the run of addresses its routings cover.
    ///
    /// Once for the table rather than once a row: three addresses under every
    /// routing is twenty-four numbers down the left-hand edge of a page, and
    /// nobody edits a program by offset. It is on the glass, in the heading,
    /// where the one thing worth saying about where these bytes are is said
    /// once.
    fn run(routings: &[Self]) -> String {
        let offsets = routings
            .iter()
            .flat_map(|routing| routing.parameters())
            .map(ParamId::offset);
        let (Some(first), Some(last)) = (offsets.clone().min(), offsets.max()) else {
            return String::new();
        };
        // A hyphen and not an en dash: this run is printed on the glass, and
        // the glass writes the characters the instrument's own display has.
        format!("{first}-{last}")
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
/// Every parameter the matrix is currently mapped onto, in `firmware`'s tables.
///
/// A destination is a value in a table of names the instrument's display prints
/// abbreviated, and until `deepmind-midi` 26.2 nothing joined `VCF Freq` to
/// [`ParamId::VcfFrequency`]: a host that wanted to mark a modulated control had
/// to match those names itself, which is the second copy of a generated table
/// this repository refuses to keep. `ValueTable::parameters_of` answers it where
/// the table lives, so this reads the eight destinations the patch holds and
/// says which parameters they move.
///
/// A destination nobody has read moves nothing, because a mark drawn from a
/// value this window has not seen is a mark that says the instrument is doing
/// something it may not be.
pub(crate) fn moved(patch: &Patch, firmware: Version) -> Vec<ParamId> {
    let mut moved = Vec::new();
    for group in Group::ORDER.iter().copied() {
        for routing in of(group).into_iter().flatten() {
            let Kind::Enumerated(table) = routing.destination.kind() else {
                continue;
            };
            let Some(value) = patch.value(routing.destination) else {
                continue;
            };
            for parameter in table.table_for(firmware).parameters_of(u16::from(value)) {
                if !moved.contains(parameter) {
                    moved.push(*parameter);
                }
            }
        }
    }
    moved
}

/// One routing as the instrument's own display would name it: where it comes
/// from, where it goes, and what backs both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Wire {
    /// Which of the group's routings this is: `3`.
    number: &'static str,
    /// How much of it arrives, as the instrument's own display would print it.
    depth: String,
    /// Where it comes from.
    from: End,
    /// Where it goes.
    to: End,
    /// The weaker of the two claims, because a line drawn between a fact and a
    /// guess is a guess.
    claim: Confidence,
}

/// One end of a routing: the name the display prints, and the picture beside it.
///
/// Two readings of one value, so two ends that are equal by name are equal by
/// picture too and the glass can key its cells on either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct End {
    /// What the instrument's own display calls it.
    name: &'static str,
    /// The library's picture of it, where there is one.
    ///
    /// A source's is `ValueTable::cell_of`, an LFO's wave, a wheel or an
    /// envelope's corner, drawn once in the library so that every host draws
    /// the same picture. A destination's is the glyph of the narrowest
    /// parameter it moves: a destination is a set of program parameters and the
    /// picture of what one of those does is [`ParamId::glyph`].
    ///
    /// `None` for an end nobody has drawn, which is what the name beside it is
    /// for.
    cell: Option<&'static Pixels>,
}

/// Returns the picture of the source `value` names, where the library has one.
///
/// `ValueTable::cell_of`, which 26.5 publishes for the modulation sources
/// ([deepmind-midi#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40)).
/// The cells are the library's for the reason the effect families' marks were:
/// a picture of `LFO 1` is a fact about the instrument, and one drawn here
/// would be this window inventing one.
fn cell_from(parameter: ParamId, value: u8, firmware: Version) -> Option<&'static Pixels> {
    let Kind::Enumerated(table) = parameter.kind() else {
        return None;
    };
    table.table_for(firmware).cell_of(u16::from(value))
}

/// Returns the picture of the destination `value` names, where there is one.
///
/// A destination is a set of program parameters rather than a value with a cell
/// of its own, so the picture is the glyph of the narrowest of them, which is
/// the same parameter [`Mapping::names`](crate::mapping::Mapping::names) would
/// have chosen coming the other way. `VCF Freq` draws a filter's corner because
/// `VCF Frequency` does, and the same picture stands on the fader itself.
///
/// `None` for a destination that moves several parameters at once with no one
/// narrowest among them, and for one whose parameters carry no glyph.
fn cell_to(parameter: ParamId, value: u8, firmware: Version) -> Option<&'static Pixels> {
    let Kind::Enumerated(table) = parameter.kind() else {
        return None;
    };
    let table = table.table_for(firmware);
    let moved = table.parameters_of(u16::from(value));
    let [only] = moved else {
        return None;
    };
    Some(only.glyph()?.pixels())
}

/// Returns the routings the patch has actually wired, in the order the matrix
/// reads them.
///
/// Both names are the library's own value tables, asked for the firmware that
/// answered, which is the call the row above makes when it draws either as a
/// list.
/// A routing with nothing at one end is not a wire and is not drawn: the
/// instrument ships with all eight sitting on `Off`, and eight lines from `Off`
/// to `Off` is a picture of nothing drawn eight times.
pub(crate) fn wiring(patch: &Patch, firmware: Version) -> Vec<Wire> {
    let named = |parameter: ParamId,
                 picture: fn(ParamId, u8, Version) -> Option<&'static Pixels>|
     -> Option<End> {
        let value = patch.value(parameter)?;
        let name = choices(parameter, firmware, Some(value))?
            .into_iter()
            .find(|choice| choice.byte() == value)?
            .name();
        // `Off` is the instrument saying this end is not wired, and it is the
        // library's own word for it rather than this window's.
        (!name.eq_ignore_ascii_case("off")).then(|| End {
            name,
            cell: picture(parameter, value, firmware),
        })
    };
    Group::ORDER
        .iter()
        .copied()
        .filter_map(of)
        .flatten()
        .filter_map(|routing| {
            Some(Wire {
                number: routing.number(),
                depth: patch
                    .value(routing.depth)
                    .map_or_else(String::new, |value| sits_at(routing.depth, value)),
                from: named(routing.source, cell_from)?,
                to: named(routing.destination, cell_to)?,
                claim: patch.claim_across([routing.source, routing.destination]),
            })
        })
        .collect()
}

/// Every routing the patch holds, against the control each one lands on.
///
/// The same walk [`moved`] makes and one more question asked of it: not only
/// which controls something can move, but which routing moves them and how
/// much is in it. Read once for a window and handed to every control, because
/// a routing being mapped is answered on all three surfaces at once and the
/// answer is the same for all of them.
///
/// A routing whose destination nobody has read reaches nothing, for the reason
/// [`moved`] draws no mark for one: a band drawn from a value this window has
/// not seen is a band that says the instrument is doing something it may not
/// be. A depth nobody has read is the same: the destination may be known and the
/// amount not, and a band of an assumed width on a known destination is the
/// worse of the two errors.
pub(crate) fn reaching(patch: &Patch, firmware: Version) -> Vec<Reach> {
    let mut reaches = Vec::new();
    for group in Group::ORDER.iter().copied() {
        for routing in of(group).into_iter().flatten() {
            let Kind::Enumerated(table) = routing.destination.kind() else {
                continue;
            };
            let (Some(value), Some(depth)) =
                (patch.value(routing.destination), patch.value(routing.depth))
            else {
                continue;
            };
            // Which way the source at the other end of this routing moves what
            // it reaches, which 26.5 publishes and which the band drawn on
            // every control it lands on is read from. A source nobody has read
            // swings nowhere, the same as a source the specification does not
            // settle: both are a band this window would be drawing blind.
            let swings = match routing.source.kind() {
                Kind::Enumerated(sources) => patch
                    .value(routing.source)
                    .and_then(|source| sources.table_for(firmware).swing_of(u16::from(source))),
                _ => None,
            };
            for parameter in table.table_for(firmware).parameters_of(u16::from(value)) {
                reaches.push(Reach::new(
                    *parameter,
                    routing.label,
                    routing.depth,
                    depth,
                    swings,
                ));
            }
        }
    }
    reaches
}

pub(crate) fn table<'a, Renderer>(
    patch: &'a Patch,
    group: Group,
    firmware: Version,
    mapper: &'a Mapper,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let routings = of(group)?;
    let mapping = mapper.mapped();
    // The other seven rows, for the eighth to be chosen against. Read once
    // here, as the rack reads it once: the same walk answers every control.
    let reaches = mapping
        .map(|_| reaching(patch, firmware))
        .unwrap_or_default();
    let sent = mapping.map(|mapping| Sent::new(mapping, &reaches));
    let heading = |parameter: ParamId, width: Length| -> Element<'a, Renderer> {
        // The heading is the parameter's own name with what the row already
        // says taken off the front, which is the rule a slot's title follows
        // against its group: `Mod 1 Destination` in the `Mod 1` row is
        // `Destination`.
        let name = parameter.name();
        let title = name.rsplit(' ').next().unwrap_or(name);
        text(title).size(11).width(width).into()
    };
    let first = routings.first().copied()?;
    // The two lists take what the window has spare, in the proportion their
    // names need it: see [`SOURCE`] and [`DESTINATION`]. Everything else in the
    // row is the size of what is drawn in it, so a wider window is a wider pair
    // of lists rather than a wider everything.
    let header = row![
        Space::new().width(Length::Fixed(LABEL)),
        heading(
            first.source,
            Length::FillPortion(SOURCE + PICTURE_SHARE + BESIDE_SHARE)
        ),
        Space::new().width(Length::Fixed(ARROW)),
        heading(
            first.destination,
            Length::FillPortion(DESTINATION + PICTURE_SHARE + BESIDE_SHARE)
        ),
        Space::new().width(Length::Fixed(POINT + 10.0)),
        heading(first.depth, Length::Fixed(DIAL)),
    ]
    .spacing(10)
    // The card the rows stand on has its own padding, and a heading that
    // ignored it would be a heading half a word to the left of the column it
    // names.
    .padding([0.0, BESIDE_ROW])
    .height(Length::Fixed(HEADING - DOWN));
    let rows = routings
        .iter()
        .copied()
        .enumerate()
        .map(|(index, routing)| {
            let reading = row![
                // Where a routing can be moved to, which is the one thing a
                // matrix of eight identical slots gives nobody a way to do.
                shifts(patch, &routings, index),
                chosen(patch, routing.source, firmware, mapper, sent, SOURCE),
                arrow(),
                chosen(
                    patch,
                    routing.destination,
                    firmware,
                    mapper,
                    sent,
                    DESTINATION
                ),
                mapping_press(routing, mapper),
                // A knob rather than a fader lying down. A depth is read about
                // its centre, `-128` at one end, `+127` at the other and no
                // modulation in the middle, and a dial is the control that
                // shows a middle by pointing at it. Eight faders at four
                // different places along their tracks are eight positions to
                // compare; eight dials are eight hands on eight clocks.
                cell(
                    patch,
                    routing.depth,
                    firmware,
                    Room::across(DIAL).turned(),
                    sent
                ),
            ]
            .spacing(10)
            .align_y(Vertical::Center);
            // Each row on a card of its own. Eight rows of controls at four
            // heights, floating on the plate the rack stands on, are eight rows
            // nothing lines up against. A routing is one sentence and this is
            // the paper it is written on.
            container(reading)
                .height(Length::Fixed(ROW))
                .padding([0.0, BESIDE_ROW])
                .align_y(Vertical::Center)
                .style(card)
                .into()
        });
    let read = column![header]
        .extend(rows)
        // What the mode is, while it is up. On the page the routing is on
        // rather than beside the control somebody is about to take hold of,
        // because the whole point of it is that they are about to go
        // somewhere else in the window.
        .extend(mapping.map(mapping_note))
        .spacing(DOWN);
    // And the shape of the whole thing beside the eight sentences, on the
    // instrument's own glass. The rows say what each routing is; the glass says
    // what they add up to, which is the question the rows cannot answer however
    // carefully somebody reads down them.
    // The table takes the room the window has and the glass takes its own: a
    // display is a count of dots at a fixed pitch, and one drawn out to fill a
    // page would be a magnified screen rather than a bigger one. So what a wide
    // window is worth goes to the two lists, which is where a long destination
    // name needs it.
    Some(
        row![
            read.width(Length::Fill),
            container(bay(patch, firmware, deep(routings.len()), &routings))
                .width(Length::Fixed(lcd::room(BAY))),
        ]
        .spacing(BESIDE)
        .align_y(Vertical::Top)
        .width(Length::Fill)
        .into(),
    )
}

/// How much panel there is between the eight rows and the glass beside them.
const BESIDE: f32 = 14.0;

/// Draws the card one routing is written on.
///
/// The face plate a rack's slots already stand on, with the seam and the lit
/// lip every cut surface in this window presents. Eight of them down a page are
/// eight things of one size, which is what the rows were missing: a control is
/// aligned against the card it is in rather than against a control two rows
/// away that happens to be the same height.
fn card(theme: &Theme) -> container::Style {
    let material = materials(theme);
    container::Style {
        background: Some(Background::Color(material.plate)),
        border: border::rounded(3).width(1.0).color(material.recess_edge),
        ..container::Style::default()
    }
}

/// How much card there is either side of what stands on it.
const BESIDE_ROW: f32 = 10.0;

/// How tall one routing's row stands.
///
/// Fixed rather than however tall the tallest thing in it happens to be,
/// because the glass beside the rows has to be exactly as deep as they are:
/// a patch bay that stopped two rows short of the table it is a picture of
/// would be a picture of something else. It is the tallest a row needs, one line
/// of controls and a little card either side of them, and every row is that
/// whatever is in it, which is the rule a rack's slots already follow.
///
/// It was half again as tall when every control printed its value underneath.
/// The footer already says what is under the pointer, which for anything in
/// this table is its name, its reading and its range, so a row that printed the
/// same number a third time was a row and a half.
///
/// What decides it now is the column that moves a routing: two five-dot arrows,
/// the numeral between them in the display's own character cell, and the card
/// either side of the three.
const ROW: f32 = 50.0;

/// How much panel there is between two of those rows.
const DOWN: f32 = 8.0;

/// How tall the heading over them stands, with its own gap under it.
const HEADING: f32 = 16.0 + DOWN;

/// Returns how many dots deep the glass beside `rows` routings is.
///
/// The table's own height, measured in the dots every display in this window is
/// drawn at. The two are locked together by construction rather than by two
/// numbers somebody has to keep in step, which is what a test holds.
fn deep(rows: usize) -> i32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a count of routings, which the library gives as eight"
    )]
    let rows = rows.max(1) as f32;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a count of dots down a table a window has room for"
    )]
    let dots = ((HEADING + rows * ROW + (rows - 1.0) * DOWN - lcd::room(0)) / crate::lcd::PITCH)
        .floor()
        .max(1.0) as i32;
    dots
}

/// Draws the patch bay: which sources are wired to which destinations.
///
/// The one thing the table cannot show. Eight rows read one sentence each, and
/// what somebody wants to know about a modulation matrix is the *shape* of it:
/// that one LFO is driving three things, that two routings are fighting over
/// the filter, that the aftertouch goes nowhere. Sources down one side,
/// destinations down the other, a line for every routing between them, and the
/// fan-out is the picture.
///
/// It is on the instrument's own glass, at the pitch every display in this
/// window shares, and it takes the band it is given as more dots rather than
/// bigger ones, which is the rule the chain's glass is cut under too.
///
/// # The cells are pictures
///
/// A 7 by 7 drawing stands against each name: an LFO's wave, a wheel, an
/// envelope's corner on one side, and what the destination's parameter does on
/// the other. 26.5 publishes the sources' as `ValueTable::cell_of` and the
/// parameters' as [`ParamId::glyph`]
/// ([deepmind-midi#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40)),
/// so a column of abbreviations is a column of pictures, which is what a patch
/// bay is for: the shape of a matrix is something you read at a glance or not at
/// all.
///
/// They are the library's for the reason the effect families' marks were: a
/// mark for `LFO 1` is a fact about the instrument and one drawn here would be
/// this window inventing it. The names stay beside them, because a picture and
/// a name say different amounts to somebody who has not met either.
fn bay<'a, Renderer>(
    patch: &Patch,
    firmware: Version,
    deep: i32,
    routings: &[Routing],
) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let wires = wiring(patch, firmware);
    // The weakest claim any wire on it makes, which is the rule everything drawn
    // as one thing out of several follows here: a glass that called itself the
    // synthesizer's because seven of its eight lines were is the one lie this
    // crate exists to avoid.
    let claim = wires.iter().fold(Confidence::Confirmed, |claim, wire| {
        match (claim, wire.claim) {
            (Confidence::Unknown, _) | (_, Confidence::Unknown) => Confidence::Unknown,
            (Confidence::Assumed, _) | (_, Confidence::Assumed) => Confidence::Assumed,
            _ => Confidence::Confirmed,
        }
    });
    lcd::lcd(
        drawn(&wires, deep, routings.len(), &Routing::run(routings)),
        claim,
    )
}

/// Draws the bay onto a screen `deep` dots tall.
///
/// Apart from [`bay`] so that what it draws can be looked at without a window
/// to draw it in, which for a picture made of dots is the only way to look at
/// it at all.
///
/// # A name is drawn once, and what leaves it is a list
///
/// One cell per source and one per destination, however many routings touch
/// them, because that is the picture: one LFO driving three things is one cell
/// with three wires out of it, and the same thing drawn as three cells reading
/// `LFO 1` is a table with lines on it.
///
/// What each of those wires *carries* is written at the end it leaves from: the
/// routing's number and its depth, one line each, down the source's own cell. So
/// the numbers are beside the wires they belong to rather than in a block along
/// the foot of the glass.
fn drawn(wires: &[Wire], deep: i32, of: usize, run: &str) -> Screen {
    let mut screen = Screen::new(BAY, deep);
    // The heading, inverted, which is how a display with one colour of light
    // says a line is a heading rather than a reading. Two facts that are true
    // of the whole table and would be a column of repetitions in it: how many
    // of the routings are wired, and where the group's bytes live.
    screen.write(
        MARGIN,
        MARGIN,
        &format!("{} OF {of}", wires.len()),
        Size::Small,
    );
    let from = BAY - MARGIN - Screen::width_of(run, Size::Small);
    screen.write(from, MARGIN, run, Size::Small);
    screen.invert(Band::new(0, 0, BAY, MARGIN * 2 + LINE));

    let ends = |pick: fn(&Wire) -> End| -> Vec<End> {
        let mut ends: Vec<End> = Vec::new();
        for end in wires.iter().map(pick) {
            if !ends.contains(&end) {
                ends.push(end);
            }
        }
        ends
    };
    let (sources, destinations) = (ends(|wire| wire.from), ends(|wire| wire.to));
    // What leaves each source: its routings, in the order the matrix reads
    // them, and how much of it each one carries.
    let leaving = |end: End| -> Vec<String> {
        wires
            .iter()
            .filter(|wire| wire.from == end)
            .map(|wire| format!("{} {}", wire.number, wire.depth).trim().to_owned())
            .collect()
    };
    let head = MARGIN * 2 + LINE;
    let depths: Vec<i32> = sources
        .iter()
        .map(|end| cell_deep(leaving(*end).len()))
        .collect();
    let left = spread(head, deep, &depths);
    let right = spread(head, deep, &vec![cell_deep(0); destinations.len()]);

    for (end, top) in sources.iter().zip(&left) {
        node(&mut screen, MARGIN, *top, *end, &leaving(*end), Side::From);
    }
    for (end, top) in destinations.iter().zip(&right) {
        node(
            &mut screen,
            BAY - MARGIN - CELL_ACROSS,
            *top,
            *end,
            &[],
            Side::To,
        );
    }
    // A wire for every routing, out of the line that says what it carries and
    // into the cell it arrives at. A track each, because two wires down the
    // same part of the glass have to be two wires and not one heavier one.
    for (lane, wire) in wires.iter().enumerate() {
        let (Some(source), Some(destination)) = (
            sources.iter().position(|end| *end == wire.from),
            destinations.iter().position(|end| *end == wire.to),
        ) else {
            continue;
        };
        let Some(top) = left.get(source).copied() else {
            continue;
        };
        let Some(arrives) = right.get(destination).copied() else {
            continue;
        };
        // Which of that source's lines this routing is, so that the wire leaves
        // beside its own number rather than from the middle of a list.
        let line = wires
            .iter()
            .filter(|other| other.from == wire.from)
            .position(|other| other.number == wire.number)
            .unwrap_or(0);
        bend(
            &mut screen,
            (MARGIN + CELL_ACROSS - 1, reading_at(top, line) + LINE / 2),
            (BAY - MARGIN - CELL_ACROSS, arrives + cell_deep(0) / 2),
            i32::try_from(lane).unwrap_or(0),
        );
    }
    screen
}

/// How tall one line of writing is on the glass.
const LINE: i32 = 7;

/// How deep a cell with `lines` readings written down it is.
///
/// The name, a line for each routing that leaves it, and the glass above and
/// below. A destination has none, because what arrives at it is written at the
/// end it left from, so it is a cell one line deep.
fn cell_deep(lines: usize) -> i32 {
    let lines = i32::try_from(lines).unwrap_or(0);
    2 + LINE + lines * (APART + LINE) + 2
}

/// Where the `line`th reading in a cell standing at `top` is written.
fn reading_at(top: i32, line: usize) -> i32 {
    top + 2 + LINE + APART + i32::try_from(line).unwrap_or(0) * (APART + LINE)
}

/// Returns where a column of cells `depths` deep stands, spread down the glass.
///
/// Spread rather than stacked, because the room is what the wires are drawn in:
/// three cells against a glass as deep as eight rows of controls leave eighty
/// dots between them, and eighty dots is a wire that visibly goes somewhere.
/// Capped and centred, so that three of them are a group in the middle of the
/// glass rather than three cells in its corners.
fn spread(under: i32, deep: i32, depths: &[i32]) -> Vec<i32> {
    let total: i32 = depths.iter().sum();
    let count = i32::try_from(depths.len()).unwrap_or(0);
    let room = deep - under - MARGIN - total;
    let apart = if count > 1 {
        (room / (count - 1)).clamp(0, PITCH)
    } else {
        0
    };
    let mut top = under + (room - apart * (count - 1).max(0)) / 2;
    let mut tops = Vec::with_capacity(depths.len());
    for depth in depths {
        tops.push(top);
        top += depth + apart;
    }
    tops
}

/// Which end of a wire a cell is, which decides where its knots sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// A source: the wires leave its right-hand edge, one per reading.
    From,
    /// A destination: they arrive at the middle of its left-hand edge.
    To,
}

/// Draws one end of a routing on the glass: its frame, the box its picture will
/// stand in, its name, whatever leaves it, and the knots the wires attach to.
fn node(screen: &mut Screen, at: i32, top: i32, end: End, lines: &[String], side: Side) {
    let deep = cell_deep(lines.len());
    screen.frame(Band::new(at, top, CELL_ACROSS, deep), Ink::Solid);
    // The library's own picture of this source or destination, seven dots by
    // seven, which is the cell this display writes a character in. An end
    // nobody has drawn keeps the empty box it always had, so a column of
    // pictures with one gap in it reads as one thing undrawn rather than as a
    // column that has not been drawn.
    let box_at = Band::new(at + GUTTER, top + 2, PICTURE, PICTURE);
    match end.cell {
        Some(cell) => screen.blit(cell, box_at.x, box_at.y),
        None => screen.frame(box_at, Ink::Solid),
    }
    // The name, in the field left over. A name too long for it scrolls rather
    // than losing its tail: `Pitch Bend` and `BreathCtrl` are ten characters in
    // a field cut for nine, and half a name is a name somebody has to already
    // know to read.
    let field = at + GUTTER + PICTURE + GUTTER;
    screen.marquee(
        field,
        top + 2,
        CELL_ACROSS - GUTTER - (field - at),
        end.name,
        Size::Small,
    );
    // What leaves it, one line per routing, against the edge the wires go out
    // of, so a reading and its own wire are the same line of the drawing.
    for (line, said) in lines.iter().enumerate() {
        let at = at + CELL_ACROSS - GUTTER - Screen::width_of(said, Size::Small);
        screen.write(at, reading_at(top, line), said, Size::Small);
    }
    // The knots: a blob on the edge a wire leaves or arrives at, so that three
    // wires out of one source are visibly three wires out of one source rather
    // than three lines that happen to converge.
    let knots = match side {
        Side::From => (0..lines.len())
            .map(|line| (at + CELL_ACROSS - 1, reading_at(top, line) + LINE / 2))
            .collect::<Vec<_>>(),
        Side::To => vec![(at, top + deep / 2)],
    };
    for (x, y) in knots {
        let out = match side {
            Side::From => 0,
            Side::To => -(KNOT - 1),
        };
        screen.fill(Band::new(x + out, y - KNOT / 2, KNOT, KNOT));
    }
}

/// Draws one wire between two knots, down the `lane`th track of the gap.
///
/// Flat out of the source, down a track of its own, flat into the destination,
/// with the corners taken off. Not a line between two points and not an ease
/// across the gap: the glass is as deep as eight rows of controls and the gap
/// is a fifth of that across, so anything drawn as a single sweep between two
/// distant nodes comes out as a near-vertical scratch that could have started
/// anywhere.
fn bend(screen: &mut Screen, from: (i32, i32), to: (i32, i32), lane: i32) {
    let (x, y) = from;
    let (across, down) = to;
    let track = x + TURN + lane * LANE;
    // Not far enough apart to take a corner off: two chamfers would meet and
    // the wire would bulge where it should be straight.
    let corner = if (down - y).abs() >= CHAMFER * 2 {
        CHAMFER
    } else {
        0
    };
    let step = if down > y { corner } else { -corner };
    screen.line((x, y), (track - corner, y), Ink::Solid);
    screen.line((track - corner, y), (track, y + step), Ink::Solid);
    screen.line((track, y + step), (track, down - step), Ink::Solid);
    screen.line((track, down - step), (track + corner, down), Ink::Solid);
    screen.line((track + corner, down), (across, down), Ink::Solid);
}

/// How far into the gap the first wire turns.
const TURN: i32 = 6;

/// How far apart two wires' tracks are.
const LANE: i32 = 2;

/// How much of each corner is taken off.
///
/// Three dots, which at this pitch is a corner that reads as turned rather than
/// as mitred, and the most a dot matrix can say about a radius.
const CHAMFER: i32 = 3;

/// How many characters of a name a cell's field holds without scrolling.
///
/// Nine, which is what the instrument's own screen prints for all but a
/// handful: `Pitch Bend` and `BreathCtrl` are the ten-character ones, and they
/// are why the field scrolls rather than clips.
const LETTERS: i32 = 9;

/// How wide the box a name's picture will stand in is, and how deep.
///
/// Seven by seven, which is the cell this display writes a character in and the
/// grid the library draws its own cells and glyphs on.
const PICTURE: i32 = 7;

/// How wide a cell is: its picture, its name's field, and the glass around
/// both.
const CELL_ACROSS: i32 = GUTTER * 3 + PICTURE + (LETTERS * glyphs::ADVANCE - glyphs::GAP);

/// How much glass there is between a cell's picture, its name and its frame.
const GUTTER: i32 = 3;

/// How much glass there is between two lines written in a cell.
///
/// Three dots. Two lines of writing with one between them are two lines a
/// reader's eye separates without being asked to; a name sitting straight on
/// the reading under it is one taller line of something illegible.
const APART: i32 = 3;

/// How big the blob a wire attaches to is.
const KNOT: i32 = 3;

/// The most glass there is between two cells in a column.
const PITCH: i32 = LINE * 3;

/// How much glass the wires have to cross.
///
/// Enough for eight tracks side by side with room to turn into and out of each:
/// the first turns [`TURN`] dots in, they are [`LANE`] apart, and a corner
/// takes [`CHAMFER`] off either end of the run.
const LINK: i32 = TURN * 2 + LANE * 8 + CHAMFER * 2;

/// How many dots across the patch bay is drawn.
///
/// Two cells and the gap the wires cross. A fixed count rather than whatever
/// the band divides into, which is the one place in this window that rule is
/// not followed: the eight rows beside it are as wide as eight rows of controls
/// are, and glass that took the room left over would be glass whose width was
/// decided by how long a destination's name is.
const BAY: i32 = MARGIN * 2 + CELL_ACROSS * 2 + LINK;

/// How much glass there is around the drawing.
const MARGIN: i32 = 3;

/// Draws the two presses that move a routing up or down the table.
///
/// The eight are read as a set and the instrument does not care which of them
/// says what, so where a routing sits is entirely for whoever has to read the
/// table next. A matrix filled in over a week is eight rows in the order they
/// were thought of; the same eight grouped by what they move is the same sound
/// and a page somebody can read.
///
/// Nothing about the sound changes. What moves is six bytes trading places, and
/// both routings keep everything they had.
///
/// A press is dead where there is nothing to trade: the top row cannot go up,
/// the bottom cannot go down, and a routing whose bytes nobody has read cannot
/// be moved anywhere, because writing a value this window has not seen into a
/// slot is the one thing it does not do.
fn shifts<'a, Renderer>(patch: &Patch, routings: &[Routing], index: usize) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let routing = routings.get(index).copied();
    // What backs the routing, carried by the one glyph that is already about
    // the routing rather than about any one of its three parameters. It was two
    // dots floating under the two lists, which is a mark that has to be asked
    // about before it says anything; a number in the claim's own colour is the
    // same three-way answer on something somebody is already reading.
    let claim = routing.map_or(Confidence::Unknown, |routing| {
        patch.claim_across(routing.parameters())
    });
    let press = |mark: crate::Badge, said: &'static str, towards: Option<usize>| {
        let swap = routing.zip(towards.and_then(|at| routings.get(at).copied()));
        let known = |routing: Routing| {
            !matches!(
                patch.claim_across(routing.parameters()),
                Confidence::Unknown
            )
        };
        let live = swap.filter(|(one, other)| known(*one) && known(*other));
        // The mark on the panel, in the metal a hand touches where the press
        // does something and most of the way back to the panel where it does
        // not, which is how a rack unit's own case says a control is not wired
        // to anything.
        let arrow = lcd::stencil(mark.screen(), move |theme: &Theme| {
            let material = materials(theme);
            style::mix(
                materials(theme).plate,
                material.metal,
                if live.is_some() { MARKED } else { DEAD },
            )
        });
        let press = button(
            container(arrow)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fixed(LABEL))
        .height(Length::Fixed(SHIFT))
        .padding(0)
        .style(style::marked)
        .on_press_maybe(live.map(|(one, other)| Message::Swap {
            one: one.parameters(),
            other: other.parameters(),
        }));
        // What it does, said in the footer while the pointer is on it. There is
        // nowhere on a mark nine dots square to write it, and a word printed
        // beside every press in the table would be two columns of words on a
        // page whose whole argument is that the rows were saying too much.
        mouse_area(press)
            .on_enter(Message::Hinted(Some(said)))
            .on_exit(Message::Hinted(None))
    };
    // The number between them, because it is the thing that moves: sending a
    // routing up the table is `3` becoming `2`, and the press above the numeral
    // is the one that does it.
    //
    // Printed in the display's own dots, which is the numeral an effect
    // engine's case already carries, and the numeral the glass beside these rows
    // prints against every source and destination the routing touches, so the
    // table and the picture say the same thing in the same hand.
    column![press(
        crate::UP,
        "Move this routing up the matrix, trading places with the one above it.",
        index.checked_sub(1)
    )]
    .push(
        container(lcd::stencil(
            Screen::of(routing.map_or("", Routing::number), Size::Small),
            move |theme: &Theme| style::mix(materials(theme).plate, tint(theme, claim), NUMERAL),
        ))
        .width(Length::Fixed(LABEL))
        .align_x(Horizontal::Center),
    )
    .push(press(
        crate::DOWN,
        "Move this routing down the matrix, trading places with the one below it.",
        Some(index + 1).filter(|at| *at < routings.len()),
    ))
    .spacing(SHIFT_APART)
    .align_x(Horizontal::Center)
    .into()
}

/// How far a live press's arrow is carried from the card towards the metal.
const MARKED: f32 = 0.82;

/// The same, for a press with nothing to trade.
///
/// Most of the way back to the card it is printed on: a mark that is still
/// there and no longer a control, which is what a dead press looks like on an
/// instrument.
const DEAD: f32 = 0.28;

/// How tall one of those presses stands.
///
/// A little more than the five-dot mark on it, so that the light a press shows
/// under the pointer is a shape around the arrow rather than a shape the arrow
/// is touching the edges of. Two of them, the numeral between them and the
/// glass either side of that are one [row](ROW) exactly.
const SHIFT: f32 = 14.0;

/// How much card there is between the two presses and the numeral they move.
const SHIFT_APART: f32 = 2.0;

/// Draws one end of a routing: its picture, and the list its name is chosen
/// from.
///
/// **One component for both ends.** A source is one of 24 names and a
/// destination one of 133, which is a difference in how long the list is and in
/// nothing else. Both are searchable lists, because the one that is hard to
/// scroll decides: three letters and `VCF Envelope Attack` is the only one left,
/// and the same three letters cost a source nothing.
///
/// Where the library has no complete table for an end, the row keeps whatever
/// control the library says that parameter is, which is not a decision this file
/// makes and is why the list is asked for rather than assumed.
///
/// # The picture
///
/// The library's own drawing of whatever this end is set to, seven dots square,
/// beside the list it was chosen from. It is the picture the patch bay puts
/// against the same name, so a routing reads the same on the row and on the
/// glass. A source's is `ValueTable::cell_of` and a destination's is the glyph
/// of the parameter it moves, both published in 26.5
/// ([deepmind-midi#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40)).
///
/// An end that is `Off`, that nobody has read, or that nobody has drawn keeps
/// the empty box, which is what this was before any of them landed.
fn chosen<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
    mapper: &'a Mapper,
    sent: Option<Sent<'_>>,
    share: u16,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let listed: Element<'a, Renderer> = match mapper.list(parameter) {
        // A list that opened while the matrix was mapping would be a menu over
        // the controls somebody is trying to map onto, so while one is up the
        // row falls back to the control `control` draws for the mode.
        Some(list) if sent.is_none() => {
            let value = patch.value(parameter);
            let selected = choices(parameter, firmware, value).and_then(|options| {
                options
                    .into_iter()
                    .find(|choice| Some(choice.byte()) == value)
            });
            container(
                combo_box(list, "search", selected.as_ref(), move |choice: Choice| {
                    Message::Edit {
                        parameter,
                        value: choice.byte(),
                    }
                })
                .size(11)
                .padding([2, 6])
                .menu_height(Length::Fixed(MENU))
                .input_style(field)
                .menu_style(shortlist)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fixed(crate::panel::BUTTON))
            .align_y(Vertical::Center)
            .into()
        }
        _ => cell(patch, parameter, firmware, Room::filling_list(), sent),
    };
    let drawn = patch
        .value(parameter)
        .and_then(|value| cell_of(parameter, value, firmware));
    row![picture(drawn), listed]
        .spacing(BESIDE_PICTURE)
        .align_y(Vertical::Center)
        .width(Length::FillPortion(share + PICTURE_SHARE + BESIDE_SHARE))
        .into()
}

/// Returns the library's picture of whatever `parameter` is set to.
///
/// A source and a destination are drawn from two different accessors, because
/// sources carry cells of their own and a destination is a set of program
/// parameters whose glyph is the picture. Which one answers is decided by which
/// of the three parameters of a routing this is. A routing's own
/// [`Routing::source`] is what says so, rather than the parameter's name.
fn cell_of(parameter: ParamId, value: u8, firmware: Version) -> Option<&'static Pixels> {
    if is_source(parameter) {
        cell_from(parameter, value, firmware)
    } else {
        cell_to(parameter, value, firmware)
    }
}

/// Returns whether `parameter` is the end a routing comes from.
///
/// Asked of the library's own tables rather than of the parameter's name: the
/// eight routings are found the same way everything else in this file finds
/// them, so a firmware that renamed them is still read correctly.
fn is_source(parameter: ParamId) -> bool {
    Group::ORDER
        .iter()
        .copied()
        .filter_map(of)
        .flatten()
        .any(|routing| routing.source == parameter)
}

/// Draws a source or destination's picture, or the box it would stand in.
///
/// Printed on the card rather than lit on glass, because it is a mark beside a
/// control and not a display: the same call the numeral beside the row goes
/// through. An end nobody has drawn keeps the empty frame, drawn as a frame
/// rather than as a dotted one, because seven dots square is too small for a
/// dotted line to read as anything but scatter.
fn picture<'a, Renderer>(cell: Option<&'static Pixels>) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let mut screen = Screen::new(PICTURE, PICTURE);
    match cell {
        Some(cell) => screen.blit(cell, 0, 0),
        None => screen.frame(Band::new(0, 0, PICTURE, PICTURE), Ink::Solid),
    }
    lcd::stencil(screen, |theme: &Theme| {
        let material = materials(theme);
        style::mix(material.plate, material.metal_low, WAITING)
    })
}

/// How far the empty picture box is carried from the card towards the metal.
///
/// Not far. It is a box with nothing in it, and a box with nothing in it drawn
/// as loudly as the name beside it would be the loudest thing in a row that is
/// about the name.
const WAITING: f32 = 0.55;

/// Draws the press that sends a routing out into the window.
///
/// A [reticle](crate::MAP) stencilled on the card, the way every other mark in
/// this window is: a press whose face is a drawing rather than a label needs no
/// rim to say where the label stops and the button starts, and a row whose
/// other controls are two recesses and a dial had a rounded rectangle sitting
/// in the middle of it.
///
/// What the press does is put the routing over the panel and wait for somebody
/// to aim it at a control, which is a thing with a picture, so it carries the
/// picture and the word is in the footer as the sentence the pointer brings up.
/// That is also what makes the press square: a press whose face is a word is as
/// wide as the word, and one whose face is a mark is the size of the mark.
///
/// What changes while the mode is up is the ink. It goes to the one saturated
/// colour on the panel, the same cyan every control the routing can reach is lit
/// in, because the press and the lit controls are one thing happening. The mark
/// does not change: a press that showed a reticle and then a cross would be two
/// marks to learn, and the colour already says which state it is in.
fn mapping_press<'a, Renderer>(routing: Routing, mapper: &Mapper) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let destination = routing.destination;
    let mapping = mapper
        .mapped()
        .is_some_and(|mapped| mapped.destination() == destination);
    let mark = lcd::stencil(crate::MAP.screen(), move |theme: &Theme| {
        if mapping {
            style::MODULATION
        } else {
            let material = materials(theme);
            style::mix(material.plate, material.metal, MARKED)
        }
    });
    let press = button(
        container(mark)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fixed(POINT))
    .height(Length::Fixed(crate::panel::BUTTON))
    .padding(0)
    .style(style::marked)
    .on_press(Message::Mapper(
        (!mapping).then(|| Mapping::new(routing.label, destination, routing.depth)),
    ));
    mouse_area(press)
        .on_enter(Message::Hinted(Some(if mapping {
            "Stop mapping: leave this routing where it is."
        } else {
            "Send this routing out into the window, and take hold of the control it should move."
        })))
        .on_exit(Message::Hinted(None))
        .into()
}

/// Says what mapping a routing at the window means while one is mapped.
///
/// One line, under the eight rows, and it is where the mode is explained
/// because it is where the mode was asked for. It says the two things somebody
/// needs and no more: that a click is the destination, and that a drag is the
/// depth as well.
fn mapping_note<'a, Renderer>(mapped: Mapping) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(format!(
            "{} is mapped onto the window. Take hold of any lit control to send it there \u{2014} \
             a click chooses it, and a drag sets the depth from how far it went. \
             What full depth is worth is assumed; the manual does not print it.",
            mapped.label()
        ))
        .size(11),
    )
    .padding([6, 8])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        border: border::rounded(3).width(1.0).color(style::MODULATION),
        ..container::Style::default()
    })
    .into()
}

/// How much room the press that maps a routing onto the window takes.
///
/// The mark on it and the card either side of it, which is the same room every
/// other mark in this window is given. It was the width of the word `MAP` set
/// small; a reticle is square, so the press is the size of a press rather than
/// the size of a label.
const POINT: f32 = 28.0;

/// How tall the list a destination is searched in opens.
///
/// Deep enough to be worth scrolling and short enough to leave the rows under
/// it visible, because what somebody is choosing between is often two rows that
/// both say `LFO 1`.
const MENU: f32 = 220.0;

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
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    container(control(parameter, value, claim, firmware, room, sent))
        .width(room.across_as())
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
    .align_y(Vertical::Center)
    .into()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{End, moved, of, routed};
    use deepmind_midi::param::{Group, Kind, ParamId};

    use super::{BAY, CELL_ACROSS, Wire, deep, drawn};
    use crate::Confidence;

    /// The screen as characters, so that a picture made of dots can be looked
    /// at in a test failure.
    fn printed(screen: &crate::Screen) -> String {
        (0..screen.rows())
            .map(|row| {
                (0..screen.columns())
                    .map(|column| {
                        if screen.is_inked(column, row) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The numbers the eight routings carry, so that a test can build wires
    /// the way `wiring` does.
    const NUMBERED: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];

    fn wired(pairs: &[(&'static str, &'static str)]) -> Vec<Wire> {
        pairs
            .iter()
            .enumerate()
            .map(|(index, (from, to))| Wire {
                number: NUMBERED.get(index).copied().unwrap_or("9"),
                depth: format!("+{}", index * 8),
                from: End {
                    name: from,
                    cell: None,
                },
                to: End {
                    name: to,
                    cell: None,
                },
                claim: Confidence::Confirmed,
            })
            .collect()
    }

    /// The same drawing every test here looks at: eight routings' worth of
    /// glass, with the heading the table's own group would give it.
    fn shown(wires: &[Wire]) -> crate::Screen {
        drawn(wires, deep(8), 8, "93-116")
    }

    #[test]
    fn both_ends_of_a_routing_carry_the_librarys_own_picture() {
        use deepmind_midi::param::DEFAULT_FIRMWARE;
        use deepmind_midi::pixels::Glyph;
        use deepmind_midi::program::ModSource;

        // A source's cell is the library's drawing of it, and a destination's
        // is the glyph of the parameter it moves. Both landed in 26.5 and
        // both are what the row and the glass now stand beside a name.
        let lfo = super::cell_from(ParamId::Mod1Source, ModSource::Lfo1.raw(), DEFAULT_FIRMWARE)
            .expect("the library draws LFO 1");
        assert!(lfo.rows().iter().any(|row| *row != 0), "a blank cell");

        // `Off` is the instrument saying an end is not wired, so it has none.
        assert!(
            super::cell_from(ParamId::Mod1Source, ModSource::Off.raw(), DEFAULT_FIRMWARE).is_none()
        );

        // And a destination that moves one parameter draws what that parameter
        // does, which is the picture the fader itself stands under.
        let Kind::Enumerated(table) = ParamId::Mod1Destination.kind() else {
            unreachable!("a destination is chosen from a table")
        };
        let narrowest = table
            .table_for(DEFAULT_FIRMWARE)
            .values_naming(ParamId::VcfFrequency)
            .next()
            .expect("the matrix reaches the filter");
        let byte = u8::try_from(narrowest).expect("a byte");
        let corner = super::cell_to(ParamId::Mod1Destination, byte, DEFAULT_FIRMWARE)
            .expect("a filter corner is drawn");
        assert_eq!(
            Some(corner),
            ParamId::VcfFrequency.glyph().map(Glyph::pixels),
            "the destination drew something other than its parameter"
        );
    }

    #[test]
    fn the_glass_is_as_deep_as_the_rows_it_stands_beside() {
        // The two are one drawing: a patch bay that stopped two rows short of
        // the table it is a picture of would be a picture of something else.
        let dots = deep(8);
        let points = f32::from(u16::try_from(dots).expect("a count of dots")) * crate::PITCH;
        let table = super::HEADING + 8.0 * super::ROW + 7.0 * super::DOWN;

        assert!(
            (table - points).abs() < crate::PITCH + 8.0,
            "the glass is {points} points against a table of {table}"
        );
    }

    #[test]
    fn a_source_with_three_destinations_is_one_cell_and_three_wires() {
        // The whole of why the bay exists. The table says it three times, once
        // per row; the glass says it once, as a fan, with each of the three
        // lines leaving that one cell saying how much it carries.
        let wires = wired(&[
            ("LFO 1", "VCF Freq"),
            ("LFO 1", "VCA Level"),
            ("LFO 1", "Pitch"),
        ]);
        let screen = shown(&wires);
        let depths = vec![super::cell_deep(3)];
        let top = super::spread(super::MARGIN * 2 + super::LINE, deep(8), &depths)
            .first()
            .copied()
            .expect("a source stands somewhere");

        // One cell on the left, three lines written down it, and a knot on the
        // edge beside each of them.
        for (line, wire) in wires.iter().enumerate() {
            let y = super::reading_at(top, line) + super::LINE / 2;
            assert!(
                screen.is_inked(super::MARGIN + CELL_ACROSS - 1, y),
                "no wire leaves the cell at line {line}:\n{}",
                printed(&screen)
            );
            let said = format!("{} {}", wire.number, wire.depth);
            let across = crate::Screen::width_of(&said, crate::Size::Small);
            let at = super::MARGIN + CELL_ACROSS - super::GUTTER - across;
            assert!(
                (0..super::LINE).any(|row| {
                    (0..across).any(|column| {
                        screen.is_inked(at + column, super::reading_at(top, line) + row)
                    })
                }),
                "nothing says what line {line} carries:\n{}",
                printed(&screen)
            );
        }
        // And three cells on the right rather than one, because three
        // destinations are three destinations.
        let arriving = super::spread(
            super::MARGIN * 2 + super::LINE,
            deep(8),
            &[super::cell_deep(0); 3],
        );
        assert_eq!(arriving.len(), 3);
        for top in arriving {
            assert!(
                screen.is_inked(BAY - super::MARGIN - CELL_ACROSS, top),
                "a destination is missing its frame:\n{}",
                printed(&screen)
            );
        }
    }

    #[test]
    fn a_wire_leaves_flat_arrives_flat_and_runs_down_a_track_of_its_own() {
        // Not a line between two points. The glass is as deep as eight rows of
        // controls and the gap is a fifth of that across, so a single sweep
        // between two distant nodes comes out as a near-vertical scratch that
        // could have started anywhere. Flat out, down a track, flat in, and a
        // track each, so that two wires down the same part of the glass are two
        // wires rather than one heavier one.
        let wires = wired(&[("A", "B"), ("A", "C"), ("A", "D"), ("A", "E")]);
        let screen = shown(&wires);
        let under = super::MARGIN * 2 + super::LINE;
        let top = super::spread(under, deep(8), &[super::cell_deep(4)])
            .first()
            .copied()
            .expect("the one source");
        let x = super::MARGIN + CELL_ACROSS - 1;

        // Flat until the corner, which is where it is allowed to start turning.
        let flat = super::TURN - super::CHAMFER;
        for line in 0..4 {
            let y = super::reading_at(top, line) + super::LINE / 2;
            for step in 0..=flat {
                assert!(
                    screen.is_inked(x + step, y),
                    "the wire is already falling {step} dots out of its source:\n{}",
                    printed(&screen)
                );
            }
        }
        // And each one turns down a column of its own rather than sharing one.
        let runs = |at: i32| {
            (0..screen.rows())
                .filter(|row| screen.is_inked(at, *row))
                .count()
        };
        for lane in [0, 3] {
            let track = x + super::TURN + lane * super::LANE;
            assert!(
                runs(track) > usize::try_from(super::CHAMFER * 2).unwrap_or(0),
                "nothing runs down the track at {track}:\n{}",
                printed(&screen)
            );
        }
    }

    #[test]
    fn a_swap_trades_two_routings_parameter_for_parameter() {
        // Six bytes and three pairs, and the pairs are the ones that mean the
        // same thing: a source for a source, a destination for a destination,
        // a depth for a depth. A swap that paired them by position in the
        // program would be a swap that put a depth where a source goes.
        let routings = of(Group::ModMatrix).expect("the matrix");
        let [one, other] = [
            routings.first().copied().expect("a first"),
            routings.get(1).copied().expect("a second"),
        ];

        assert_eq!(
            one.parameters(),
            [
                ParamId::Mod1Source,
                ParamId::Mod1Destination,
                ParamId::Mod1Depth
            ]
        );
        assert_eq!(
            other.parameters(),
            [
                ParamId::Mod2Source,
                ParamId::Mod2Destination,
                ParamId::Mod2Depth
            ]
        );
    }

    #[test]
    fn nothing_is_drawn_for_a_matrix_nobody_has_wired() {
        // The instrument ships with all eight sitting on `Off`, and eight lines
        // from `Off` to `Off` is a picture of nothing drawn eight times.
        // The heading is still printed, because it says nothing is wired, which
        // is a reading rather than a drawing, so what has to be empty is the
        // glass under it.
        let screen = shown(&[]);
        let under = super::MARGIN * 2 + super::LINE;

        assert!(
            (under..screen.rows())
                .all(|row| (0..screen.columns()).all(|column| !screen.is_inked(column, row))),
            "something is drawn for a matrix nobody has wired:\n{}",
            printed(&screen)
        );
    }

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
    fn the_glass_says_where_the_whole_group_lives() {
        let routings = of(Group::ModMatrix).expect("the matrix");
        let first = routings.first().copied().expect("a first routing");
        let last = routings.last().copied().expect("a last routing");

        // Twenty-four bytes in one run, said once in the heading rather than
        // three at a time down the left-hand edge of the table: nobody edits a
        // program by offset, and the one thing worth saying about where these
        // bytes are is where they start and where they stop.
        let run = format!("{}-{}", first.source.offset(), last.depth.offset());
        assert_eq!(super::Routing::run(&routings), run);
        assert_eq!(first.depth.offset(), first.source.offset() + 2);
    }

    #[test]
    fn a_routing_is_numbered_by_what_the_library_calls_it() {
        let routings = of(Group::ModMatrix).expect("the matrix");

        // The digits off the end of the name, so that a library which numbered
        // them differently is what the row prints.
        for (index, routing) in routings.iter().enumerate() {
            assert_eq!(routing.number(), (index + 1).to_string());
        }
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

    #[test]
    fn a_destination_names_the_parameter_it_moves() {
        use deepmind_midi::ids::ProtocolVersion;
        use deepmind_midi::param::DEFAULT_FIRMWARE;
        use deepmind_midi::program::Program;

        use crate::Patch;

        let mut patch = Patch::new();
        // Nothing is read, so nothing is moved: a mark drawn from a value this
        // window has not seen says the instrument is doing something it may not.
        assert!(moved(&patch, DEFAULT_FIRMWARE).is_empty());

        patch.confirm(Program::new(ProtocolVersion::V7));
        let routings = of(Group::ModMatrix).expect("the matrix is one");
        let destination = routings.first().expect("eight of them").destination;
        let Kind::Enumerated(table) = destination.kind() else {
            unreachable!("a destination is chosen from a table")
        };
        // Whatever the first named destination moves, mapping a routing at it
        // is what puts that parameter in the answer.
        let entry = table
            .table_for(DEFAULT_FIRMWARE)
            .entries
            .iter()
            .find(|entry| {
                !table
                    .table_for(DEFAULT_FIRMWARE)
                    .parameters_of(entry.value)
                    .is_empty()
            })
            .expect("some destination moves a parameter");
        let expected = table
            .table_for(DEFAULT_FIRMWARE)
            .parameters_of(entry.value)
            .to_vec();
        patch.edit(
            destination,
            u8::try_from(entry.value).expect("a destination is a byte"),
        );

        let answered = moved(&patch, DEFAULT_FIRMWARE);
        for parameter in expected {
            assert!(
                answered.contains(&parameter),
                "{parameter:?} is moved by the routing pointed at it"
            );
        }
    }
}

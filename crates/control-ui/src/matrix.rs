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
//! answered, which is the same call the rack makes when it draws that parameter
//! — and where the library has no complete table, the row keeps whatever
//! control the library says the parameter is, with nothing here to decide.
//!
//! **Mapping**, for somebody who knows the *control*. `map` sends the
//! routing out into the window: every control the matrix can reach lights up on
//! all three surfaces, everything else is passed over, and taking hold of one
//! is the answer — a click chooses it, and a drag sets the depth as well, from
//! how far the drag would have moved it. See [`Mapper`](crate::Mapper), which is also
//! where the one assumption on this page is written down.
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
//! `Source` in it — the oscillators have one — is not a matrix and is not
//! drawn as one.
//!
//! The same rule covers where a routing may be mapped. Which controls light up
//! is `ValueEntry::parameters` read backwards — the destinations that name the
//! parameter under the pointer — so a firmware that moves a destination lights
//! a different set of controls with nothing in this file to edit.

use deepmind_midi::param::{Group, Kind, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Vertical;
use iced_core::{Background, Font, Length, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{Space, button, column, combo_box, container, row, text};

use crate::glyphs;
use crate::lcd::{self, Band, Ink, Screen, Size};
use crate::mapping::{Mapper, Mapping, Reach, Sent};
use crate::panel::{Choice, Message, Room, choices, control, readout};
use crate::style::{self, chrome, field, materials, shortlist};
use crate::{Confidence, Element, Patch, tint};

/// How much room the two presses that move a routing are given.
///
/// This was the routing's own number, and before that its number over the three
/// addresses it occupies. Neither needed to be there: the footer already names
/// whatever is under the pointer, which for any control in this table is
/// `Mod 4 Depth`, and a number printed eight times down the edge of a page is a
/// number somebody reads once.
///
/// What the room is spent on instead is the one thing a matrix of eight
/// identical slots gives nobody a way to do.
const LABEL: f32 = 20.0;

/// How much room a source is chosen in.
const SOURCE: f32 = 108.0;

/// How much room a destination is chosen in.
///
/// Wider than a source, because there are 133 of them and their names are the
/// long ones: `VCF Envelope Attack` has to be readable to be chosen.
const DESTINATION: f32 = 158.0;

/// How long the depth fader's travel is.
///
/// Shorter than it was, because the glass beside the rows wanted the room and
/// wanted it more: a depth is one byte and ninety points of travel is already
/// finer than a hand can be, where the patch bay is eight routings and cannot
/// be read at all if the two columns of names meet in the middle.
const DEPTH: f32 = 80.0;

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
    /// third. Read rather than counted, so that a library that numbered them
    /// differently — or named them something other than `Mod` — is what the
    /// row prints.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Wire {
    /// Which of the group's routings this is: `3`.
    number: &'static str,
    /// The source's name, as the display prints it.
    from: &'static str,
    /// The destination's, the same way.
    to: &'static str,
    /// The weaker of the two claims, because a line drawn between a fact and a
    /// guess is a guess.
    claim: Confidence,
}

/// Returns the routings the patch has actually wired, in the order the matrix
/// reads them.
///
/// Both names are the library's own value tables, asked for the firmware that
/// answered — the same call the row above makes when it draws either as a list.
/// A routing with nothing at one end is not a wire and is not drawn: the
/// instrument ships with all eight sitting on `Off`, and eight lines from `Off`
/// to `Off` is a picture of nothing drawn eight times.
pub(crate) fn wiring(patch: &Patch, firmware: Version) -> Vec<Wire> {
    let named = |parameter: ParamId| -> Option<&'static str> {
        let value = patch.value(parameter)?;
        let name = choices(parameter, firmware, Some(value))?
            .into_iter()
            .find(|choice| choice.byte() == value)?
            .name();
        // `Off` is the instrument saying this end is not wired, and it is the
        // library's own word for it rather than this window's.
        (!name.eq_ignore_ascii_case("off")).then_some(name)
    };
    Group::ORDER
        .iter()
        .copied()
        .filter_map(of)
        .flatten()
        .filter_map(|routing| {
            Some(Wire {
                number: routing.number(),
                from: named(routing.source)?,
                to: named(routing.destination)?,
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
/// be. A depth nobody has read is the same — the destination may be known and
/// the amount not, and a band of an assumed width on a known destination is the
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
            for parameter in table.table_for(firmware).parameters_of(u16::from(value)) {
                reaches.push(Reach::new(*parameter, routing.label, routing.depth, depth));
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
        heading(first.destination, DESTINATION + POINT + 6.0),
        heading(first.depth, DEPTH),
    ]
    .spacing(10)
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
                cell(patch, routing.source, firmware, Room::listed(SOURCE), sent),
                arrow(),
                going(patch, routing, firmware, mapper, sent),
                cell(patch, routing.depth, firmware, Room::across(DEPTH), sent),
            ]
            .spacing(10)
            .align_y(Vertical::Bottom);
            container(reading)
                .height(Length::Fixed(ROW))
                .align_y(Vertical::Bottom)
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
    Some(
        row![
            read,
            container(bay(patch, firmware, deep(routings.len()), &routings))
                .width(Length::Fixed(lcd::room(BAY))),
        ]
        .spacing(BESIDE)
        .align_y(Vertical::Top)
        .into(),
    )
}

/// How much panel there is between the eight rows and the glass beside them.
const BESIDE: f32 = 14.0;

/// How tall one routing's row stands.
///
/// Fixed rather than however tall the tallest thing in it happens to be,
/// because the glass beside the rows has to be exactly as deep as they are:
/// a patch bay that stopped two rows short of the table it is a picture of
/// would be a picture of something else. It is the tallest a row needs — a
/// control and the reading under it — and every row is that whatever is in it,
/// which is the rule a rack's slots already follow.
const ROW: f32 = 65.0;

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
/// what somebody wants to know about a modulation matrix is the *shape* of it —
/// that one LFO is driving three things, that two routings are fighting over
/// the filter, that the aftertouch goes nowhere. Sources down one side,
/// destinations down the other, a line for every routing between them, and the
/// fan-out is the picture.
///
/// It is on the instrument's own glass, at the pitch every display in this
/// window shares, and it takes the band it is given — more dots, not bigger
/// ones, which is the rule the chain's glass is cut under too.
///
/// # What is not drawn yet
///
/// A **7 by 7 cell for each name**, which is what the instrument's own display
/// would have room for and what would turn two columns of abbreviations into
/// two columns of pictures: an LFO's wave, an envelope's corner, a wheel, a
/// filter's knee. Those are the library's to publish for the same reason the
/// effect families' marks were — a mark for `LFO 1` is a fact about the
/// instrument and a drawing invented here would be this window making one up.
/// Asked for in
/// [deepmind-midi#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40),
/// recorded in [`docs/waiting.md`](https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/waiting.md),
/// and until it lands the cells are the names the display already prints.
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

    let ends = |pick: fn(&Wire) -> &'static str| -> Vec<&'static str> {
        let mut ends: Vec<&'static str> = Vec::new();
        for name in wires.iter().map(pick) {
            if !ends.contains(&name) {
                ends.push(name);
            }
        }
        ends
    };
    let (sources, destinations) = (ends(|wire| wire.from), ends(|wire| wire.to));
    // Which routings touch each end, so that a destination three of them are
    // fighting over says `1 4 7` rather than making somebody trace three wires
    // back. It is the number the row beside it is printing, which is what makes
    // the two one page rather than two.
    let using = |name: &'static str, pick: fn(&Wire) -> &'static str| -> String {
        wires
            .iter()
            .filter(|wire| pick(wire) == name)
            .map(|wire| wire.number)
            .collect::<Vec<_>>()
            .join(" ")
    };
    let head = MARGIN * 2 + LINE;
    let left = Column::new(MARGIN, head, deep, sources.len());
    let right = Column::new(BAY - MARGIN - CELL_ACROSS, head, deep, destinations.len());
    for (index, name) in sources.iter().enumerate() {
        left.cell(
            &mut screen,
            index,
            name,
            &using(name, |wire| wire.from),
            Side::From,
        );
    }
    for (index, name) in destinations.iter().enumerate() {
        right.cell(
            &mut screen,
            index,
            name,
            &using(name, |wire| wire.to),
            Side::To,
        );
    }
    // A wire for every routing, bending out of one node and into the other. A
    // straight line between two cells three quarters of a glass apart is a
    // diagonal that crosses every other diagonal at the same angle and says
    // nothing about which end is which; a wire that leaves flat, turns, and
    // arrives flat reads as going from somewhere to somewhere, and two wires
    // leaving the same node are visibly two wires leaving the same node.
    for (lane, wire) in wires.iter().enumerate() {
        let (Some(from), Some(to)) = (
            sources.iter().position(|name| *name == wire.from),
            destinations.iter().position(|name| *name == wire.to),
        ) else {
            continue;
        };
        bend(
            &mut screen,
            left.knot(from),
            right.knot(to),
            i32::try_from(lane).unwrap_or(0),
        );
    }
    screen
}

/// How tall one line of writing is on the glass.
const LINE: i32 = 7;

/// Which end of a wire a cell is, which decides where its knot sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// A source: the wire leaves its right-hand edge.
    From,
    /// A destination: the wire arrives at its left-hand edge.
    To,
}

/// One column of the bay: where its cells stand and where its wires attach.
#[derive(Debug, Clone, Copy)]
struct Column {
    /// The left edge of every cell in it.
    at: i32,
    /// How far apart two cells are, top to top.
    apart: i32,
    /// Where the first one starts.
    first: i32,
}

impl Column {
    /// A column of `count` cells, spread down `deep` dots of glass.
    ///
    /// Spread rather than stacked, because the room is what the curves are for:
    /// three sources against a glass as deep as eight rows of controls leave
    /// eighty dots between them, and eighty dots is a wire that visibly goes
    /// somewhere. Stacked at the top they would be three cells and a lot of
    /// empty glass, with the same three diagonals in the corner of it.
    fn new(at: i32, under: i32, deep: i32, count: usize) -> Self {
        let count = i32::try_from(count).unwrap_or(1).max(1);
        let room = deep - under - MARGIN - CELL_DEEP;
        // As far apart as the glass allows, up to a limit: three sources spread
        // over a glass as deep as eight rows of controls are three cells in the
        // corners of an empty screen with three long verticals between them,
        // which is a picture of the glass rather than of the patch. Capped,
        // they are a group — and the group is centred, because a column of
        // three is not eight rows with five missing.
        let apart = if count > 1 {
            (room / (count - 1)).min(PITCH)
        } else {
            0
        };
        let first = under + (room - apart * (count - 1)) / 2;
        Self { at, apart, first }
    }

    /// Where the `index`th cell's top edge is.
    fn top(self, index: usize) -> i32 {
        self.first + i32::try_from(index).unwrap_or(0) * self.apart
    }

    /// Draws one cell: its frame, the box its picture will stand in, its name,
    /// and the knot a wire attaches to.
    fn cell(self, screen: &mut Screen, index: usize, name: &'static str, using: &str, side: Side) {
        let top = self.top(index);
        let frame = Band::new(self.at, top, CELL_ACROSS, CELL_DEEP);
        screen.frame(frame, Ink::Solid);
        // Where the library's own picture of this source or destination will
        // go: seven dots by seven, which is the cell this display writes a
        // character in. Dotted, because an empty solid box is a box and a
        // dotted one is a box waiting for something — and nothing is drawn in
        // it, because a picture of `LFO 1` is a fact about the instrument and
        // one invented here would be this window making one up. Asked for in
        // deepmind-midi#40.
        screen.frame(
            Band::new(self.at + 2, top + 2, PICTURE, PICTURE),
            Ink::Dotted,
        );
        let written: String = name
            .chars()
            .take(usize::try_from(LETTERS).unwrap_or(0))
            .collect();
        screen.write(
            self.at + 2 + PICTURE + GUTTER,
            top + 3,
            &written,
            Size::Small,
        );
        // And which routings use it, against the far edge. A destination three
        // of them are fighting over says `1 4 7` rather than making somebody
        // trace three wires back to find out, and the numbers are the ones the
        // rows beside the glass are printing — which is what makes the table
        // and the picture one page rather than two.
        let numbers = Screen::width_of(using, Size::Small).min(NUMBERS);
        screen.write(
            self.at + CELL_ACROSS - 2 - numbers,
            top + 3,
            using,
            Size::Small,
        );
        // The knot: a blob on the edge the wire leaves or arrives at, so that
        // three wires out of one source are visibly three wires out of one
        // source rather than three lines that happen to converge.
        let (x, y) = self.knot(index);
        let out = match side {
            Side::From => 0,
            Side::To => -(KNOT - 1),
        };
        screen.fill(Band::new(x + out, y - KNOT / 2, KNOT, KNOT));
    }

    /// Where a wire attaches to the `index`th cell.
    fn knot(self, index: usize) -> (i32, i32) {
        let x = match self.at {
            // The left-hand column's wires leave its right-hand edge; the
            // right-hand column's arrive at its left.
            at if at == MARGIN => at + CELL_ACROSS - 1,
            at => at,
        };
        (x, self.top(index) + CELL_DEEP / 2)
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
///
/// A track each, so that two wires down the same part of the glass are two
/// wires rather than one heavier one — which is the whole question somebody
/// looks at a patch bay to answer.
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
/// as mitred — the most a dot matrix can say about a radius.
const CHAMFER: i32 = 3;

/// How many characters of a name a cell has room for./// How many characters of a name a cell has room for.
///
/// A display clips, which is what a display does and what this one is a picture
/// of. Ten is what the instrument's own screen prints for all but a handful:
/// `Pitch Bend`, `BreathCtrl` and `VCF Freq` are ten, nine and eight, and the
/// few that run past lose their tail rather than push the two columns into each
/// other.
const LETTERS: i32 = 9;

/// How wide the box a name's picture will stand in is, and how deep.
///
/// Seven by seven, which is the cell this display writes a character in and the
/// size asked of the library in deepmind-midi#40.
const PICTURE: i32 = 7;

/// How wide a cell is: its frame, the picture, the name, and the routings
/// that use it.
const CELL_ACROSS: i32 =
    2 + PICTURE + GUTTER + (LETTERS * glyphs::ADVANCE - glyphs::GAP) + APART + NUMBERS + 2;

/// How deep a cell is: its frame, and the picture with a dot of glass either
/// side of it.
const CELL_DEEP: i32 = PICTURE + 4;

/// How much glass there is between a cell's picture, its name and its frame.
const GUTTER: i32 = 3;

/// How much glass there is between the longest name a cell holds and the
/// routings printed after it.
///
/// Wider than the gutter before the name, because a name that fills its field
/// ends where the numbers begin and two runs of writing that touch are one run
/// of writing.
const APART: i32 = 5;

/// How much room a cell keeps for the routings that use it.
///
/// Three of them. A destination four routings reach is a patch somebody built
/// on purpose and will recognise from the three it does print; a cell sized for
/// all eight is a cell that is mostly empty on every patch anybody writes.
const NUMBERS: i32 = 3 * glyphs::ADVANCE - glyphs::GAP;

/// How big the blob a wire attaches to is.
const KNOT: i32 = 3;

/// The most glass there is between two cells in a column, top to top.
///
/// A cell and twice the same again: eight of them fill a glass as deep as the
/// eight rows beside it, which is the case this is laid out for, and three of
/// them are a group in the middle of one rather than three cells in its
/// corners.
const PITCH: i32 = CELL_DEEP * 3;

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
/// be moved anywhere — there is nothing to move, and writing a value this window
/// has not seen into a slot is the one thing it does not do.
fn shifts<'a, Renderer>(patch: &Patch, routings: &[Routing], index: usize) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let routing = routings.get(index).copied();
    let press = |label: &'static str, towards: Option<usize>| {
        let swap = routing.zip(towards.and_then(|at| routings.get(at).copied()));
        let known = |routing: Routing| {
            !matches!(
                patch.claim_across(routing.parameters()),
                Confidence::Unknown
            )
        };
        button(text(label).size(9).center())
            .width(Length::Fixed(LABEL))
            .height(Length::Fixed(SHIFT))
            .padding(0)
            .style(chrome)
            .on_press_maybe(
                swap.filter(|(one, other)| known(*one) && known(*other))
                    .map(|(one, other)| Message::Swap {
                        one: one.parameters(),
                        other: other.parameters(),
                    }),
            )
    };
    column![
        press("\u{25b2}", index.checked_sub(1)),
        press(
            "\u{25bc}",
            Some(index + 1).filter(|at| *at < routings.len())
        ),
    ]
    .spacing(2)
    .into()
}

/// How tall one of those presses stands.
///
/// Two of them and the glass between are a control's own height, so the pair
/// stands in the row the way everything else in it does.
const SHIFT: f32 = 15.0;

/// Draws where a routing goes: the name, the search it is found in, and the/// Draws where a routing goes: the name, the search it is found in, and the
/// press that asks the window instead.
///
/// Two ways to answer one question, side by side, because they are good at
/// opposite things. Typing is how somebody who knows the name of the thing
/// finds it among 133 — three letters and `VCF Envelope Attack` is the only one
/// left. Mapping is how somebody who knows the *control* finds it: the name of
/// a destination is an abbreviation the instrument's display prints, and
/// knowing which abbreviation stands over the fader you have in mind is the
/// whole of what makes this column hard.
fn going<'a, Renderer>(
    patch: &Patch,
    routing: Routing,
    firmware: Version,
    mapper: &'a Mapper,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let destination = routing.destination;
    let mapping = mapper
        .mapped()
        .is_some_and(|mapped| mapped.destination() == destination);
    let press = button(text(if mapping { "stop" } else { "map" }).size(10).center())
        .width(Length::Fixed(POINT))
        .height(Length::Fixed(crate::panel::BUTTON))
        .padding(0)
        .style(move |theme: &Theme, status| {
            if mapping {
                lamp(theme)
            } else {
                chrome(theme, status)
            }
        })
        .on_press(Message::Mapper(
            (!mapping).then(|| Mapping::new(routing.label, destination, routing.depth)),
        ));
    // A searchable list where the library names every value the parameter
    // accepts, and whatever the library says the parameter is where it does
    // not. Which of the two it is, is not a decision this file makes: the list
    // exists exactly where [`Mapper`] could build one, which is where
    // `ParamId::choices_for` answered.
    let chosen: Element<'a, Renderer> = match mapper.list(destination) {
        Some(list) => {
            let value = patch.value(destination);
            let selected = choices(destination, firmware, value).and_then(|options| {
                options
                    .into_iter()
                    .find(|choice| Some(choice.byte()) == value)
            });
            container(
                combo_box(list, "search", selected.as_ref(), move |choice: Choice| {
                    Message::Edit {
                        parameter: destination,
                        value: choice.byte(),
                    }
                })
                .size(11)
                .padding([2, 6])
                .menu_height(Length::Fixed(MENU))
                .input_style(field)
                .menu_style(shortlist)
                .width(Length::Fixed(DESTINATION)),
            )
            .height(Length::Fixed(crate::panel::BUTTON))
            .align_y(Vertical::Center)
            .into()
        }
        None => cell(
            patch,
            destination,
            firmware,
            Room::listed(DESTINATION),
            sent,
        ),
    };
    column![
        row![chosen, press].spacing(6).align_y(Vertical::Center),
        readout(
            destination,
            patch.value(destination),
            patch.claim(destination),
            firmware
        ),
    ]
    .spacing(3)
    .width(Length::Fixed(DESTINATION + POINT + 6.0))
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

/// The style the press wears while its own routing is the one being mapped.
///
/// The one saturated colour on the panel, which is what every control the
/// routing can reach is outlined in at the same moment: the press and the lit
/// controls are one thing happening, so they are one colour.
fn lamp(theme: &Theme) -> button::Style {
    button::Style {
        background: Some(Background::Color(style::MODULATION)),
        text_color: materials(theme).panel,
        border: border::rounded(3).width(1.0).color(style::MODULATION),
        ..button::Style::default()
    }
}

/// How much room the press that points a routing at the window takes.
const POINT: f32 = 44.0;

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
    column![
        control(parameter, value, claim, firmware, room, sent),
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
    .align_y(Vertical::Center)
    .into()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::{Group, Kind, ParamId};

    use super::{moved, of, routed};

    use super::{BAY, CELL_ACROSS, CELL_DEEP, Wire, deep, drawn};
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
                from,
                to,
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
        // per row; the glass says it once, as a fan.
        let screen = shown(&wired(&[
            ("LFO 1", "VCF Freq"),
            ("LFO 1", "VCA Level"),
            ("LFO 1", "Pitch"),
        ]));
        let ink = |band: crate::Band| {
            (0..band.height)
                .flat_map(|row| (0..band.width).map(move |column| (column, row)))
                .filter(|(column, row)| screen.is_inked(band.x + column, band.y + row))
                .count()
        };
        // One cell on the left, three on the right, and the glass between them
        // carrying three wires rather than one.
        let gap = crate::Band::new(
            super::MARGIN + CELL_ACROSS,
            0,
            BAY - super::MARGIN * 2 - CELL_ACROSS * 2,
            screen.rows(),
        );
        assert!(
            ink(gap) > 0,
            "nothing crosses the glass between the columns:\n{}",
            printed(&screen)
        );
        // The one source sits in the middle of its column rather than at the
        // top, because a column of one is not the top of a list.
        let middle = screen.rows() / 2;
        assert!(
            (0..CELL_DEEP).any(|row| screen.is_inked(super::MARGIN, middle - CELL_DEEP / 2 + row)),
            "the lone source is not in the middle:\n{}",
            printed(&screen)
        );
    }

    #[test]
    fn a_wire_leaves_flat_arrives_flat_and_runs_down_a_track_of_its_own() {
        // Not a line between two points. The glass is as deep as eight rows of
        // controls and the gap is a fifth of that across, so a single sweep
        // between two distant nodes comes out as a near-vertical scratch that
        // could have started anywhere. Flat out, down a track, flat in — and a
        // track each, so that two wires down the same part of the glass are two
        // wires rather than one heavier one.
        let dots = deep(8);
        // One source and four destinations, so that the two wires under test
        // are the ones furthest apart and their vertical runs are long enough
        // to tell from a corner.
        let screen = shown(&wired(&[("A", "B"), ("A", "C"), ("A", "D"), ("A", "E")]));
        let under = super::MARGIN * 2 + super::LINE;
        let left = super::Column::new(super::MARGIN, under, dots, 1);
        let right = super::Column::new(BAY - super::MARGIN - CELL_ACROSS, under, dots, 4);
        let (x, y) = left.knot(0);

        // Flat until the corner, which is where it is allowed to start turning.
        let flat = super::TURN - super::CHAMFER;
        for step in 0..=flat {
            assert!(
                screen.is_inked(x + step, y),
                "the wire is already falling {step} dots out of its source:\n{}",
                printed(&screen)
            );
        }
        for index in 0..4 {
            let (to, row) = right.knot(index);
            for step in 0..=flat {
                assert!(
                    screen.is_inked(to - step, row),
                    "the wire is still falling {step} dots short of its destination:\n{}",
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
        // The first and the last, which are the two furthest from the source
        // and so the two with a run long enough to tell from a corner.
        for lane in [0, 3] {
            let track = x + super::TURN + lane * super::LANE;
            assert!(
                runs(track) > usize::try_from(super::PITCH).unwrap_or(0),
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
        // The heading is still printed — it says nothing is wired, which is a
        // reading rather than a drawing — so what has to be empty is the glass
        // under it.
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

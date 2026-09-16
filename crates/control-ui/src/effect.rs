//! The effects: four engines, and what the algorithm in one says its twelve
//! bytes are.
//!
//! Fifty-eight parameters, and fifty-six of them are `FX 1 Param 3`. That name
//! is the truth about the wire and useless on a panel: the byte is `Size` on a
//! Room Reverb, `Depth` on a Phaser, and nothing at all on a TC Deep Reverb,
//! which uses five of its twelve. A rack of fifty-eight slots under those names
//! is complete, honest and unreadable, which is what stage 3 left here and what
//! this replaces.
//!
//! `deepmind-midi` 26.2 published the table that settles it. [`Engine`] is which
//! parameters one engine addresses, [`Algorithm`] is the 35 and which byte
//! selects one on a given firmware, and [`FxSlot`] is what a slot of the loaded
//! algorithm is called. So the panel is four plates, each of them the engine's
//! own settings and then its slots under the names the algorithm gives them.
//!
//! # Nothing here is transcribed
//!
//! Which group holds the engines is [`Engine::algorithm_parameter`]'s own group,
//! not a name written down. Which slot a parameter is under the loaded algorithm
//! is [`Algorithm::slot_of`]. Which algorithm a byte selects is
//! [`Algorithm::for_value`] for the firmware a device inquiry reported, because
//! firmware 1.1 inserted Vintage Pitch rather than appending it and 33 is Rotary
//! Speaker on 1.0. A fifth engine, a thirty-sixth algorithm or a renamed slot
//! arrives with nothing in this file to edit.
//!
//! # The page is the instrument's own, and so is the grid
//!
//! 26.3 published the rest of it
//! ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)):
//! the grid the synthesizer's own FX page lays a slot out on, measured off the
//! 35 screenshots in the manual, and what the figure printed beside each
//! algorithm is made of. Six columns and two rows, and
//! [`FxSlot::position`] says which cell a slot is drawn in.
//!
//! The cell is what this draws it in. Six columns, each the same width and each
//! there whether or not a slot stands in it, spread across the whole page — so
//! the third column of the second row stands under the third column of the
//! first, and an algorithm using five slots on one row and six on the next
//! draws the five where the hardware draws them. Reading the grid as the *order*
//! of a row instead, and packing each row against the left at the width of a
//! rack's slot, is what left two rows that did not line up in the left half of
//! an empty plate.
//!
//! [`Panel::control`] is the shape: 29 of the 35 figures are rotary knobs, five
//! are faders and one is a numeric display, and a slot on a knob panel is drawn
//! as [a knob](crate::knob).
//!
//! # Four at once, and what that costs the livery
//!
//! All four engines are on the page together, in the two-by-two the four of
//! them make. One at a time behind a row of tabs was an engine you could see
//! and three you had to remember, on a page whose whole subject is what four
//! effects are doing *together* — and the chain drawn across the top of it is a
//! picture of exactly that. Each engine's own case carries the strip that says
//! what it is running, which is also the list that changes it, and how loud it
//! comes out; there is no separate header, because a header was a second place
//! saying which algorithm an engine was running and neither place was the
//! engine.
//!
//! The library publishes four measured colours per algorithm. How many of them
//! this page can afford depends on how many units stand on it. With one open at
//! a time it wore the lot — case, face and cap — because a single unit can be
//! that unit without the window becoming a shelf of other people's boxes. Four
//! stand on it now, and four liveries side by side are exactly the collage this
//! editor's whole argument is against. So the face went back to the window's
//! own plate and the cap back to the window's own metal, and what an engine
//! wears is its case and a hairline of its accent: enough to tell the reverb
//! from the distortion at arm's length, which is all four units at once can
//! afford to say.
//!
//! # Nothing is printed in a colour that cannot be read on what is under it
//!
//! Half of the 35 measured chassis are pale and half are dark, and no word on
//! this page names an ink of its own. It names the surface it lands on — the
//! face, a case, a band's strip, the recess the chain is cut into — and
//! [`ink`] and [`legend`] answer with whichever of the instrument's two can be
//! read there, measured rather than judged. A reading keeps the colour of its
//! claim, because that is what it means, and is lifted until it clears the
//! same threshold: amber stays amber and green stays green, and what moves is
//! how light it is.
//!
//! # A named slot is the same control it was
//!
//! The library says two things about a slot and they are not the same thing.
//! `FxSlot::kind` is how the *display* reads the byte, and the parameter table
//! is what the byte *is*: `Freeze` is two states on the panel and a parameter
//! that accepts 256 values on the wire, and the curve between the two is not
//! published. So the control is still [`control`] over the parameter's own
//! range, chosen by the same code from the same table as every other slot in
//! the editor, and what the algorithm supplies is the naming: the title, the
//! abbreviation the instrument's display prints, the band the slot belongs to,
//! and the two ends of the reading it shows. Hand layout changes the arrangement
//! and never what a control is, and that rule is what keeps this panel from
//! inventing a mapping the manual does not give.
//!
//! For the same reason a slot whose display shows names — `Ambience`, `Church`,
//! `Gate` — is not drawn as a list. The manual prints those names and never the
//! bytes they sit at, [`FxSlot::values`] says so, and a list that sent one of
//! them would be sending a guess. The names are printed under the plate as what
//! the display will show, and the byte stays draggable.
//!
//! # Twelve bytes, however many the algorithm uses
//!
//! An engine holds twelve whatever it is running, and
//! [`Algorithm::slots`](deepmind_midi::effect::Algorithm::slots) is as short as
//! five. The ones the algorithm has a name for are drawn as that name; the rest
//! are drawn at the end of the plate under the library's own `Fx 1 Param 6`,
//! marked as doing nothing, because a byte in the program that no panel reaches
//! is a byte the modulation matrix can still be pointed at. An engine whose
//! algorithm this firmware's table does not name — or whose type nobody has read
//! — draws all twelve that way, which is stage 3's rack for exactly as long as
//! there is nothing better to say.

use deepmind_midi::effect::{Algorithm, Colour, Control, Engine, FxSlot, Panel, grid};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Color, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{Space, column, container, row, text};

use crate::chain;
use crate::fader;
use crate::panel::{Room, control, lit_rather_than_listed, modulated, shown};
use crate::style::{self, materials, reading as reading_face};
use crate::{Confidence, Element, Patch, tint};

/// How much room one of the group's own settings is chosen in.
///
/// `Parallel 1/2, parallel 3/4` is what the routing's names read like, and a
/// list that clips one is a list that offers ten indistinguishable topologies.
const SETTING: f32 = 192.0;

/// How much room one whose choices are lit rather than listed is given.
///
/// `Insert`, `Send` and `Bypass` are three short words in a column of lamps,
/// and the room a list of ten topologies needs is room taken off the display
/// beside it — which is the one thing on this page that has to be wide.
const LIT_SETTING: f32 = 86.0;

/// How much room an engine's algorithm is chosen in.
///
/// The display's own abbreviations — `RoomRev`, `MulBndDist` — because the
/// control is the parameter's value table and not this panel's prose. What the
/// algorithm is called in full is printed beside it.
const ALGORITHM: f32 = 104.0;

/// How long the output gain's fader is, along the top of an engine's case.
const GAIN: f32 = 84.0;

/// How thick it is across its travel.
///
/// A band of the strip rather than a rack's fader lying on its side: the strip
/// is one line of an engine's case, and a control as deep as a rack fader is
/// wide would be most of it.
const GAIN_THICK: f32 = 16.0;

/// Height of the box a slot's title is set in.
///
/// Two lines of it. A column of the grid is about a sixth of half the page, so
/// `Crossover Frequency 1` wraps and `Decay` does not, and both of them have to
/// leave the line under them in the same place. Fixed and set in the middle, so
/// a title of one line sits the same distance under its value as its units sit
/// under it rather than at one end of a gap.
const TITLE: f32 = 26.0;

/// Height of the line a slot's reading is described on.
///
/// One line, always taken whether or not there is anything to say on it, so
/// that the units of one row land on one line however many of the slots above
/// them have any.
const HINT: f32 = 12.0;

/// Height of the strip a band's name is knocked out of.
const BAND: f32 = 13.0;

/// How big the thing a hand takes hold of is drawn on an effect plate.
///
/// Smaller than a rack's slot rather than larger, which is what changed when
/// all four engines came onto one page: this is twenty-four controls, four
/// cases and a chain on one surface, and the rack's forty have a page each.
///
/// The grid publishes the instrument's own proportion — a control 11.9 wide in
/// a column pitched 20, on a 128 point display — and that is not what decides
/// this. It measures a dot matrix drawing its own labels in dots, and it does
/// not survive a title set in a real face. What this window takes from the grid
/// is the arrangement, which is published and exact.
const BODY: f32 = 40.0;

/// How far a fader on one of them runs.
///
/// Shorter than a rack's, the way the instrument's own front panel runs its
/// faders short to hold two rows of them. A knob shrinks in both directions at
/// once and a fader only along its travel, so the two come down to the same
/// weight in a column at different numbers.
const TRAVEL: f32 = 50.0;

/// How big one is drawn where the algorithm does not use the byte.
const SPARE: f32 = 22.0;

/// One of an engine's twelve bytes, and what the loaded algorithm calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lane {
    /// The parameter that addresses the byte, which is the same one whatever
    /// the engine is running.
    parameter: ParamId,
    /// What the algorithm says the byte is, when it uses it at all.
    slot: Option<&'static FxSlot>,
}

/// One row of the instrument's own FX grid: what stands in each of its columns.
///
/// As long as the grid is wide, always, and a column with nothing in it is a
/// column with nothing in it rather than a column that is not there. That is
/// the whole difference between this and a row of slots: six of them are drawn
/// whatever the algorithm uses, so the second row can be read down against the
/// first.
type Line = Vec<Option<Lane>>;

/// Returns the engines `group` holds, when it is the one that holds them.
///
/// Asked of the library rather than written down: an engine's type parameter
/// knows its own group, so the effects are wherever the parameter table puts
/// them and a group with no engine in it is every other group.
pub(crate) fn engines(group: Group) -> Option<Vec<Engine>> {
    let found: Vec<Engine> = Engine::ALL
        .into_iter()
        .filter(|engine| engine.algorithm_parameter().group() == group)
        .collect();
    (!found.is_empty()).then_some(found)
}

/// Returns the parameters this panel draws rather than leaving to the rack.
///
/// The whole group, where the group holds the engines. Four plates of fourteen
/// parameters each are not a layout a rack of two leftovers sits under: the
/// settings that are not an engine's are the first thing on the panel instead,
/// which is where a connection mode belongs and where a parameter a later
/// library adds to this group will appear.
pub(crate) fn claimed(group: Group) -> Vec<ParamId> {
    match engines(group) {
        Some(_) => group.parameters().collect(),
        None => Vec::new(),
    }
}

/// Returns the group's own settings, which are what no engine addresses.
///
/// The connection mode and whether the effects are inserted, sent or bypassed.
/// Found by subtraction rather than by name, so that this is "everything in the
/// effects that is not one of the four engines" and stays that whatever the
/// library adds.
fn settings(group: Group) -> Vec<ParamId> {
    let engines = engines(group).unwrap_or_default();
    group
        .parameters()
        .filter(|parameter| !engines.iter().any(|engine| addresses(*engine, *parameter)))
        .collect()
}

/// Returns whether `engine` addresses `parameter`: its type, its gain, or one
/// of its twelve.
fn addresses(engine: Engine, parameter: ParamId) -> bool {
    engine.algorithm_parameter() == parameter
        || engine.gain_parameter() == parameter
        || engine.slot_parameters().contains(&parameter)
}

/// Returns what `engine` is running, as far as this window knows.
///
/// `None` for a type nobody has read, and for a byte this firmware's table does
/// not name, which is what a dump from a unit running the other firmware can
/// hand over. Both of those are "there is nothing to call these twelve bytes",
/// and drawing them under the library's own names is the honest answer to it.
fn algorithm(patch: &Patch, engine: Engine, firmware: Version) -> Option<&'static Algorithm> {
    let value = patch.value(engine.algorithm_parameter())?;
    Algorithm::for_value(value, firmware)
}

/// Returns the twelve bytes of `engine` and what the loaded algorithm calls
/// each.
pub(crate) fn lanes(patch: &Patch, engine: Engine, firmware: Version) -> Vec<Lane> {
    let algorithm = algorithm(patch, engine, firmware);
    engine
        .slot_parameters()
        .iter()
        .copied()
        .map(|parameter| Lane {
            parameter,
            slot: algorithm.and_then(|algorithm| algorithm.slot_of(engine, parameter)),
        })
        .collect()
}
/// Gathers the lanes onto the grid the instrument's own FX page draws them on.
///
/// Six columns and two rows, and every slot in the column and row
/// [`FxSlot::position`] measured for it. That is the whole of the arrangement:
/// a slot is not placed by counting along a row, it is placed where the
/// instrument places it, so the third column of the second row stands under the
/// third column of the first however many slots either row happens to hold.
///
/// Returns the rows, and whatever has no place on the grid. The second is the
/// bytes the loaded algorithm does not use — as many as seven of the twelve —
/// and, if a later library ever measured a slot outside the grid it publishes,
/// that slot as well. Either way nothing is dropped: a byte this panel does not
/// draw is a byte the modulation matrix can still be pointed at.
///
/// Everything with no place, where nothing is known about the algorithm. That
/// is stage 3's rack: twelve bytes under the library's own names, and no page
/// to lay them out by.
fn placed(lanes: &[Lane], panel: Option<&'static Panel>) -> (Vec<Line>, Vec<Lane>) {
    let columns = usize::from(grid().columns());
    let Some(panel) = panel else {
        return (Vec::new(), lanes.to_vec());
    };
    let mut rows: Vec<Line> = vec![vec![None; columns]; panel.rows().len()];
    let mut spare = Vec::new();
    for lane in lanes.iter().copied() {
        let cell = lane.slot.map(FxSlot::position).and_then(|at| {
            rows.get_mut(usize::from(at.row()))
                .and_then(|row| row.get_mut(usize::from(at.column())))
        });
        match cell {
            // An occupied cell means two slots measured to the same place,
            // which the specification does not contain and which this would
            // rather draw at the end than lose.
            Some(cell) if cell.is_none() => *cell = Some(lane),
            _ => spare.push(lane),
        }
    }
    (rows, spare)
}

/// Returns the runs of columns one band covers, along one row.
///
/// In column order and covering every column, so that a strip built from this
/// and the row of slots under it divide the same width the same way: a run of
/// three labelled columns is one entry three wide, and a column the library
/// labelled nothing is an entry of its own one wide, whether or not the column
/// beside it is also unlabelled. Two unlabelled slots side by side are two
/// slots and not a pair, which is the rule the clusters followed before the
/// grid replaced them, and an empty column is not a band either.
fn spans(line: &Line) -> Vec<(Option<&'static str>, usize)> {
    let mut runs: Vec<(Option<&'static str>, usize)> = Vec::new();
    for cell in line {
        let label = cell.and_then(|lane| lane.slot).and_then(|slot| slot.group);
        match runs.last_mut() {
            Some((last, run)) if *last == label && label.is_some() => *run += 1,
            _ => runs.push((label, 1)),
        }
    }
    runs
}

/// Which of the effects page's surfaces a word is printed on.
///
/// Every colour in this window is chosen for what it means, and what it lands
/// on here is not always chosen at all: an engine's case is a colour somebody
/// measured off a photograph of a rack unit, and half of the 35 are pale. So
/// nothing on this page names an ink directly. It names the surface, and
/// [`ink`] and [`legend`] work out what can be read on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum On {
    /// The face an engine's controls stand on, which is the window's own plate.
    Face,
    /// The case around them, carrying the algorithm's own measured chassis.
    Case(Option<(Colour, Colour)>),
    /// The strip a band's name is knocked out of.
    Strip,
    /// The panel the chain and the unnamed bytes are cut into.
    Recess,
}

impl On {
    /// Returns the colour of the surface itself.
    fn colour(self, theme: &Theme) -> Color {
        let material = materials(theme);
        match self {
            Self::Face => material.plate,
            Self::Case(figure) => chassis(theme, figure),
            Self::Strip => banding(theme),
            Self::Recess => material.panel,
        }
    }
}

/// Returns the ink a word on `on` is printed in.
///
/// Whichever of the instrument's two the surface can be read against. Nothing
/// on this page picks an ink itself, because half the surfaces on it are
/// measured off somebody else's rack unit and a window that printed every title
/// in the one ink it uses everywhere else would have picked the wrong one for
/// half the algorithms.
fn ink(theme: &Theme, on: On) -> Color {
    style::ink_on(on.colour(theme), theme)
}

/// The same ink, at the strength a legend is silkscreened in.
///
/// Half way back to the surface it is printed on and then lifted until it is
/// readable, which is to say: the dimmest a legend can be on this surface and
/// still be a legend. A grey chosen once for a dark window is a grey that
/// disappears on a cream one, and the tint of a value nobody has read — which
/// is what these words used to be drawn in — said of a printed word that its
/// value was unknown.
fn legend(theme: &Theme, on: On) -> Color {
    let surface = on.colour(theme);
    let full = style::ink_on(surface, theme);
    style::legible(style::mix(full, surface, 0.5), surface, theme)
}

/// Grey text at legend strength, on `on`.
fn printing<'a, Renderer>(what: String, on: On) -> iced_widget::Text<'a, Theme, Renderer>
where
    Renderer: TextRenderer,
{
    text(what).style(move |theme: &Theme| text::Style {
        color: Some(legend(theme, on)),
    })
}

/// Draws what a parameter is holding, in the colour of what backs it.
///
/// [`panel::readout`](crate::panel::readout) with the surface taken into
/// account: a claim is amber or green because of what it means, and an engine
/// wearing a cream case is a surface neither of them was chosen against. What
/// moves is how light the colour is and never which colour it is, so an amber
/// claim stays amber and the distinction the whole editor turns on survives
/// being printed on somebody else's rack unit.
fn reading<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
    on: On,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    text(shown(parameter, value, firmware))
        .size(11)
        .font(reading_face())
        .style(move |theme: &Theme| text::Style {
            color: Some(style::legible(tint(theme, claim), on.colour(theme), theme)),
        })
        .into()
}

/// Draws the effects: the chain, what shapes it, and the four engines under it.
///
/// All four at once, in the two-by-two the four of them make. One at a time
/// behind a row of tabs was an engine you could see and three you had to
/// remember, on a page whose whole subject is what four effects are doing
/// together — and the chain drawn across the top of it is a picture of exactly
/// that. A tab row is the right shape for a section of an instrument and the
/// wrong one for four things wired to each other.
pub(crate) fn panels<'a, Renderer>(
    patch: &Patch,
    group: Group,
    firmware: Version,
    moved: &[ParamId],
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let engines = engines(group)?;
    // The picture, and beside it the two settings that shape it. The routing
    // *is* the chain — which engine feeds which is the drawing, and whether the
    // four of them are inserted, sent or bypassed is the rest of the same
    // sentence — so they stand together, and standing beside the display rather
    // than under it is what keeps the whole block one strip deep.
    let mut block = row(settings(group).into_iter().map(|parameter| {
        let room = if lit_rather_than_listed(parameter, firmware) {
            LIT_SETTING
        } else {
            SETTING
        };
        cell(
            patch,
            parameter,
            firmware,
            Room::listed(room),
            parameter.short_name(),
            moved,
            On::Recess,
        )
    }))
    .spacing(10);
    if let Some(note) = chain::note(patch) {
        block = block
            .push(container(printing(note.to_owned(), On::Recess).size(9)).width(Length::Fill));
    }
    let wiring = container(
        row![
            container(chain::display(patch, firmware)).width(Length::Fill),
            container(block).width(Length::Shrink),
        ]
        .spacing(14)
        .align_y(Vertical::Center),
    )
    .padding(8)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let material = materials(theme);
        container::Style {
            background: Some(Background::Color(material.panel)),
            border: Border {
                color: material.recess_edge,
                width: 1.0,
                radius: 3.into(),
            },
            ..container::Style::default()
        }
    });
    let mut body = column![wiring].spacing(8);
    // Two across, in the order the instrument numbers them. `chunks` rather
    // than a pair of indexes so that a library that ever published a fifth
    // engine gets a third row rather than a panel nobody can reach.
    for pair in engines.chunks(2) {
        body = body.push(
            row(pair
                .iter()
                .map(|engine| plate(patch, *engine, firmware, moved)))
            .spacing(8),
        );
    }
    Some(body.into())
}

/// Draws one row of a plate's grid: six columns, each of them a slot or empty.
///
/// Every column is the same width and is there whether or not anything stands
/// in it, which is what makes this a grid rather than six things in a row: an
/// algorithm using five slots on its first row and six on its second draws the
/// five under the first five, where the instrument draws them, instead of
/// spreading them over a row of its own.
fn line_of<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    line: &Line,
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let cells = line.iter().map(|cell| match cell {
        Some(byte) => slot(patch, engine, *byte, firmware, moved, figure),
        None => Element::from(Space::new().width(Length::Fill)),
    });
    row(cells).spacing(0).into()
}

/// Draws the bands printed over one row of the grid.
///
/// The library groups the slots that are one side of a stereo engine or one
/// band of an equaliser, and says so as a label it derived from the parameter
/// names rather than as a fact the manual prints. It is a convention for laying
/// a panel out, which is what this uses it for: a pale strip over the columns
/// the band covers with its name knocked out of it, which is how a `DeepMind`
/// prints `ARP / SEQ` and `VCF` across the top of a group and is the first
/// thing the eye follows across the instrument's own panel.
///
/// The strip divides the row exactly the way the row below divides itself,
/// because both are laid out from [`spans`] in the grid's own columns, so a
/// band of three stands over its three and not over two and a half.
fn over<'a, Renderer>(line: &Line) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let spans = spans(line);
    // Nothing at all where the library grouped nothing on this row, which is
    // most algorithms: a strip of six blanks is a line of empty space over a
    // row that did not ask for one.
    if spans.iter().all(|(label, _)| label.is_none()) {
        return None;
    }
    let runs = spans.into_iter().map(|(label, across)| {
        let across = u16::try_from(across).unwrap_or(1);
        match label {
            None => Element::from(Space::new().width(Length::FillPortion(across))),
            Some(label) => container(text(label).size(8).font(reading_face()).style(
                move |theme: &Theme| text::Style {
                    color: Some(ink(theme, On::Strip)),
                },
            ))
            .width(Length::FillPortion(across))
            .height(Length::Fixed(BAND))
            .padding([0, 4])
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(move |theme: &Theme| container::Style {
                background: Some(Background::Color(banding(theme))),
                border: Border {
                    color: materials(theme).metal,
                    width: 1.0,
                    radius: 2.into(),
                },
                ..container::Style::default()
            })
            .into(),
        }
    });
    Some(row(runs).spacing(0).into())
}

/// What an algorithm's own printed figure says about how to draw its slots.
///
/// Two things, and both of them answered by the same `Option`: an engine
/// running something this firmware cannot name has no figure, and then a slot
/// is a fader on the window's own plate, which is what it always was.
///
/// The colours are not in here. Four engines stand on this page at once, and
/// four measured liveries side by side are the collage this window has always
/// refused to be — so what an engine wears is its case and a hairline of its
/// accent, and both of those belong to the plate rather than to a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Figure {
    /// What shape a control that sweeps a range takes.
    control: Option<Control>,
}

impl Figure {
    /// Returns what the algorithm's own panel is made of.
    fn of(panel: Option<&'static Panel>) -> Self {
        Self {
            control: panel.map(Panel::control),
        }
    }

    /// Returns the room one of the algorithm's slots is drawn in.
    ///
    /// A knob where the figure draws one, which is 29 of the 35. The size is
    /// the page's rather than the rack's: four engines on one surface is
    /// twenty-four controls and a chain, so a slot here is smaller than a slot
    /// in a rack of forty rather than larger, and the travel comes down with it
    /// the way the instrument's own front panel brings it down to hold two rows.
    fn room(self, spare: bool) -> Room {
        let body = if spare { SPARE } else { BODY };
        match self.control {
            Some(Control::Knob) => Room::SLOT.turned().sized(body),
            // A fader keeps the width a fader is, and gives up travel instead:
            // there is one cap in this window and it is as wide as it is.
            _ => Room::lane(fader::WIDTH, if spare { SPARE } else { TRAVEL }),
        }
    }
}

/// Draws one of an engine's twelve bytes, under the name the algorithm gives it.
///
/// The control is the same one the rest of the editor draws from the same
/// table, in the shape the algorithm's own figure uses: 29 of the 35 are knobs,
/// five are faders and one is a numeric display, and the library says which
/// without saying anything about what the byte is. A knob and a fader are the
/// same control over the same range with the same drag — see [`knob`](crate::knob)
/// — so the shape is an arrangement, which is the one thing hand layout here is
/// allowed to choose.
///
/// The display is drawn as a fader. It is the one shape this window does not
/// have and the one algorithm that wants it, and a control invented for a
/// single figure would be a worse lie than the fader that is already honest
/// about the byte underneath.
///
/// The title and the reading under it are two fixed boxes rather than one, so
/// that the units of a row land on one line whether the title above them ran to
/// one line or to two. A slot whose title wraps pushes nothing down but itself.
fn slot<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    lane: Lane,
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let title = lane
        .slot
        .map_or_else(|| within(engine, lane.parameter), |slot| slot.title);
    column![
        row![
            at(lane.parameter, On::Face),
            reference(lane.slot),
            Space::new().width(Length::Fixed(1.0)),
            modulated(
                moved.contains(&lane.parameter),
                lane.slot.is_none_or(|slot| slot.modulatable),
            ),
        ]
        .spacing(3)
        .align_y(Vertical::Center),
        control(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
            figure.room(false),
        ),
        reading(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
            On::Face,
        ),
        container(
            text(title)
                .size(10)
                .center()
                .style(move |theme: &Theme| text::Style {
                    color: Some(ink(theme, On::Face)),
                })
        )
        .height(Length::Fixed(TITLE))
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center),
        container(printing(hint(lane.slot), On::Face).size(9).center())
            .height(Length::Fixed(HINT))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(2)
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// Draws the bytes the loaded algorithm has no name for.
///
/// Under the grid and not on it. An algorithm can leave seven of its twelve
/// unused, and seven controls the size of the five that do something is a plate
/// whose loudest half is the half that does nothing. So they are a strip cut
/// into the plate below the two rows: smaller, and still every one of them
/// draggable — a byte in the program that no panel reaches is a byte the
/// modulation matrix can still be pointed at.
///
/// Nothing said over them. They are under the grid, cut into the plate, at half
/// the size, and named `Param 9` by the library rather than by the algorithm,
/// which is four ways of saying the same thing; a sentence saying it a fifth
/// time is a sentence taking room from the controls it is about.
///
/// Nothing at all for an algorithm that uses all twelve, which is most of them.
fn spare<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    lanes: &[Lane],
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    if lanes.is_empty() {
        return None;
    }
    let bytes = lanes.iter().map(|lane| {
        column![
            row![
                at(lane.parameter, On::Recess),
                Space::new().width(Length::Fixed(1.0)),
                modulated(moved.contains(&lane.parameter), true),
            ]
            .spacing(3)
            .align_y(Vertical::Center),
            control(
                lane.parameter,
                patch.value(lane.parameter),
                patch.claim(lane.parameter),
                firmware,
                figure.room(true),
            ),
            printing(within(engine, lane.parameter).to_owned(), On::Recess).size(9),
        ]
        .spacing(2)
        .width(Length::Fixed(SPARE + 20.0))
        .align_x(Horizontal::Center)
        .into()
    });
    Some(
        container(row(bytes).spacing(4).wrap())
            .width(Length::Fill)
            .padding(5)
            .style(|theme: &Theme| {
                let material = materials(theme);
                container::Style {
                    background: Some(Background::Color(material.panel)),
                    border: Border {
                        color: material.recess_edge,
                        width: 1.0,
                        radius: 2.into(),
                    },
                    ..container::Style::default()
                }
            })
            .into(),
    )
}

/// Draws one engine: what it is running, and what that makes its twelve bytes.
///
/// On the grid the instrument's own FX page uses. 26.3 published it — six
/// columns and two rows, measured off the 35 screenshots in the manual, and
/// every slot's column and row on it
/// ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22))
/// — so a plate is not twelve slots wrapped into whatever width the window
/// happened to have, and it is not six things in a row either. It is the six
/// columns, each of them there whether or not a slot stands in it, spread
/// across the whole of this engine's half of the page. That is the arrangement
/// anybody who has edited an effect on the hardware already knows, and it is
/// the arrangement a second row can be read down against.
///
/// [`Row::align`] is what the library offers a host that lays a partial row out
/// some other way, and this is not one: a slot here is in the column that was
/// measured for it, so a row that does not fill the grid is already packed
/// where the page packs it and there is no room left over to place.
fn plate<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let lanes = lanes(patch, engine, firmware);
    let panel = algorithm(patch, engine, firmware).map(Algorithm::panel);
    let figure = Figure::of(panel);
    let (grid, left) = placed(&lanes, panel);
    let mut drawn = column![].spacing(6);
    for line in &grid {
        drawn = drawn.extend(over(line));
        drawn = drawn.push(line_of(patch, engine, line, firmware, moved, figure));
    }
    // What the display will show where a slot shows names rather than a number,
    // printed once under the grid: the manual gives the names and never the
    // bytes they sit at, so this is a reading and not something to send.
    let shown: Vec<Element<'a, Renderer>> = displays(&lanes).map(Element::from).collect();
    if !shown.is_empty() {
        drawn = drawn.push(column(shown).spacing(1));
    }
    let drawn = container(drawn)
        .width(Length::Fill)
        .padding(7)
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
                background: Some(Background::Color(material.plate)),
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 2.into(),
                },
                ..container::Style::default()
            }
        });
    let body = column![header(patch, engine, firmware, moved, panel)]
        .push(drawn)
        .extend(spare(patch, engine, &left, firmware, moved, figure))
        .spacing(6);
    let figure_colours = panel.map(|panel| (panel.chassis(), panel.accent()));
    container(body)
        .padding(6)
        .width(Length::Fill)
        // Its own height, and not its neighbour's. A case is as deep as the
        // algorithm in it needs — one row of slots or two, a strip of unnamed
        // bytes or none — and stretching the shallower of a pair to match would
        // be a panel with empty case at the bottom of it, which is a worse lie
        // than two cases of different depths.
        .style(move |theme: &Theme| {
            // Cut into the group's face plate rather than raised off it: the
            // plate is what the four engines are recessed into, which is the
            // same trick the section bar plays with the panel it is cut from.
            container::Style {
                background: Some(Background::Color(chassis(theme, figure_colours))),
                border: Border {
                    color: rim(theme, figure_colours),
                    width: 1.0,
                    radius: 3.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Returns the colour the case an engine is drawn in is painted.
///
/// The measured chassis of the algorithm's own figure, most of the way back to
/// the panel this window is. The library publishes four colours per algorithm —
/// [deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)
/// — and they are the colours of 35 imaginary rack units: a cream fader panel,
/// a black one, a blue-grey one.
///
/// Two of the four are spent and two are not, and which two depends on how many
/// units are on the surface. With one open at a time this page wore the lot,
/// case and face and cap, because a single unit can be that unit without the
/// window becoming a shelf of other people's boxes. Four stand on it now, and
/// four liveries side by side are exactly the collage this editor's whole
/// argument is against — so the face went back to the window's own plate and
/// the cap back to the window's own metal, and what is left is the case and a
/// hairline of the accent: enough to tell the reverb from the distortion at
/// arm's length, which is all four units at once can afford to say.
fn chassis(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    let Some((chassis, _)) = figure else {
        return material.panel;
    };
    let worn = colour(chassis);
    // As much of the case as can be worn and still be printed on. A measured
    // chassis carried half way into the panel lands, for the cream ones, in the
    // exact middle of the instrument's two inks — where the best either can do
    // is 4.45 against it, and every word on the strip is a word somebody has to
    // squint at. So the carry is not a number somebody picked: it is the most
    // of the livery that leaves the case readable, found by backing off towards
    // the panel until it is.
    //
    // Backing off darkens, because the panel is dark, so this always terminates
    // at the panel itself — which is what an engine with no figure wears.
    (0..=CARRY)
        .rev()
        .map(|step| {
            style::mix(
                material.panel,
                worn,
                f32::from(step) / f32::from(CARRY) * FULL,
            )
        })
        .find(|case| style::contrast(*case, style::ink_on(*case, theme)) >= style::READABLE)
        .unwrap_or(material.panel)
}

/// How far the measured chassis is carried into the panel, at most.
///
/// Far enough to be the colour of the thing rather than a tint on the window,
/// and not so far that four cases side by side stop being one instrument.
const FULL: f32 = 0.5;

/// How finely [`chassis`] looks for the most of it that can be printed on.
const CARRY: u8 = 20;

/// Returns the colour of the strip a band's name is knocked out of.
///
/// The instrument's own device: a pale bar across the top of a group, the same
/// one the front panel prints `ARP / SEQ` and `VCF` on. Off the plate rather
/// than off the case, because a band belongs to the grid it is printed over and
/// the grid stands on the window's own plate whatever the engine is wearing.
fn banding(theme: &Theme) -> Color {
    let material = materials(theme);
    style::mix(material.plate, material.metal_low, 0.62)
}

/// Returns the colour the plate's edge is drawn in.
///
/// The figure's accent, which the library defines as the most saturated colour
/// covering a visible share of the printed panel: the lit label strip on a
/// cream unit, the LED on a black one. A hairline of it is what says which
/// engine is which without a second word on the plate.
fn rim(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    figure.map_or(material.recess_edge, |(_, accent)| {
        style::mix(material.recess_edge, colour(accent), 0.55)
    })
}

/// Returns a colour the library measured, as a colour this window can draw.
///
/// Three components rather than a parse, because three components are what the
/// library publishes and what a window wants; the one thing this does is put
/// them on the scale `iced` uses.
fn colour(measured: Colour) -> Color {
    let [red, green, blue] = measured.to_rgb();
    Color::from_rgb8(red, green, blue)
}

/// Draws the strip across the top of one engine's case.
///
/// Everything an engine is, on one line: its number, what it is running, and
/// how loud it comes out. The algorithm is the parameter's own value table
/// under the abbreviations the instrument's display prints — a list, which is
/// to say the thing you press to change it — with what those stand for written
/// out beside it rather than substituted into it, because the control is the
/// library's and the prose is the panel's.
///
/// This is where the page it replaced had a whole header band of its own, under
/// a row of tabs that had the name on it as well. Two places saying which
/// algorithm an engine is running, and neither of them where the engine was.
fn header<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
    panel: Option<&'static Panel>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let loaded = algorithm(patch, engine, firmware);
    let named = loaded.map_or_else(
        || {
            patch.value(engine.algorithm_parameter()).map_or_else(
                || "nothing has been read".to_owned(),
                |value| format!("{value}, firmware {firmware} does not name it"),
            )
        },
        |algorithm| algorithm.full_name.to_owned(),
    );
    let category = loaded.map_or("", |algorithm| algorithm.category);
    let on = On::Case(panel.map(|panel| (panel.chassis(), panel.accent())));
    let gain = engine.gain_parameter();
    row![
        text(format!("FX {}", engine.number()))
            .size(13)
            .font(reading_face())
            .style(move |theme: &Theme| text::Style {
                color: Some(ink(theme, on)),
            }),
        control(
            engine.algorithm_parameter(),
            patch.value(engine.algorithm_parameter()),
            patch.claim(engine.algorithm_parameter()),
            firmware,
            Room::listed(ALGORITHM),
        ),
        container(
            column![
                text(named)
                    .size(12)
                    .style(move |theme: &Theme| text::Style {
                        color: Some(ink(theme, on)),
                    }),
                printing(category.to_owned(), on).size(9),
            ]
            .spacing(0)
        )
        .width(Length::Fill),
        // Cut into the case rather than printed on it. A reading carries its
        // claim in its colour, and the claims were chosen against this
        // window's own dark panel: on the palest of the 35 cases both of them
        // have to be lifted so far to be read that they arrive as the same
        // ink, which would spend the one distinction this editor exists to
        // draw on a livery. So the one value on the strip sits in a recess,
        // which is also where a control belongs.
        container(
            row![
                printing("gain".to_owned(), On::Recess).size(9),
                control(
                    gain,
                    patch.value(gain),
                    patch.claim(gain),
                    firmware,
                    Room::across(GAIN).sized(GAIN_THICK),
                ),
                modulated(moved.contains(&gain), true),
                reading(
                    gain,
                    patch.value(gain),
                    patch.claim(gain),
                    firmware,
                    On::Recess,
                ),
            ]
            .spacing(5)
            .align_y(Vertical::Center)
        )
        .padding([2, 6])
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
                background: Some(Background::Color(material.panel)),
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 3.into(),
                },
                ..container::Style::default()
            }
        }),
    ]
    .spacing(8)
    .align_y(Vertical::Center)
    .into()
}

/// Draws one parameter that is a setting rather than a slot.
///
/// A slot of the rack with its title moved off the bottom, because what these
/// stand in is a row and not a column: the address and the modulation mark
/// above, the control, and the reading under it.
fn cell<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
    room: Room,
    title: &'a str,
    moved: &[ParamId],
    on: On,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        row![
            at(parameter, on),
            Space::new().width(Length::Fixed(2.0)),
            modulated(moved.contains(&parameter), true),
            Space::new().width(Length::Fixed(4.0)),
            text(title)
                .size(10)
                .style(move |theme: &Theme| text::Style {
                    color: Some(ink(theme, on)),
                }),
            Space::new().width(Length::Fill),
            reading(parameter, value, claim, firmware, on),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
        control(parameter, value, claim, firmware, room),
    ]
    .spacing(3)
    .width(Length::Fixed(room.width()))
    .into()
}

/// Draws where a parameter lives, in the ink of the surface it is printed on.
///
/// The NRPN number, which is also the parameter's byte offset in a dump, so one
/// number is both its name on the wire and its address in memory.
/// [`panel::address`](crate::panel::address) is the same thing on the window's
/// own plate; this page needs it per surface, because the grey a rack prints it
/// in is a grey half the 35 chassis are.
fn at<'a, Renderer>(parameter: ParamId, on: On) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    printing(parameter.offset().to_string(), on)
        .size(9)
        .font(reading_face())
        .into()
}

/// Draws the abbreviation the instrument's display prints for a slot.
///
/// `PST`, `DCY`, `HMQ`: the word a player reads on the hardware while looking
/// at this panel, so that the two say the same thing. Nothing for a byte the
/// algorithm does not use, which has no display name because it is not on the
/// display.
fn reference<'a, Renderer>(slot: Option<&'static FxSlot>) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let name = slot.map_or("", |slot| slot.reference);
    printing(name.to_owned(), On::Face)
        .size(9)
        .font(reading_face())
        .into()
}

/// The line under a slot's title: what the instrument's display reads there.
///
/// The two ends of the reading and its unit, which the manual prints, and never
/// a value worked out from the byte, which it does not: the curve between `0.1`
/// and `6.0 s` is not published, so the fader's own readout stays the byte and
/// this says what the ends of it mean. A slot showing names says how many
/// rather than which, because which one a byte shows is the part nobody has
/// published.
fn hint(slot: Option<&'static FxSlot>) -> String {
    // A byte the algorithm does not use is on a strip of its own under the
    // grid, at half the size, under the library's own name for it. There is
    // nothing left for a line here to add.
    let Some(slot) = slot else {
        return String::new();
    };
    if slot.is_selector() {
        return match slot.values.len() {
            1 => "1 setting".to_owned(),
            names => format!("{names} settings"),
        };
    }
    match (slot.min, slot.max) {
        (Some(low), Some(high)) => match slot.unit {
            Some(unit) => format!("{low}\u{2013}{high} {unit}"),
            None => format!("{low}\u{2013}{high}"),
        },
        _ => String::new(),
    }
}

/// The names a plate's slots show on the instrument's display.
///
/// One line per slot that shows names instead of a number, printed under the
/// grid rather than in the slot, because that is what they are: a reading and
/// not something to send. The manual gives the names and never the bytes they
/// sit at, so the control stays the byte and this says what the display will
/// make of it.
fn displays<'a, Renderer>(
    lanes: &[Lane],
) -> impl Iterator<Item = iced_widget::Text<'a, Theme, Renderer>>
where
    Renderer: TextRenderer<Font = Font>,
{
    let printed: Vec<String> = lanes
        .iter()
        .filter_map(|lane| lane.slot)
        .filter(|slot| slot.is_selector())
        .map(|slot| {
            format!(
                "{} {} shows {}",
                slot.reference,
                slot.title,
                slot.values.join(", ")
            )
        })
        .collect();
    printed
        .into_iter()
        .map(|line| printing(line, On::Face).size(9).font(reading_face()))
}

/// Returns a parameter's name with the engine's own taken off the front.
///
/// The rule a slot's title follows against its group, one level down: the plate
/// says `FX 1` once, so `FX 1 Type` in it is `Type`. Case-insensitively,
/// because the parameter table writes the same engine `FX 1 Type` and `Fx 1
/// Output Gain`. A name that does not start with the engine is returned whole,
/// which is how a library that renumbers them says so.
fn within(engine: Engine, parameter: ParamId) -> &'static str {
    let name = parameter.short_name();
    let prefix = format!("{engine} ");
    if name.len() > prefix.len() && name.to_ascii_uppercase().starts_with(&prefix) {
        name.get(prefix.len()..).unwrap_or(name)
    } else {
        name
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::effect::{Algorithm, Engine, SLOTS_PER_ENGINE, grid};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, ParamId};
    use deepmind_midi::program::Program;
    use deepmind_midi::sysex::inquiry::Version;

    use super::{
        Lane, On, claimed, engines, hint, ink, lanes, legend, placed, settings, spans, within,
    };
    use crate::style::{READABLE, contrast, deepmind, legible, tint};
    use crate::{Confidence, Patch};

    /// Firmware 1.0, which numbers the algorithms differently.
    const FIRMWARE_1_0: Version = Version { major: 1, minor: 0 };

    /// A sound the synthesizer described, with `engine` running `algorithm`.
    fn running(algorithm: &str) -> Patch {
        let mut patch = Patch::new();
        patch.confirm(Program::new(ProtocolVersion::V7));
        let value = Algorithm::by_name(algorithm)
            .expect("an algorithm of that name")
            .value_for(DEFAULT_FIRMWARE)
            .expect("the newest firmware offers it");
        // Not asserted as a change: a program at its floor is already running
        // the algorithm the first byte selects, and asking for the one it holds
        // is what `Patch::edit` refuses.
        patch.edit(Engine::One.algorithm_parameter(), value);
        assert_eq!(patch.value(Engine::One.algorithm_parameter()), Some(value));
        patch
    }

    /// The twelve bytes of `engine` under `algorithm`, whatever the patch says.
    fn under(algorithm: &'static Algorithm, engine: Engine) -> Vec<Lane> {
        engine
            .slot_parameters()
            .iter()
            .copied()
            .map(|parameter| Lane {
                parameter,
                slot: algorithm.slot_of(engine, parameter),
            })
            .collect()
    }

    #[test]
    fn every_byte_of_an_engine_is_drawn_once_whatever_is_loaded() {
        // The plate is laid out from the library's own grid, and the one thing
        // a layout read out of a table can do wrong is lose a row of it: a byte
        // with no control on it is a byte nobody can edit, and a byte with two
        // is a plate that disagrees with itself. Neither is visible in a
        // screenshot of the one algorithm somebody happened to load.
        for algorithm in Algorithm::all() {
            for engine in Engine::ALL {
                let lanes = under(algorithm, engine);
                let (rows, spare) = placed(&lanes, Some(algorithm.panel()));

                let mut drawn: Vec<ParamId> = rows
                    .into_iter()
                    .flatten()
                    .flatten()
                    .chain(spare)
                    .map(|lane| lane.parameter)
                    .collect();
                let count = drawn.len();
                drawn.sort_unstable_by_key(|parameter| parameter.offset());
                drawn.dedup();

                assert_eq!(
                    count, SLOTS_PER_ENGINE,
                    "{engine} running {} draws {count} of its twelve bytes",
                    algorithm.full_name
                );
                assert_eq!(
                    drawn.len(),
                    count,
                    "{engine} running {} draws a byte twice",
                    algorithm.full_name
                );
            }
        }
    }

    #[test]
    fn a_slot_stands_in_the_column_the_instrument_draws_it_in() {
        // The point of the grid, and the thing a row of clusters could not do:
        // every row is as wide as the grid, and a slot is at its own measured
        // column of it, so the two rows of a plate line up down the page.
        let columns = usize::from(grid().columns());
        for algorithm in Algorithm::all() {
            let lanes = under(algorithm, Engine::One);
            let (rows, _) = placed(&lanes, Some(algorithm.panel()));

            assert_eq!(
                rows.len(),
                algorithm.panel().rows().len(),
                "{} draws a row the library does not",
                algorithm.full_name
            );
            for line in &rows {
                assert_eq!(
                    line.len(),
                    columns,
                    "{} has a short row",
                    algorithm.full_name
                );
                for (column, cell) in line.iter().enumerate() {
                    let Some(slot) = cell.and_then(|lane| lane.slot) else {
                        continue;
                    };
                    assert_eq!(
                        usize::from(slot.position().column()),
                        column,
                        "{} draws {} out of its column",
                        algorithm.full_name,
                        slot.title
                    );
                }
            }
        }
    }

    #[test]
    fn the_bytes_an_algorithm_does_not_use_are_the_ones_off_the_grid() {
        // TC Deep Reverb uses five of its twelve, so seven are left for the
        // strip under the plate and the grid holds exactly the five.
        let deep = Algorithm::by_name("TC-DeepVRB").expect("the TC reverb");

        let (rows, spare) = placed(&under(deep, Engine::One), Some(deep.panel()));

        assert_eq!(spare.len(), 7);
        assert!(
            spare.iter().all(|lane| lane.slot.is_none()),
            "a named slot fell off the grid"
        );
        let on = rows.iter().flatten().flatten().count();
        assert_eq!(on, 5, "the grid holds the five the algorithm names");
    }

    #[test]
    fn nothing_known_about_the_algorithm_leaves_every_byte_off_the_grid() {
        // Stage 3's rack, which is what a dump from a firmware whose table this
        // build does not have still deserves: twelve bytes, no page to lay them
        // out by, and every one of them draggable.
        let patch = Patch::new();
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        let (rows, spare) = placed(&lanes, None);

        assert!(rows.is_empty());
        assert_eq!(spare.len(), SLOTS_PER_ENGINE);
    }

    /// Every surface a word on this page can land on.
    ///
    /// The four the page has, with the case in all 35 of its liveries: half the
    /// measured chassis are pale and half are dark, and a page that reads on
    /// one of them proves nothing about the other 34.
    fn surfaces() -> Vec<On> {
        let mut found = vec![On::Face, On::Strip, On::Recess, On::Case(None)];
        found.extend(
            Algorithm::all()
                .iter()
                .map(Algorithm::panel)
                .map(|panel| On::Case(Some((panel.chassis(), panel.accent())))),
        );
        found
    }

    #[test]
    fn every_word_can_be_read_on_every_surface_it_lands_on() {
        // The rule this page is drawn under, checked rather than eyeballed. A
        // colour that is right in a palette and unreadable on a panel is not
        // right, and a screenshot of the one algorithm somebody happened to
        // load is not a check.
        let theme = deepmind();
        for on in surfaces() {
            let surface = on.colour(&theme);
            let title = contrast(ink(&theme, on), surface);
            assert!(
                title >= READABLE,
                "a title on {on:?} is {title:.2} against it"
            );
            let printed = contrast(legend(&theme, on), surface);
            assert!(
                printed >= READABLE,
                "a legend on {on:?} is {printed:.2} against it"
            );
        }
    }

    #[test]
    fn a_reading_keeps_its_claim_and_is_still_readable() {
        // The two surfaces a value is ever printed on, and the reason the list
        // is two rather than four: a claim is amber or green because of what it
        // means, and both were chosen against this window's own dark panel. On
        // the palest of the 35 measured cases each of them has to be lifted so
        // far to be read that they arrive as the same ink — so nothing on this
        // page prints a value on a case. The one value the strip across a case
        // carries sits in a recess cut into it, which is where a control
        // belongs anyway.
        let theme = deepmind();
        for on in [On::Face, On::Recess] {
            let surface = on.colour(&theme);
            for claim in [
                Confidence::Unknown,
                Confidence::Assumed,
                Confidence::Confirmed,
            ] {
                let reading = legible(tint(&theme, claim), surface, &theme);
                let against = contrast(reading, surface);
                assert!(
                    against >= READABLE,
                    "a {claim:?} reading on {on:?} is {against:.2} against it"
                );
            }
            // And the two that carry a colour still carry it. Contrast is a
            // measure of light and says nothing about hue, so what is asserted
            // is the warmth: an assumed value is the copper of the wooden
            // cheeks and a confirmed one is green, and lifting either of them
            // until it can be read must not turn one into the other.
            let warmth = |claim: Confidence| {
                let reading = legible(tint(&theme, claim), surface, &theme);
                reading.r - reading.g
            };
            assert!(
                warmth(Confidence::Assumed) > warmth(Confidence::Confirmed),
                "on {on:?} an assumed reading came out no warmer than a confirmed one"
            );
        }
    }

    #[test]
    fn one_group_holds_the_engines_and_the_rest_are_racks() {
        let found: Vec<Group> = Group::ALL
            .iter()
            .copied()
            .filter(|group| engines(*group).is_some())
            .collect();

        assert_eq!(found, vec![Group::Effects], "found {found:?}");
        assert_eq!(engines(Group::Effects).map(|found| found.len()), Some(4));
    }

    #[test]
    fn the_panel_claims_the_whole_group_and_leaves_every_other_alone() {
        let drawn = claimed(Group::Effects);

        assert_eq!(drawn.len(), Group::Effects.parameters().count());
        for parameter in Group::Effects.parameters() {
            assert!(drawn.contains(&parameter), "{parameter} is not drawn");
        }
        assert!(claimed(Group::Vcf).is_empty());
        assert!(claimed(Group::ModMatrix).is_empty());
    }

    #[test]
    fn the_settings_are_what_no_engine_addresses() {
        let settings = settings(Group::Effects);

        // The connection mode and whether the effects are in the signal path.
        // Named here to say what the subtraction found, not to find it.
        assert_eq!(settings.len(), 2);
        for parameter in &settings {
            assert!(
                Engine::of(*parameter).is_none(),
                "{parameter} is a slot of an engine"
            );
            for engine in Engine::ALL {
                assert_ne!(*parameter, engine.algorithm_parameter());
                assert_ne!(*parameter, engine.gain_parameter());
            }
        }
    }

    #[test]
    fn every_parameter_is_either_a_setting_or_something_an_engine_addresses() {
        let settings = settings(Group::Effects);
        let engines = engines(Group::Effects).expect("the effects hold them");
        let addressed = engines.len() * (SLOTS_PER_ENGINE + 2);

        assert_eq!(
            settings.len() + addressed,
            Group::Effects.parameters().count(),
            "nothing falls between the engines and the settings"
        );
    }

    #[test]
    fn a_slot_is_named_by_the_algorithm_that_is_loaded() {
        let patch = running("RoomRev");

        let room = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(room.len(), SLOTS_PER_ENGINE);
        let third = room.get(2).copied().expect("twelve of them");
        assert_eq!(third.parameter, ParamId::Fx1Param3);
        assert_eq!(third.slot.map(|slot| slot.title), Some("Size"));
        // And the same byte is something else under another algorithm, which is
        // the whole reason this panel exists.
        let phaser = running("Phaser");
        let third = lanes(&phaser, Engine::One, DEFAULT_FIRMWARE)
            .get(2)
            .copied()
            .expect("twelve of them");
        assert_ne!(third.slot.map(|slot| slot.title), Some("Size"));
    }

    #[test]
    fn an_algorithm_that_uses_fewer_leaves_the_rest_as_the_bytes_they_are() {
        // TC Deep Reverb uses five of its twelve.
        let patch = running("TC-DeepVRB");

        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(lanes.len(), SLOTS_PER_ENGINE, "twelve bytes either way");
        let named = lanes.iter().filter(|lane| lane.slot.is_some()).count();
        assert_eq!(named, 5);
        for lane in lanes.iter().skip(named) {
            assert_eq!(lane.slot, None, "{} is not used", lane.parameter);
            assert!(
                hint(lane.slot).is_empty(),
                "the strip's heading says it once"
            );
        }
    }

    #[test]
    fn a_sound_nobody_has_read_has_nothing_to_call_its_slots() {
        let patch = Patch::new();

        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(lanes.len(), SLOTS_PER_ENGINE);
        assert!(
            lanes.iter().all(|lane| lane.slot.is_none()),
            "a byte nobody has read is under the library's own name"
        );
    }

    #[test]
    fn an_algorithm_this_firmware_does_not_name_leaves_the_slots_as_bytes() {
        // Vintage Pitch is firmware 1.1's, inserted rather than appended, and
        // its byte selects Rotary Speaker on 1.0.
        let patch = running("Vintage Pitch");

        let newer = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let older = lanes(&patch, Engine::One, FIRMWARE_1_0);

        assert_eq!(
            newer
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            Algorithm::by_name("Vintage Pitch")
                .and_then(|algorithm| algorithm.slot(1))
                .map(|slot| slot.title)
        );
        assert_ne!(
            older
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            newer
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            "the same byte is another algorithm on the older firmware"
        );
    }

    #[test]
    fn a_band_covers_the_columns_of_the_row_it_runs_along() {
        // The Midas equaliser's slots are four bands and an output, and the
        // strip over a row has to divide that row exactly the way the row
        // divides itself or a band of three stands over two and a half.
        let patch = running("MidasEQ");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let panel = Algorithm::by_name("MidasEQ").expect("the Midas").panel();

        let (rows, _) = placed(&lanes, Some(panel));

        let mut named: Vec<&str> = Vec::new();
        for line in &rows {
            let runs = spans(line);
            assert_eq!(
                runs.iter().map(|(_, across)| across).sum::<usize>(),
                line.len(),
                "the strip and the row divide different widths"
            );
            named.extend(runs.iter().filter_map(|(label, _)| *label));
        }
        // `high-mid` twice, because the band runs across the break between the
        // two rows and each row prints what stands over it.
        assert_eq!(
            named,
            vec!["low", "low-mid", "high-mid", "high-mid", "high"]
        );
    }

    #[test]
    fn two_slots_the_library_labelled_nothing_are_not_a_band() {
        let patch = running("RoomRev");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let panel = Algorithm::by_name("RoomRev").expect("the room").panel();

        let (rows, _) = placed(&lanes, Some(panel));

        for line in &rows {
            for (label, across) in spans(line) {
                assert!(
                    label.is_some() || across == 1,
                    "unlabelled columns ran together into a band"
                );
            }
        }
    }

    #[test]
    fn a_reading_says_the_ends_the_manual_prints_and_never_the_byte_between() {
        let deep = Algorithm::by_name("TC-DeepVRB").expect("the TC reverb");

        assert_eq!(hint(deep.slot(2)), "0.1\u{2013}6.0 s");
        // A slot the display shows names on says how many, because which one a
        // byte shows is the part nobody has published.
        assert_eq!(hint(deep.slot(1)), "11 settings");
        assert!(hint(None).is_empty());
    }

    #[test]
    fn a_plate_says_the_engine_once() {
        assert_eq!(within(Engine::One, ParamId::Fx1Type), "Type");
        assert_eq!(within(Engine::One, ParamId::Fx1OutputGain), "Output Gain");
        assert_eq!(within(Engine::Four, ParamId::Fx4Param12), "Param 12");
        // A parameter that is not the engine's keeps its whole name.
        assert_eq!(
            within(Engine::One, ParamId::Fx2Type),
            ParamId::Fx2Type.short_name()
        );
    }
}

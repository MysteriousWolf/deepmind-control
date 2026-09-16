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
//! # One unit, in its own livery
//!
//! The four measured colours were spent as an identity rather than as a finish
//! while four plates stood side by side, because four of them painted as
//! measured would have been a collage of other people's instruments in a window
//! whose whole argument is that it is one instrument. One plate stands on the
//! surface now, and a single unit can wear its own case: the chassis, the face
//! it is inset with, a hairline of the accent round the edge — and the cap,
//! which is the colour of the knob body or the fader cap printed on the
//! algorithm's own figure.
//!
//! The cap is the one that used to be refused, on the grounds that a control
//! repainted to match a figure would break the one rule this window keeps
//! everywhere. It is the same reversal the chassis already went through and it
//! has the same reason: the rule was protecting a surface with four liveries on
//! it, and there is one. What the livery may not touch is what a control
//! *means* — the recess it is cut into, the scale printed round it, how far a
//! drag runs, and above all the claim, which is still the fill for a value the
//! synthesizer reported and the stroke for one this window put there. A cap
//! that painted an assumed value the same as a confirmed one would be buying a
//! colour with the one distinction this editor exists to draw.
//!
//! Everything printed on the face is printed in the ink that face can be read
//! against, because a measured face is a cream panel as often as a black one.
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
use iced_widget::{Space, button, column, container, row, text};

use crate::chain;
use crate::panel::{Message, Room, control, modulated, readout};
use crate::style::{self, materials, reading};
use crate::{Element, Patch};

/// How much room one of the group's own settings is chosen in.
///
/// `Parallel 1/2, parallel 3/4` is what the routing's names read like, and a
/// list that clips one is a list that offers ten indistinguishable topologies.
const SETTING: f32 = 210.0;

/// How much room an engine's algorithm is chosen in.
///
/// The display's own abbreviations — `RoomRev`, `MulBndDist` — because the
/// control is the parameter's value table and not this panel's prose. What the
/// algorithm is called in full is printed beside it.
const ALGORITHM: f32 = 116.0;

/// How long the output gain's fader is.
///
/// Long enough to be the right-hand end of the unit's top strip rather than a
/// control parked at it: the band has an engine's number, its algorithm and its
/// name along the left, and what filled the rest was nothing.
const GAIN: f32 = 240.0;

/// Height of the box a slot's title is set in.
///
/// Two lines of it. `Low-Mid Frequency Spread` is the longest title any of the
/// 35 algorithms gives a slot, and a column of the grid spread across the page
/// is wide enough to set it on one line; two is the room for a face that sets
/// it wider. Fixed, and separate from the line under it, so that a title
/// running to two lines pushes nothing down but itself. Set in the middle of
/// the box, so that a title of one line sits the same distance under its value
/// as its units sit under it rather than at one end of a gap.
const TITLE: f32 = 30.0;

/// Height of the line a slot's reading is described on.
///
/// One line, always taken whether or not there is anything to say on it, so
/// that the units of one row land on one line however many of the slots above
/// them have any.
const HINT: f32 = 14.0;

/// Height of the strip a band's name is knocked out of.
const BAND: f32 = 15.0;

/// How big the thing a hand takes hold of is drawn on an effect plate.
///
/// Half again what a rack's slot is cut to. The library publishes the
/// instrument's own proportion — a control 11.9 wide in a column pitched 20 on
/// a 128 point display — and a page this wide would make that a knob a hundred
/// points across, with the title under it set in a face a tenth of its size.
/// That proportion is a measurement of a dot matrix drawing its own labels in
/// dots, so what this window takes from the grid is the arrangement, which is
/// published and exact, and not the size, which does not survive being read in
/// a real face. Twelve controls on a page have room the rack's forty do not,
/// and this is that room spent on the thing a hand actually touches.
const BODY: f32 = 72.0;

/// How wide a fader is drawn on one, for the same reason.
///
/// Less than a knob's diameter. A knob grows in both directions at once and a
/// fader only across its travel, which is already as long as a rack's.
const TRACK: f32 = 58.0;

/// How big one of them is drawn where the algorithm does not use the byte.
///
/// A rack slot's own control, near enough: the strip under the grid is where a
/// byte the algorithm has no name for goes, and it is not competing with the
/// twelve above it.
const SPARE: f32 = 34.0;

/// How much room the engine's number is stencilled in.
const NUMERAL: f32 = 52.0;

/// How far the open engine's tab is lifted off whatever its chassis is.
///
/// A step of the window's own light rather than of the livery, so that which
/// tab is open is the same step whether the unit on it is cream or black.
const LIFT: f32 = 0.12;

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

/// Draws the effects, when `group` is the group that holds them.
pub(crate) fn panels<'a, Renderer>(
    patch: &Patch,
    group: Group,
    firmware: Version,
    moved: &[ParamId],
    opened: Option<Engine>,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let engines = engines(group)?;
    // The picture and the two settings that shape it, on one surface. The
    // routing *is* the chain: which engine feeds which is the drawing, and
    // whether the four of them are inserted, sent or bypassed is the rest of
    // the same sentence. Drawn as a display over the controls it is a drawing
    // of, which is the arrangement every plate of the front panel has, and cut
    // into the group's face rather than standing loose on it, which is what
    // two controls with nothing under them were doing.
    let block = settings(group)
        .into_iter()
        .map(|parameter| {
            cell(
                patch,
                parameter,
                firmware,
                Room::listed(SETTING),
                parameter.short_name(),
                moved,
                None,
            )
        })
        .collect::<Vec<_>>();
    let mut wiring = row(block).spacing(16).align_y(Vertical::Bottom);
    if let Some(note) = chain::note(patch) {
        wiring = wiring.push(
            container(printing(note.to_owned(), None).size(11))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Vertical::Center),
        );
    }
    let mut body = column![
        container(column![chain::display(patch, firmware), wiring].spacing(8))
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
            })
    ]
    .spacing(10);
    // One engine at a time, chosen along the top. Four plates stacked was four
    // algorithms' worth of controls on one surface and a scroll to reach the
    // fourth, and on an instrument that gives its own FX page to one engine at
    // a time it was also the wrong shape.
    let opened = opened.filter(|engine| engines.contains(engine));
    let opened = opened.or_else(|| engines.first().copied())?;
    body = body.push(chooser(patch, &engines, firmware, opened));
    body = body.push(plate(patch, opened, firmware, moved));
    Some(body.into())
}

/// Draws the row that chooses which engine is open.
///
/// A tab each, carrying the engine's number and the algorithm it is running,
/// because `FX 2` on its own says which of four and nothing about what it does.
/// Each is painted in its own algorithm's measured chassis and edged in its
/// accent, so the row is four recognisable units rather than four words: which
/// one is the reverb is a thing to see rather than to read.
fn chooser<'a, Renderer>(
    patch: &Patch,
    engines: &[Engine],
    firmware: Version,
    opened: Engine,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let tabs = engines.iter().copied().map(|engine| {
        let loaded = algorithm(patch, engine, firmware);
        let panel = loaded.map(Algorithm::panel);
        let figure = panel.map(|panel| (panel.chassis(), panel.accent()));
        let here = engine == opened;
        let running = loaded.map_or_else(
            || "\u{2014}".to_owned(),
            |algorithm| algorithm.full_name.to_owned(),
        );
        let category = loaded.map_or("", |algorithm| algorithm.category);
        button(
            column![
                row![
                    text(format!("FX {}", engine.number()))
                        .size(10)
                        .font(reading()),
                    Space::new().width(Length::Fill),
                    text(category.to_owned()).size(9).font(reading()),
                ]
                .align_y(Vertical::Center),
                text(running).size(13),
            ]
            .spacing(2),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status| tab(theme, figure, here, status))
        .on_press(Message::Open(engine))
        .into()
    });
    row(tabs).spacing(6).into()
}

/// What one of the engine tabs is drawn as.
///
/// The open one is its algorithm's own unit, face up: the measured chassis
/// carried most of the way rather than a sixth of the way, because one panel on
/// the surface can wear its own livery without the window becoming a shelf of
/// other people's boxes. The rest are the same unit seen edge on — the panel
/// this window is, with a hairline of each one's accent, which is enough to
/// pick the reverb out of four and not enough to compete with the one open.
///
/// Which one is open is a step of the window's own light, and never the
/// chassis. A closed tab carries no chassis at all and an open one is lifted a
/// fixed amount off whatever its chassis left it at, so a cream unit sitting
/// closed cannot outshine a black one standing open — which is what a tint
/// mixed into every tab did, and it read as the wrong engine being the one on
/// the page.
fn tab(
    theme: &Theme,
    figure: Option<(Colour, Colour)>,
    here: bool,
    status: button::Status,
) -> button::Style {
    let material = materials(theme);
    let hover = matches!(status, button::Status::Hovered);
    let face = if here {
        let worn = figure.map_or(material.plate, |(chassis, _)| {
            style::mix(material.plate, colour(chassis), 0.62)
        });
        style::mix(worn, material.metal_low, LIFT)
    } else if hover {
        style::mix(material.panel, material.plate, 0.6)
    } else {
        material.panel
    };
    let edge = figure.map_or(material.recess_edge, |(_, accent)| {
        style::mix(
            material.recess_edge,
            colour(accent),
            if here { 0.9 } else { 0.55 },
        )
    });
    button::Style {
        background: Some(Background::Color(face)),
        // The open tab is read against its own chassis, which a measured one
        // can be either side of; the rest are edge on against the panel.
        text_color: if here {
            style::ink_on(face, theme)
        } else {
            material.metal_low
        },
        border: Border {
            color: edge,
            width: if here { 2.0 } else { 1.0 },
            radius: 3.into(),
        },
        ..button::Style::default()
    }
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
fn over<'a, Renderer>(line: &Line, figure: Figure) -> Option<Element<'a, Renderer>>
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
            Some(label) => container(text(label).size(9).font(reading()).style(
                move |theme: &Theme| text::Style {
                    color: Some(style::ink_on(banding(theme, figure.surface), theme)),
                },
            ))
            .width(Length::FillPortion(across))
            .height(Length::Fixed(BAND))
            .padding([0, 6])
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(move |theme: &Theme| container::Style {
                background: Some(Background::Color(banding(theme, figure.surface))),
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

/// What a plate has been given to wear, which is what its slots are drawn in.
///
/// Three colours the library measured off the figure printed beside the
/// algorithm in the manual, carried together because every one of them is
/// answered by the same `Option`: an engine running something this firmware
/// cannot name has no figure, and then a slot is the window's own metal on the
/// window's own plate, which is what it always was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Figure {
    /// The surface the controls stand on.
    surface: Option<Colour>,
    /// The body of a knob, or the cap of a fader.
    cap: Option<Colour>,
    /// What shape a control that sweeps a range takes.
    control: Option<Control>,
}

impl Figure {
    /// Returns what the algorithm's own panel is made of.
    fn of(panel: Option<&'static Panel>) -> Self {
        Self {
            surface: panel.map(Panel::face),
            cap: panel.map(Panel::cap),
            control: panel.map(Panel::control),
        }
    }

    /// Returns the room one of the algorithm's slots is drawn in.
    ///
    /// A knob where the figure draws one, which is 29 of the 35, and the size
    /// an effect plate draws a control at rather than the size a rack slot is
    /// cut to. The measured cap goes on whatever the shape is, and everything
    /// else about the room — how far a drag runs, most of all — is untouched.
    fn room(self, spare: bool) -> Room {
        let room = match self.control {
            Some(Control::Knob) => Room::SLOT.turned().sized(if spare { SPARE } else { BODY }),
            // A fader is widened rather than turned, and by less: a knob grows
            // in both directions at once and a fader only across its travel,
            // so the two end up the same weight in a column at different
            // numbers.
            _ if spare => Room::SLOT,
            _ => Room::SLOT.sized(TRACK),
        };
        match self.cap {
            Some(worn) => room.worn(colour(worn)),
            None => room,
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
    let surface = figure.surface;
    column![
        row![
            at(lane.parameter, surface),
            reference(lane.slot, surface),
            Space::new().width(Length::Fixed(2.0)),
            modulated(
                moved.contains(&lane.parameter),
                lane.slot.is_none_or(|slot| slot.modulatable),
            ),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
        control(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
            figure.room(false),
        ),
        readout(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
        ),
        container(
            text(title)
                .size(11)
                .center()
                .style(move |theme: &Theme| text::Style {
                    color: Some(writing(theme, surface)),
                })
        )
        .height(Length::Fixed(TITLE))
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center),
        container(printing(hint(lane.slot), surface).size(10).center())
            .height(Length::Fixed(HINT))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(5)
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// Draws the bytes the loaded algorithm has no name for.
///
/// Under the grid and not on it. An algorithm can leave seven of its twelve
/// unused, and seven controls the size of the five that do something is a plate
/// whose loudest half is the half that does nothing. So they are a strip cut
/// into the plate below the two rows: smaller, under one heading that says what
/// they are, and still every one of them draggable — a byte in the program that
/// no panel reaches is a byte the modulation matrix can still be pointed at.
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
        let title = within(engine, lane.parameter);
        column![
            row![
                at(lane.parameter, None),
                Space::new().width(Length::Fixed(2.0)),
                modulated(moved.contains(&lane.parameter), true),
            ]
            .spacing(4)
            .align_y(Vertical::Center),
            control(
                lane.parameter,
                patch.value(lane.parameter),
                patch.claim(lane.parameter),
                firmware,
                figure.room(true),
            ),
            readout(
                lane.parameter,
                patch.value(lane.parameter),
                patch.claim(lane.parameter),
                firmware,
            ),
            printing(title.to_owned(), None).size(10),
        ]
        .spacing(3)
        .width(Length::Fixed(SPARE + 34.0))
        .align_x(Horizontal::Center)
        .into()
    });
    Some(
        container(
            column![
                printing(
                    match lanes.len() {
                        1 => "one byte this algorithm does not use".to_owned(),
                        many => format!("{many} bytes this algorithm does not use"),
                    },
                    None,
                )
                .size(10),
                row(bytes).spacing(6).wrap(),
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .padding(8)
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
/// across the whole page rather than packed at the left of it. That is the
/// arrangement anybody who has edited an effect on the hardware already knows,
/// and it is the arrangement a second row can be read down against.
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
    let mut drawn = column![].spacing(10);
    for line in &grid {
        drawn = drawn.extend(over(line, figure));
        drawn = drawn.push(line_of(patch, engine, line, firmware, moved, figure));
    }
    // What the display will show where a slot shows names rather than a number,
    // printed once under the grid: the manual gives the names and never the
    // bytes they sit at, so this is a reading and not something to send.
    let shown: Vec<Element<'a, Renderer>> = displays(&lanes, figure.surface)
        .map(Element::from)
        .collect();
    if !shown.is_empty() {
        drawn = drawn.push(column(shown).spacing(1));
    }
    // The controls sit on the figure's own face, inside its chassis, which is
    // the two colours the library measures for exactly those two things.
    let surface = figure.surface;
    let drawn = container(drawn)
        .width(Length::Fill)
        .padding(10)
        .style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(face(theme, surface))),
            border: Border {
                color: materials(theme).recess_edge,
                width: 1.0,
                radius: 2.into(),
            },
            ..container::Style::default()
        });
    let body = column![header(patch, engine, firmware, moved, figure)]
        .push(drawn)
        .extend(spare(patch, engine, &left, firmware, moved, figure))
        .spacing(8);
    let figure_colours = panel.map(|panel| (panel.chassis(), panel.accent()));
    container(body)
        .padding(8)
        .width(Length::Fill)
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

/// Returns the colour the plate an engine is drawn on is painted.
///
/// The measured chassis of the algorithm's own figure, most of the way back to
/// the panel this window is. The library publishes four colours per algorithm —
/// [deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)
/// — and they are the colours of 35 imaginary rack units: a cream fader panel,
/// a black one, a blue-grey one. Painted as measured, four of them side by side
/// in this window would be a collage of other people's instruments, and the one
/// thing this editor is is one instrument.
///
/// So the measurement was spent as an identity rather than as a finish, back
/// when four plates stood on the surface at once. One does now, and that is
/// what changed: a single unit can wear its own case without the window turning
/// into a shelf of other people's boxes, so the chassis is carried far enough
/// to be the colour of the thing rather than a tint on the panel.
fn chassis(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    figure.map_or(material.panel, |(chassis, _)| {
        style::mix(material.panel, colour(chassis), 0.55)
    })
}

/// Returns the colour the controls of an open engine stand on.
///
/// The library measures the case and the surface inside it separately, because
/// a rack unit has both: a cream panel screwed into a black chassis, a dark
/// face inset in a grey one. Drawing only the case threw half of that away and
/// left the controls floating on it.
fn face(theme: &Theme, surface: Option<Colour>) -> Color {
    let material = materials(theme);
    surface.map_or(material.plate, |face| {
        style::mix(material.plate, colour(face), 0.5)
    })
}

/// Returns the colour of the strip a band's name is knocked out of.
///
/// The instrument's own device — a pale bar across the top of a group — carried
/// onto whatever the plate is wearing, so that the bar reads as printed on this
/// unit rather than borrowed from the window behind it.
fn banding(theme: &Theme, surface: Option<Colour>) -> Color {
    style::mix(face(theme, surface), materials(theme).metal_low, 0.62)
}

/// Returns the ink a word on an engine's face is printed in.
///
/// Whichever of the instrument's two the face can be read against. A measured
/// face is a cream panel as often as it is a black one, and a window that
/// printed every title in the one ink it uses everywhere else would have picked
/// the wrong one for half the algorithms.
fn writing(theme: &Theme, surface: Option<Colour>) -> Color {
    style::ink_on(face(theme, surface), theme)
}

/// The same ink, at the strength a legend is silkscreened in.
///
/// Part of the way back to the surface it is printed on, rather than a grey
/// chosen for a dark window: a unit and a legend do not have to be told apart
/// twice, and the one thing a legend must be is readable on the face it is on.
/// What this replaced was the tint of a value nobody has read, which said of a
/// printed word that its value was unknown.
fn printing<'a, Renderer>(
    what: String,
    surface: Option<Colour>,
) -> iced_widget::Text<'a, Theme, Renderer>
where
    Renderer: TextRenderer,
{
    text(what).style(move |theme: &Theme| text::Style {
        color: Some(style::mix(
            writing(theme, surface),
            face(theme, surface),
            0.45,
        )),
    })
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

/// Draws an engine's own settings: which algorithm, and how loud its output.
///
/// The strip across the top of the unit, cut into its case: the number
/// stencilled on the left, the algorithm chosen from the parameter's own value
/// table under the abbreviations the instrument's display prints, what those
/// stand for written out beside the list rather than substituted into it, and
/// the output gain lying along the right of the band. One surface and one
/// baseline, because four controls floating at four heights over the same empty
/// space was the part of this page that read as unfinished.
fn header<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let loaded = algorithm(patch, engine, firmware);
    let named = loaded.map_or_else(
        || {
            patch.value(engine.algorithm_parameter()).map_or_else(
                || "nothing has been read".to_owned(),
                |value| format!("{value}, which firmware {firmware} does not name"),
            )
        },
        |algorithm| algorithm.full_name.to_owned(),
    );
    let category = loaded.map_or("", |algorithm| algorithm.category);
    let surface = figure.surface;
    container(
        row![
            container(
                text(format!("FX {}", engine.number()))
                    .size(18)
                    .font(reading())
                    .style(move |theme: &Theme| text::Style {
                        color: Some(writing(theme, surface)),
                    })
            )
            .width(Length::Fixed(NUMERAL))
            .height(Length::Fill)
            .align_y(Vertical::Center),
            cell(
                patch,
                engine.algorithm_parameter(),
                firmware,
                Room::listed(ALGORITHM),
                within(engine, engine.algorithm_parameter()),
                moved,
                surface,
            ),
            container(
                column![
                    text(named)
                        .size(15)
                        .style(move |theme: &Theme| text::Style {
                            color: Some(writing(theme, surface)),
                        }),
                    printing(category.to_owned(), surface).size(11),
                ]
                .spacing(1)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Vertical::Center),
            cell(
                patch,
                engine.gain_parameter(),
                firmware,
                Room::across(GAIN),
                within(engine, engine.gain_parameter()),
                moved,
                surface,
            ),
        ]
        .spacing(14)
        .align_y(Vertical::Bottom),
    )
    .width(Length::Fill)
    .padding([6, 10])
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(style::mix(
            face(theme, surface),
            materials(theme).recess,
            0.3,
        ))),
        border: Border {
            color: materials(theme).recess_edge,
            width: 1.0,
            radius: 2.into(),
        },
        ..container::Style::default()
    })
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
    surface: Option<Colour>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        row![
            at(parameter, surface),
            Space::new().width(Length::Fixed(2.0)),
            modulated(moved.contains(&parameter), true),
            Space::new().width(Length::Fixed(4.0)),
            text(title)
                .size(11)
                .style(move |theme: &Theme| text::Style {
                    color: Some(writing(theme, surface)),
                }),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
        control(parameter, value, claim, firmware, room),
        readout(parameter, value, claim, firmware),
    ]
    .spacing(4)
    .width(Length::Fixed(room.width()))
    .into()
}

/// Draws where a parameter lives, in the ink of the face it is printed on.
///
/// The NRPN number, which is also the parameter's byte offset in a dump, so one
/// number is both its name on the wire and its address in memory.
/// [`panel::address`](crate::panel::address) is the same thing on the window's
/// own plate; a plate wearing a measured face needs it in that face's ink,
/// because the grey a rack prints it in is a grey half the 35 chassis are.
fn at<'a, Renderer>(parameter: ParamId, surface: Option<Colour>) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    printing(parameter.offset().to_string(), surface)
        .size(10)
        .font(reading())
        .into()
}

/// Draws the abbreviation the instrument's display prints for a slot.
///
/// `PST`, `DCY`, `HMQ`: the word a player reads on the hardware while looking
/// at this panel, so that the two say the same thing. Nothing for a byte the
/// algorithm does not use, which has no display name because it is not on the
/// display.
fn reference<'a, Renderer>(
    slot: Option<&'static FxSlot>,
    surface: Option<Colour>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let name = slot.map_or("", |slot| slot.reference);
    printing(name.to_owned(), surface)
        .size(10)
        .font(reading())
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
    // A byte the algorithm does not use has its own strip under the grid, and
    // the strip's heading says what they are once rather than seven times.
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
    surface: Option<Colour>,
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
        .map(move |line| printing(line, surface).size(10).font(reading()))
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

    use super::{Lane, claimed, engines, hint, lanes, placed, settings, spans, within};
    use crate::Patch;

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

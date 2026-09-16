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
//! # The page is the instrument's own, and so is the shape
//!
//! 26.3 published the rest of it
//! ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)):
//! the grid the synthesizer's own FX page lays a slot out on, measured off the
//! 35 screenshots in the manual, and what the figure printed beside each
//! algorithm is made of. So a plate is no longer twelve slots wrapped into
//! whatever width the window happened to have — it is the rows that page puts
//! them in, six across and two down, which is the arrangement anybody who has
//! edited an effect on the hardware already knows. [`Panel::control`] is the
//! shape: 29 of the 35 figures are rotary knobs, five are faders and one is a
//! numeric display, and a slot on a knob panel is drawn as [a knob](crate::knob).
//!
//! The four measured colours are spent as an identity rather than as a finish.
//! They are the colours of 35 imaginary rack units, and four of them painted as
//! measured, side by side, would be a collage of other people's instruments in
//! a window whose whole argument is that it is one instrument. So the plate
//! carries enough of the algorithm's chassis to tell the reverb from the
//! distortion at arm's length and a hairline of its accent around the edge, and
//! the controls stay the editor's own: a cap is what carries the claim, and a
//! cap repainted to match a figure would be the one rule this window keeps
//! everywhere, broken where it is least affordable.
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

use deepmind_midi::effect::{Algorithm, Align, Colour, Control, Engine, FxSlot, Panel};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Color, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{Space, column, container, row, text};

use crate::chain;
use crate::panel::{NAME, Room, SLOT, address, control, modulated, readout};
use crate::style::{self, materials, reading};
use crate::{Confidence, Element, Patch, tint};

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
const GAIN: f32 = 120.0;

/// Height of the line a slot's reading is described on.
///
/// One line, always taken whether or not there is anything to say on it, so
/// that the slots of one plate line up however many of them have units.
const HINT: f32 = 14.0;

/// Height of the heading over a band of slots.
const BAND: f32 = 15.0;

/// One of an engine's twelve bytes, and what the loaded algorithm calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lane {
    /// The parameter that addresses the byte, which is the same one whatever
    /// the engine is running.
    parameter: ParamId,
    /// What the algorithm says the byte is, when it uses it at all.
    slot: Option<&'static FxSlot>,
}

/// What a run of lanes belongs together as.
///
/// The library groups the slots that are one side of a stereo engine or one
/// band of an equaliser, and says so as a label it derived from the parameter
/// names rather than as a fact the manual prints. It is a convention for laying
/// a panel out, which is exactly what this uses it for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Band {
    /// The slots the library labelled, which are always a run of them.
    Named(&'static str),
    /// A slot the library labelled nothing, which stands on its own.
    Alone,
    /// The bytes the loaded algorithm does not use.
    Unused,
}

impl Band {
    /// Returns the heading printed over the band.
    ///
    /// Empty for a slot standing on its own, which still takes the line, so
    /// that a plate of clusters lines up whether or not its algorithm has
    /// bands in it.
    const fn heading(self) -> &'static str {
        match self {
            Self::Named(label) => label,
            Self::Alone => "",
            Self::Unused => "not used",
        }
    }

    /// Returns whether another lane of this band joins the one before it.
    ///
    /// A named band and the unused tail are runs; a slot with no label is its
    /// own cluster, because two unlabelled slots side by side are two slots and
    /// not a pair.
    const fn runs(self) -> bool {
        !matches!(self, Self::Alone)
    }

    /// Returns the band a lane belongs to.
    const fn of(lane: Lane) -> Self {
        match lane.slot {
            None => Self::Unused,
            Some(slot) => match slot.group {
                Some(label) => Self::Named(label),
                None => Self::Alone,
            },
        }
    }
}

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

/// Gathers the lanes into the clusters they are drawn in.
///
/// The library's own runs: a band of an equaliser, a side of a stereo engine,
/// and the tail of bytes the algorithm does not use. Everything else stands
/// alone.
fn bands(lanes: &[Lane]) -> Vec<(Band, Vec<Lane>)> {
    let mut clusters: Vec<(Band, Vec<Lane>)> = Vec::new();
    for lane in lanes.iter().copied() {
        let band = Band::of(lane);
        match clusters.last_mut() {
            Some((last, run)) if *last == band && band.runs() => run.push(lane),
            _ => clusters.push((band, vec![lane])),
        }
    }
    clusters
}

/// Draws the effects, when `group` is the group that holds them.
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
            )
        })
        .collect::<Vec<_>>();
    // The picture first and the settings under it, which is the arrangement
    // every plate of the front panel has: a display over the controls it is a
    // drawing of.
    let mut body = column![chain::display(patch, firmware)].spacing(10);
    if let Some(note) = chain::note(patch) {
        body = body.push(muted(note.to_owned()).size(11));
    }
    if !block.is_empty() {
        body = body.push(row(block).spacing(10).wrap());
    }
    for engine in engines {
        body = body.push(plate(patch, engine, firmware, moved));
    }
    Some(body.into())
}

/// Gathers the lanes into the rows the instrument's own FX page draws them in.
///
/// The library's grid, row by row, with the alignment it measured for each;
/// then whatever the algorithm does not use, on a row of its own. A slot the
/// page has no place for cannot happen — the rows are generated from the same
/// specification as the slots — and if one ever did it would follow the unused
/// tail rather than disappear, because a byte this panel does not draw is a
/// byte the modulation matrix can still be pointed at.
///
/// One row of everything where nothing is known about the algorithm, which is
/// stage 3's rack: twelve bytes under the library's own names, and no page to
/// lay them out by.
fn rows(lanes: &[Lane], panel: Option<&'static Panel>) -> Vec<(Vec<Lane>, Align)> {
    let mut drawn = Vec::new();
    let mut placed: Vec<ParamId> = Vec::new();
    if let Some(panel) = panel {
        for row in panel.rows() {
            let line: Vec<Lane> = row
                .slots()
                .iter()
                .filter_map(|number| {
                    lanes
                        .iter()
                        .find(|lane| lane.slot.is_some_and(|slot| slot.slot == *number))
                        .copied()
                })
                .collect();
            placed.extend(line.iter().map(|lane| lane.parameter));
            if !line.is_empty() {
                drawn.push((line, row.align()));
            }
        }
    }
    let left: Vec<Lane> = lanes
        .iter()
        .filter(|lane| !placed.contains(&lane.parameter))
        .copied()
        .collect();
    if !left.is_empty() {
        drawn.push((left, Align::Left));
    }
    drawn
}

/// Draws one row of a plate: its clusters, each under the band it belongs to.
fn line_of<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    line: &[Lane],
    firmware: Version,
    moved: &[ParamId],
    panel: Option<&'static Panel>,
) -> Vec<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    bands(line)
        .into_iter()
        .map(|(band, run)| {
            let slots = run
                .into_iter()
                .map(|lane| slot(patch, engine, lane, firmware, moved, panel));
            column![
                container(muted(band.heading().to_owned()).size(10)).height(Length::Fixed(BAND)),
                row(slots).spacing(0),
            ]
            .into()
        })
        .collect()
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
fn slot<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    lane: Lane,
    firmware: Version,
    moved: &[ParamId],
    panel: Option<&'static Panel>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let title = lane
        .slot
        .map_or_else(|| within(engine, lane.parameter), |slot| slot.title);
    let room = if panel.is_some_and(|panel| matches!(panel.control(), Control::Knob)) {
        Room::SLOT.turned()
    } else {
        Room::SLOT
    };
    column![
        row![
            address(lane.parameter),
            reference(lane.slot),
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
            room,
        ),
        readout(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
        ),
        // The title and what the display reads there are one label and not two
        // rows: a slot whose title runs to three lines pushes its own ends down
        // rather than the whole plate's. The box is the rack's name box and one
        // line more, so that the slots of a band still begin at the same height.
        container(
            column![
                text(title).size(11).center(),
                muted(hint(lane.slot)).size(10).center(),
            ]
            .spacing(2)
            .align_x(Horizontal::Center)
        )
        .height(Length::Fixed(NAME + HINT))
        .width(Length::Fill)
        .align_x(Horizontal::Center),
    ]
    .spacing(5)
    .width(Length::Fixed(SLOT))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws one engine: what it is running, and what that makes its twelve bytes.
///
/// In the rows the instrument's own FX page puts them in. 26.3 published that
/// grid — six columns and two rows, measured off the 35 screenshots in the
/// manual, and every slot's place on it
/// ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22))
/// — so a plate is no longer twelve slots wrapped into whatever width the
/// window happened to have. It is the arrangement the instrument's own display
/// uses, which is the arrangement anybody who has edited an effect on the
/// hardware already knows.
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
    let mut drawn = column![].spacing(2);
    for (line, align) in rows(&lanes, panel) {
        let clusters = line_of(patch, engine, &line, firmware, moved, panel);
        drawn = drawn.push(
            container(row(clusters).spacing(8).wrap())
                .width(Length::Fill)
                .align_x(match align {
                    Align::Right => Horizontal::Right,
                    Align::Centre => Horizontal::Center,
                    // Left, and anything a later library adds: every row of
                    // every algorithm as measured is packed against the start,
                    // and a row this build has no arrangement for is drawn the
                    // way the other 35 are rather than guessed at.
                    _ => Horizontal::Left,
                }),
        );
    }
    let body = column![header(patch, engine, firmware, moved)]
        .push(drawn)
        .extend(displays(&lanes).map(Element::from))
        .spacing(8);
    let figure = panel.map(|panel| (panel.chassis(), panel.accent()));
    container(body)
        .padding(8)
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            // Cut into the group's face plate rather than raised off it: the
            // plate is what the four engines are recessed into, which is the
            // same trick the section bar plays with the panel it is cut from.
            container::Style {
                background: Some(Background::Color(chassis(theme, figure))),
                border: Border {
                    color: rim(theme, figure),
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
/// So the measurement is spent as an identity rather than as a finish: enough
/// of the chassis to tell the reverb from the delay at arm's length, and not so
/// much that the plate stops being the panel the rest of the window is.
fn chassis(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    figure.map_or(material.panel, |(chassis, _)| {
        style::mix(material.panel, colour(chassis), 0.16)
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
/// The algorithm is chosen from the parameter's own value table, under the
/// abbreviations the instrument's display prints, and what those stand for is
/// written out beside the list rather than substituted into it: the control is
/// the library's, and the prose is the panel's.
fn header<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
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
    row![
        container(text(engine.to_string()).size(15))
            .width(Length::Fixed(44.0))
            .height(Length::Fill)
            .align_y(Vertical::Center),
        cell(
            patch,
            engine.algorithm_parameter(),
            firmware,
            Room::listed(ALGORITHM),
            within(engine, engine.algorithm_parameter()),
            moved,
        ),
        container(column![text(named).size(13), muted(category.to_owned()).size(11)].spacing(2))
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
        ),
    ]
    .spacing(12)
    .align_y(Vertical::Bottom)
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
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        row![
            address(parameter),
            Space::new().width(Length::Fixed(2.0)),
            modulated(moved.contains(&parameter), true),
            Space::new().width(Length::Fixed(4.0)),
            text(title).size(11),
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
    text(name)
        .size(10)
        .font(reading())
        .style(move |theme: &Theme| text::Style {
            color: Some(tint(theme, Confidence::Unknown)),
        })
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
    // The heading over the cluster says the algorithm does not use these, once
    // rather than seven times.
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
/// plate rather than in the slot, because that is what they are: a reading and
/// not something to send. The manual gives the names and never the bytes they
/// sit at, so the control stays the byte and this says what the display will
/// make of it.
fn displays<'a, Renderer>(
    lanes: &[Lane],
) -> impl Iterator<Item = iced_widget::Text<'a, Theme, Renderer>>
where
    Renderer: TextRenderer,
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
    printed.into_iter().map(|line| muted(line).size(10))
}

/// Grey text, for what is not a value.
fn muted<'a, Renderer>(what: String) -> iced_widget::Text<'a, Theme, Renderer>
where
    Renderer: TextRenderer,
{
    text(what).style(move |theme: &Theme| text::Style {
        color: Some(tint(theme, Confidence::Unknown)),
    })
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
    use deepmind_midi::effect::{Algorithm, Engine, SLOTS_PER_ENGINE};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, ParamId};
    use deepmind_midi::program::Program;
    use deepmind_midi::sysex::inquiry::Version;

    use super::{Band, Lane, bands, claimed, engines, hint, lanes, rows, settings, within};
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

    #[test]
    fn every_byte_of_an_engine_is_drawn_once_whatever_is_loaded() {
        // The plate is laid out from the library's own grid now, and the one
        // thing a layout read out of a table can do wrong is lose a row of it:
        // a byte with no control on it is a byte nobody can edit, and a byte
        // with two is a plate that disagrees with itself. Neither is visible in
        // a screenshot of the one algorithm somebody happened to load.
        for algorithm in Algorithm::all() {
            for engine in Engine::ALL {
                let lanes: Vec<Lane> = engine
                    .slot_parameters()
                    .iter()
                    .copied()
                    .map(|parameter| Lane {
                        parameter,
                        slot: algorithm.slot_of(engine, parameter),
                    })
                    .collect();
                let mut drawn: Vec<ParamId> = rows(&lanes, Some(algorithm.panel()))
                    .into_iter()
                    .flat_map(|(line, _)| line)
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
            assert!(hint(lane.slot).is_empty(), "the band heading says it once");
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
    fn the_bands_are_the_runs_the_library_groups() {
        // The Midas equaliser's slots are four bands and an output.
        let patch = running("MidasEQ");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        let bands = bands(&lanes);

        let named: Vec<&str> = bands
            .iter()
            .filter_map(|(band, _)| match band {
                Band::Named(label) => Some(*label),
                _ => None,
            })
            .collect();
        assert_eq!(named, vec!["low", "low-mid", "high-mid", "high"]);
        // Every one of the twelve is in exactly one cluster, in slot order.
        let clustered: Vec<ParamId> = bands
            .iter()
            .flat_map(|(_, run)| run.iter().map(|lane| lane.parameter))
            .collect();
        assert_eq!(
            clustered,
            lanes.iter().map(|lane| lane.parameter).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_slot_the_library_labelled_nothing_stands_on_its_own() {
        let patch = running("RoomRev");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        let bands = bands(&lanes);

        for (band, run) in &bands {
            if *band == Band::Alone {
                assert_eq!(run.len(), 1, "two unlabelled slots became a pair");
            }
        }
    }

    #[test]
    fn the_unused_bytes_are_one_cluster_at_the_end() {
        let patch = running("TC-DeepVRB");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        let bands = bands(&lanes);

        let unused: Vec<&(Band, Vec<super::Lane>)> = bands
            .iter()
            .filter(|(band, _)| *band == Band::Unused)
            .collect();
        assert_eq!(unused.len(), 1, "the tail is one cluster");
        assert_eq!(unused.first().map(|(_, run)| run.len()), Some(7));
        assert_eq!(bands.last().map(|(band, _)| *band), Some(Band::Unused));
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

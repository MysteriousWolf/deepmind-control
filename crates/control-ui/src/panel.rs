//! One section of the instrument, laid out as the instrument lays it out.
//!
//! A rack of slots rather than a list of rows. Each slot is one parameter: its
//! address, the control, the value, and the name, in that order down the panel,
//! wrapping onto the next line when the window runs out of room. It is the
//! arrangement the library's own effect panels use, and the arrangement the
//! synthesizer's display uses, because they are drawings of the same thing.
//!
//! What a slot draws depends on what the library says the parameter is: a sweep
//! gets a fader, two states get a lamp, a named set gets its names. Nothing here
//! decides what a value means; that is the library's table and this is the
//! pixels.
//!
//! A panel laid out by hand changes the arrangement and never what a control
//! is. The [name](crate::name) is the one place even that is bent, because
//! seventeen parameters holding one character each are one word to the person
//! reading them: the slots are still the library's, and seventeen of them are
//! drawn as the display the instrument shows them on.
//!
//! What a hand layout may do is give a control more room, or a different way to
//! run in it. [`Room`] is how it says so, and it is the only thing that changes
//! between a rack's slot and the [matrix](crate::matrix)'s rows: the control in
//! either is whatever the library says the parameter is, chosen by the same
//! code from the same table.

use core::fmt;

use deepmind_midi::param::{Group, Kind, ParamId, Shape};
use deepmind_midi::program::ProgramName;
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::gradient::Linear;
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Border, Color, Font, Gradient, Length, Radians, Rectangle, Size, Theme, Widget,
    border, mouse, renderer, text::Renderer as TextRenderer,
};
use iced_widget::{Space, button, column, container, mouse_area, pick_list, row, stack, text};

use crate::effect;
use crate::envelope;
use crate::fader::{self, Axis, fader};
use crate::knob::{self, knob};
use crate::mapping::{Mapper, Mapping, Sent};
use crate::matrix;
use crate::name;
use crate::sequencer;
use crate::style::{self, materials, reading};
use crate::{Confidence, Element, Patch, tint};

/// Width of one slot, which is the fader plus the room a name needs either side.
pub(crate) const SLOT: f32 = 88.0;

/// Height of the name, so that slots in a row line up whatever their names do.
///
/// Three lines of it. `VCF Envelope Velocity Sensitivity` is the longest name
/// in the instrument once its group is taken off the front, and a box that
/// clips it is a box that lies about which fader is which.
pub(crate) const NAME: f32 = 44.0;

/// How tall a control that is not a fader stands in a row of them.
///
/// A fader lying on its side, which is what makes a row of switches, lists and
/// lamps one band whatever is standing in it. Named because the front panel has
/// to know it: the band along the foot of a plate is as tall as this, and a
/// plate is as tall as its parts.
pub(crate) const BUTTON: f32 = fader::WIDTH;

/// How wide the cap of a button that lights is.
///
/// A fader's width, so that a lamp in a rack's slot is the same width as the
/// fader in the slot beside it and a row of controls is one row.
pub(crate) const CAP: f32 = fader::WIDTH;

/// How tall that cap stands.
///
/// Square-ish, and taller than a line of text needs. The buttons on a
/// `DeepMind` are moulded rubber about half again as wide as they are tall, and
/// a cap drawn as tall as its own label is a menu item with a light behind it:
/// the height is what makes it read as something a finger presses rather than
/// something a pointer clicks.
pub(crate) const PRESS: f32 = 28.0;

/// How tall one lit legend of a named set stands.
///
/// One line of the reading face at the size a legend is set in, and no padding
/// above or below it: a column of seven has to fit beside two faders on the
/// front panel, which is the tightest room a named set is ever lit in.
///
/// Given rather than taken, because a column is laid out into the room it was
/// given and a legend past the end of that room is drawn no lines tall. That is
/// a set silently naming fewer things than the library says it has, rather than
/// one that overflows where somebody would see it.
pub(crate) const LIT: f32 = 13.0;

/// How far apart two lit legends stand.
pub(crate) const BETWEEN: f32 = 1.0;

/// How tall a column of `count` lit legends stands.
///
/// What a hand layout has to give a named set for all of it to be drawn. The
/// front panel asks, because the instrument lights its LFO shapes beside the
/// faders rather than under them and the room beside a fader is the room a
/// fader runs in.
pub(crate) fn lit_band(count: usize) -> f32 {
    let count = f32::from(u16::try_from(count).unwrap_or(u16::MAX));
    (count * LIT + (count - 1.0) * BETWEEN).max(0.0)
}

/// Longest named set that is drawn as lit legends rather than as a list.
///
/// Beyond this a list is the honest control: the modulation matrix has 130
/// destinations, and a column of 130 legends is a joke at the reader's expense.
const LEGENDS: usize = 6;

/// Returns whether a parameter's choices are lit rather than listed.
///
/// The rule [`drawn`] follows, asked in advance: a hand layout that has to
/// decide how much room a control needs before it draws it wants the same
/// answer the control will give itself, and a second rule written here would
/// be a second rule to keep in step.
pub(crate) fn lit_rather_than_listed(parameter: ParamId, firmware: Version) -> bool {
    choices(parameter, firmware, None).is_some_and(|options| options.len() <= LEGENDS)
}

/// How much of the panel a control has been given, and which way it runs.
///
/// A slot in the rack gives a fader its own width and a list no more than the
/// slot it stands in; a row of a hand-laid-out table gives a list the width its
/// names need and turns a fader onto its side. It is the whole of what hand
/// layout is allowed to change about a control, which is why it is one type
/// rather than an argument each.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Room {
    /// Which way a fader runs in it.
    axis: Axis,
    /// How much room across the panel, which is a fader's travel when it runs
    /// across it.
    width: f32,
    /// How tall the control stands, so that a rack's slots line up whatever is
    /// in them and a row is only as tall as what it holds.
    height: Length,
    /// How long a fader's travel is when it runs down the panel.
    travel: f32,
    /// Whether there is room to light a named set rather than list it.
    ///
    /// A list is the honest control for a set too long to read at a glance,
    /// and how long that is depends on the room: a rack's slot has none to
    /// spare, and the strip of legends beside the instrument's own LFO faders
    /// has seven. It is room and not identity: the same enumerated parameter,
    /// from the same table, with the same values under it.
    legends: bool,
    /// Which shape a control that sweeps a range takes.
    ///
    /// The last thing hand layout is allowed to change about a control, and
    /// the newest: the library publishes what shape each effect algorithm's
    /// own figure draws, and 29 of the 35 are knobs. A knob and a fader are the
    /// same control over the same byte with the same drag (see
    /// [`knob`](crate::knob)), so this belongs beside the axis a fader runs
    /// along rather than anywhere near what a parameter is.
    form: Form,
    /// How big the thing a hand takes hold of is drawn, when the room is more
    /// than the control's own size.
    ///
    /// A rack's slot is cut to its fader, so the two are the same number and
    /// this is `None`. An effect plate's column is not: the instrument's own FX
    /// page puts six of them across a whole page, so a column there is twice a
    /// slot wide and a control drawn at the column's width would be a knob the
    /// size of a fist.
    body: Option<f32>,
    /// How many columns a lit set stands in.
    ///
    /// One, everywhere the instrument itself lights a set: `Sine`, `Triangle`,
    /// `Square` beside an LFO's faders are a column, because that is how they
    /// are silkscreened. A set too long to read down in one, such as the ten
    /// topologies the effects block can be wired in, is the same lamps in
    /// several columns, which is a shape a page has room for where a column of
    /// ten is a page of nothing else.
    across: usize,
    /// Whether the control takes the room left over rather than its own.
    ///
    /// A rack's slot is cut to its fader and a table's cell to its longest
    /// name, so both of those are a number. A set of lamps laid out in columns
    /// across a band of its own is not: what it wants is the band, and a width
    /// written down for it is a band half full with the rest of the page blank
    /// beside it.
    fills: bool,
}

/// What a control that sweeps a range is drawn as.
///
/// Named for the thing a hand touches rather than for the value's own shape,
/// which is what the library's [`Shape`] already means: one says a step is read
/// about its centre and the other says the control is round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Form {
    /// A fader, which is every control the instrument itself has.
    #[default]
    Fader,
    /// A knob, which is what an effect algorithm's own panel draws.
    Knob,
}

impl Room {
    /// The room a slot in the rack gives.
    ///
    /// Every slot is the same height whatever its control, because a rack of
    /// forty that jostles is a rack nobody can read across.
    pub(crate) const SLOT: Self = Self {
        axis: Axis::Down,
        width: fader::WIDTH + 28.0,
        height: Length::Fixed(fader::HEIGHT),
        travel: fader::HEIGHT,
        legends: false,
        form: Form::Fader,
        body: None,
        across: 1,
        fills: false,
    };

    /// Room for one lane of the instrument's own front panel.
    ///
    /// Narrower than a slot and shorter, because the panel holds two rows of
    /// eighteen and the hardware's own faders are a third the length of the
    /// ones in a rack.
    pub(crate) const fn lane(width: f32, travel: f32) -> Self {
        Self {
            axis: Axis::Down,
            width,
            height: Length::Fixed(travel),
            travel,
            legends: false,
            form: Form::Fader,
            body: None,
            across: 1,
            fills: false,
        }
    }

    /// The same lane, with room to light a named set rather than list it.
    pub(crate) const fn lamps(width: f32, height: f32) -> Self {
        Self {
            axis: Axis::Down,
            width,
            height: Length::Fixed(height),
            travel: height,
            legends: true,
            form: Form::Fader,
            body: None,
            across: 1,
            fills: false,
        }
    }

    /// Room to light a long named set in `across` columns of legends.
    ///
    /// What a set too long to read down in one column gets when the page has the
    /// width for it: the same lamps, in the same order, wrapped. The height is
    /// taken rather than given, as many rows as the columns need, because a
    /// legend past the end of the room is one of the choices silently missing.
    pub(crate) const fn spread(width: f32, across: usize) -> Self {
        Self {
            axis: Axis::Down,
            width,
            height: Length::Shrink,
            travel: fader::HEIGHT,
            legends: true,
            form: Form::Fader,
            body: None,
            across,
            fills: true,
        }
    }

    /// Returns how much room across, as a length a widget can be given.
    ///
    /// A number for everything cut to its own contents, and the room left over
    /// for a set of lamps laid out across a band of its own.
    pub(crate) const fn across_as(self) -> Length {
        if self.fills {
            Length::Fill
        } else {
            Length::Fixed(self.width)
        }
    }

    /// Room for something chosen from a list in a row, `width` points of it.
    ///
    /// As tall as a fader lying on its side, so that a table's cells are one
    /// band whatever is standing in them and its columns line up.
    pub(crate) const fn listed(width: f32) -> Self {
        Self {
            axis: Axis::Down,
            width,
            height: Length::Fixed(BUTTON),
            travel: fader::HEIGHT,
            legends: false,
            form: Form::Fader,
            body: None,
            across: 1,
            fills: false,
        }
    }

    /// The same, taking the room it is given rather than a width of its own.
    ///
    /// What a column of a table that fills the window asks for: the width is
    /// the row's to share out, and a control that carried one of its own would
    /// be a column that stopped where it was written to stop.
    pub(crate) const fn filling_list() -> Self {
        Self {
            fills: true,
            ..Self::listed(0.0)
        }
    }

    /// Room for one lane of a strip, `width` points across.
    ///
    /// Narrower than a slot and as tall, so a row of thirty-two stands as one
    /// block the width of the rack beneath it.
    pub(crate) const fn step(width: f32) -> Self {
        Self {
            axis: Axis::Down,
            width,
            height: Length::Fixed(fader::HEIGHT),
            travel: fader::HEIGHT,
            legends: false,
            form: Form::Fader,
            body: None,
            across: 1,
            fills: false,
        }
    }

    /// Room for a fader lying on its side, `width` points of travel long.
    pub(crate) const fn across(width: f32) -> Self {
        Self {
            axis: Axis::Across,
            width,
            height: Length::Fixed(fader::WIDTH),
            travel: fader::HEIGHT,
            legends: false,
            form: Form::Fader,
            body: None,
            across: 1,
            fills: false,
        }
    }

    /// The same room, with a knob in it rather than a fader.
    ///
    /// What an effect slot is given where the algorithm's own panel draws a
    /// knob. Everything else about the room is untouched, because everything
    /// else about the control is: the library says which shape the figure
    /// beside an algorithm uses, and a shape is an arrangement.
    pub(crate) const fn turned(self) -> Self {
        Self {
            form: Form::Knob,
            ..self
        }
    }

    /// The same room, with the control drawn `across` points rather than
    /// filling it.
    ///
    /// For a panel whose columns are wider than its controls, which is what the
    /// effects page became when it took the instrument's own six-column grid
    /// and gave it a whole page to lie on. The travel is untouched: how far a
    /// drag runs is what makes every control in this window move at one rate,
    /// and it is not a thing a layout may bargain with.
    ///
    /// A fader standing up takes it only to narrow, because a fader is as wide
    /// as its cap and there is one cap in this window; a knob is as wide as it
    /// is drawn, and a fader lying down is as thick as it is asked to be.
    pub(crate) const fn sized(self, across: f32) -> Self {
        Self {
            body: Some(across),
            ..self
        }
    }

    /// Returns which way the control in this room travels.
    ///
    /// Asked by what is drawn *over* a control rather than by the control
    /// itself: a band saying how far a routing can push it has to run the way
    /// the control runs, or it is a bar beside a fader rather than a reading of
    /// it. A knob keeps whichever axis its room was cut with, because a knob's
    /// sweep is not a direction on the panel and a bar along one edge of it is
    /// a scale rather than a picture of the dial.
    pub(crate) const fn along(self) -> Axis {
        self.axis
    }
}

/// What a view in this crate asks for.
///
/// Seven things, and the last four never reach a wire: a parameter should move,
/// the program should be called something, a section should be the one on the
/// screen, the pointer has come to rest on a control or on a press, a routing
/// is being mapped onto the window, or a drag while it is mapped has said where
/// and how much. What an edit costs on a wire, when it goes out and what it
/// goes out behind is the host crate's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Message {
    /// A parameter should move to this value.
    Edit {
        /// The parameter to move.
        parameter: ParamId,
        /// Where to move it, as the program byte it is stored as.
        value: u8,
    },
    /// The program should carry this name.
    ///
    /// The one message that is not about a single parameter, because the one
    /// control that is not: a name is seventeen of them, and a person editing
    /// it is editing a word. [`Patch::rename`](crate::Patch::rename) is what
    /// turns the word back into the parameters that moved.
    Rename(ProgramName),
    /// A section should be the one on the screen.
    ///
    /// Two hundred and forty-two parameters do not fit on a screen and are not
    /// laid out on the instrument as one surface either. Which panel is in
    /// front of somebody is the application's state and not the synthesizer's,
    /// so this is the one message that goes nowhere near the port.
    Show(Group),
    /// The pointer is over this control, or has left the one it was over.
    ///
    /// A panel of forty faders under four-letter legends is only readable
    /// because a hand can ask what one of them is, and this is the asking. The
    /// answer is drawn somewhere else, because the application decides where a
    /// footer goes, so all a view does is say what is under the pointer.
    ///
    /// It never reaches a wire. Looking at a control is not editing it.
    Pointed(Option<ParamId>),
    /// The pointer is over a press, or has left the one it was over.
    ///
    /// The same question [`Pointed`](Message::Pointed) asks about a control,
    /// for the things in this window that are not parameters: the marks along
    /// the header and the foot, and the two presses that move a routing up and
    /// down the matrix. A press whose whole face is a nine-dot mark has nowhere
    /// to put a word, and a word beside it is a word on the panel whether or not
    /// anybody is asking, so the answer goes where this window already says what
    /// is under the pointer.
    ///
    /// It is what the press says about itself rather than anything read from
    /// the instrument, which is why it is a string and not a parameter.
    Hinted(Option<&'static str>),
    /// This routing is being mapped onto the window, or none is any more.
    ///
    /// While one is, every control the matrix can reach is lit across all three
    /// surfaces and none of them edits anything: the next one somebody takes
    /// hold of is where the routing goes. See [`Mapper`].
    ///
    /// It never reaches a wire either. Choosing where to point something is not
    /// pointing it.
    Mapper(Option<Mapping>),
    /// A drag on a control while a routing is mapped onto the window.
    ///
    /// Both bytes are already worked out, by the view that knows which control
    /// the drag is on and what it was holding before the drag began: the
    /// destination the control answers to, and the depth its travel asks for.
    /// Two [`Edit`](Message::Edit)s, in other words, and they are one message
    /// because they are one gesture.
    Reach {
        /// Where the routing goes, and the byte that names what was taken hold
        /// of.
        destination: ParamId,
        /// That byte.
        at: u8,
        /// How much of it arrives, and the byte the drag's travel asks for.
        depth: ParamId,
        /// That byte.
        by: u8,
    },
    /// Two routings should trade places.
    ///
    /// Six parameters, three at a time, and the pairs are already worked out by
    /// the view that knows which two rows are being swapped. It is one message
    /// because it is one gesture, for the reason [`Reach`](Message::Reach) is:
    /// somebody moving a routing up the table is moving a routing, not editing
    /// six bytes.
    ///
    /// Nothing about the sound changes. The eight routings are read as a set
    /// and the instrument does not care which of them says what, so this is a
    /// rearrangement for whoever has to read the table next, which is the one
    /// thing a matrix of eight identical slots gives somebody no way to do.
    Swap {
        /// One routing's source, destination and depth.
        one: [ParamId; 3],
        /// The other's, in the same order.
        other: [ParamId; 3],
    },
}

/// Draws one group of parameters.
///
/// `firmware` is what a device inquiry reported, and it is not decoration:
/// firmware 1.1 renumbered three value tables, so 17 of 23 modulation sources
/// mean something else on 1.0. Until a synthesizer has answered, the caller
/// passes the library's default and says so on the screen.
#[must_use]
pub fn group<'a, Renderer>(
    patch: &'a Patch,
    group: Group,
    firmware: Version,
    mapper: &'a Mapper,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // Seventeen parameters the library calls `Program Name Char 1` to `17`
    // are one field, drawn where the first of them sits, so the rack keeps the
    // instrument's own order and loses seventeen faders nobody could name a
    // sound on.
    //
    // A panel drawn as a table takes its own parameters out of the rack, and
    // leaves anything it did not claim in it: a group that grows a parameter no
    // row knows about keeps it as a slot rather than losing it to a layout.
    // The routing mapped onto the window, if one is. Read once for the panel:
    // it reaches every control drawn below and it is the same answer for all of
    // them.
    let mapping = mapper.mapped();
    // Read once for the rack, for the same reason `moved` is: every control
    // below asks the same question of the same eight routings.
    let reaches = mapping
        .map(|_| matrix::reaching(patch, firmware))
        .unwrap_or_default();
    let sent = mapping.map(|mapping| Sent::new(mapping, &reaches));
    let routed = matrix::routed(group);
    let stepped = sequencer::stepped(group);
    let claimed = effect::claimed(group);
    // What the eight routings are pointed at, so a slot the matrix moves says
    // so. Read once for the panel rather than once per slot: it is eight
    // lookups either way, and forty slots asking the same question is forty.
    let moved = matrix::moved(patch, firmware);
    let slots: Vec<Element<'a, Renderer>> = group
        .parameters()
        .filter(|parameter| {
            !routed.contains(parameter)
                && !stepped.contains(parameter)
                && !claimed.contains(parameter)
        })
        .filter_map(|parameter| {
            if name::holds(parameter) {
                return name::begins(parameter).then(|| name::field(patch));
            }
            Some(slot(patch, parameter, firmware, &moved, sent))
        })
        .collect();
    // An envelope's meaning is a picture, so the picture goes above its rack,
    // and the modulation matrix is eight sentences, so they are read across
    // above what is left of one. The slots themselves are untouched: hand
    // layout changes the arrangement and never what a control is.
    let mut body = column![].spacing(10);
    if let Some(shape) = envelope::shape(patch, group) {
        body = body.push(shape);
    }
    if let Some(table) = matrix::table(patch, group, firmware, mapper) {
        body = body.push(table);
    }
    if let Some(strip) = sequencer::strip(patch, group, firmware, sent) {
        body = body.push(strip);
    }
    if let Some(engines) = effect::panels(patch, group, firmware, &moved, sent) {
        body = body.push(engines);
    }
    if !slots.is_empty() {
        body = body.push(row(slots).spacing(0).wrap());
    }
    container(body)
        .padding(8)
        .style(|theme: &Theme| {
            // The face plate the library's own panels draw their slots on.
            let material = materials(theme);
            container::Style {
                // Raised off the panel, so the recesses cut into it read as
                // cut into something. A plate the colour of its own slots is a
                // plate with invisible slots.
                background: Some(Background::Color(material.plate)),
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 3.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Draws one parameter: its address, its control, its value and its name.
fn slot<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
    moved: &[ParamId],
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        row![
            address(parameter),
            Space::new().width(Length::Fixed(4.0)),
            modulated(moved.contains(&parameter), true),
        ]
        .align_y(Vertical::Center),
        control(parameter, value, claim, firmware, Room::SLOT, sent),
        readout(parameter, value, claim, firmware),
        container(text(parameter.short_name()).size(11).center())
            .height(Length::Fixed(NAME))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(5)
    .width(Length::Fixed(SLOT))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws where a parameter lives.
///
/// The NRPN number, which is also the parameter's byte offset in a dump, so one
/// number is both its name on the wire and its address in memory.
pub(crate) fn address<'a, Renderer>(parameter: ParamId) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    text(parameter.offset().to_string())
        .size(10)
        .font(reading())
        .style(move |theme: &Theme| text::Style {
            color: Some(tint(theme, Confidence::Unknown)),
        })
        .into()
}

/// Draws what a parameter is holding, in the colour of what backs it.
pub(crate) fn readout<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    text(shown(parameter, value, firmware))
        .size(13)
        .font(reading())
        .style(move |theme: &Theme| text::Style {
            color: Some(tint(theme, claim)),
        })
        .into()
}

/// Draws the control a parameter is edited with.
///
/// A parameter of a sound nobody has read still draws its control, because an
/// empty slot in a rack of forty is harder to read than a fader with no cap on
/// it. The control says so by having nothing to take hold of.
pub(crate) fn control<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
    room: Room,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // Every control in this editor is drawn through here: a lane of the front
    // panel, a slot of a rack, a step of the sequencer, a byte of an effect. So
    // this is the one place that has to notice a pointer for all of them to say
    // what they are. It is also the one place that has to answer the modulation
    // matrix when a routing is mapped onto the window, for the same reason: a
    // mode that lit the controls of one panel would be a mode that stopped at
    // the edge of the panel somebody is looking at.
    let asked = For::of(sent, parameter, firmware);
    let drawn = drawn(parameter, value, claim, firmware, room, asked);
    let Some(sent) = sent else {
        return mouse_area(drawn)
            .on_enter(Message::Pointed(Some(parameter)))
            .on_exit(Message::Pointed(None))
            .into();
    };
    // Over the control rather than around it. A border drawn in a container
    // would be two points of layout this panel does not have, and every control
    // in the window would move the moment a routing was pointed, which is a
    // window that jumps when somebody is about to map onto something in it.
    let reached = sent.mapping().names(parameter, firmware);
    // What is already there, drawn on the control it is already there on.
    // Somebody choosing where a routing goes is choosing against the other
    // seven, and a second routing onto the same filter corner is a thing people
    // do on purpose and a thing people do by accident.
    let swings: Vec<(f32, f32)> = value
        .filter(|_| reached.is_some())
        .map(|value| {
            sent.already(parameter)
                .map(|reach| reach.swing(value))
                .collect()
        })
        .unwrap_or_default();
    let lit = stack![
        drawn,
        mapping_light(reached.is_some(), room.along(), swings)
    ];
    let area = mouse_area(lit)
        .on_enter(Message::Pointed(Some(parameter)))
        .on_exit(Message::Pointed(None));
    // A press is taken by the control under it, which is what makes a drag
    // possible; the release is what says *this one*. A control the matrix
    // cannot reach answers nothing at all, so a click on it neither points the
    // routing nor loses what was already being mapped.
    match reached {
        Some(value) => area.on_release(Message::Edit {
            parameter: sent.mapping().destination(),
            value,
        }),
        None => area,
    }
    .into()
}

/// What is laid over a control while a routing is mapped onto the window.
///
/// Three things, and the first two are the whole of the mode. A control the
/// matrix can reach is lit; a control it cannot reach is covered by the panel
/// it stands on until it is barely there. Lit and dimmed rather than lit and
/// left alone, because a page of forty faders with six outlined is a page
/// somebody has to search; the same page with thirty-four of them faded is a
/// page with six faders on it.
///
/// The third is what is already there: a band along the control's own travel
/// for each of the other seven routings that lands on it, showing how far that
/// one can push it. See [`Reach::swing`](crate::Reach::swing) for what that
/// band assumes.
///
/// None of them takes an event. A stack hands what lands on it to what is
/// under it, and what is under it is the control.
fn mapping_light<'a, Renderer>(
    reached: bool,
    along: Axis,
    swings: Vec<(f32, f32)>,
) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    Element::new(Light {
        reached,
        along,
        swings,
    })
}

/// The lamp, and what is already wired to the control under it.
#[derive(Debug)]
struct Light {
    reached: bool,
    along: Axis,
    swings: Vec<(f32, f32)>,
}

impl Light {
    /// Where one routing's band is drawn, in `bounds`.
    ///
    /// Along the control's own travel and against the near edge of it: a fader
    /// that runs down the panel gets a bar up its left-hand side, and one that
    /// runs across gets a bar along its top. `lane` is which of them this is,
    /// so that two routings onto one control are two bars rather than one bar
    /// drawn twice.
    fn band(&self, bounds: Rectangle, swing: (f32, f32), lane: usize) -> Rectangle {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a lane of the few a control has room for"
        )]
        let out = EDGE + lane as f32 * (BAND + APART);
        let (from, to) = swing;
        match self.along {
            // Nothing at the foot and one at the head, which is the way up
            // every drawing in this window reads and the way a fader's own cap
            // sits on its travel.
            Axis::Down => {
                let top = bounds.y + bounds.height * (1.0 - to);
                Rectangle {
                    x: bounds.x + out,
                    y: top,
                    width: BAND,
                    height: (bounds.height * (to - from)).max(BAND),
                }
            }
            Axis::Across => Rectangle {
                x: bounds.x + bounds.width * from,
                y: bounds.y + out,
                width: (bounds.width * (to - from)).max(BAND),
                height: BAND,
            },
        }
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Light
where
    Renderer: iced_core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
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
        // A control that is not part of the question, covered by the panel it
        // stands on. What is left is enough to see that there is a control
        // there and not enough to read it.
        if !self.reached {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: PASSED_OVER,
                    ..material.panel
                }),
            );
            return;
        }
        // The lamp: light coming up through the panel from behind it, which is
        // how a `DeepMind` says a press is on. It was a hairline rectangle
        // round the control, and a hairline rectangle is a focus ring on a web
        // page rather than anything on a piece of equipment.
        //
        // Brightest at the foot and falling away across it, because that is
        // what a lamp behind a panel does and it is the rule every other lit
        // surface in this window is drawn under; the display's own glass is two
        // stops of the same argument. A flat fill of one colour is a highlight;
        // this is a light.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default().rounded(3.0),
                ..renderer::Quad::default()
            },
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(core::f32::consts::PI))
                    .add_stop(
                        0.0,
                        Color {
                            a: FAR,
                            ..style::MODULATION
                        },
                    )
                    .add_stop(
                        1.0,
                        Color {
                            a: NEAR,
                            ..style::MODULATION
                        },
                    ),
            )),
        );
        // And the wall the light is coming past, lit hardest of all: the one
        // thing every cut surface in this window has in common is that it
        // catches the light along the edge the light reaches. A track does it,
        // a plate does it, and a display does it.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + 1.0,
                    y: bounds.y + bounds.height - EDGE - 1.0,
                    width: (bounds.width - 2.0).max(0.0),
                    height: EDGE,
                },
                border: Border::default().rounded(EDGE / 2.0),
                ..renderer::Quad::default()
            },
            Background::Color(Color {
                a: CATCH,
                ..style::MODULATION
            }),
        );
        for (lane, swing) in self.swings.iter().copied().enumerate() {
            let band = self.band(bounds, swing, lane);
            // A band that has run off the control is a band nobody can read
            // against it, which is what happens to the fourth routing onto one
            // slot. The three that fit are the ones drawn.
            if !bounds.contains(band.position()) {
                break;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: band,
                    border: Border::default().rounded(BAND / 2.0),
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: ALREADY,
                    ..style::MODULATION
                }),
            );
        }
    }
}

/// How hard the lamp shows through the panel at the foot of a lit control.
///
/// Where the light enters. Faint even here: it is a ground the control is still
/// read against, and a fill that competed with the cap on a fader would be a
/// mode that hid the values it was asking somebody to choose between.
const NEAR: f32 = 0.20;

/// How hard it still shows at the head of one, which is most of the way to
/// nothing.
const FAR: f32 = 0.03;

/// How hard the wall the light comes past catches it.
const CATCH: f32 = 0.6;

/// How thick that lit edge is.
const EDGE: f32 = 1.5;

/// How thick the band for one routing already landing here is.
const BAND: f32 = 2.5;

/// How much panel there is between two of those bands.
const APART: f32 = 1.5;

/// How hard a band is drawn.
///
/// Harder than the lamp and softer than the aperture: what is already wired to
/// a control is a fact about the patch, and the lamp is a question about it.
const ALREADY: f32 = 0.75;

/// How much of the panel is laid over a control the matrix cannot reach.
///
/// Most of it. What is left is enough to see that there is a control there and
/// not enough to read it, which is the right amount for something that is not
/// part of the question being asked.
const PASSED_OVER: f32 = 0.72;

/// Draws the control itself, as whatever the library says the parameter is.
fn drawn<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
    room: Room,
    asked: For,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let low = u8::try_from(parameter.min()).unwrap_or(u8::MIN);
    let high = u8::try_from(parameter.max()).unwrap_or(u8::MAX);
    let idle = idle(parameter);
    match parameter.kind() {
        // A switch is two states, and the library says which parameters are
        // switches. Where it also says one accepts 256 values, the two answers
        // contradict each other and the lamp is the one that cannot be right:
        // it offers 0 and 1 and no way to reach the other 254. `Seq Step Value
        // 9` and `11` are the two that say it today, from a `kind = "switch"`
        // in `spec/parameters.toml` that their own range and their own note
        // disagree with. Drawing the sweep is the reading that loses nothing.
        Kind::Switch if parameter.max() <= 1 => lamp(parameter, value, claim, room, asked),
        Kind::Enumerated(_) => match choices(parameter, firmware, value) {
            Some(options) if options.len() <= LEGENDS || room.legends => {
                legends(parameter, &options, value, claim, room, asked)
            }
            Some(options) => list(parameter, options, value, room, asked),
            // A table that does not name this value is a table that would drop
            // the value on the next click, so the raw number stays draggable.
            None => sweep(
                parameter,
                low..=high,
                value.unwrap_or(idle),
                claim,
                room,
                asked,
            ),
        },
        // A sweep, and anything a later library adds that this build has not
        // heard of: every parameter is a number underneath.
        _ => sweep(
            parameter,
            low..=high,
            value.unwrap_or(idle),
            claim,
            room,
            asked,
        ),
    }
}

/// Returns where a control stands when this window has no value to stand it at.
///
/// Nothing is drawn to take hold of either way (see the cap in
/// [`fader`](crate::fader)), so this is where the *track* is read from, and for
/// a value read about a centre that is the centre. A modulation depth nobody
/// has read, drawn at the floor of its own range, is a control sitting at
/// `-128` on a page whose whole subject is how much of something arrives: the
/// window would be showing full negative modulation where it means to be
/// showing that it has not asked.
fn idle(parameter: ParamId) -> u8 {
    let low = u8::try_from(parameter.min()).unwrap_or(u8::MIN);
    match parameter.shape() {
        Shape::Bipolar { centre } => u8::try_from(centre).unwrap_or(low),
        // Unipolar, and anything a later library adds: a range that counts up
        // from somewhere starts where it counts from.
        _ => low,
    }
}

/// What a control is being drawn for.
///
/// Three states, and the second two are the modulation matrix pointing a
/// routing at the window. They decide what a control *does* and never what it
/// is: a lamp is a lamp in all three, a list says which value is chosen in all
/// three, and a fader has its cap in all three. What changes is whether taking
/// hold of one moves the sound, moves the routing, or does nothing at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum For {
    /// Editing the sound, which is every control almost all of the time.
    Editing,
    /// Answering a routing that is mapped onto the window, at a control the
    /// matrix can reach: the routing, and the destination byte that names this
    /// control.
    Mapping(Mapping, u8),
    /// The same, at a control it cannot reach. The control is drawn and does
    /// nothing, because it is not part of the question being asked.
    Passed,
}

impl For {
    /// What a control is being drawn for, given what the matrix is asking.
    fn of(sent: Option<Sent<'_>>, parameter: ParamId, firmware: Version) -> Self {
        match sent {
            None => Self::Editing,
            Some(sent) => sent
                .mapping()
                .names(parameter, firmware)
                .map_or(Self::Passed, |at| Self::Mapping(sent.mapping(), at)),
        }
    }

    /// Returns whether a control drawn for this is one somebody can move.
    ///
    /// Only while editing. A routing being mapped is a question about where it
    /// goes, and a control that answered it by also changing the sound would be
    /// answering a question nobody asked.
    const fn edits(self) -> bool {
        matches!(self, Self::Editing)
    }
}

/// Draws a fader over a parameter's whole range.
fn sweep<'a, Renderer>(
    parameter: ParamId,
    range: core::ops::RangeInclusive<u8>,
    value: u8,
    claim: Confidence,
    room: Room,
    asked: For,
) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let low = *range.start();
    let high = *range.end();
    let held = value.clamp(low, high);
    // The one gesture the two modes share. While the matrix is mapping, the
    // drag is the same drag over the same range with the same relative grab,
    // and what comes out of it is the depth that travel asks for rather than
    // the value it would have reached, so the number under the hand is still
    // "how far did I move it", which is the only question a hand can answer
    // about an amount it has not heard yet.
    let moved = move |to: u8| match asked {
        For::Mapping(mapped, at) => Message::Reach {
            destination: mapped.destination(),
            at,
            depth: mapped.depth(),
            by: mapped.depth_of(parameter, held, to),
        },
        // A control the matrix cannot reach never publishes, because it is
        // built inert below; this is the arm that says so.
        _ if !asked.edits() => Message::Pointed(Some(parameter)),
        _ => Message::Edit {
            parameter,
            value: to,
        },
    };
    let live = !matches!(asked, For::Passed);
    if matches!(room.form, Form::Knob) {
        // A knob is as wide as it is tall and takes the room across the panel
        // it was given, which for a slot in the rack is the fader's own width
        // and for a column of an effect plate is the size that plate asked for.
        return knob(range, held, claim, moved)
            .live(live)
            .size(room.body.unwrap_or_else(|| room.width.min(knob::SIZE)))
            .into();
    }
    let fader = fader(range, held, claim, moved).live(live);
    let across = room.body.unwrap_or(room.width);
    match room.axis {
        // A fader takes as much room across as it is given and never more than
        // it needs: a slot gives it more than its width, and a strip's lane
        // gives it less, which is the lane it draws in. How long it runs is the
        // room's too, because the instrument's own panel holds two rows of them.
        Axis::Down if across < fader::WIDTH => fader.narrow(across).travel(room.travel).into(),
        Axis::Down => fader.travel(room.travel).into(),
        // A fader lying down is narrowed the same way, so that a gain along the
        // top of an effect panel is a band of that panel's header rather than a
        // rack fader turned on its side in a row half its height.
        Axis::Across if room.body.is_some() => fader.across(room.width).narrow(across).into(),
        Axis::Across => fader.across(room.width).into(),
    }
}

/// Draws a lamp: a moulded cap, lit for on, with what it says on its face.
///
/// Not a checkbox and not something that slides. An instrument says *on* with a
/// light, and this is the only place a saturated colour appears.
///
/// The cap is [the panel's own](CAP): square-ish, rounded the way a rubber
/// button is moulded, and lit across its crown rather than filled flat, so that
/// a row of them along the foot of a plate reads as the row of buttons a
/// photograph of the instrument shows. What the panel prints about it goes
/// beside the cap and never on it, since the front panel prints its legends
/// under the buttons, which is [`home`](crate::home)'s business rather than this
/// one's. What stays on the face is the reading, which is the one thing the
/// hardware's own lamp says by being lit.
///
/// A switch nobody has read is drawn as the switch it is, because the shape of a
/// control does not depend on whether a sound has arrived, but as the hole
/// without the cap in it, the way a fader nobody has read is a track with
/// nothing to take hold of. That distinction used to be carried by the word
/// printed inside the cap, which is the one job the words were really doing;
/// taking them off means the cap has to do it, which is where it belonged.
fn lamp<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    room: Room,
    asked: For,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let on = value.is_some_and(|value| value != 0);
    let next = u8::from(!on);
    let live = !matches!(claim, Confidence::Unknown) && asked.edits();
    // Nothing is written on the cap. A button on the instrument is a blank
    // piece of rubber that is lit or is not, and the word `off` printed inside
    // an unlit one is this window explaining a control the control already
    // states, and every legend the panel does print is silkscreened beside the
    // cap, where a finger cannot cover it.
    let face = button(Space::new())
        .width(Length::Fixed(CAP.min(room.width)))
        .height(Length::Fixed(PRESS))
        .padding(0)
        .style(move |theme: &Theme, status| capped(theme, on, claim, status));
    let face = if live {
        face.on_press(Message::Edit {
            parameter,
            value: next,
        })
    } else {
        face
    };
    container(face)
        .height(room.height)
        .width(Length::Fixed(room.width))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
}

/// Draws a named set as legends, one of them lit.
///
/// A column of them, which is how the instrument silkscreens a set beside the
/// faders it belongs to, or several columns where the room says so (see
/// [`Room::spread`]). Down each column and then across, so that reading it the
/// way the instrument numbers the values is reading down.
fn legends<'a, Renderer>(
    parameter: ParamId,
    options: &[Choice],
    value: Option<u8>,
    claim: Confidence,
    room: Room,
    asked: For,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let live = !matches!(claim, Confidence::Unknown) && asked.edits();
    let lamp = |choice: &Choice| {
        let on = Some(choice.byte()) == value;
        let byte = choice.byte();
        let face = button(text(choice.name).size(10).font(reading()))
            .padding([0, 5])
            .width(Length::Fill)
            .height(Length::Fixed(LIT))
            .style(move |theme: &Theme, _status| lit(theme, on, claim));
        if live {
            face.on_press(Message::Edit {
                parameter,
                value: byte,
            })
            .into()
        } else {
            Element::from(face)
        }
    };
    let across = room.across.max(1);
    // As many rows as it takes, so that the last column is the short one: a set
    // of ten in three columns is four, four and two, and the one that is two is
    // the last rather than one somewhere in the middle of the reading.
    let down = options.len().div_ceil(across);
    let columns = options.chunks(down.max(1)).map(|chunk| {
        Element::from(
            column(chunk.iter().map(lamp))
                .spacing(BETWEEN)
                .width(Length::Fill),
        )
    });
    container(
        row(columns)
            .spacing(if across > 1 { BESIDE } else { 0.0 })
            .width(room.across_as()),
    )
    .height(room.height)
    .align_y(Vertical::Center)
    .into()
}

/// How far apart two columns of lit legends stand.
///
/// Wider than the gap down a column, so that a set read down reads as columns
/// rather than as a block of words.
const BESIDE: f32 = 6.0;

/// Draws a named set too long for legends as the list it is.
fn list<'a, Renderer>(
    parameter: ParamId,
    options: Vec<Choice>,
    value: Option<u8>,
    room: Room,
    asked: For,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let selected = options
        .iter()
        .find(|choice| Some(choice.byte()) == value)
        .copied();
    if !asked.edits() {
        // A list that opened while the matrix was pointing would be a list
        // whose choice nobody wanted made, over a menu covering the controls
        // somebody is trying to map onto. So it is the name it is showing, in the
        // recess the list was in, and the click goes to the control.
        return container(
            text(selected.map_or_else(|| "\u{2014}".to_owned(), |choice| choice.name.to_owned()))
                .size(11),
        )
        .padding([2, 6])
        .width(Length::Fixed(room.width))
        .height(room.height)
        .align_y(Vertical::Center)
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
                background: Some(Background::Color(material.recess)),
                border: border::rounded(2).width(1.0).color(material.recess_edge),
                text_color: Some(material.metal_low),
                ..container::Style::default()
            }
        })
        .into();
    }
    container(
        pick_list(options, selected, move |choice: Choice| Message::Edit {
            parameter,
            value: choice.byte(),
        })
        .text_size(11)
        .padding([2, 6])
        .width(Length::Fixed(room.width)),
    )
    .height(room.height)
    .align_y(Vertical::Center)
    .into()
}

/// The style one legend of a lit set is drawn in.
///
/// Flat, and not [a cap](capped): a strip of legends is the row of lamps beside
/// the instrument's own LFO faders, where the light is behind the name rather
/// than under a finger, and seven moulded caps in the room two faders leave
/// would be seven buttons nobody can press. So this carries the claim the same
/// way a cap does, filled for a fact, an outline for a claim and neither for a
/// value nobody has read, and nothing else about it is a button.
fn lit(theme: &Theme, on: bool, claim: Confidence) -> button::Style {
    let material = materials(theme);
    let colour = tint(theme, claim);
    let confirmed = claim.is_confirmed();
    button::Style {
        background: Some(Background::Color(if on && confirmed {
            colour
        } else {
            material.panel
        })),
        text_color: if on {
            if confirmed { material.panel } else { colour }
        } else {
            material.metal_low
        },
        border: border::rounded(2)
            .width(1.0)
            .color(if on { colour } else { material.recess_edge }),
        ..button::Style::default()
    }
}

/// The style a cap is drawn in: lit, outlined, or dark.
///
/// The same rule the fader's cap follows. Filled for what the synthesizer
/// reported, an outline for what this window claims, and neither for a value
/// nobody has read, so the fill carries the difference and the colour agrees
/// with it.
///
/// Whatever it is carrying, it is [moulded](style::moulded): an unlit button on
/// the instrument is still a rubber cap standing in the panel, and drawing that
/// one as a hole and the lit one as a light would be two controls wearing one
/// name. So the dark state is the panel's own colour moulded, and what lighting
/// it changes is the colour and not the shape.
fn capped(theme: &Theme, on: bool, claim: Confidence, status: button::Status) -> button::Style {
    let material = materials(theme);
    let colour = tint(theme, claim);
    let confirmed = claim.is_confirmed();
    let held = matches!(status, button::Status::Pressed);
    // A cap a pointer is over is lit a little before it is pressed, which is
    // the whole of what hovering means on a panel that has no cursor.
    let hover = matches!(status, button::Status::Hovered);
    // Nothing read is the hole with no cap in it: the recess the cap would be
    // moulded into, flat and unlit, which is the same thing a fader says by
    // drawing its track and no cap. Nothing is written on these any more, so
    // this is what tells an unread switch from one that is switched off, and a
    // flat recess against a moulded cap is a difference in relief rather than
    // in colour, so it survives the greyscale the rest of the panel survives.
    if matches!(claim, Confidence::Unknown) {
        return button::Style {
            background: Some(Background::Color(material.recess)),
            text_color: material.metal_low,
            border: border::rounded(style::MOULD)
                .width(1.0)
                .color(material.recess_edge),
            ..button::Style::default()
        };
    }
    let face = match (on, confirmed) {
        (true, true) => colour,
        (true, false) => style::mix(material.panel, colour, 0.22),
        (false, _) => style::mix(
            material.panel,
            material.metal_low,
            if hover { 0.22 } else { 0.12 },
        ),
    };
    button::Style {
        background: Some(style::moulded(face, held)),
        text_color: if on {
            if confirmed { material.panel } else { colour }
        } else {
            material.metal_low
        },
        border: border::rounded(style::MOULD).width(1.0).color(if on {
            colour
        } else {
            material.recess_edge
        }),
        ..button::Style::default()
    }
}

/// Draws the mark that says the modulation matrix is mapped onto this parameter.
///
/// The one saturated thing on the panel, and it means one thing: something
/// other than a hand can move this control. A parameter nothing is mapped at
/// keeps the space, so a rack does not jostle when a routing changes.
///
/// `heeded` is whether the value arriving there does anything, which is a
/// question only the effects can answer no to: the library says of a slot
/// whether its engine acts on modulation reaching it, and every slot is
/// addressable from the matrix regardless. A routing mapped somewhere the
/// engine ignores gets the mark as an outline, because the matrix really is
/// pointed there and really is doing nothing, and an editor that drew that the
/// same way as an effective routing would be hiding the reason a sound is not
/// moving.
pub(crate) fn modulated<'a, Renderer>(moved: bool, heeded: bool) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    container(Space::new())
        .width(Length::Fixed(5.0))
        .height(Length::Fixed(5.0))
        .style(move |_theme: &Theme| container::Style {
            background: (moved && heeded).then_some(Background::Color(style::MODULATION)),
            border: if moved && !heeded {
                border::rounded(3).width(1.0).color(style::MODULATION)
            } else {
                border::rounded(3)
            },
            ..container::Style::default()
        })
        .into()
}

/// Draws the dot that says what backs a value.
pub(crate) fn dot<'a, Renderer>(claim: Confidence) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    container(Space::new())
        .width(Length::Fixed(9.0))
        .height(Length::Fixed(9.0))
        .style(move |theme: &Theme| {
            let colour = tint(theme, claim);
            container::Style {
                // Filled for what was reported, outlined for what was claimed:
                // the difference survives a screen nobody can see colour on.
                background: claim.is_confirmed().then_some(Background::Color(colour)),
                border: border::rounded(5).width(1.0).color(colour),
                ..container::Style::default()
            }
        })
        .into()
}

/// The number, or the name the instrument's display would show for it.
///
/// Raw where the library has no table, because inventing a plausible "2.4 kHz"
/// for a byte is wrong in a way nobody can see. A measured curve arrives in the
/// library, parameter by parameter, and this picks it up when it upgrades.
pub(crate) fn shown(parameter: ParamId, value: Option<u8>, firmware: Version) -> String {
    let Some(value) = value else {
        return "\u{2014}".to_owned();
    };
    match parameter.kind() {
        // The control already carries the name, so this carries the byte.
        Kind::Switch | Kind::Enumerated(_) => sits_at(parameter, value),
        _ => parameter
            .label_for(u16::from(value), firmware)
            .map_or_else(|| sits_at(parameter, value), str::to_owned),
    }
}

/// What the raw byte reads as, which is not always the raw byte.
///
/// Two things the library publishes about a value beyond the range it sits in,
/// and both of them change what a number means rather than how it is drawn:
///
/// - A **bipolar** parameter is read about a centre, so `128` on a modulation
///   depth is not "half way up" but *no modulation at all*, and the reading is
///   the signed distance from there. A matrix of eight depths set to nothing
///   reading `128` eight times is a panel stating the wrong musical fact in the
///   most confident way available to it.
/// - An **inactive** value means "not set" rather than the smallest setting.
///   Zero on a sequencer step skips the step; it is not the quietest one.
///
/// Both are [`ParamId::shape`] and [`ParamId::inactive`], published by
/// `deepmind-midi` 26.3, and until then this printed the byte and the strip's
/// own documentation said which fact it was getting wrong.
pub(crate) fn sits_at(parameter: ParamId, value: u8) -> String {
    if parameter.inactive() == Some(u16::from(value)) {
        return "skip".to_owned();
    }
    match parameter.shape() {
        Shape::Bipolar { centre } => {
            let from = i32::from(value) - i32::from(centre);
            // A sign on every reading of a bipolar control, the zero included:
            // `0` and `+0` are the same number and only one of them says the
            // control it is under has two directions.
            format!("{from:+}")
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
#[expect(clippy::panic, reason = "a failed expectation is the test failure")]
mod readings {
    use super::sits_at;
    use deepmind_midi::param::{ParamId, Shape};

    #[test]
    fn a_control_nobody_has_read_stands_where_its_range_is_read_from() {
        // The floor for a value that counts up from one, and the centre for a
        // value read about one. Both are places the control is drawn with
        // nothing to take hold of; only one of them is a picture of full
        // negative modulation.
        let Shape::Bipolar { centre } = ParamId::Mod1Depth.shape() else {
            panic!("a depth is read about its centre");
        };

        assert_eq!(
            super::idle(ParamId::Mod1Depth),
            u8::try_from(centre).unwrap_or_default()
        );
        assert_eq!(
            super::idle(ParamId::Lfo1Rate),
            u8::try_from(ParamId::Lfo1Rate.min()).unwrap_or_default()
        );
    }

    #[test]
    fn a_bipolar_value_is_read_about_its_centre_and_not_from_the_floor() {
        // The fact the library published and this used to get wrong: a
        // modulation depth at 128 is no modulation, and eight of them reading
        // `128` was a matrix stating the wrong musical fact eight times.
        let depth = ParamId::Mod1Depth;
        let Shape::Bipolar { centre } = depth.shape() else {
            panic!("{depth} is the bipolar one this guards");
        };

        assert_eq!(
            sits_at(depth, u8::try_from(centre).unwrap_or_default()),
            "+0"
        );
        assert_eq!(
            sits_at(depth, u8::try_from(centre).unwrap_or_default() + 40),
            "+40"
        );
        assert_eq!(
            sits_at(depth, u8::try_from(centre).unwrap_or_default() - 12),
            "-12"
        );
    }

    #[test]
    fn a_unipolar_value_is_still_the_byte() {
        assert_eq!(sits_at(ParamId::VcfFrequency, 200), "200");
        assert_eq!(sits_at(ParamId::VcfFrequency, 0), "0");
    }

    #[test]
    fn a_step_of_nothing_is_a_skipped_step_and_not_the_quietest_one() {
        let step = ParamId::SeqStepValue1;

        assert_eq!(step.inactive(), Some(0), "the library says zero skips it");
        assert_eq!(sits_at(step, 0), "skip");
        assert_ne!(sits_at(step, 1), "skip", "and only zero does");
    }
}

/// One value of a named set, as a list shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Choice {
    value: u16,
    name: &'static str,
}

impl Choice {
    /// Returns the value as the program byte it is stored as.
    pub(crate) fn byte(self) -> u8 {
        u8::try_from(self.value).unwrap_or(u8::MAX)
    }

    /// Returns the name the instrument's own display prints for it.
    pub(crate) const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// The names a table gives, in the order the table gives them.
///
/// `None` where the library says the table does not name every value the
/// parameter accepts: some of them list only the start of a documented run, and
/// a control that silently drops the values it has no name for is a control
/// that moves the sound when somebody opens it. `ParamId::choices_for` is that
/// question answered where the table lives, from `deepmind-midi` 26.2; this
/// crate counted the run itself until then.
///
/// Also `None` when the table names every value the parameter accepts and the
/// one it is holding is not among them, which a dump from hardware can do. The
/// list would open on nothing, so the raw number stays draggable instead.
///
/// A parameter nobody has read is not that case: there is no value to be
/// missing from the table, and the control is drawn as the named set it is with
/// nothing chosen in it.
pub(crate) fn choices(
    parameter: ParamId,
    firmware: Version,
    value: Option<u8>,
) -> Option<Vec<Choice>> {
    let entries = parameter.choices_for(firmware)?;
    let options: Vec<Choice> = entries
        .iter()
        .map(|entry| Choice {
            value: entry.value,
            name: entry.name,
        })
        .collect();
    if let Some(value) = value {
        options
            .iter()
            .find(|choice| choice.value == u16::from(value))?;
    }
    Some(options)
}

#[cfg(test)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};

    use crate::effect;
    use crate::matrix;
    use crate::sequencer;

    #[test]
    fn every_parameter_is_drawn_once_whatever_its_panel_is_laid_out_as() {
        // A hand-laid-out table takes its parameters out of the rack, and what
        // it did not claim stays in it. Two ways to lose a parameter: a table
        // that claims one twice, and a rack that filters one out for a table
        // that was not drawing it. A group nobody has laid out by hand passes
        // this trivially, which is the point: the rule is the same for all
        // fourteen.
        for group in Group::ALL.iter().copied() {
            let routed = matrix::routed(group);
            let stepped = sequencer::stepped(group);
            let claimed = effect::claimed(group);
            let slots: Vec<ParamId> = group
                .parameters()
                .filter(|parameter| {
                    !routed.contains(parameter)
                        && !stepped.contains(parameter)
                        && !claimed.contains(parameter)
                })
                .collect();

            let mut drawn: Vec<ParamId> = routed;
            drawn.extend(stepped);
            drawn.extend(claimed);
            drawn.extend(slots);
            let count = drawn.len();
            drawn.sort_unstable_by_key(|parameter| parameter.offset());
            drawn.dedup();

            assert_eq!(drawn.len(), count, "{group} draws a parameter twice");
            assert_eq!(
                drawn.len(),
                group.parameters().count(),
                "{group} loses a parameter between its hand layout and its rack"
            );
        }
    }
}

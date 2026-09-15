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

use core::fmt;

use deepmind_midi::param::{Group, Kind, ParamId, TableId};
use deepmind_midi::program::ProgramName;
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Font, Length, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{Space, button, column, container, pick_list, row, text};

use crate::envelope;
use crate::fader::{self, fader};
use crate::name;
use crate::style::materials;
use crate::{Confidence, Element, Patch, tint};

/// Width of one slot, which is the fader plus the room a name needs either side.
pub(crate) const SLOT: f32 = 88.0;

/// Height of the name, so that slots in a row line up whatever their names do.
///
/// Three lines of it. `VCF Envelope Velocity Sensitivity` is the longest name
/// in the instrument once its group is taken off the front, and a box that
/// clips it is a box that lies about which fader is which.
pub(crate) const NAME: f32 = 44.0;

/// Longest named set that is drawn as lit legends rather than as a list.
///
/// Beyond this a list is the honest control: the modulation matrix has 130
/// destinations, and a column of 130 legends is a joke at the reader's expense.
const LEGENDS: usize = 6;

/// What a view in this crate asks for.
///
/// Three things, and the last one never reaches a wire: a parameter should
/// move, the program should be called something, or a section should be the one
/// on the screen. What an edit costs on a wire, when it goes out and what it
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
}

/// Draws one group of parameters.
///
/// `firmware` is what a device inquiry reported, and it is not decoration:
/// firmware 1.1 renumbered three value tables, so 17 of 23 modulation sources
/// mean something else on 1.0. Until a synthesizer has answered, the caller
/// passes the library's default and says so on the screen.
#[must_use]
pub fn group<'a, Renderer>(patch: &Patch, group: Group, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // Seventeen parameters the library calls `Program Name Char 1` to `17`
    // are one field, drawn where the first of them sits, so the rack keeps the
    // instrument's own order and loses seventeen faders nobody could name a
    // sound on.
    let slots = group.parameters().filter_map(|parameter| {
        if name::holds(parameter) {
            return name::begins(parameter).then(|| name::field(patch));
        }
        Some(slot(patch, group, parameter, firmware))
    });
    let rack = row(slots).spacing(0).wrap();
    // An envelope's meaning is a picture, so the picture goes above its rack.
    // The slots are untouched: hand layout changes the arrangement and never
    // what a control is.
    let body: Element<'a, Renderer> = match envelope::shape(patch, group) {
        Some(shape) => column![shape, rack].spacing(10).into(),
        None => rack.into(),
    };
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

/// Draws what the three drawings of a value mean.
#[must_use]
pub fn legend<'a, Renderer>() -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    row![
        dot(Confidence::Confirmed),
        muted("reported"),
        Space::new().width(Length::Fixed(14.0)),
        dot(Confidence::Assumed),
        muted("claimed"),
        Space::new().width(Length::Fixed(14.0)),
        dot(Confidence::Unknown),
        muted("unread"),
    ]
    .spacing(6)
    .align_y(Vertical::Center)
    .into()
}

/// Draws one parameter: its address, its control, its value and its name.
fn slot<'a, Renderer>(
    patch: &Patch,
    group: Group,
    parameter: ParamId,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        // The NRPN number, which is also the parameter's byte offset in a dump,
        // so one number is both its name on the wire and its address in memory.
        text(parameter.offset().to_string())
            .size(10)
            .font(Font::MONOSPACE)
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, Confidence::Unknown)),
            }),
        control(parameter, value, claim, firmware),
        text(reading(parameter, value, firmware))
            .size(13)
            .font(Font::MONOSPACE)
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, claim)),
            }),
        container(text(short(group, parameter)).size(11).center())
            .height(Length::Fixed(NAME))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(5)
    .width(Length::Fixed(SLOT))
    .align_x(Horizontal::Center)
    .into()
}

/// The parameter's name without the group's, which is printed above the rack.
///
/// `VCF Envelope Depth` in the VCF panel is `Envelope Depth`: the group is the
/// panel's own heading, and repeating it in every slot costs the width the rest
/// of the name needs. A name that does not start with its group is left alone.
fn short(group: Group, parameter: ParamId) -> &'static str {
    let name = parameter.name();
    name.strip_prefix(group.name())
        .map_or(name, |rest| rest.trim_start())
}

/// Draws the control a parameter is edited with.
///
/// A parameter of a sound nobody has read still draws its control, because an
/// empty slot in a rack of forty is harder to read than a fader with no cap on
/// it. The control says so by having nothing to take hold of.
fn control<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let low = u8::try_from(parameter.min()).unwrap_or(u8::MIN);
    let high = u8::try_from(parameter.max()).unwrap_or(u8::MAX);
    let Some(value) = value else {
        return fader(low..=high, low, Confidence::Unknown, move |value| {
            Message::Edit { parameter, value }
        })
        .into();
    };
    match parameter.kind() {
        Kind::Switch => lamp(
            parameter,
            value != 0,
            claim,
            u8::from(value == 0),
            if value == 0 { "off" } else { "on" },
        ),
        Kind::Enumerated(table) => match choices(table, parameter, firmware, value) {
            Some(options) if options.len() <= LEGENDS => legends(parameter, &options, value, claim),
            Some(options) => list(parameter, options, value),
            // A table that does not name this value is a table that would drop
            // the value on the next click, so the raw number stays draggable.
            None => sweep(parameter, low..=high, value, claim),
        },
        // A sweep, and anything a later library adds that this build has not
        // heard of: every parameter is a number underneath.
        _ => sweep(parameter, low..=high, value, claim),
    }
}

/// Draws a fader over a parameter's whole range.
fn sweep<'a, Renderer>(
    parameter: ParamId,
    range: core::ops::RangeInclusive<u8>,
    value: u8,
    claim: Confidence,
) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let low = *range.start();
    let high = *range.end();
    fader(range, value.clamp(low, high), claim, move |value| {
        Message::Edit { parameter, value }
    })
    .into()
}

/// Draws a lamp: lit for on, and what it says under it.
///
/// Not a checkbox and not something that slides. An instrument says *on* with a
/// light, and this is the only place a saturated colour appears.
fn lamp<'a, Renderer>(
    parameter: ParamId,
    on: bool,
    claim: Confidence,
    next: u8,
    label: &'a str,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let live = !matches!(claim, Confidence::Unknown);
    let face = button(
        container(text(label).size(11).font(Font::MONOSPACE).center())
            .width(Length::Fixed(fader::WIDTH))
            .align_x(Horizontal::Center),
    )
    .padding(6)
    .style(move |theme: &Theme, _status| lit(theme, on, claim));
    let face = if live {
        face.on_press(Message::Edit {
            parameter,
            value: next,
        })
    } else {
        face
    };
    container(face)
        .height(Length::Fixed(fader::HEIGHT))
        .align_y(Vertical::Center)
        .into()
}

/// Draws a named set as a column of legends, one of them lit.
fn legends<'a, Renderer>(
    parameter: ParamId,
    options: &[Choice],
    value: u8,
    claim: Confidence,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let live = !matches!(claim, Confidence::Unknown);
    let rows = options.iter().map(|choice| {
        let on = choice.byte() == value;
        let byte = choice.byte();
        let face = button(text(choice.name).size(10).font(Font::MONOSPACE))
            .padding([1, 5])
            .width(Length::Fill)
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
    });
    container(
        column(rows)
            .spacing(2)
            .width(Length::Fixed(fader::WIDTH + 28.0)),
    )
    .height(Length::Fixed(fader::HEIGHT))
    .align_y(Vertical::Center)
    .into()
}

/// Draws a named set too long for legends as the list it is.
fn list<'a, Renderer>(parameter: ParamId, options: Vec<Choice>, value: u8) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let selected = options
        .iter()
        .find(|choice| choice.byte() == value)
        .copied();
    container(
        pick_list(options, selected, move |choice: Choice| Message::Edit {
            parameter,
            value: choice.byte(),
        })
        .text_size(11)
        .padding([2, 6])
        .width(Length::Fixed(fader::WIDTH + 28.0)),
    )
    .height(Length::Fixed(fader::HEIGHT))
    .align_y(Vertical::Center)
    .into()
}

/// The style a lamp is drawn in: lit, outlined, or dark.
///
/// The same rule the fader's cap follows. Filled for what the synthesizer
/// reported, an outline for what this window claims, and neither for a value
/// nobody has read, so the fill carries the difference and the colour agrees
/// with it.
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
fn reading(parameter: ParamId, value: Option<u8>, firmware: Version) -> String {
    let Some(value) = value else {
        return "\u{2014}".to_owned();
    };
    match parameter.kind() {
        // The control already carries the name, so this carries the byte.
        Kind::Switch | Kind::Enumerated(_) => value.to_string(),
        _ => parameter
            .label_for(u16::from(value), firmware)
            .map_or_else(|| value.to_string(), str::to_owned),
    }
}

/// Grey text, for what is not a value.
fn muted<Renderer>(what: &str) -> iced_widget::Text<'_, Theme, Renderer>
where
    Renderer: TextRenderer,
{
    text(what).size(14).style(move |theme: &Theme| text::Style {
        color: Some(tint(theme, Confidence::Unknown)),
    })
}

/// One value of a named set, as a list shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Choice {
    value: u16,
    name: &'static str,
}

impl Choice {
    /// Returns the value as the program byte it is stored as.
    fn byte(self) -> u8 {
        u8::try_from(self.value).unwrap_or(u8::MAX)
    }
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// The names a table gives, in the order the table gives them.
///
/// `None` when the table does not name every value the parameter accepts, or
/// does not name the one it holds. Some tables list only the start of a
/// documented run, and a control that silently drops the values it has no name
/// for is a control that moves the sound when somebody opens it.
fn choices(
    table: TableId,
    parameter: ParamId,
    firmware: Version,
    value: u8,
) -> Option<Vec<Choice>> {
    let table = table.table_for(firmware);
    let options: Vec<Choice> = table
        .entries
        .iter()
        .filter(|entry| parameter.accepts(entry.value))
        .map(|entry| Choice {
            value: entry.value,
            name: entry.name,
        })
        .collect();
    let named = u32::try_from(options.len()).ok()?;
    let span = u32::from(parameter.max() - parameter.min()) + 1;
    if named != span {
        return None;
    }
    options
        .iter()
        .find(|choice| choice.value == u16::from(value))?;
    Some(options)
}

//! One section of the instrument, laid out as rows.
//!
//! A row per parameter, in the order the protocol numbers them, which is the
//! order the instrument's own display walks. What a row draws depends on what
//! the library says the parameter is: a sweep gets a slider, a named set gets a
//! list of its names, and two states get a switch. Nothing here decides what a
//! value means; that is the library's table and this is the pixels.

use core::fmt;

use deepmind_midi::param::{Group, Kind, ParamId, TableId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Vertical;
use iced_core::{Background, Length, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{Space, column, container, pick_list, row, slider, text, toggler};

use crate::{Confidence, Element, Patch, tint};

/// Width of the parameter name column.
const NAME: f32 = 230.0;

/// Width of the value column.
const READING: f32 = 70.0;

/// What a view in this crate asks for.
///
/// One thing, because one thing is all a view knows how to want. What it costs
/// on a wire, when it goes out and what it goes out behind is the host crate's
/// business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Message {
    /// A parameter should move to this value.
    Edit {
        /// The parameter to move.
        parameter: ParamId,
        /// Where to move it, as the program byte it is stored as.
        value: u8,
    },
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
    Renderer: TextRenderer + 'a,
{
    let rows = group
        .parameters()
        .map(|parameter| row_for(patch, parameter, firmware));
    column(rows).spacing(6).into()
}

/// Draws what the two colours mean, for a window that uses them.
#[must_use]
pub fn legend<'a, Renderer>() -> Element<'a, Renderer>
where
    Renderer: TextRenderer + 'a,
{
    row![
        dot(Confidence::Confirmed),
        muted(Confidence::Confirmed.name()),
        Space::new().width(Length::Fixed(14.0)),
        dot(Confidence::Assumed),
        muted(Confidence::Assumed.name()),
    ]
    .spacing(6)
    .align_y(Vertical::Center)
    .into()
}

/// Draws one parameter.
fn row_for<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    row![
        text(parameter.name()).size(14).width(Length::Fixed(NAME)),
        control(parameter, value, claim, firmware),
        text(reading(parameter, value, firmware))
            .size(14)
            .width(Length::Fixed(READING))
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, claim)),
            }),
        dot(claim),
    ]
    .spacing(12)
    .align_y(Vertical::Center)
    .into()
}

/// Draws the control a parameter is edited with.
///
/// A parameter of a sound nobody has read has nothing to edit: there is no value
/// to move away from, and the host would refuse the edit anyway, so the row says
/// so instead of offering a control that lies about where the knob is.
fn control<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer + 'a,
{
    let Some(value) = value else {
        // Nothing to move, and nothing to move it from. The reading column says
        // so once per row, which is once more than a control that would lie
        // about where the knob is.
        return Space::new().width(Length::Fill).into();
    };
    match parameter.kind() {
        Kind::Switch => container(
            toggler(value != 0)
                .on_toggle(move |on| Message::Edit {
                    parameter,
                    value: u8::from(on),
                })
                .style(move |theme: &Theme, status| {
                    let mut style = toggler::default(theme, status);
                    // On in the colour of the claim, like a slider's handle. The
                    // theme's own on-colour is the one every other control uses,
                    // which on a panel of greys leaves on and off too alike.
                    if value != 0 {
                        style.background = Background::Color(tint(theme, claim));
                    }
                    style
                }),
        )
        .width(Length::Fill)
        .into(),
        Kind::Enumerated(table) => match choices(table, parameter, firmware, value) {
            Some((options, selected)) => {
                pick_list(options, Some(selected), move |choice: Choice| {
                    Message::Edit {
                        parameter,
                        value: choice.byte(),
                    }
                })
                .text_size(14)
                .width(Length::Fill)
                .into()
            }
            // A table that does not name this value is a table that would drop
            // the value on the next click, so the raw number stays editable.
            None => sweep(parameter, value, claim),
        },
        // A sweep, and anything a later library adds that this build has not
        // heard of: every parameter is a number underneath.
        _ => sweep(parameter, value, claim),
    }
}

/// Draws a slider over a parameter's whole range.
fn sweep<'a, Renderer>(parameter: ParamId, value: u8, claim: Confidence) -> Element<'a, Renderer>
where
    Renderer: TextRenderer + 'a,
{
    let low = u8::try_from(parameter.min()).unwrap_or(u8::MIN);
    let high = u8::try_from(parameter.max()).unwrap_or(u8::MAX);
    slider(low..=high, value.clamp(low, high), move |value| {
        Message::Edit { parameter, value }
    })
    .style(move |theme: &Theme, status| {
        let mut style = slider::default(theme, status);
        // The handle is where the claim shows: a knob standing where the
        // instrument says it stands looks different from one standing where
        // this window put it.
        style.handle.background = Background::Color(tint(theme, claim));
        style
    })
    .width(Length::Fill)
    .into()
}

/// Draws the dot that says what backs a value.
fn dot<'a, Renderer>(claim: Confidence) -> Element<'a, Renderer>
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

/// The names a table gives, and the one the parameter is sitting on.
///
/// `None` when the table does not name every value the parameter accepts, or
/// does not name the one it holds. Some tables list only the start of a
/// documented run, and a list that silently drops the values it has no name for
/// is a list that moves the sound when somebody opens it.
fn choices(
    table: TableId,
    parameter: ParamId,
    firmware: Version,
    value: u8,
) -> Option<(Vec<Choice>, Choice)> {
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
    let selected = options
        .iter()
        .find(|choice| choice.value == u16::from(value))
        .copied()?;
    Some((options, selected))
}

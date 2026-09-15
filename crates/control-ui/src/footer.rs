//! What is under the pointer, along the foot of the window.
//!
//! A front panel is twenty-odd faders under four-letter legends and a rack is
//! forty slots under abbreviated ones, and both of them are readable only
//! because a hand can ask what one of them is. `KYBD` over a fader on the
//! filter plate is `VCF Keyboard Tracking`, it accepts `0` to `255`, and
//! controller 74 drives it — none of which fits over a lane forty-six points
//! wide, and all of which fits along the bottom of a window.
//!
//! # Everything here is the library's answer
//!
//! The footer writes down nothing about a parameter. The name, the section, the
//! range, the name of the value it is sitting on and the controller that drives
//! it are five questions put to `deepmind-midi`, which is the same rule the
//! controls themselves are drawn under: what a parameter *is* is the library's
//! table, and this crate is the pixels.
//!
//! # What it cannot say yet
//!
//! What the manual says a parameter *does* — the paragraph, not the name — is
//! the one thing here that has no getter to ask. The library carries the
//! specification and deliberately not the prose: `param/mod.rs` says it "does
//! not carry the manual's prose", and that value tables carry "names and not
//! descriptions for the same reason", which is that the targets it is built for
//! do not want the kilobytes.
//!
//! That is the right decision for a `no_std` library and it leaves a window
//! with nowhere to look, so
//! [deepmind-midi#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28)
//! asks for it behind a feature: a `ParamId::description` that is `None` when
//! the feature is off, so a host that wants the prose pays for it and a
//! synthesizer on a microcontroller does not. The day it lands this footer
//! gains a sentence and nothing else here changes.

use deepmind_midi::param::{Controller, Kind, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Vertical;
use iced_core::{Background, Border, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{container, row, text};

use crate::style::{materials, printed, reading};
use crate::{Confidence, Element, Patch};

/// Draws what the pointer is over, or what to do with the panel when it is over
/// nothing.
///
/// The empty state is not a blank strip. A footer that vanishes is a footer
/// nobody learns is there, and the row it stands in would jump every time the
/// pointer crossed a gap between two faders.
#[must_use]
pub fn footer<'a, Renderer>(
    pointed: Option<ParamId>,
    patch: &Patch,
    firmware: Version,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let line = match pointed {
        Some(parameter) => described(parameter, patch, firmware),
        None => vec![Part::Quiet(
            "Point at a control to read what it is.".to_owned(),
        )],
    };
    let mut across = row![].spacing(10).align_y(Vertical::Center);
    for part in line {
        across = across.push(part.draw());
    }
    container(across)
        .width(Length::Fill)
        .padding([5, 10])
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
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

/// One piece of the line, and how loudly it is said.
enum Part {
    /// The parameter's own name, which is what somebody pointed at it to read.
    Name(String),
    /// A reading: what it is sitting on, in the display's own face.
    Reading(String),
    /// Everything that is true of the parameter whatever its value is.
    Quiet(String),
}

impl Part {
    /// Draws it in the face and the ink its kind is said in.
    fn draw<'a, Renderer>(self) -> Element<'a, Renderer>
    where
        Renderer: TextRenderer<Font = Font> + 'a,
    {
        match self {
            Self::Name(said) => text(said)
                .size(13)
                .font(printed())
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_high),
                })
                .into(),
            Self::Reading(said) => text(said)
                .size(13)
                .font(reading())
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal),
                })
                .into(),
            Self::Quiet(said) => text(said)
                .size(12)
                .font(printed())
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_low),
                })
                .into(),
        }
    }
}

/// Everything the library will say about `parameter`, in the order it is read.
///
/// The name first, because that is the question; then what it is sitting on,
/// because that is the second one; then the facts that are true of it whatever
/// it is sitting on.
fn described(parameter: ParamId, patch: &Patch, firmware: Version) -> Vec<Part> {
    let mut line = vec![
        Part::Name(parameter.name().to_owned()),
        Part::Quiet(parameter.group().name().to_owned()),
    ];
    line.push(Part::Reading(reading_of(parameter, patch, firmware)));
    line.push(Part::Quiet(range_of(parameter)));
    if let Some(controller) = Controller::for_parameter(parameter) {
        line.push(Part::Quiet(format!("CC {}", controller.cc)));
    }
    line
}

/// What the parameter is sitting on, and what backs that.
///
/// The claim is in words here rather than in a colour. A footer is a sentence,
/// and a sentence that said what backs a value by being a different colour
/// would be saying it only to the readers who see the colour.
fn reading_of(parameter: ParamId, patch: &Patch, firmware: Version) -> String {
    let claim = patch.claim(parameter);
    let Some(value) = patch.value(parameter) else {
        return "unread".to_owned();
    };
    let shown = parameter
        .label_for(u16::from(value), firmware)
        .map_or_else(|| value.to_string(), ToOwned::to_owned);
    match claim {
        Confidence::Confirmed => shown,
        // The distinction the whole editor is built on, said plainly: this is
        // where the window put it and not where the synthesizer says it is.
        Confidence::Assumed => format!("{shown} (claimed)"),
        Confidence::Unknown => format!("{shown} (unread)"),
    }
}

/// What the parameter accepts, in the library's own numbers.
///
/// Raw bytes and never the number the synthesizer's display shows, because the
/// manual prints the two ends of a range and almost never the curve between
/// them, and a footer that invented the curve would be the one place in this
/// window that guessed.
fn range_of(parameter: ParamId) -> String {
    match parameter.kind() {
        Kind::Switch if parameter.max() <= 1 => "off or on".to_owned(),
        Kind::Enumerated(_) => parameter.choices().map_or_else(
            || format!("{}\u{2013}{}", parameter.min(), parameter.max()),
            |options| format!("{} named values", options.len()),
        ),
        _ => format!("{}\u{2013}{}", parameter.min(), parameter.max()),
    }
}

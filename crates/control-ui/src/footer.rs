//! What is under the pointer, along the foot of the window.
//!
//! A front panel is twenty-odd faders under four-letter legends and a rack is
//! forty slots under abbreviated ones, and both of them are readable only
//! because a hand can ask what one of them is. `KYBD` over a fader on the
//! filter plate is `VCF Keyboard Tracking`, it accepts `0` to `255`, and
//! controller 74 drives it. None of that fits over a lane forty-six points
//! wide, and all of it fits along the bottom of a window.
//!
//! # Everything here is the library's answer
//!
//! The footer writes down nothing about a parameter. The name, the section, the
//! range, the name of the value it is sitting on and the controller that drives
//! it are five questions put to `deepmind-midi`, which is the same rule the
//! controls themselves are drawn under: what a parameter *is* is the library's
//! table, and this crate is the pixels.
//!
//! # And what it does
//!
//! The sixth question, and the one somebody points at a control to ask. It had
//! no getter when this file was written: the library carries the specification
//! and deliberately not the prose, to keep 31 kB of sentences out of the binary
//! on the microcontrollers it is also built for.
//!
//! [deepmind-midi#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28)
//! asked for it behind a feature rather than asking for that decision to be
//! reversed, and 26.3 answers with exactly that: `ParamId::description` returns
//! `None` unless the `descriptions` feature is on, so the signature is the same
//! either way and a host writes one code path. This workspace turns the feature
//! on, because a window with a footer along the bottom of it is precisely the
//! host that has somewhere to print the answer.
//!
//! A parameter with no sentence is still drawn, with the five facts it has.
//! The library writes `None` rather than a guess where its own specification
//! does not establish what a parameter does, which is the same refusal every
//! other drawing in this crate is under.
//!
//! # And the picture of it
//!
//! The seventh question, and the first one answered: 26.5 publishes a glyph per
//! parameter and per standard controller, a decay as a tail, a mix as wet
//! against dry, a pedal as a treadle, on the same seven by seven grid the
//! effects' marks and the modulation sources' cells are on. It stands at the
//! head of the line, before the name, because a picture is read before a word
//! is and because somebody who points at the same control twice should stop
//! needing the word.
//!
//! 177 of the 242 carry one. The rest are the effect slots, whose picture
//! depends on which algorithm the engine is running and is `FxSlot::glyph`
//! instead, and the seventeen characters of the program's name, which are
//! letters rather than a control. A parameter with no glyph draws none, the
//! same as one with no sentence.

use deepmind_midi::param::{Controller, Kind, ParamId};
use deepmind_midi::pixels::Pixels;
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::Vertical;
use iced_core::{Background, Border, Font, Length, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{button, container, row, text};

use crate::lcd::{self, Screen};
use crate::mapping::{Mapping, Reach};
use crate::matrix;
use crate::panel::{Message, sits_at};
use crate::style::{self, materials, printed, reading};
use crate::{Confidence, Element, Patch};

/// Draws what the pointer is over, or what to do with the panel when it is over
/// nothing.
///
/// `hinted` is what a press under the pointer says about itself, for the
/// presses that are a mark rather than a word.
///
/// The empty state is not a blank strip. A footer that vanishes is a footer
/// nobody learns is there, and the row it stands in would jump every time the
/// pointer crossed a gap between two faders.
#[must_use]
pub fn footer<'a, Renderer>(
    pointed: Option<ParamId>,
    patch: &Patch,
    firmware: Version,
    mapped: Option<Mapping>,
    hinted: Option<&'static str>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let line = match (pointed, hinted) {
        (Some(parameter), _) => described(parameter, patch, firmware),
        // A press whose whole face is a nine-dot mark says what it does here,
        // because there is nowhere on a nine-dot mark to write it and a word
        // printed beside every one of them is a row of words on the panel
        // whether or not anybody is asking. It is one sentence and no facts:
        // what the library knows about a parameter is what the rest of this
        // line is for, and a press is not a parameter.
        (None, Some(said)) => vec![Part::Name(said.to_owned())],
        (None, None) => vec![Part::Quiet(
            "Point at a control to read what it is.".to_owned(),
        )],
    };
    let mut across = row![].spacing(10).align_y(Vertical::Center);
    // A routing mapped onto the window is a mode, and a mode with nothing on the
    // screen saying it is up is a window that has stopped answering for reasons
    // nobody can see. The footer is where it is said, because the footer is the
    // one thing under every surface and under every sheet, and somebody in this
    // mode is by definition somewhere other than the page they turned it on
    // from.
    if let Some(mapped) = mapped {
        across = across.push(mode(mapped));
        // And what is already wired to whatever the pointer is over, by name.
        // The control itself draws how far each of them can push it; a band on
        // a fader cannot say *which* routing drew it, and which one it is is
        // the thing somebody about to add a second one wants to know.
        if let Some(already) = pointed.and_then(|at| already(at, patch, firmware)) {
            across = across.push(already);
        }
    }
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
    /// The library's picture of what the control does.
    Pictured(&'static Pixels),
    /// The parameter's own name, which is what somebody pointed at it to read.
    Name(String),
    /// A reading: what it is sitting on, in the display's own face.
    Reading(String),
    /// Everything that is true of the parameter whatever its value is.
    Quiet(String),
    /// What the parameter does, in the library's own sentence.
    Said(String),
}

impl Part {
    /// Draws it in the face and the ink its kind is said in.
    fn draw<'a, Renderer>(self) -> Element<'a, Renderer>
    where
        Renderer: TextRenderer<Font = Font> + 'a,
    {
        match self {
            // Stencilled on the card rather than lit on glass, because it is a
            // mark beside a word and not a display, which is the call every
            // other mark in this window goes through. In the ink the name is in,
            // so
            // that the picture and the word it stands before read as one thing.
            Self::Pictured(cell) => {
                let mut screen = Screen::new(lcd::CELL, lcd::CELL);
                screen.blit(cell, 0, 0);
                lcd::stencil(screen, |theme: &Theme| materials(theme).metal_high)
            }
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
            // Takes the rest of the line and wraps in it, because a sentence
            // clipped half way through is a sentence that stops being one.
            Self::Said(said) => text(said)
                .size(12)
                .font(printed())
                .width(Length::Fill)
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal),
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
    let mut line = Vec::new();
    // The picture first, where the library has one. A control that is driven by
    // a standard controller is pictured the way the parameter is, so the two
    // agree without this asking twice.
    if let Some(glyph) = parameter.glyph() {
        line.push(Part::Pictured(glyph.pixels()));
    }
    line.push(Part::Name(parameter.name().to_owned()));
    line.push(Part::Quiet(
        crate::section::name(parameter.group()).to_owned(),
    ));
    line.push(Part::Reading(reading_of(parameter, patch, firmware)));
    line.push(Part::Quiet(range_of(parameter)));
    if let Some(controller) = Controller::for_parameter(parameter) {
        line.push(Part::Quiet(format!("CC {}", controller.cc)));
    }
    // And what it does, which is the reason somebody pointed at it. Last,
    // because it is the longest and the other five are what a reader who
    // already knows the parameter came back for.
    if let Some(sentence) = parameter.description() {
        line.push(Part::Said(sentence.to_owned()));
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
        .map_or_else(|| sits_at(parameter, value), ToOwned::to_owned);
    match claim {
        Confidence::Confirmed => shown,
        // The distinction the whole editor is built on, said plainly: this is
        // where the window put it and not where the synthesizer says it is.
        Confidence::Assumed => format!("{shown} (claimed)"),
        Confidence::Unknown => format!("{shown} (unread)"),
    }
}

/// What the parameter accepts.
///
/// The two ends the manual prints, where the library has them, so
/// [`ParamId::display`] is `50.0 Hz to 20000.0 Hz` for the filter's corner, and
/// the raw range where it does not, which is 216 of the 242. Never a number
/// between the two ends: the manual publishes what a range runs from and to and
/// almost never the curve across it, so a footer that turned this byte into a
/// frequency would be the one place in this window that guessed.
fn range_of(parameter: ParamId) -> String {
    if let Some(printed) = parameter.display() {
        return printed.to_owned();
    }
    match parameter.kind() {
        Kind::Switch if parameter.max() <= 1 => "off or on".to_owned(),
        Kind::Enumerated(_) => parameter.choices().map_or_else(
            || ends(parameter),
            |options| format!("{} named values", options.len()),
        ),
        _ => ends(parameter),
    }
}

/// The two ends of the raw range, read the way the control reads them.
///
/// A bipolar parameter's ends are the two extremes either side of its centre,
/// not `0` and `255`: a depth that runs `-128` to `+127` said to run `0` to
/// `255` is the same wrong fact the readings used to state.
fn ends(parameter: ParamId) -> String {
    let low = u8::try_from(parameter.min()).unwrap_or(u8::MIN);
    let high = u8::try_from(parameter.max()).unwrap_or(u8::MAX);
    format!(
        "{}\u{2013}{}",
        sits_at(parameter, low),
        sits_at(parameter, high)
    )
}

/// Says which of the other routings already land on the control under the
/// pointer.
///
/// Only while one is being mapped, because it is only then that somebody is
/// choosing against them. Nothing at all where nothing else lands there, which
/// is most controls and is the answer rather than a gap.
fn already<'a, Renderer>(
    at: ParamId,
    patch: &Patch,
    firmware: Version,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let names: Vec<&'static str> = matrix::reaching(patch, firmware)
        .into_iter()
        .filter(|reach| reach.at() == at)
        .map(Reach::label)
        .collect();
    let said = match names.split_last()? {
        (last, []) => (*last).to_owned(),
        (last, rest) => format!("{} and {last}", rest.join(", ")),
    };
    Some(
        text(format!("{said} already \u{2192} here"))
            .size(11)
            .style(|_theme: &Theme| text::Style {
                color: Some(style::MODULATION),
            })
            .into(),
    )
}

/// Says which routing is mapped onto the window, and how to stop.
///
/// The one saturated colour on the panel, which is the colour every control the
/// routing can reach is outlined in at the same moment. It is a press as well
/// as a statement: a mode somebody can leave only by going back to the page
/// they started it on is a mode somebody gets stuck in.
fn mode<'a, Renderer>(mapped: Mapping) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    button(
        row![
            text(format!("{} \u{2192}", mapped.label())).size(12),
            text("take hold of a lit control, or press to stop").size(11),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding([2, 8])
    .style(|theme: &Theme, _status| button::Style {
        background: Some(Background::Color(style::MODULATION)),
        text_color: materials(theme).panel,
        border: border::rounded(3).width(1.0).color(style::MODULATION),
        ..button::Style::default()
    })
    .on_press(Message::Mapper(None))
    .into()
}

//! The shelf, drawn.
//!
//! The other half of the window: not the sound in front of somebody but the
//! sounds they keep. A pack is 128 programs and the instrument shows them eight
//! at a time on a two-line display, so the one thing this surface can offer that
//! the front panel cannot is all of them at once, in slot order, with the names
//! readable.
//!
//! It lives here rather than in `control-ui` on purpose. Bank and librarian
//! operations are desktop only: a preset pack is thirty-five kilobytes of
//! `SysEx`, a plugin event buffer is sized for a few notes, and a twelve-second
//! transfer through a DAW's MIDI output is twelve seconds of a host wondering
//! why its instrument stopped answering. Editing is the same in both builds
//! because it is the same crate; this is the part that is not.

use control_ui::materials;
use deepmind_midi::ids::{BANK_COUNT, Bank};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, progress_bar, row, scrollable, space, text};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{App, Message};
use crate::shelf::{Held, Shelf};

/// Width of one program on the shelf.
///
/// A name is sixteen characters and a slot is four, so this is the two of them
/// side by side with room to breathe. Narrow enough that a window at its opening
/// width shows four across, which puts a whole bank in eight screenfuls rather
/// than in a column somebody scrolls through twice.
const CARD: f32 = 190.0;

/// Room kept clear down the right for the scroll bar.
///
/// A shelf that ran under its own scroll bar would be a shelf whose last column
/// is a column somebody has to guess at.
const GUTTER: f32 = 16.0;

/// The whole librarian.
pub fn view(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    column![
        actions(app),
        banks(app.bank(), app.is_connected()),
        standing(shelf),
    ]
    .extend(progress(shelf))
    .push(programs(shelf))
    .spacing(12)
    .padding([0, 12])
    .into()
}

/// A button that is not a parameter, in the instrument's own materials.
///
/// The window's own, so that the two surfaces press the same button.
fn chrome(label: &str) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(label).size(13))
        .padding([5, 12])
        .style(control_ui::chrome)
}

/// What can be done to a shelf, and to the sound beside it.
fn actions(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    let reading = shelf.transfer().is_some();
    row![
        chrome("Open\u{2026}").on_press(Message::Open),
        chrome("Save the sound\u{2026}")
            .on_press_maybe(app.patch().is_known().then_some(Message::SavePatch)),
        chrome("Save the shelf\u{2026}")
            .on_press_maybe((!shelf.is_empty()).then_some(Message::SavePack)),
        space().width(Fill),
        chrome("Stop").on_press_maybe(reading.then_some(Message::Cancel)),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// The eight banks, and the one a read would read.
fn banks<'a>(chosen: Bank, open: bool) -> Element<'a, Message> {
    let letters = (0..BANK_COUNT).filter_map(|index| Bank::new(index).ok());
    let tabs = letters.map(|bank| {
        let pressed = bank == chosen;
        button(text(bank.letter()).size(12))
            .padding([4, 10])
            .style(move |theme: &Theme, _status| pressed_like(theme, pressed))
            .on_press(Message::ChooseBank(bank))
            .into()
    });
    row![text("Bank").size(13)]
        .extend(tabs)
        .push(chrome("Read it onto the shelf").on_press_maybe(open.then_some(Message::ReadBank)))
        .spacing(6)
        .align_y(Center)
        .into()
}

/// Where what is on the shelf came from, in words.
///
/// The one thing a librarian must never be vague about. A pack read off a
/// synthesizer and a pack read off a disk look identical on the screen and are
/// not the same claim, and which one somebody is about to write over the other
/// with depends entirely on knowing which is which.
fn standing(shelf: &Shelf) -> Element<'_, Message> {
    let held = shelf.held().len();
    let sound = if held == 1 { "program" } else { "programs" };
    let where_from = match shelf.source() {
        Some(source) => format!("{held} {sound} \u{00b7} {source}"),
        None => "Nothing on the shelf. Open a `.syx` file, or read a bank off the synthesizer."
            .to_owned(),
    };
    let unreadable = (shelf.skipped() > 0).then(|| {
        text(format!(
            "{} frame(s) in that file could not be read, and were skipped rather than losing the \
             rest.",
            shelf.skipped()
        ))
        .size(12)
    });
    column![text(where_from).size(13)]
        .extend(unreadable.map(Element::from))
        .spacing(4)
        .into()
}

/// How far a bank read has got, while one is going.
///
/// Twelve seconds of MIDI is something to watch rather than something to wait
/// through, which is the whole reason the device loop is on a thread of its own.
fn progress(shelf: &Shelf) -> Option<Element<'_, Message>> {
    let transfer = shelf.transfer()?;
    Some(
        column![
            text(format!(
                "Bank {}: {} of {}",
                transfer.bank, transfer.received, transfer.expected
            ))
            .size(12),
            progress_bar(0.0..=1.0, transfer.fraction()).girth(Length::Fixed(6.0)),
        ]
        .spacing(4)
        .into(),
    )
}

/// Everything on the shelf, in the order it is held.
fn programs(shelf: &Shelf) -> Element<'_, Message> {
    let cards = shelf
        .held()
        .iter()
        .enumerate()
        .map(|(index, held)| card(index, held, shelf.loaded() == Some(index)));
    scrollable(container(row(cards).spacing(6).wrap()).padding(Padding::ZERO.right(GUTTER)))
        .height(Fill)
        .into()
}

/// One program: where it lives, and what it is called.
fn card(index: usize, held: &Held, loaded: bool) -> Element<'_, Message> {
    let face = row![
        container(text(held.address()).size(12))
            .width(Length::Fixed(38.0))
            .align_x(Horizontal::Right),
        text(held.name()).size(13),
    ]
    .spacing(8)
    .align_y(Vertical::Center);
    button(face)
        .width(Length::Fixed(CARD))
        .padding([6, 8])
        .style(move |theme: &Theme, _status| pressed_like(theme, loaded))
        .on_press(Message::Load(index))
        .into()
}

/// The style a chosen thing is drawn in, which is the surface switch's.
///
/// Not a colour of its own: the thing that has been pressed is the face plate a
/// rack sits on, and everything else is the panel it is cut into. It is the same
/// trick the instrument plays with a lit section button, and the librarian
/// borrows it so that one window has one idea of what "this one" looks like.
fn pressed_like(theme: &Theme, pressed: bool) -> button::Style {
    let material = materials(theme);
    let palette = theme.extended_palette();
    button::Style {
        background: Some(Background::Color(if pressed {
            material.plate
        } else {
            material.panel
        })),
        text_color: palette.background.base.text,
        border: border::rounded(3).width(1.0).color(if pressed {
            material.lit
        } else {
            material.recess_edge
        }),
        ..button::Style::default()
    }
}

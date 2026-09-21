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

use deepmind_midi::ids::{BANK_COUNT, Bank};
use deepmind_midi::sysex::inquiry::Version;
use iced::widget::{button, column, progress_bar, row, space, text};
use iced::{Center, Element, Fill, Length, Theme};

use crate::app::{App, Browsing, Message};
use crate::shelf::Shelf;

/// The whole librarian: every sound, or what it takes to publish one.
///
/// Two things and not two lists. **Where a sound lives is a column**, not a
/// surface of its own — see [`crate::sounds`] — so the only switch here is
/// between reading and contributing.
pub fn view(app: &App) -> Element<'_, Message> {
    column![crate::sounds::switch(app.browsing())]
        .push(match app.browsing() {
            Browsing::Sounds => sounds(app),
            Browsing::Publish => crate::sharing::view(app),
        })
        .spacing(12)
        .padding([0, 12])
        .into()
}

/// Every sound, and what fills the shelf they are drawn from.
///
/// The two presses that put something *on* the shelf stay here — opening a
/// file, reading a bank — because they are about this machine and this
/// instrument rather than about the list. Everything below them is
/// [`crate::sounds`], which is one table of every sound wherever it is.
fn sounds(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    column![
        actions(app),
        banks(app.bank(), app.is_connected()),
        standing(shelf, app.firmware()),
    ]
    .extend(progress(shelf))
    .push(crate::sounds::view(app))
    .spacing(10)
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
    // The same press the table's own filters are, because that is what the
    // bank row is: a chooser among eight, above a list. One window, one idea of
    // what "this one" looks like.
    let tabs = letters.map(|bank| {
        crate::sounds::filter(
            bank.letter().to_string(),
            None,
            bank == chosen,
            Message::ChooseBank(bank),
        )
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
fn standing(shelf: &Shelf, firmware: Version) -> Element<'_, Message> {
    let held = shelf.held().len();
    let sound = if held == 1 { "program" } else { "programs" };
    // And how many of them the search left, where one is narrowing the shelf.
    // Said as a count of what is held rather than instead of it: the shelf is
    // still holding 128 whatever the screen is showing, and a librarian whose
    // own count changed as somebody typed is a librarian that has lost track of
    // what it has.
    let showing = shelf.showing(firmware).len();
    let where_from = match shelf.source() {
        Some(source) if showing == held => format!("{held} {sound} \u{00b7} {source}"),
        Some(source) => format!("{showing} of {held} {sound} \u{00b7} {source}"),
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

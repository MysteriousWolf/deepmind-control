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
use iced::widget::{column, pick_list, progress_bar, row, space, text};
use iced::{Center, Element, Fill, Length};

use crate::app::{App, Message};
use crate::shelf::Shelf;

/// The whole librarian: every sound, and the sheet that describes one of them.
///
/// **One list.** Where a sound lives is a column — see [`crate::sounds`] — and
/// describing one to share it is a sheet over that list rather than a second
/// list beside it: sharing is done *to one sound*, so it belongs where the
/// sound is. It was a tab, and a tab meant leaving the table to describe a row
/// and having no way, once there, to tell which row was being described.
pub fn view(app: &App) -> Element<'_, Message> {
    let under = column![chrome(app)]
        .push(sounds(app))
        .spacing(12)
        .padding([0, 12]);
    if app.sharing().is_none() {
        return under.into();
    }
    control_ui::modal(under, crate::sharing::sheet(app), Message::CloseSharing)
}

/// The line above the table: what is on the shelf, how to put another bank on
/// it, and the four presses that are about files.
///
/// **Left is what you have, right is what you do to the disk**, which is the
/// arrangement the whole librarian is laid out under: the seven verbs in
/// [`crate::sounds`] sit under this on the same principle, with what is about
/// the chosen sound on the left of their row and what is about the library on
/// the right of it.
///
/// The four are marks and not words for the reason every mark in this window
/// is: `Save the shelf\u{2026}` beside `Save the sound\u{2026}` beside `Open\u{2026}` is a
/// sentence along the top of a list, and what each one does is said in full in
/// the footer while the pointer is on it.
fn chrome(app: &App) -> Element<'_, Message> {
    row![
        standing(app.shelf(), app.firmware()),
        reading(app.bank(), app.is_connected()),
        space().width(Fill),
        files(app),
    ]
    .spacing(14)
    .align_y(Center)
    .into()
}

/// Every sound, and what fills the shelf they are drawn from.
///
/// The bank row stays here — which of the eight a read would read, and the
/// press that reads it — because it is about this instrument rather than about
/// the list. Everything below it is [`crate::sounds`], which is one table of
/// every sound wherever it is.
fn sounds(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    column([])
        .extend(progress(shelf))
        .push(crate::sounds::view(app))
        .spacing(10)
        .into()
}

/// The four presses that are about files rather than about any one sound.
///
/// A `.syx` in and a `.syx` out, the whole shelf out as a pack, and the one
/// that calls off a transfer already going. None of them is a verb from
/// [`crate::app::Action`]: those act on the row somebody chose, and these act
/// on the shelf and the disk, which is why they stand in a different corner.
fn files(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    let reading = shelf.transfer().is_some();
    row![
        crate::sounds::press(
            control_ui::OPEN,
            "Open\u{2026}",
            Some(Message::Open),
            "Open a `.syx` file and put what it holds on the shelf.",
        ),
        crate::sounds::press(
            control_ui::EXPORT,
            "Save one\u{2026}",
            app.patch().is_known().then_some(Message::SavePatch),
            "Write the sound on the screen out as one `.syx` file.",
        ),
        crate::sounds::press(
            control_ui::PACK,
            "Save all\u{2026}",
            (!shelf.is_empty()).then_some(Message::SavePack),
            "Write the whole shelf out as one `.syx` pack.",
        ),
    ]
    // Nothing to stop until something is going. A press that spends every
    // moment but twelve seconds greyed out is a press that teaches somebody
    // the toolbar is mostly dead, and this one has a transfer to belong to.
    .extend(reading.then(|| {
        crate::sounds::press(
            control_ui::SHUT,
            "Stop",
            Some(Message::Cancel),
            "Stop the bank read that is going out now.",
        )
    }))
    .spacing(1)
    .align_y(Center)
    .into()
}

/// Reading a bank off the instrument: one press, and which bank it reads.
///
/// **It was eight chips and it read as a second filter.** They were drawn with
/// [`crate::sounds::filter`], which is the press the table's own `BANK` column
/// heading narrows with, so the surface showed two bank choosers that looked
/// identical and did different things — one asked the synthesizer for a bank,
/// the other hid rows. A person cannot be expected to tell those apart by
/// where they sit.
///
/// So the eight are a picker on the press that uses them, and the press says
/// what it does. There is exactly one bank filter on this surface now, and it
/// is in the column called `BANK`.
fn reading<'a>(chosen: Bank, open: bool) -> Element<'a, Message> {
    let banks: Vec<Bank> = (0..BANK_COUNT)
        .filter_map(|index| Bank::new(index).ok())
        .collect();
    row![
        crate::sounds::press(
            control_ui::READ,
            "Read bank",
            open.then_some(Message::ReadBank),
            "Read that bank off the synthesizer onto this machine's shelf.",
        ),
        pick_list(banks, Some(chosen), Message::ChooseBank)
            .text_size(12)
            .padding([3, 7])
            .width(Length::Fixed(BANK_PICKER))
            .style(control_ui::selector)
            .menu_style(control_ui::shortlist),
    ]
    .spacing(4)
    .align_y(Center)
    .into()
}

/// How wide the bank picker stands: one letter and the mark that opens it.
const BANK_PICKER: f32 = 56.0;

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

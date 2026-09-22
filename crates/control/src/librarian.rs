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
use iced::widget::{column, progress_bar, row, space, text};
use iced::{Center, Element, Fill, Length, Theme};

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
        .push(
            column([])
                .extend(progress(app.shelf()))
                .push(crate::sounds::view(app))
                .spacing(10),
        )
        .spacing(12)
        .padding([0, 12]);
    if app.sharing().is_none() {
        return under.into();
    }
    control_ui::modal(under, crate::sharing::sheet(app), Message::CloseSharing)
}

/// The one line above the table: three groups of presses, spread across it.
///
/// **Grouped by what they act on, and the group says so.** They were two rows
/// pushed to the far edges with a gap between — the width of two columns of
/// nothing across the middle of the window — and the reason was that nothing
/// said what belonged with what, so the only arrangement available was *left*
/// and *right*. Naming the three things this window deals with, and putting
/// each group's presses under its name, both fills the row and answers the
/// question the gap was hiding: which of these does what to which.
///
/// The three are the whole of it. A sound is the row somebody chose; the shelf
/// is what this machine is holding; the library is what other people have
/// published. Every press here moves something between two of them.
fn chrome(app: &App) -> Element<'_, Message> {
    row![
        group("THIS SOUND", crate::sounds::toolbar(app)),
        space().width(Fill),
        group("SHELF", shelf_presses(app)),
        space().width(Fill),
        group("LIBRARY", crate::sounds::library(app.catalogue())),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

/// A named group of presses.
///
/// The name is printed in the ink a legend is, small and upright, so it reads
/// as the label on a panel rather than as another press. It stands to the left
/// of what it names rather than above it, because above would make this line
/// two lines tall and the line is what the gap was spent on.
fn group<'a>(name: &'a str, presses: Element<'a, Message>) -> Element<'a, Message> {
    row![
        text(name).size(9).style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }),
        presses,
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

/// What fills this machine's shelf, and what empties it.
///
/// Three, and each one names the other end: a shelf is filled from the
/// synthesizer or from a file and emptied into a file, so `Read synth`,
/// `Open file\u{2026}` and `Save shelf\u{2026}` say the whole of what they do
/// without the group's name having to be read twice.
fn shelf_presses(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    let reading = shelf.transfer().is_some();
    row![
        crate::sounds::press(
            "Read synth",
            app.is_connected().then_some(Message::ReadAll),
            "Read every bank off the synthesizer onto this machine's shelf. About a \
             minute and a half.",
        ),
        crate::sounds::press(
            "Open file\u{2026}",
            Some(Message::Open),
            "Open a `.syx` file and put what it holds on the shelf.",
        ),
        crate::sounds::press(
            "Save shelf\u{2026}",
            (!shelf.is_empty()).then_some(Message::SavePack),
            "Write the whole shelf out as one `.syx` pack.",
        ),
    ]
    // Nothing to stop until something is going. A press that spends every
    // moment but ninety seconds greyed out is a press that teaches somebody
    // this line is mostly dead, and this one has a transfer to belong to.
    .extend(reading.then(|| {
        crate::sounds::press(
            "Stop",
            Some(Message::Cancel),
            "Stop the read that is going out now.",
        )
    }))
    .spacing(2)
    .align_y(Center)
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

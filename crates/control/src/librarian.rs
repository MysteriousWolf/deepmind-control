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
use deepmind_midi::sysex::inquiry::Version;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, column, container, progress_bar, row, scrollable, space, text, text_input,
};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{App, Browsing, Message};
use crate::shelf::{Held, ORDERS, Shelf};

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

/// The whole librarian: this machine's shelf, or the shared one.
///
/// Two shelves and one surface. They answer the same question — *what sounds
/// can I have* — and the second one's answer becomes the first one's the moment
/// anything on it is pressed, because a shared patch put on the shelf is a
/// shared patch this machine holds. See [`crate::shared`].
pub fn view(app: &App) -> Element<'_, Message> {
    column![crate::shared::switch(app.browsing())]
        .push(match app.browsing() {
            Browsing::Shelf => shelf(app),
            Browsing::Shared => crate::shared::view(app),
            Browsing::Publish => crate::sharing::view(app),
        })
        .spacing(12)
        .padding([0, 12])
        .into()
}

/// This machine's own shelf.
fn shelf(app: &App) -> Element<'_, Message> {
    let shelf = app.shelf();
    column![
        actions(app),
        banks(app.bank(), app.is_connected()),
        finding(shelf),
        standing(shelf, app.firmware()),
    ]
    .extend(progress(shelf))
    .push(programs(shelf, app.firmware(), app.catalogue().held()))
    .spacing(12)
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

/// How to find one sound among 128, and which way round they stand.
///
/// A bank is 128 programs and the instrument shows eight of them at a time, so
/// the one thing this surface has that the front panel does not is all of them
/// at once. That stops being an advantage at the moment somebody is looking for
/// one: a wall of 128 names is a wall, and the name is what they remember.
///
/// So: a field to type into, which narrows the wall to whatever the words are
/// anywhere in — a name, a slot, or the category a program calls itself — and
/// three ways round to draw what is left. Neither of them moves anything on the
/// shelf. What a save writes is the pack that was opened, in its own order,
/// whatever the screen is sorted by or searched for, because a librarian that
/// wrote out the screen would turn a search into a deletion.
///
/// The field is drawn whether or not there is anything on the shelf, and does
/// nothing while there is not: a control that appeared when a file was opened
/// is a control nobody knows is there.
fn finding(shelf: &Shelf) -> Element<'_, Message> {
    let orders = ORDERS.map(|order| {
        let chosen = order == shelf.order();
        button(text(order.label()).size(12))
            .padding([4, 10])
            .style(move |theme: &Theme, _status| pressed_like(theme, chosen))
            .on_press(Message::SortBy(order))
            .into()
    });
    row![
        text("Find").size(13),
        text_input("a name, a slot, or a category", shelf.query())
            .on_input(Message::Search)
            .size(13)
            .padding([5, 10])
            .style(control_ui::field)
            .width(Length::Fixed(FIELD)),
        space().width(Fill),
        text("Order").size(13),
    ]
    .extend(orders)
    .spacing(6)
    .align_y(Center)
    .into()
}

/// How wide the field somebody types a name into stands.
///
/// A program name is sixteen characters and what gets typed into this is three
/// or four of them, so it is cut for the phrase rather than for the name.
const FIELD: f32 = 260.0;

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

/// Everything a search left, in whichever order the shelf is being read in.
///
/// A search that matched nothing says so. A blank surface where the programs
/// were is a surface that reads as an empty shelf, which is a different and
/// much worse thing to believe about a librarian holding a pack somebody has
/// not saved yet.
fn programs<'a>(
    shelf: &'a Shelf,
    firmware: Version,
    known: Option<&'a crate::catalogue::Held>,
) -> Element<'a, Message> {
    let showing = shelf.showing(firmware);
    if showing.is_empty() && !shelf.is_empty() {
        return container(
            text(format!(
                "Nothing on this shelf is called {}, sits at it, or is one.",
                shelf.query()
            ))
            .size(13),
        )
        .padding([12, 0])
        .into();
    }
    let cards = showing
        .into_iter()
        .map(|(index, held)| card(index, held, shelf.loaded() == Some(index), firmware, known));
    scrollable(container(row(cards).spacing(6).wrap()).padding(Padding::ZERO.right(GUTTER)))
        .height(Fill)
        .into()
}

/// One program: where it lives, what it is called, and what it calls itself.
///
/// The category under the name rather than beside it, in the ink a legend is
/// printed in. Beside it would be a second column that every short name has a
/// gap in the middle of, and the category is the thing on this card somebody
/// reads second: it is what the shelf can be sorted and searched by, so it has
/// to be on the card the sort moved, and it is not what the sound is called.
fn card<'a>(
    index: usize,
    held: &'a Held,
    loaded: bool,
    firmware: Version,
    known: Option<&'a crate::catalogue::Held>,
) -> Element<'a, Message> {
    let said = column![text(held.name()).size(13)]
        .extend(held.category(firmware).map(|category| {
            text(category)
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_low),
                })
                .into()
        }))
        .extend(published(held, known))
        .spacing(1);
    let face = row![
        container(text(held.address()).size(12))
            .width(Length::Fixed(38.0))
            .align_x(Horizontal::Right),
        said,
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

/// What the shared library knows about a program on this shelf.
///
/// **The fingerprint is identity of a sound**, which is the interesting part:
/// `deepmind_patches::Fingerprint` is taken over the bytes that decide what is
/// heard, with the name, the slot and the file left out. So a program somebody
/// renamed after loading still matches the patch it came from, and one they
/// edited does not — which is the honest answer, because it is not that sound
/// any more.
///
/// That makes a bank read off an instrument legible in a way it was not: this
/// row is `Acid Growl` by `nyx`, at the version it was published as, and this
/// one is something the library has never seen. And where the version it
/// matched is not the newest one, it says so, which is the whole of what an
/// update offer is — no digest to compare and nothing to poll, because the
/// index already carries every version each patch has ever had.
///
/// Nothing at all where the catalogue is not open, which is the common case.
fn published<'a>(
    held: &Held,
    known: Option<&crate::catalogue::Held>,
) -> Option<Element<'a, Message>> {
    let found = known?.matching(&held.program)?;
    let said = if found.latest {
        format!("{} · {}", found.patch.author, found.patch.name)
    } else {
        format!(
            "{} · {} v{} · v{} published",
            found.patch.author, found.patch.name, found.version, found.patch.version
        )
    };
    let stale = !found.latest;
    Some(
        text(said)
            .size(10)
            .style(move |theme: &Theme| text::Style {
                // An out-of-date sound is worth noticing and is not a fault, so
                // it is lit in the colour this window already spends on *there
                // is something here* rather than in one it would have to invent.
                color: Some(if stale {
                    control_ui::WAY_IN
                } else {
                    materials(theme).metal_low
                }),
            })
            .into(),
    )
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

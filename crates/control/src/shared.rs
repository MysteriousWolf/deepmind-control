//! The other shelf: the sounds other people have published, drawn.
//!
//! The librarian holds what is on this machine. This is the same surface asking
//! the same question of
//! [`deepmind-patches`](https://github.com/MysteriousWolf/deepmind-patches) —
//! one `.syx` and one `.toml` per sound, published as a release on every merge —
//! and pressing anything on it puts that sound on the shelf every other route
//! already fills. There is no third surface and no second window: a catalogue
//! is a shelf somebody else filled.
//!
//! # It draws the library's own pictures
//!
//! Every patch carries a 7x7 one-bit icon, and so does every category and every
//! vocabulary term. They arrive in `index.toml` as
//! [`deepmind_patches::Icon`], which converts to the same
//! [`Pixels`](deepmind_midi::pixels::Pixels) this window blits a glyph and a
//! modulation cell from — so a patch's picture is drawn on the instrument's own
//! dots at the instrument's own pitch, with nothing invented here.
//!
//! An author who draws one gets it drawn. An author who does not gets their
//! category's, which the repository draws once for all twelve.
//!
//! # Two routes in, and neither of them blocks
//!
//! **A folder**, which is `git clone` and a file dialog, and needs no network at
//! all. **A release**, which is one download of the index and one of the
//! bundle, on a thread, folded in on the same tick everything else is. See
//! [`crate::catalogue`].

use control_ui::{Screen, materials, stencil};
use deepmind_patches::Icon;
use deepmind_patches::index::IndexPatch;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{App, Browsing, Message};
use crate::catalogue::{Catalogue, Held, State};

/// Room kept clear down the right for the scroll bar, as the shelf keeps it.
const GUTTER: f32 = 16.0;

/// How many dots a patch's icon is drawn on, across and down.
///
/// Seven, which is what the library draws them at. Not scaled up: a one-bit
/// picture stretched to look important is a picture with fatter dots and no
/// more in it, and this window already draws every other small mark at the
/// pitch it was drawn for.
const ICON: i32 = 7;

/// The shared shelf.
pub fn view(app: &App) -> Element<'_, Message> {
    let catalogue = app.catalogue();
    column![actions(catalogue), standing(app)]
        .extend(catalogue.held().map(|held| finding(app, held)))
        .push(patches(app))
        .spacing(12)
        .into()
}

/// A button that is not a parameter, in the instrument's own materials.
fn chrome(label: &str) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(label).size(13))
        .padding([5, 12])
        .style(control_ui::chrome)
}

/// What can be done to a catalogue.
fn actions(catalogue: &Catalogue) -> Element<'_, Message> {
    let working = catalogue.working();
    let loadable = catalogue.held().is_some_and(Held::loadable);
    row![
        chrome("Fetch the newest").on_press_maybe((!working).then_some(Message::FetchPatches)),
        chrome("Open a checkout\u{2026}")
            .on_press_maybe((!working).then_some(Message::OpenPatches)),
        space().width(Fill),
        chrome("Put these on the shelf").on_press_maybe(loadable.then_some(Message::ShelveShowing)),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// What the catalogue is, in one line.
///
/// The same job the shelf's own standing line does: how many there are, where
/// they came from, and how much of that a search is showing. A catalogue that
/// has its index but not its sounds says so, because browsing works and loading
/// does not yet, and a press that did nothing without saying why would read as
/// a broken button.
fn standing(app: &App) -> Element<'_, Message> {
    let catalogue = app.catalogue();
    let said = match catalogue.state() {
        State::Empty => {
            "Nothing yet. Fetch the newest patches, or open a checkout of them.".to_owned()
        }
        State::Working(doing) => (*doing).to_owned(),
        State::Failed(why) => format!("That did not work: {why}"),
        State::Ready => match catalogue.held() {
            None => "Nothing yet.".to_owned(),
            Some(held) => {
                let all = held.patches().len();
                let showing = held.showing(app.looking()).len();
                let count = if app.looking().asking() {
                    format!("{showing} of {all} patches")
                } else {
                    format!("{all} patches")
                };
                let version = held.index().catalogue.version;
                if held.loadable() {
                    format!("{count} \u{b7} library {version}")
                } else {
                    format!("{count} \u{b7} library {version} \u{b7} the sounds are still coming")
                }
            }
        },
    };
    text(said)
        .size(13)
        .style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        })
        .into()
}

/// How to find one sound among a library of them.
///
/// One field asking one question of everything the catalogue knows about a
/// sound without opening it — its name, who made it, what it says it is, its
/// category, and every vocabulary term and tag it carries — which is the rule
/// the shelf's own search is already under.
///
/// And the categories, which are the twelve the instrument itself has, drawn
/// only where the catalogue has something filed under them: a filter that
/// returns nothing is a filter nobody can use.
fn finding<'a>(app: &'a App, held: &'a Held) -> Element<'a, Message> {
    let chosen = app.looking().category;
    let all = button(text("All").size(12))
        .padding([4, 10])
        .style(move |theme: &Theme, _status| pressed_like(theme, chosen.is_none()))
        .on_press(Message::PatchCategory(None));
    let tabs = held.categories().into_iter().map(move |category| {
        let pressed = chosen == Some(category);
        button(text(category.label()).size(12))
            .padding([4, 10])
            .style(move |theme: &Theme, _status| pressed_like(theme, pressed))
            .on_press(Message::PatchCategory((!pressed).then_some(category)))
            .into()
    });
    column![
        row![text("Find").size(13), field(app)]
            .spacing(8)
            .align_y(Center),
        row![all].extend(tabs).spacing(6).align_y(Center).wrap(),
    ]
    .spacing(8)
    .into()
}

/// The search field itself.
fn field(app: &App) -> Element<'_, Message> {
    text_input("a name, a maker, a sound", &app.looking().find)
        .on_input(Message::FindPatch)
        .size(13)
        .padding([5, 8])
        .width(Length::Fixed(260.0))
        .into()
}

/// Everything a search left, as a table.
///
/// A grid of cards was the wrong shape for this. A card is right for the
/// librarian's own shelf, where every entry is a slot and a name and the
/// question is *which of these 128*; a shared patch carries a maker, a
/// category, four vocabularies and a sentence, and a grid of those is a grid of
/// paragraphs. A table puts one fact per column, so the eye runs down the
/// column it cares about — every author, or every mood — rather than reading
/// each card to find the one field it wanted.
///
/// A search that matched nothing says so, for the reason the shelf's does: a
/// blank surface where the sounds were reads as an empty catalogue, which is a
/// different and much worse thing to believe.
fn patches(app: &App) -> Element<'_, Message> {
    let Some(held) = app.catalogue().held() else {
        return space().height(Fill).into();
    };
    let showing = held.showing(app.looking());
    if showing.is_empty() {
        return container(
            text("Nothing in the library is called that, made by them, or one of those.").size(13),
        )
        .padding([12, 0])
        .into();
    }
    let loadable = held.loadable();
    let trying = app.trying();
    let rows = showing
        .into_iter()
        .enumerate()
        .map(|(at, patch)| line(held, patch, at, loadable, trying == Some(patch.id.as_str())));
    column![heading()]
        .push(
            scrollable(container(column(rows).spacing(1)).padding(Padding::ZERO.right(GUTTER)))
                .height(Fill),
        )
        .spacing(4)
        .into()
}

/// What each column holds, printed once above them.
///
/// In the ink a legend is printed in, which is what every other label on this
/// panel is set in: a header that competed with the rows would be a header
/// somebody reads twice.
fn heading<'a>() -> Element<'a, Message> {
    let label = |said: &'static str, width: f32| {
        container(text(said).size(10).style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }))
        .width(Length::Fixed(width))
    };
    container(
        row![
            container(space()).width(Length::Fixed(ICON_COLUMN)),
            label("SOUND", NAME),
            label("MAKER", MAKER),
            label("CATEGORY", CATEGORY),
            label("IS", TERMS),
            text("ABOUT").size(10).style(|theme: &Theme| text::Style {
                color: Some(materials(theme).metal_low),
            }),
        ]
        .spacing(COLUMN_GAP)
        .align_y(Vertical::Center),
    )
    .padding(Padding::from([0.0, 8.0]).right(GUTTER + 8.0))
    .into()
}

/// One patch, as a row of the table.
///
/// Banded, faintly: a row of six columns read across is a row somebody loses
/// their place in, and a plate that alternates is what a printed table has
/// always done about it.
fn line<'a>(
    held: &'a Held,
    patch: &'a IndexPatch,
    at: usize,
    loadable: bool,
    trying: bool,
) -> Element<'a, Message> {
    let banded = at % 2 == 1;
    let terms = row(chips(held, patch)
        .into_iter()
        .map(|(said, colour)| chip(said, colour)))
    .spacing(4);
    let face = row![
        container(drawn(
            held.icon(patch),
            held.category_colour(patch.category)
        ))
        .width(Length::Fixed(ICON_COLUMN))
        .align_x(Horizontal::Center),
        container(text(patch.name.as_str()).size(13)).width(Length::Fixed(NAME)),
        container(
            text(patch.author.as_str())
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_low),
                })
        )
        .width(Length::Fixed(MAKER)),
        container(chip(
            patch.category.label().to_owned(),
            held.category_colour(patch.category),
        ))
        .width(Length::Fixed(CATEGORY)),
        container(terms).width(Length::Fixed(TERMS)),
        // One line, cut where the column ends. A table whose rows are three
        // lines tall because one of six columns wraps is a list of paragraphs
        // with a table drawn round it, and the thing somebody is running their
        // eye down is the column to the left of it.
        //
        // Clipped as well as unwrapped: a line that does not wrap still draws
        // its whole length, which is a sentence running out under the press at
        // the end of the row.
        container(
            text(patch.about.as_str())
                .size(12)
                .wrapping(iced::widget::text::Wrapping::None)
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_low),
                }),
        )
        .width(Fill)
        .clip(true),
    ]
    .spacing(COLUMN_GAP)
    .align_y(Vertical::Center);
    // The row plays it and the press at the end keeps it. That way round
    // because hearing a sound is what somebody came here to do and putting it
    // on a shelf is what they do about the one they liked, and because playing
    // it costs nothing: the edit buffer is the sound in front of them, not one
    // of the instrument's 1024.
    let played = button(face)
        .width(Length::Fill)
        .padding([5, 8])
        .style(move |theme: &Theme, status| banded_like(theme, banded, trying, status))
        .on_press_maybe(loadable.then(|| Message::AuditionPatch(patch.id.clone())));
    row![played, keep(patch, loadable)]
        .spacing(2)
        .align_y(Vertical::Center)
        .into()
}

/// The press at the end of a row that puts that one sound on the shelf.
///
/// A mark rather than a word, like every other press in this window that is not
/// a way in: the band already spends its words on the columns, and `Keep` in a
/// column of twelve `Keep`s is a column of one word repeated.
fn keep(patch: &IndexPatch, loadable: bool) -> Element<'_, Message> {
    button(
        Element::from(stencil(control_ui::DOWN.screen(), |theme: &Theme| {
            materials(theme).metal_low
        }))
        .map(Message::Ui),
    )
    .padding([5, 8])
    .style(control_ui::chrome)
    .on_press_maybe(loadable.then(|| Message::ShelvePatch(patch.id.clone())))
    .into()
}

/// The terms a row prints, with the colour of the axis each came from.
///
/// All four vocabularies, in the order somebody reads them — what it is like
/// first, then what it does and what it is for — and capped, because a patch
/// that carries nine terms is a patch whose row would be nothing but chips.
/// What is dropped is still searched: the field reads every term a patch has,
/// whether or not the column had room to print it.
///
/// Every axis rather than a chosen two, because a patch is free to fill any
/// of them and a column that could be empty for a patch that said plenty about
/// itself is a column that looks broken.
fn chips(held: &Held, patch: &IndexPatch) -> Vec<(String, Option<[u8; 3]>)> {
    let mut said = Vec::new();
    for (axis, terms) in [
        (deepmind_patches::Axis::Mood, &patch.mood),
        (deepmind_patches::Axis::Timbre, &patch.timbre),
        (deepmind_patches::Axis::Role, &patch.role),
        (deepmind_patches::Axis::Genre, &patch.genre),
    ] {
        let colour = held.axis_colour(axis);
        said.extend(terms.iter().map(|term| (term.clone(), colour)));
    }
    said.truncate(CHIPS);
    said
}

/// How many vocabulary terms one row prints.
const CHIPS: usize = 4;

/// One word on a tinted ground.
///
/// The colour is the library's and the tint is this window's. A chip filled
/// with `#E4572E` at full strength on a dark panel is a chip that is the
/// loudest thing on the surface; carried most of the way back to the panel it
/// is a ground that says *these two belong together* without shouting, and the
/// word on it stays in the metal a legend is printed in.
fn chip<'a>(said: String, colour: Option<[u8; 3]>) -> Element<'a, Message> {
    container(text(said).size(11).style(move |theme: &Theme| text::Style {
        color: Some(match colour {
            Some(rgb) => control_ui::legible(of(rgb), ground(theme, colour), theme),
            None => materials(theme).metal_low,
        }),
    }))
    .padding([1, 6])
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(ground(theme, colour))),
        border: border::rounded(3),
        ..container::Style::default()
    })
    .into()
}

/// The ground a chip's word is printed on.
fn ground(theme: &Theme, colour: Option<[u8; 3]>) -> iced::Color {
    let material = materials(theme);
    match colour {
        Some(rgb) => control_ui::mix(material.recess, of(rgb), CHIP),
        None => material.recess,
    }
}

/// How far a chip's ground is carried from the recess towards the library's
/// own colour.
const CHIP: f32 = 0.28;

/// A colour the library named, as one this window can draw.
fn of(rgb: [u8; 3]) -> iced::Color {
    let [red, green, blue] = rgb;
    iced::Color::from_rgb8(red, green, blue)
}

/// The room the icon column takes.
const ICON_COLUMN: f32 = 22.0;
/// The room a sound's name takes.
const NAME: f32 = 170.0;
/// The room a maker's name takes.
const MAKER: f32 = 110.0;
/// The room a category chip takes.
const CATEGORY: f32 = 92.0;
/// The room the vocabulary chips take.
const TERMS: f32 = 210.0;
/// The gap between two columns.
const COLUMN_GAP: f32 = 10.0;

/// A patch's icon, on the instrument's own dots, in its category's colour.
fn drawn<'a>(icon: Icon, colour: Option<[u8; 3]>) -> Element<'a, Message> {
    let mut screen = Screen::new(ICON, ICON);
    screen.blit(&icon.pixels(), 0, 0);
    // Through the view layer's own message and back, the way every other mark
    // this window borrows from `control_ui` is drawn: the stencil carries no
    // press, so the mapping is a formality that keeps one drawing in one crate.
    Element::from(stencil(screen, move |theme: &Theme| match colour {
        // Lifted off the library's own colour rather than set to it: a seven-dot
        // picture in `#5DB56A` on a dark panel is a smudge, and what a category
        // colour is for is telling two of them apart at a glance.
        Some(rgb) => control_ui::mix(materials(theme).metal, of(rgb), MARK),
        None => materials(theme).metal,
    }))
    .map(Message::Ui)
}

/// How far an icon is carried from the metal towards its category's colour.
const MARK: f32 = 0.55;

/// The ground one row of the table stands on.
fn banded_like(theme: &Theme, banded: bool, trying: bool, status: button::Status) -> button::Style {
    let material = materials(theme);
    let palette = theme.extended_palette();
    let ground = match status {
        button::Status::Hovered | button::Status::Pressed => material.plate,
        _ if trying => material.plate,
        _ if banded => control_ui::mix(material.panel, material.plate, BAND),
        _ => material.panel,
    };
    button::Style {
        background: Some(Background::Color(ground)),
        text_color: palette.background.base.text,
        // The one being tried carries the lit edge every chosen thing in this
        // window carries, because that is what it is: the sound this instrument
        // is making, out of a list of sounds it could be making.
        border: border::rounded(2)
            .width(if trying { 1.0 } else { 0.0 })
            .color(if trying { material.lit } else { material.panel }),
        ..button::Style::default()
    }
}

/// How far a banded row is carried from the panel towards a plate.
const BAND: f32 = 0.45;

/// The two shelves, and which one is showing.
///
/// A switch rather than a tab bar: there are two of them, they answer the same
/// question, and a bar of two is a bar that looks like it is missing the rest.
pub fn switch<'a>(browsing: Browsing) -> Element<'a, Message> {
    let one = |label: &'static str, which: Browsing| {
        let pressed = browsing == which;
        button(text(label).size(12))
            .padding([4, 12])
            .style(move |theme: &Theme, _status| pressed_like(theme, pressed))
            .on_press(Message::Browse(which))
    };
    row![
        one("This machine", Browsing::Shelf),
        one("Shared", Browsing::Shared),
        one("Share one", Browsing::Publish),
    ]
    .spacing(6)
    .align_y(Center)
    .into()
}

/// The style a chosen thing is drawn in, which is the surface switch's.
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

#[cfg(test)]
mod tests {
    use deepmind_patches::Category;

    #[test]
    fn the_category_column_is_wide_enough_for_every_category() {
        // The line under a patch's name is `author · Category`, and the twelve
        // category labels are the part this window cannot shorten. A card that
        // clipped one would be a card that lies about what a sound is.
        let longest = Category::ALL
            .into_iter()
            .map(|category| category.label().len())
            .max()
            .unwrap_or_default();
        // Eleven-point type at roughly six points a character, plus the chip's
        // own padding either side. A column that clipped a chip would be a
        // column that lies about what a sound is, and the twelve labels are the
        // part this window cannot shorten.
        let needed = u16::try_from(longest).unwrap_or(u16::MAX);
        assert!(
            super::CATEGORY > f32::from(needed).mul_add(6.0, 12.0),
            "a {longest}-character category does not fit the category column"
        );
    }
}

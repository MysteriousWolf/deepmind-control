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

/// Width of one patch on the shared shelf.
///
/// Wider than a program on the librarian's own shelf, because a card here
/// carries a picture, a name, who made it and what it is, where one there
/// carries a slot and a name. Still narrow enough that a window at its opening
/// width shows three across.
const CARD: f32 = 232.0;

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

/// Everything a search left.
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
    let cards = showing
        .into_iter()
        .map(|patch| card(patch, held.icon(patch), loadable));
    scrollable(container(row(cards).spacing(6).wrap()).padding(Padding::ZERO.right(GUTTER)))
        .height(Fill)
        .into()
}

/// One patch: its picture, what it is called, who made it and what it is.
///
/// The author under the name rather than beside it, in the ink a legend is
/// printed in, which is where the shelf's own cards put a category. Somebody
/// reads the name first and the maker second, and a second column would put a
/// gap in the middle of every short name.
fn card(patch: &IndexPatch, icon: Icon, loadable: bool) -> Element<'_, Message> {
    let said = column![
        text(patch.name.as_str()).size(13),
        text(format!(
            "{} \u{b7} {}",
            patch.author,
            patch.category.label()
        ))
        .size(11)
        .style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }),
    ]
    .spacing(1);
    let face = row![
        container(drawn(icon))
            .width(Length::Fixed(20.0))
            .align_x(Horizontal::Center),
        said,
    ]
    .spacing(8)
    .align_y(Vertical::Center);
    button(face)
        .width(Length::Fixed(CARD))
        .padding([6, 8])
        .style(move |theme: &Theme, _status| pressed_like(theme, false))
        .on_press_maybe(loadable.then(|| Message::ShelvePatch(patch.id.clone())))
        .into()
}

/// A patch's icon, on the instrument's own dots.
fn drawn<'a>(icon: Icon) -> Element<'a, Message> {
    let mut screen = Screen::new(ICON, ICON);
    screen.blit(&icon.pixels(), 0, 0);
    // Through the view layer's own message and back, the way every other mark
    // this window borrows from `control_ui` is drawn: the stencil carries no
    // press, so the mapping is a formality that keeps one drawing in one crate.
    Element::from(stencil(screen, |theme: &Theme| materials(theme).metal)).map(Message::Ui)
}

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

    use super::CARD;

    #[test]
    fn a_card_is_wide_enough_for_the_longest_thing_printed_under_a_name() {
        // The line under a patch's name is `author · Category`, and the twelve
        // category labels are the part this window cannot shorten. A card that
        // clipped one would be a card that lies about what a sound is.
        let longest = Category::ALL
            .into_iter()
            .map(|category| category.label().len())
            .max()
            .unwrap_or_default();
        // Eleven-point type, the icon and the padding, at roughly six points a
        // character: the check is that the card has room for a label plus a
        // maker's name beside it, not that it is any exact width.
        let room = (CARD - 20.0 - 16.0) / 6.0;
        let needed = u16::try_from(longest).unwrap_or(u16::MAX);
        assert!(
            room > f32::from(needed) + 8.0,
            "a {longest}-character category leaves no room for a maker"
        );
    }
}

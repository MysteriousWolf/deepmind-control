//! Every sound this window can see, in one table.
//!
//! Three places a sound can be — on the synthesizer, on this machine, in the
//! [shared library](https://github.com/MysteriousWolf/deepmind-patches) — and
//! **where it is, is a column.**
//!
//! It used to be a tab per place, and that was wrong for a reason worth writing
//! down: they were three lists of the same kind of thing. Somebody looking for
//! a pad had to look three times, and nothing could tell them that the pad on
//! their shelf and the pad in the library were *the same sound*. One list with
//! a column says both at once, and the search and the filters run over all of
//! it rather than over whichever third was showing.
//!
//! # A shelf program borrows what the library knows about it
//!
//! A program read off an instrument carries a name, a slot and 242 bytes, and
//! nothing else: no maker, no description, no vocabulary. But
//! [`Fingerprint`](deepmind_patches::Fingerprint) is identity of a *sound*, so
//! where the library has ever published those bytes, the row can say who made
//! it, what it is and what version it is — and where that version is not the
//! newest, say that too.
//!
//! So the columns are the same whichever place a row came from. A row that
//! matched nothing published prints its name and its category and leaves the
//! rest blank, which is the honest answer.
//!
//! # The colours are the library's
//!
//! `resources/palette.toml` names twelve category colours and one per
//! vocabulary axis, carried in `index.toml` so a reader needs no second file.
//! Every chip on this surface is tinted from it — the category cells, the
//! vocabulary terms, and the filters above them — because a filter in a
//! different colour from the thing it filters is a filter somebody has to read
//! rather than recognise.

use control_ui::{Screen, materials, stencil};
use deepmind_midi::sysex::inquiry::Version;
use deepmind_patches::index::IndexPatch;
use deepmind_patches::{Category, Icon};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{App, Browsing, Message, Where};
use crate::catalogue::{Catalogue, Held, Looking, State};
use crate::shelf::Held as OnShelf;

/// Room kept clear down the right for the scroll bar.
const GUTTER: f32 = 16.0;

/// How many dots a patch's icon is drawn on, across and down.
const ICON: i32 = 7;

/// The room the icon column takes.
const ICON_COLUMN: f32 = 22.0;
/// The room the place column takes.
const PLACE: f32 = 104.0;
/// The room a sound's name takes.
const NAME: f32 = 156.0;
/// The room a maker's name takes.
const MAKER: f32 = 96.0;
/// The room a category chip takes.
const CATEGORY: f32 = 84.0;
/// The room the vocabulary chips take.
const TERMS: f32 = 168.0;
/// The room the version takes.
const VERSION: f32 = 62.0;
/// The gap between two columns.
const COLUMN_GAP: f32 = 10.0;
/// How far a chip's ground is carried towards the library's own colour.
const CHIP: f32 = 0.28;
/// How far a chosen filter is carried towards it, which is further: a filter
/// that is on has to be legible across the row of ones that are not.
const CHOSEN: f32 = 0.5;
/// How far an icon is carried from the metal towards its category's colour.
const MARK: f32 = 0.55;
/// How far a banded row is carried from the panel towards a plate.
const BAND: f32 = 0.45;

/// One line of the table.
///
/// Two kinds and one shape. Everything below asks a row for a column and does
/// not care which it is, which is the whole point of merging the lists.
enum Row<'a> {
    /// A program on this machine's shelf, and what the library knows about it.
    Held {
        /// Where it sits on the shelf, which is what a press loads.
        at: usize,
        /// The program.
        held: &'a OnShelf,
        /// The patch it is, where the library has ever published those bytes.
        known: Option<deepmind_patches::index::Match<'a>>,
        /// Which of the two nearby places it is in.
        place: Where,
    },
    /// A patch in the shared library.
    Patch(&'a IndexPatch),
}

impl Row<'_> {
    /// Where this sound is.
    const fn place(&self) -> Where {
        match self {
            Self::Held { place, .. } => *place,
            Self::Patch(_) => Where::Library,
        }
    }

    /// What it is called.
    fn name(&self) -> String {
        match self {
            Self::Held { held, .. } => held.name(),
            Self::Patch(patch) => patch.name.clone(),
        }
    }

    /// Who made it, where anybody knows.
    fn maker(&self) -> Option<&str> {
        match self {
            Self::Held { known, .. } => known.map(|found| found.patch.author.as_str()),
            Self::Patch(patch) => Some(patch.author.as_str()),
        }
    }

    /// What it calls itself.
    fn category(&self) -> Option<Category> {
        match self {
            Self::Held { held, .. } => Category::of(&held.program),
            Self::Patch(patch) => Some(patch.category),
        }
    }

    /// The patch it is, where there is one.
    const fn patch(&self) -> Option<&IndexPatch> {
        match self {
            Self::Held { known, .. } => match known {
                Some(found) => Some(found.patch),
                None => None,
            },
            Self::Patch(patch) => Some(patch),
        }
    }

    /// Which version it is, and whether that is the newest published.
    const fn version(&self) -> Option<(u32, bool)> {
        match self {
            Self::Held { known, .. } => match known {
                Some(found) => Some((found.version, found.latest)),
                None => None,
            },
            Self::Patch(patch) => Some((patch.version, true)),
        }
    }

    /// What it sounds like, where anybody has said.
    fn about(&self) -> &str {
        self.patch().map_or("", |patch| patch.about.as_str())
    }

    /// The slot it names, for a program that names one.
    fn slot(&self) -> Option<String> {
        match self {
            Self::Held { held, .. } => held.slot.map(|slot| slot.to_string()),
            Self::Patch(_) => None,
        }
    }

    /// What pressing it does. Every row plays; nothing is kept by a press.
    fn played(&self) -> Message {
        match self {
            Self::Held { at, .. } => Message::Load(*at),
            Self::Patch(patch) => Message::AuditionPatch(patch.id.clone()),
        }
    }

    /// Whether a search's words are anywhere in what this row says.
    fn said(&self, firmware: Version) -> Vec<String> {
        let mut said = vec![self.name(), self.place().label().to_owned()];
        if let Some(slot) = self.slot() {
            said.push(slot);
        }
        if let Some(maker) = self.maker() {
            said.push(maker.to_owned());
        }
        if let Some(category) = self.category() {
            said.push(category.label().to_owned());
            said.push(category.name().to_owned());
        }
        if let Self::Held { held, .. } = self
            && let Some(category) = held.category(firmware)
        {
            said.push(category.to_owned());
        }
        if let Some(patch) = self.patch() {
            said.push(patch.about.clone());
            said.extend(patch.genre.iter().cloned());
            said.extend(patch.mood.iter().cloned());
            said.extend(patch.timbre.iter().cloned());
            said.extend(patch.role.iter().cloned());
            said.extend(patch.tags.iter().cloned());
        }
        said
    }

    /// Whether the filters leave it standing.
    fn kept(&self, looking: &Looking, firmware: Version) -> bool {
        if let Some(wanted) = looking.place
            && self.place() != wanted
        {
            return false;
        }
        if let Some(wanted) = looking.category
            && self.category() != Some(wanted)
        {
            return false;
        }
        for (axis, term) in &looking.terms {
            let carried = self
                .patch()
                .is_some_and(|patch| terms_of(patch, *axis).iter().any(|held| held == term));
            if !carried {
                return false;
            }
        }
        let find = looking.find.trim().to_lowercase();
        find.is_empty()
            || self
                .said(firmware)
                .into_iter()
                .any(|said| said.to_lowercase().contains(&find))
    }
}

/// The whole surface.
pub fn view(app: &App) -> Element<'_, Message> {
    column![actions(app.catalogue()), standing(app), finding(app)]
        .push(table(app))
        .spacing(10)
        .into()
}

/// A button that is not a parameter, in the instrument's own materials.
fn chrome(label: &str) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(label).size(13))
        .padding([5, 12])
        .style(control_ui::chrome)
}

/// What can be done to the library.
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

/// What is on the table, in one line.
fn standing(app: &App) -> Element<'_, Message> {
    let catalogue = app.catalogue();
    let said = match catalogue.state() {
        State::Working(doing) => (*doing).to_owned(),
        State::Failed(why) => format!("That did not work: {why}"),
        State::Empty | State::Ready => {
            let all = rows(app).len();
            let showing = showing(app).len();
            let count = if app.looking().asking() {
                format!("{showing} of {all} sounds")
            } else {
                format!("{all} sounds")
            };
            match catalogue.held() {
                None => format!("{count} \u{b7} no shared library open yet"),
                Some(held) if held.loadable() => count,
                Some(_) => format!("{count} \u{b7} the shared sounds are still coming"),
            }
        }
    };
    text(said)
        .size(13)
        .style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        })
        .into()
}

/// How to find one sound among all of them.
///
/// One field asking one question of every column, and three rows of filters:
/// where it is, what it calls itself, and what it is like. All of them in the
/// library's own colours, so a filter is the same colour as the cells it keeps.
fn finding(app: &App) -> Element<'_, Message> {
    let looking = app.looking();
    let held = app.catalogue().held();
    let places = row![all(looking.place.is_none(), Message::PatchPlace(None))]
        .extend(Where::ALL.map(|place| {
            let pressed = looking.place == Some(place);
            filter(
                place.label().to_owned(),
                None,
                pressed,
                Message::PatchPlace((!pressed).then_some(place)),
            )
        }))
        .spacing(5)
        .align_y(Center)
        .wrap();
    let categories = row![all(
        looking.category.is_none(),
        Message::PatchCategory(None)
    )]
    .extend(seen(app).into_iter().map(|category| {
        let pressed = looking.category == Some(category);
        filter(
            category.label().to_owned(),
            held.and_then(|held| held.category_colour(category)),
            pressed,
            Message::PatchCategory((!pressed).then_some(category)),
        )
    }))
    .spacing(5)
    .align_y(Center)
    .wrap();
    column![
        row![
            text("Find").size(13),
            text_input("a name, a maker, a slot, a sound", &looking.find)
                .on_input(Message::FindPatch)
                .size(13)
                .padding([5, 8])
                .width(Length::Fixed(280.0)),
        ]
        .spacing(8)
        .align_y(Center),
        places,
        categories,
    ]
    .spacing(6)
    .into()
}

/// The press that clears one row of filters.
fn all<'a>(pressed: bool, said: Message) -> Element<'a, Message> {
    filter("All".to_owned(), None, pressed, said)
}

/// One filter, in the colour of what it keeps.
pub fn filter<'a>(
    label: String,
    colour: Option<[u8; 3]>,
    pressed: bool,
    said: Message,
) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([3, 9])
        .style(move |theme: &Theme, _status| {
            let material = materials(theme);
            let ground = tinted(theme, colour, if pressed { CHOSEN } else { 0.0 });
            button::Style {
                background: Some(Background::Color(ground)),
                text_color: if pressed {
                    control_ui::legible(material.metal, ground, theme)
                } else {
                    material.metal_low
                },
                border: border::rounded(3).width(1.0).color(if pressed {
                    material.lit
                } else {
                    material.recess_edge
                }),
                ..button::Style::default()
            }
        })
        .on_press(said)
        .into()
}

/// Every category anything on the table is filed under, in the instrument's
/// own order.
fn seen(app: &App) -> Vec<Category> {
    let rows = rows(app);
    Category::ALL
        .into_iter()
        .filter(|category| rows.iter().any(|row| row.category() == Some(*category)))
        .collect()
}

/// Every sound, wherever it is.
///
/// The shelf first and the library after it, because the nearer a sound is the
/// sooner somebody wants to see it: what is in the instrument, then what is on
/// the machine, then what could be.
fn rows(app: &App) -> Vec<Row<'_>> {
    let shelf = app.shelf();
    let place = match shelf.source() {
        Some(crate::shelf::Source::Bank(_)) => Where::Instrument,
        _ => Where::Machine,
    };
    let known = app.catalogue().held();
    let mut rows: Vec<Row<'_>> = shelf
        .held()
        .iter()
        .enumerate()
        .map(|(at, held)| Row::Held {
            at,
            held,
            known: known.and_then(|known| known.matching(&held.program)),
            place,
        })
        .collect();
    if let Some(known) = known {
        rows.extend(known.patches().iter().map(Row::Patch));
    }
    rows
}

/// Every sound the filters leave standing.
fn showing(app: &App) -> Vec<Row<'_>> {
    let looking = app.looking();
    let firmware = app.firmware();
    rows(app)
        .into_iter()
        .filter(|row| row.kept(looking, firmware))
        .collect()
}

/// The table.
fn table(app: &App) -> Element<'_, Message> {
    let standing = showing(app);
    if standing.is_empty() {
        let said = if rows(app).is_empty() {
            "Nothing yet. Read a bank off the instrument, open a file, or fetch the shared \
             patches."
        } else {
            "Nothing here is called that, made by them, or one of those."
        };
        return container(text(said).size(13)).padding([12, 0]).into();
    }
    let known = app.catalogue().held();
    let lines = standing
        .into_iter()
        .enumerate()
        .map(|(at, row)| line(&row, at, known, app.trying()));
    column![heading()]
        .push(
            scrollable(container(column(lines).spacing(1)).padding(Padding::ZERO.right(GUTTER)))
                .height(Fill),
        )
        .spacing(4)
        .into()
}

/// What each column holds, printed once above them.
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
            label("WHERE", PLACE),
            label("SOUND", NAME),
            label("MAKER", MAKER),
            label("CATEGORY", CATEGORY),
            label("IS", TERMS),
            label("VERSION", VERSION),
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

/// One sound, as a row.
fn line<'a>(
    row: &Row<'a>,
    at: usize,
    known: Option<&'a Held>,
    trying: Option<&str>,
) -> Element<'a, Message> {
    let banded = at % 2 == 1;
    let category = row.category();
    let colour = known
        .zip(category)
        .and_then(|(known, category)| known.category_colour(category));
    let lit = matches!(row, Row::Patch(patch) if trying == Some(patch.id.as_str()));
    let face = row![
        container(drawn(icon_of(row, known), colour))
            .width(Length::Fixed(ICON_COLUMN))
            .align_x(Horizontal::Center),
        container(place_cell(row)).width(Length::Fixed(PLACE)),
        container(text(row.name()).size(13)).width(Length::Fixed(NAME)),
        container(dim(row.maker().unwrap_or("\u{2014}").to_owned(), 12.0))
            .width(Length::Fixed(MAKER)),
        container(match category {
            Some(category) => chip(category.label().to_owned(), colour),
            None => dim("\u{2014}".to_owned(), 11.0),
        })
        .width(Length::Fixed(CATEGORY)),
        container(terms(row, known)).width(Length::Fixed(TERMS)),
        container(version_cell(row)).width(Length::Fixed(VERSION)),
        container(
            text(row.about().to_owned())
                .size(12)
                .wrapping(iced::widget::text::Wrapping::None)
                .style(|theme: &Theme| text::Style {
                    color: Some(materials(theme).metal_low),
                })
        )
        .width(Fill)
        .clip(true),
    ]
    .spacing(COLUMN_GAP)
    .align_y(Vertical::Center);
    // Every row plays, whichever place it came from, and nothing a press does
    // is kept: a shelf program goes to the edit buffer and so does a library
    // one. The press at the end is the one that keeps, and only a library row
    // has anything to keep.
    let played = button(face)
        .width(Length::Fill)
        .padding([5, 8])
        .style(move |theme: &Theme, status| banded_like(theme, banded, lit, status))
        .on_press(row.played());
    let keeps = match row {
        Row::Patch(patch) if known.is_some_and(Held::loadable) => Some(patch.id.clone()),
        _ => None,
    };
    row![played, keep(keeps)]
        .spacing(2)
        .align_y(Vertical::Center)
        .into()
}

/// Where the sound is, and the slot it names.
fn place_cell<'a>(row: &Row<'a>) -> Element<'a, Message> {
    let place = row.place();
    let mut cell = row![chip(place.label().to_owned(), place_colour(place))].spacing(5);
    if let Some(slot) = row.slot() {
        cell = cell.push(dim(slot, 11.0));
    }
    cell.align_y(Vertical::Center).into()
}

/// The colour a place is drawn in.
///
/// Not the library's, because the library names colours for what a sound *is*
/// and not for where it happens to be. These are this window's own three, taken
/// from what it already spends: the instrument's lamp for what is in the
/// instrument, and nothing at all for the other two, which are ground rather
/// than figure.
const fn place_colour(place: Where) -> Option<[u8; 3]> {
    match place {
        Where::Instrument => Some([0xff, 0xbe, 0x3d]),
        Where::Machine | Where::Library => None,
    }
}

/// Which version it is, in amber where a newer one is published.
fn version_cell<'a>(row: &Row<'a>) -> Element<'a, Message> {
    let Some((version, latest)) = row.version() else {
        return dim("\u{2014}".to_owned(), 11.0);
    };
    let said = if latest {
        format!("v{version}")
    } else {
        format!("v{version} \u{2191}")
    };
    text(said)
        .size(11)
        .style(move |theme: &Theme| text::Style {
            color: Some(if latest {
                materials(theme).metal_low
            } else {
                control_ui::WAY_IN
            }),
        })
        .into()
}

/// The vocabulary terms a row prints.
fn terms<'a>(of_row: &Row<'a>, known: Option<&'a Held>) -> Element<'a, Message> {
    let Some(patch) = of_row.patch() else {
        return space().into();
    };
    let mut said = Vec::new();
    for axis in [
        deepmind_patches::Axis::Mood,
        deepmind_patches::Axis::Timbre,
        deepmind_patches::Axis::Role,
        deepmind_patches::Axis::Genre,
    ] {
        let colour = known.and_then(|known| known.axis_colour(axis));
        said.extend(
            terms_of(patch, axis)
                .iter()
                .map(|term| (term.clone(), colour)),
        );
    }
    said.truncate(CHIPS);
    row(said.into_iter().map(|(term, colour)| chip(term, colour)))
        .spacing(4)
        .into()
}

/// How many vocabulary terms one row prints.
const CHIPS: usize = 3;

/// One axis of a patch's vocabulary terms.
fn terms_of(patch: &IndexPatch, axis: deepmind_patches::Axis) -> &Vec<String> {
    match axis {
        deepmind_patches::Axis::Genre => &patch.genre,
        deepmind_patches::Axis::Mood => &patch.mood,
        deepmind_patches::Axis::Timbre => &patch.timbre,
        deepmind_patches::Axis::Role => &patch.role,
    }
}

/// The icon a row is drawn by: the patch's own, or its category's.
fn icon_of(row: &Row<'_>, known: Option<&Held>) -> Option<Icon> {
    if let Some(patch) = row.patch() {
        return Some(patch.icon);
    }
    known
        .zip(row.category())
        .and_then(|(known, category)| known.category_icon(category))
}

/// Text in the ink a legend is printed in.
fn dim<'a>(said: String, size: f32) -> Element<'a, Message> {
    text(said)
        .size(size)
        .style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        })
        .into()
}

/// One word on a tinted ground.
pub fn chip<'a>(said: String, colour: Option<[u8; 3]>) -> Element<'a, Message> {
    container(text(said).size(11).style(move |theme: &Theme| text::Style {
        color: Some(match colour {
            Some(rgb) => control_ui::legible(of(rgb), tinted(theme, colour, CHIP), theme),
            None => materials(theme).metal_low,
        }),
    }))
    .padding([1, 6])
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(tinted(theme, colour, CHIP))),
        border: border::rounded(3),
        ..container::Style::default()
    })
    .into()
}

/// A ground carried `how_far` from the recess towards a colour the library
/// named.
pub fn tinted(theme: &Theme, colour: Option<[u8; 3]>, how_far: f32) -> iced::Color {
    let material = materials(theme);
    match colour {
        Some(rgb) if how_far > 0.0 => control_ui::mix(material.recess, of(rgb), how_far),
        Some(_) | None if how_far > 0.0 => material.plate,
        _ => material.recess,
    }
}

/// A colour the library named, as one this window can draw.
pub const fn of(rgb: [u8; 3]) -> iced::Color {
    let [red, green, blue] = rgb;
    iced::Color::from_rgb8(red, green, blue)
}

/// A patch's icon, on the instrument's own dots, in its category's colour.
fn drawn<'a>(icon: Option<Icon>, colour: Option<[u8; 3]>) -> Element<'a, Message> {
    let mut screen = Screen::new(ICON, ICON);
    if let Some(icon) = icon {
        screen.blit(&icon.pixels(), 0, 0);
    }
    Element::from(stencil(screen, move |theme: &Theme| match colour {
        // Lifted off the library's own colour rather than set to it: a
        // seven-dot picture in `#5DB56A` on a dark panel is a smudge, and what
        // a category colour is for is telling two of them apart at a glance.
        Some(rgb) => control_ui::mix(materials(theme).metal, of(rgb), MARK),
        None => materials(theme).metal,
    }))
    .map(Message::Ui)
}

/// The press at the end of a row that puts that one sound on the shelf.
fn keep<'a>(id: Option<String>) -> Element<'a, Message> {
    // Nothing at all where there is nothing to keep, rather than a press that
    // does not go down: a column of empty bordered boxes down the side of a
    // shelf is a column of controls that look broken. The room is held, so the
    // rows still line up.
    let Some(id) = id else {
        return space().width(Length::Fixed(KEEP)).into();
    };
    container(
        button(
            Element::from(stencil(control_ui::DOWN.screen(), |theme: &Theme| {
                materials(theme).metal_low
            }))
            .map(Message::Ui),
        )
        .padding([5, 8])
        .style(control_ui::chrome)
        .on_press(Message::ShelvePatch(id)),
    )
    .width(Length::Fixed(KEEP))
    .align_x(Horizontal::Center)
    .into()
}

/// The room the press that keeps a sound takes.
const KEEP: f32 = 30.0;

/// The ground one row of the table stands on.
fn banded_like(theme: &Theme, banded: bool, lit: bool, status: button::Status) -> button::Style {
    let material = materials(theme);
    let palette = theme.extended_palette();
    let ground = match status {
        button::Status::Hovered | button::Status::Pressed => material.plate,
        _ if lit => material.plate,
        _ if banded => control_ui::mix(material.panel, material.plate, BAND),
        _ => material.panel,
    };
    button::Style {
        background: Some(Background::Color(ground)),
        text_color: palette.background.base.text,
        border: border::rounded(2)
            .width(if lit { 1.0 } else { 0.0 })
            .color(if lit { material.lit } else { material.panel }),
        ..button::Style::default()
    }
}

/// The two things the librarian does, and which one is showing.
pub fn switch<'a>(browsing: Browsing) -> Element<'a, Message> {
    let one = |label: &'static str, which: Browsing| {
        let pressed = browsing == which;
        button(text(label).size(12))
            .padding([4, 12])
            .style(move |theme: &Theme, _status| {
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
            })
            .on_press(Message::Browse(which))
    };
    row![
        one("Sounds", Browsing::Sounds),
        one("Share one", Browsing::Publish),
    ]
    .spacing(6)
    .align_y(Center)
    .into()
}

#[cfg(test)]
mod tests {
    use deepmind_patches::Category;

    use crate::app::Where;

    use super::{CATEGORY, PLACE};

    #[test]
    fn the_category_column_is_wide_enough_for_every_category() {
        // Eleven-point type at roughly six points a character, plus the chip's
        // own padding either side. The twelve labels are the part this window
        // cannot shorten, and a column that clipped one would lie about what a
        // sound is.
        let longest = Category::ALL
            .into_iter()
            .map(|category| category.label().len())
            .max()
            .unwrap_or_default();
        let needed = u16::try_from(longest).unwrap_or(u16::MAX);
        assert!(
            CATEGORY > f32::from(needed).mul_add(6.0, 12.0),
            "a {longest}-character category does not fit the category column"
        );
    }

    #[test]
    fn the_place_column_holds_a_place_and_a_slot() {
        // `Synth` and `A128` side by side is the widest this cell gets, and the
        // whole reason the column exists is that a person can see where a sound
        // is without pressing anything.
        let longest = Where::ALL
            .into_iter()
            .map(|place| place.label().len())
            .max()
            .unwrap_or_default();
        let needed = u16::try_from(longest).unwrap_or(u16::MAX);
        // The place chip, its padding, a gap, and four characters of slot.
        assert!(
            PLACE > f32::from(needed).mul_add(6.0, 12.0 + 5.0 + 26.0),
            "a {longest}-character place leaves no room for a slot beside it"
        );
    }
}

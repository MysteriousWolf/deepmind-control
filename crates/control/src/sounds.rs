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
use iced::widget::{
    button, column, container, pick_list, row, scrollable, space, text, text_input,
};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{App, Browsing, By, Message, Sorting, Where};
use crate::catalogue::{Catalogue, Held, Looking, State};
use crate::shelf::Held as OnShelf;
use deepmind_midi::ids::{BANK_COUNT, Bank};

/// Room kept clear down the right for the scroll bar.
const GUTTER: f32 = 16.0;

/// How many dots a patch's icon is drawn on, across and down.
const ICON: i32 = 7;

/// The room the icon column takes.
const ICON_COLUMN: f32 = 22.0;
/// The room the place column takes.
const PLACE: f32 = 88.0;
/// The room the bank column takes.
const BANK: f32 = 74.0;
/// The room the number column takes.
const NUMBER: f32 = 58.0;
/// The room a sound's name takes.
const NAME: f32 = 128.0;
/// The room a maker's name takes.
const MAKER: f32 = 80.0;
/// The room a category chip takes.
const CATEGORY: f32 = 96.0;
/// The room the vocabulary chips take.
const TERMS: f32 = 96.0;
/// The room the version takes.
const VERSION: f32 = 70.0;
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

    /// Whether a newer version of this sound has been published.
    const fn stale(&self) -> bool {
        match self.version() {
            Some((_, latest)) => !latest,
            None => false,
        }
    }

    /// What it sounds like, where anybody has said.
    fn about(&self) -> &str {
        self.patch().map_or("", |patch| patch.about.as_str())
    }

    /// The bank it sits in, for a sound that sits in one.
    ///
    /// Its own column rather than half of the place, because it is its own
    /// fact: a person asking *what is in bank C* is asking something different
    /// from *what is on the synthesizer*, and a cell holding both can be
    /// filtered by neither.
    fn bank(&self) -> Option<Bank> {
        match self {
            Self::Held { held, .. } => held.slot.map(|slot| slot.bank),
            Self::Patch(_) => None,
        }
    }

    /// The number within that bank.
    fn number(&self) -> Option<u8> {
        match self {
            Self::Held { held, .. } => held.slot.map(|slot| slot.number.get()),
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
        if let Some(bank) = self.bank() {
            said.push(bank.letter().to_string());
            if let Some(number) = self.number() {
                said.push(format!("{}{}", bank.letter(), number.saturating_add(1)));
            }
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
        if let Some(wanted) = looking.bank
            && self.bank() != Some(wanted)
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
/// One field asking one question of every column: a name, a maker, a slot, a
/// category, a vocabulary term, a tag. Everything that narrows to *one of a
/// known set* is in the column heading instead, where the column it narrows
/// is, rather than in a row of chips somebody has to match up by eye.
fn finding(app: &App) -> Element<'_, Message> {
    let looking = app.looking();
    row![
        text("Find").size(13),
        text_input("a name, a maker, a slot, a sound", &looking.find)
            .on_input(Message::FindPatch)
            .size(13)
            .padding([5, 8])
            .width(Length::Fixed(300.0)),
        space().width(Fill),
        clearing(looking),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

/// The press that puts every column's chooser back to showing everything.
///
/// Drawn only while something is narrowed, because a press that does nothing
/// is a press somebody has to try to find out. Three headings and a field is
/// enough places to have left something switched on that finding them all
/// again is worth one press.
fn clearing<'a>(looking: &Looking) -> Element<'a, Message> {
    if !looking.asking() {
        return space().into();
    }
    button(text("Show everything").size(12))
        .padding([3, 9])
        .style(control_ui::chrome)
        .on_press(Message::ShowEverything)
        .into()
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

/// Every sound the filters leave standing, in the order the heading asks for.
///
/// Sorting never touches the shelf, which is the rule its own search is already
/// under: this is which way round the same sounds are drawn, and what a save
/// writes is still the pack that was opened in its own order.
fn showing(app: &App) -> Vec<Row<'_>> {
    let looking = app.looking();
    let firmware = app.firmware();
    let mut standing: Vec<Row<'_>> = rows(app)
        .into_iter()
        .filter(|row| row.kept(looking, firmware))
        .collect();
    let sorting = app.sorting();
    standing.sort_by(|one, other| {
        let order = match sorting.by {
            // Nearest first, which is the order `rows` already builds and the
            // order the table opens in.
            By::Where => std::cmp::Ordering::Equal,
            By::Bank => one.bank().cmp(&other.bank()),
            By::Number => one.number().cmp(&other.number()),
            By::Sound => one.name().to_lowercase().cmp(&other.name().to_lowercase()),
            By::Maker => one
                .maker()
                .map(str::to_lowercase)
                .cmp(&other.maker().map(str::to_lowercase)),
            By::Category => one
                .category()
                .map(Category::label)
                .cmp(&other.category().map(Category::label)),
            // Stale first when it is turned round, because *what needs
            // updating* is the question somebody sorts this column to ask.
            By::Version => one.stale().cmp(&other.stale()),
        };
        if sorting.down { order.reverse() } else { order }
    });
    standing
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
    column![heading(app)]
        .push(
            scrollable(container(column(lines).spacing(1)).padding(Padding::ZERO.right(GUTTER)))
                .height(Fill),
        )
        .spacing(4)
        .into()
}

/// The heading, which is where the table is both sorted and narrowed.
///
/// One press per column lays the table out by it, and pressing the one it is
/// already under turns it round. Three of them narrow as well — where, bank and
/// category — because those are the columns with a small, known set of values,
/// and a filter belongs on the column it filters rather than in a row of chips
/// somebody has to match up by eye.
///
/// The rest are sort-only. A maker's name is not a set anybody can be offered,
/// and the search field already reads it.
fn heading(app: &App) -> Element<'_, Message> {
    let sorting = app.sorting();
    let looking = app.looking();

    let banks: Vec<Bank> = (0..BANK_COUNT)
        .filter_map(|at| Bank::new(at).ok())
        .collect();
    container(
        row![
            container(space()).width(Length::Fixed(ICON_COLUMN)),
            narrowed(
                By::Where,
                sorting,
                PLACE,
                Where::ALL
                    .map(|place| (place.label().to_owned(), Some(place)))
                    .to_vec(),
                looking.place,
                Message::PatchPlace,
            ),
            narrowed(
                By::Bank,
                sorting,
                BANK,
                banks
                    .iter()
                    .map(|bank| (bank.letter().to_string(), Some(*bank)))
                    .collect(),
                looking.bank,
                Message::PatchBank,
            ),
            sorted(By::Number, sorting, NUMBER),
            sorted(By::Sound, sorting, NAME),
            sorted(By::Maker, sorting, MAKER),
            narrowed(
                By::Category,
                sorting,
                CATEGORY,
                seen(app)
                    .into_iter()
                    .map(|category| (category.label().to_owned(), Some(category)))
                    .collect(),
                looking.category,
                Message::PatchCategory,
            ),
            plain("IS", TERMS),
            sorted(By::Version, sorting, VERSION),
            plain("ABOUT", ABOUT),
        ]
        .spacing(COLUMN_GAP)
        .align_y(Vertical::Center),
    )
    .padding(Padding::from([0.0, 8.0]).right(GUTTER + 8.0 + KEEP))
    .into()
}

/// A column heading that only sorts.
fn sorted<'a>(by: By, sorting: Sorting, width: f32) -> Element<'a, Message> {
    container(caret(by, sorting))
        .width(Length::Fixed(width))
        .into()
}

/// A column heading that sorts and narrows.
///
/// The chooser carries the heading when nothing is chosen and the value when
/// something is, so the column says what it is showing without a second line.
fn narrowed<'a, T>(
    by: By,
    sorting: Sorting,
    width: f32,
    values: Vec<(String, Option<T>)>,
    chosen: Option<T>,
    said: impl Fn(Option<T>) -> Message + 'a,
) -> Element<'a, Message>
where
    T: Clone + PartialEq + 'a,
{
    let mut options: Vec<Narrowing<T>> = vec![Narrowing {
        said: by.heading().to_owned(),
        value: None,
    }];
    options.extend(
        values
            .into_iter()
            .map(|(said, value)| Narrowing { said, value }),
    );
    // Always something chosen, never `None`: a column showing everything is
    // showing its own heading, and `None` would draw an empty box that says the
    // column has lost its name rather than that it is not narrowed.
    let now = Some(match chosen {
        Some(value) => Narrowing {
            said: options
                .iter()
                .find(|one| one.value.as_ref() == Some(&value))
                .map_or_else(String::new, |one| one.said.clone()),
            value: Some(value),
        },
        None => Narrowing {
            said: by.heading().to_owned(),
            value: None,
        },
    });
    // The chooser *is* the heading. It carries the column's name while nothing
    // is chosen and the chosen value afterwards, so the strip says what each
    // column holds and what it is showing in one line rather than two saying
    // the same word twice.
    row![
        pick_list(options, now, move |one| said(one.value))
            .text_size(10)
            .padding([2, 5])
            .width(Length::Fixed(width - ARROW))
            .style(control_ui::selector)
            .menu_style(control_ui::shortlist),
        arrow(by, sorting),
    ]
    .spacing(0)
    .align_y(Vertical::Center)
    .width(Length::Fixed(width))
    .into()
}

/// How much room the sort mark takes beside a chooser.
const ARROW: f32 = 16.0;

/// The mark beside a chooser that lays the table out by its column.
///
/// A caret and nothing else, because the chooser has already said the name. It
/// is drawn dim until the table is under that column, which is how somebody
/// tells at a glance which of the columns it is laid out by.
fn arrow<'a>(by: By, sorting: Sorting) -> Element<'a, Message> {
    let under = sorting.by == by;
    let said = if under && sorting.down {
        "\u{25be}"
    } else {
        "\u{25b4}"
    };
    button(text(said).size(9).style(move |theme: &Theme| text::Style {
        color: Some(if under {
            materials(theme).metal
        } else {
            materials(theme).recess_edge
        }),
    }))
    .padding([2, 3])
    .style(|_theme: &Theme, _status| button::Style::default())
    .on_press(Message::SortSounds(by))
    .into()
}

/// One option of a column's chooser.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Narrowing<T> {
    /// What it is called.
    said: String,
    /// What it narrows to, or nothing for the heading itself.
    value: Option<T>,
}

impl<T> std::fmt::Display for Narrowing<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.said)
    }
}

/// The heading as a press, with the mark that says which way round it is.
fn caret<'a>(by: By, sorting: Sorting) -> Element<'a, Message> {
    let under = sorting.by == by;
    let said = if under {
        // The arrow points the way the column runs, which is what an arrow in a
        // table heading has always meant.
        format!(
            "{} {}",
            by.heading(),
            if sorting.down { "\u{2193}" } else { "\u{2191}" }
        )
    } else {
        by.heading().to_owned()
    };
    button(text(said).size(10).style(move |theme: &Theme| text::Style {
        color: Some(if under {
            materials(theme).metal
        } else {
            materials(theme).metal_low
        }),
    }))
    .padding([1, 2])
    .style(|_theme: &Theme, _status| button::Style::default())
    .on_press(Message::SortSounds(by))
    .into()
}

/// A heading that is neither sorted nor narrowed.
///
/// Two columns are: what a sound *is*, which is a handful of chips rather than
/// one value to sort by, and what it sounds like, which is a sentence. Both are
/// still read by the search field.
fn plain<'a>(said: &'static str, width: f32) -> Element<'a, Message> {
    container(text(said).size(10).style(|theme: &Theme| text::Style {
        color: Some(materials(theme).metal_low),
    }))
    .padding([1, 2])
    .width(Length::Fixed(width))
    .into()
}

/// The room what a sound is like takes, which is what is left.
const ABOUT: f32 = 132.0;

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
        glass(
            row.bank().map(|bank| bank.letter().to_string()),
            BANK,
            BANK_DOTS,
        ),
        glass(
            row.number()
                .map(|number| number.saturating_add(1).to_string()),
            NUMBER,
            NUMBER_DOTS,
        ),
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
        .width(Length::Fixed(ABOUT))
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

/// Where the sound is.
fn place_cell<'a>(of_row: &Row<'a>) -> Element<'a, Message> {
    let place = of_row.place();
    chip(place.label().to_owned(), place_colour(place))
}

/// The bank a sound sits in, on the instrument's own glass.
///
/// A bank letter and a program number are what the front panel's display shows
/// and nothing else in this window writes, so they are written the way it
/// writes them: dark dots on a lit field, at the pitch every other display here
/// is drawn at. Sixty of them down a table reads as a rack of little screens,
/// which is what a list of slots in a synthesizer is.
fn glass<'a>(said: Option<String>, width: f32, dots: i32) -> Element<'a, Message> {
    let Some(said) = said else {
        return container(dim("\u{2014}".to_owned(), 11.0))
            .width(Length::Fixed(width))
            .align_x(Horizontal::Center)
            .into();
    };
    // Sized to the column rather than to the word, so every cell down the
    // column is the same display rather than a row of screens that grow and
    // shrink with what is on them. A `B` and a `128` are the same slot written
    // shorter, not a smaller instrument.
    let mut screen = Screen::new(dots, LINE);
    screen.centre(0, &said, control_ui::Size::Small);
    container(
        Element::from(control_ui::lcd(screen, control_ui::Confidence::Confirmed)).map(Message::Ui),
    )
    .width(Length::Fixed(width))
    .align_x(Horizontal::Center)
    .into()
}

/// How many dots deep one line of a display is.
const LINE: i32 = 7;
/// How many dots wide the bank's own display is: one character, with a dot
/// either side so the glass is a frame rather than a fit.
const BANK_DOTS: i32 = 7;
/// And the number's: three characters, which is `128`.
const NUMBER_DOTS: i32 = 19;

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

    use crate::app::{By, Where};

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
    fn the_place_column_holds_the_longest_place() {
        // The bank and the number are their own columns now, so this cell holds
        // one chip and nothing else. It still has to hold the longest of the
        // three without clipping, because a place that read `Librar` would be a
        // place nobody trusts.
        let longest = Where::ALL
            .into_iter()
            .map(|place| place.label().len())
            .max()
            .unwrap_or_default();
        let needed = u16::try_from(longest).unwrap_or(u16::MAX);
        assert!(
            PLACE > f32::from(needed).mul_add(6.0, 12.0),
            "a {longest}-character place does not fit the place column"
        );
    }

    #[test]
    fn every_column_that_sorts_is_drawn() {
        // The heading is the only place the table can be sorted from, so a
        // column in `By::ALL` that nothing draws a heading for is a sort
        // nobody can reach.
        for by in By::ALL {
            assert!(!by.heading().is_empty(), "{by:?} has no heading to press");
        }
    }
}

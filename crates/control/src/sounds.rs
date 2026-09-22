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
    button, column, container, mouse_area, pick_list, responsive, row, scrollable, space, stack,
    text, text_input,
};
use iced::{Background, Center, Element, Fill, Length, Padding, Theme, border};

use crate::app::{Action, App, By, Chosen, Message, Where};
use crate::catalogue::{Catalogue, Held, Looking, State};
use crate::shelf::Held as OnShelf;
use deepmind_midi::ids::Bank;

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

    /// The newest version of this sound the library has published.
    ///
    /// Its own number rather than the one this copy is, so the press that takes
    /// it can say what it is about to become: `v2 \u{2192} v4` is a press
    /// somebody can weigh, and an arrow on its own is one they have to try.
    const fn newest(&self) -> u32 {
        match self {
            Self::Held { known, .. } => match known {
                Some(found) => found.patch.version,
                None => 0,
            },
            Self::Patch(patch) => patch.version,
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

    /// Which sound this row is, as something that outlives the drawing.
    ///
    /// A row is rebuilt every frame and moves every time the table is sorted,
    /// so what a press hands on cannot be the row: a place on the shelf and a
    /// patch's id both survive both.
    fn chosen(&self) -> Chosen {
        match self {
            Self::Held { at, .. } => Chosen::Held(*at),
            Self::Patch(patch) => Chosen::Patch(patch.id.clone()),
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

    /// What one column says about this row, as a filter reads it.
    ///
    /// **Not always what the column prints.** The version column prints
    /// `v2 \u{2192} v4` and says `v2 behind`, because *behind* is the thing
    /// anybody would narrow that column to and an arrow is not a word. Every
    /// other column says what it shows.
    ///
    /// This is also what the column is sorted by, so a column sorts and
    /// narrows on one answer rather than two that can disagree.
    fn said_in(&self, by: By) -> String {
        match by {
            By::Where => self.place().label().to_owned(),
            By::Bank => self
                .bank()
                .map(|bank| bank.letter().to_string())
                .unwrap_or_default(),
            // Padded, so that sorting the text sorts the numbers: `9` after
            // `10` is what a table that compares `"9"` with `"10"` shows.
            By::Number => self
                .number()
                .map(|number| format!("{:03}", number.saturating_add(1)))
                .unwrap_or_default(),
            By::Name => self.name(),
            By::Maker => self.maker().unwrap_or_default().to_owned(),
            By::Category => self
                .category()
                .map(Category::label)
                .unwrap_or_default()
                .to_owned(),
            By::Tags => self.tags().join(" "),
            // How many there are, so the column sorts *most demos first* when
            // it is turned round and narrows to `1` or `2`. The cell itself is
            // presses and has no text to read.
            By::Demos => match self.demos().len() {
                0 => String::new(),
                many => many.to_string(),
            },
            By::Version => match self.version() {
                Some((version, true)) => format!("v{version} current"),
                Some((version, false)) => format!("v{version} behind"),
                None => String::new(),
            },
            By::About => self.about().to_owned(),
        }
    }

    /// Every recording of this sound the library published.
    ///
    /// Only a library row has any: a program off a shelf is 242 bytes and a
    /// recording is a file somebody uploaded beside a patch. Where a shelf
    /// program matched a published patch by its fingerprint it borrows that
    /// patch's, the same as it borrows the maker and the description.
    fn demos(&self) -> &[deepmind_patches::index::IndexDemo] {
        self.patch().map_or(&[], |patch| patch.demos.as_slice())
    }

    /// Every vocabulary term this row carries, in the order they are drawn.
    fn tags(&self) -> Vec<String> {
        let Some(patch) = self.patch() else {
            return Vec::new();
        };
        [
            deepmind_patches::Axis::Mood,
            deepmind_patches::Axis::Timbre,
            deepmind_patches::Axis::Role,
            deepmind_patches::Axis::Genre,
        ]
        .into_iter()
        .flat_map(|axis| terms_of(patch, axis).iter().cloned())
        .collect()
    }

    /// Whether the filters leave it standing.
    fn kept(&self, looking: &Looking, firmware: Version) -> bool {
        if !looking.narrows(|by| self.said_in(by)) {
            return false;
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
///
/// Measured before it is drawn, for one reason: a menu opens where the pointer
/// was, and a menu that opens below the bottom of the window is a menu nobody
/// can reach. How much room there is is the only thing that can bring it back
/// inside, and it is the only thing this closure uses it for.
pub fn view(app: &App) -> Element<'_, Message> {
    responsive(move |room| surface(app, room)).into()
}

/// The surface, once it knows how much room it has.
fn surface(app: &App, room: iced::Size) -> Element<'_, Message> {
    // The pointer is tracked here rather than on the rows, because a row's own
    // coordinates start at the row: a menu placed from those would open at the
    // top of the table every time.
    let laid = mouse_area(
        column![actions(app), finding(app)]
            .push(table(app))
            .spacing(10),
    )
    .on_move(Message::PointerAt);
    let Some(at) = app.menu() else {
        return laid.into();
    };
    stack![laid, shade(), floating(app, at, room)].into()
}

/// What can be done to the sound that is chosen, and what can be done to the
/// library.
///
/// **Two groups and a gap between them**, which is the whole of the layout: on
/// the left the seven verbs, every one of them about the one row somebody
/// pressed; on the right the three presses that are about the library itself
/// rather than about any sound in it. A row where *export this patch* stood
/// beside *fetch the newest release* would be a row somebody has to read
/// before using.
fn actions(app: &App) -> Element<'_, Message> {
    row![toolbar(app), space().width(Fill), library(app.catalogue())]
        .spacing(14)
        .align_y(Center)
        .into()
}

/// The seven verbs, along the top.
///
/// The same seven the menu a right-press opens carries, in the same order, with
/// the same marks and greyed out by the same rule — [`App::can`] decides for
/// both. A toolbar that offered what its own menu refused would be a window
/// that disagrees with itself.
///
/// **Each one carries its word.** They were marks alone, with the word left to
/// the footer, and a nine-dot drawing nobody has been introduced to is a
/// drawing nobody can read: an arrow onto a shelf and an arrow into a memory
/// are the same arrow until somebody has been told which is which. The mark
/// earns its place by being the thing recognised at the second glance, and the
/// word is how there is a first one.
///
/// The sentence in the footer is still there, and it is the longer answer: the
/// toolbar says `Copy here` and the footer says what that means.
fn toolbar(app: &App) -> Element<'_, Message> {
    row(Action::TOOLBAR.map(|action| tool(app, action)))
        .spacing(2)
        .align_y(Center)
        .into()
}

/// One verb: the mark, and the word under it.
///
/// **Under and not beside.** A word beside a mark makes a press as wide as the
/// word is long, so a row of them is a ragged line of different-sized boxes and
/// the long ones (`Save shelf\u{2026}`) crowd out the short. Stacked, every press
/// is the same narrow column, the words line up along one baseline, and a word
/// too long for the column wraps onto a second line instead of widening it.
fn tool(app: &App, action: Action) -> Element<'_, Message> {
    let usable = app.can(action);
    hinting(
        button(stacked(action_badge(action), action.label(), usable))
            .padding([3, 4])
            .style(control_ui::marked)
            .on_press_maybe(usable.then_some(Message::Act(action))),
        action.about(),
    )
}

/// A mark with its word under it, dimmed together while the press is refused.
fn stacked(badge: control_ui::Badge, label: &str, usable: bool) -> Element<'_, Message> {
    column![
        mark(badge, usable),
        text(label)
            .size(10)
            .center()
            .wrapping(iced::widget::text::Wrapping::Word)
            .style(move |theme: &Theme| text::Style {
                color: Some(if usable {
                    theme.extended_palette().background.base.text
                } else {
                    materials(theme).metal_low
                }),
            }),
    ]
    .spacing(3)
    .width(Length::Fixed(STACKED))
    .align_x(Horizontal::Center)
    .into()
}

/// How wide a press with its word under it stands.
///
/// Enough for `Checkout` on one line and for `Shelve these` on two. A column
/// this narrow is what makes the words wrap rather than the presses widen,
/// which is the whole point of stacking them.
const STACKED: f32 = 62.0;

/// What is done to the library itself, up in the corner.
///
/// Three, and none of them is about a sound: where the shared patches come
/// from, and the one press that takes everything the filters left standing onto
/// this machine at once. Top right because that is where a window's own
/// housekeeping goes and because the left of this row belongs to the sound
/// somebody has in hand.
fn library(catalogue: &Catalogue) -> Element<'_, Message> {
    let working = catalogue.working();
    let loadable = catalogue.held().is_some_and(Held::loadable);
    row![
        press(
            control_ui::FETCH,
            "Fetch",
            (!working).then_some(Message::FetchPatches),
            "Fetch the newest published library over the network.",
        ),
        press(
            control_ui::FOLDER,
            "Checkout\u{2026}",
            (!working).then_some(Message::OpenPatches),
            "Read a checkout of the shared patches off a folder on this machine.",
        ),
        press(
            control_ui::SHELVE,
            "Shelve these",
            loadable.then_some(Message::ShelveShowing),
            "Put every sound the filters left standing onto this machine's shelf.",
        ),
    ]
    .spacing(2)
    .align_y(Center)
    .into()
}

/// One press whose whole face is a mark, and what it says about itself.
///
/// The one press the whole librarian is built from, here and in
/// [`crate::librarian`]: a nine-dot mark on the bare panel, greyed while it is
/// refused, with its sentence in the footer under the pointer.
pub fn press<'a>(
    badge: control_ui::Badge,
    label: &'a str,
    said: Option<Message>,
    about: &'static str,
) -> Element<'a, Message> {
    let usable = said.is_some();
    hinting(
        button(stacked(badge, label, usable))
            .padding([3, 4])
            .style(control_ui::marked)
            .on_press_maybe(said),
        about,
    )
}

/// The menu a right-press opens, standing where the press was.
///
/// The same seven verbs the toolbar carries, and here they carry their words as
/// well: a menu is what somebody opens when the mark alone was not enough, so a
/// menu of marks would be a menu that answers nothing. The mark stays beside
/// the word so that the two are learnt together, which is the only reason the
/// toolbar can be marks alone.
fn menu(app: &App) -> Element<'_, Message> {
    container(column(Action::ALL.map(|action| entry(app, action))).spacing(1))
        .width(Length::Fixed(MENU_WIDE))
        .padding(4)
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
                background: Some(Background::Color(material.plate)),
                border: border::rounded(4).width(1.0).color(material.metal_low),
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                    offset: iced::Vector::new(0.0, 3.0),
                    blur_radius: 9.0,
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// One line of the menu: the mark, and the word beside it.
fn entry(app: &App, action: Action) -> Element<'_, Message> {
    let usable = app.can(action);
    hinting(
        button(
            row![
                container(mark(action_badge(action), usable))
                    .width(Length::Fixed(MENU_MARK))
                    .align_x(Horizontal::Center),
                text(action.label())
                    .size(12)
                    .style(move |theme: &Theme| text::Style {
                        color: Some(if usable {
                            theme.extended_palette().background.base.text
                        } else {
                            materials(theme).metal_low
                        }),
                    }),
            ]
            .spacing(8)
            .align_y(Center),
        )
        .width(Fill)
        .padding([4, 6])
        .style(control_ui::marked)
        .on_press_maybe(usable.then_some(Message::Act(action))),
        action.about(),
    )
}

/// How wide the menu stands.
///
/// Wide enough for the longest verb and its mark and no wider. It is a fixed
/// number rather than the toolkit's own shrink because the menu is placed by
/// hand: a floating thing has to be brought back inside the window when it
/// opens near an edge, and nothing can do that arithmetic without knowing how
/// much room the thing takes.
const MENU_WIDE: f32 = 190.0;

/// The column the marks down the menu stand in.
///
/// Wide enough for a nine-dot mark at the pitch every drawing in this window
/// shares, which is what decides it: a column narrower than the mark is a
/// column the mark sits on top of the word in.
const MENU_MARK: f32 = 24.0;

/// How tall the menu stands: seven lines, their gaps, and the rim.
const MENU_TALL: f32 = 7.0 * 31.0 + 6.0 * 1.0 + 8.0;

/// The mark a verb wears, everywhere it is offered.
///
/// One mark per verb and the same one in both places, which is what makes a
/// toolbar of marks readable at all: somebody learns the seven once, in the
/// menu where the words are, and reads them thereafter along the top.
const fn action_badge(action: Action) -> control_ui::Badge {
    match action {
        Action::Load => control_ui::PLAY,
        // The front panel itself, because that is where the press goes: `Edit`
        // hears the sound and puts somebody in front of the instrument.
        Action::Edit => control_ui::PANEL,
        Action::Copy => control_ui::SHELVE,
        Action::Write => control_ui::STORE,
        Action::Export => control_ui::EXPORT,
        Action::Update => control_ui::UPDATE,
    }
}

/// A mark, stencilled on the panel, dim while the press it is on is refused.
pub fn mark<'a>(badge: control_ui::Badge, usable: bool) -> Element<'a, Message> {
    Element::from(stencil(badge.screen(), move |theme: &Theme| {
        let material = materials(theme);
        if usable {
            material.metal
        } else {
            material.metal_low
        }
    }))
    .map(Message::Ui)
}

/// Says something the library wrote in the footer while the pointer is on it.
///
/// The twin of [`hinting`] for a sentence that is not known until the index is
/// read: a demo's own note, or the variant it was played with. It goes through
/// this window's own `Saying` rather than the view layer's `Hinted`, which
/// takes a `&'static str` — right for every press this window draws itself and
/// wrong for one drawn out of somebody else's file.
fn hinted<'a>(what: impl Into<Element<'a, Message>>, said: String) -> Element<'a, Message> {
    mouse_area(what.into())
        .on_enter(Message::Saying(Some(said)))
        .on_exit(Message::Saying(None))
        .into()
}

/// Says what a press does in the footer while the pointer is on it.
///
/// The same arrangement the window's own chrome is under: a press whose face is
/// a nine-dot mark has nowhere to carry a word, so the word goes where this
/// window already says what is under the pointer.
pub fn hinting<'a>(
    what: impl Into<Element<'a, Message>>,
    said: &'static str,
) -> Element<'a, Message> {
    mouse_area(what.into())
        .on_enter(Message::Ui(control_ui::Message::Hinted(Some(said))))
        .on_exit(Message::Ui(control_ui::Message::Hinted(None)))
        .into()
}

/// Nothing to look at, and a press anywhere on it puts the menu away.
///
/// Laid over the whole surface under the menu rather than around it, because
/// the way out of an open menu is *somewhere else* and somewhere else is
/// everywhere. It takes the press rather than passing it on, so the row
/// underneath is not also chosen by the press that closed the menu.
fn shade<'a>() -> Element<'a, Message> {
    mouse_area(space().width(Fill).height(Fill))
        .on_press(Message::CloseMenu)
        .on_right_press(Message::CloseMenu)
        .into()
}

/// The menu, brought inside the window.
///
/// It opens where the press was, and it is moved back up or left by however
/// much of it would otherwise be off the edge. A menu that opens seven lines
/// below the bottom of the window is a menu nobody can reach, and the last row
/// of a full table is exactly where somebody right-presses.
fn floating(app: &App, at: iced::Point, room: iced::Size) -> Element<'_, Message> {
    let left = at.x.min((room.width - MENU_WIDE).max(0.0)).max(0.0);
    let top = at.y.min((room.height - MENU_TALL).max(0.0)).max(0.0);
    container(menu(app))
        .padding(Padding::ZERO.left(left).top(top))
        .into()
}

/// What is on the table, in a few words.
///
/// It stood on a line of its own and now it shares one with the search field,
/// which is where it belongs: *how many there are* and *how you narrow them*
/// are one thought, and the count is what tells somebody their search did
/// anything.
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

/// The press that takes the newer version of every sound that has one.
///
/// Drawn only while something is stale, and saying how many, because a press
/// that would do nothing is a press somebody has to try to find out about. It
/// stands on the line that says what is on the table, which is where it
/// belongs: *forty-four sounds, eleven of them behind* is one fact about the
/// shelf said twice, and the press is the second half of it.
///
/// The lamp, and the same press a filter is, for the reason every colour on
/// this surface is somebody else's: this window has one way of drawing *look
/// here* and one way of drawing a press with a colour in it, and a button
/// invented for this would be a third thing to learn.
fn updating<'a>(stale: usize) -> Element<'a, Message> {
    if stale == 0 {
        return space().into();
    }
    hinting(
        filter(
            format!("Update all {stale}"),
            Some(LAMP),
            true,
            Message::UpdateEverything,
        ),
        "Replace every sound on the shelf that the library has published a newer version of.",
    )
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
        clearing(looking),
        space().width(Fill),
        standing(app),
        updating(app.stale()),
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

/// Every sound, wherever it is.
///
/// The shelf first and the library after it, because the nearer a sound is the
/// sooner somebody wants to see it: what is in the instrument, then what is on
/// the machine, then what could be.
fn rows(app: &App) -> Vec<Row<'_>> {
    let shelf = app.shelf();
    let place = match shelf.source() {
        Some(crate::shelf::Source::Instrument(_)) => Where::Instrument,
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
        // Laid out by the same answer the column is narrowed by, so a column
        // cannot sort on one thing and filter on another. `Where` is the
        // exception and is not really one: it sorts by *nearness* rather than
        // by the word, so what is in the instrument stands above what is only
        // published, which is not what `Instrument` before `Library`
        // alphabetically would give.
        let order = match sorting.by {
            By::Where => one.place().nearness().cmp(&other.place().nearness()),
            by => one
                .said_in(by)
                .to_lowercase()
                .cmp(&other.said_in(by).to_lowercase()),
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
    let picked = app.picked();
    let playing = app.hearing();
    let lines = standing
        .into_iter()
        .enumerate()
        .map(|(at, row)| line(&row, at, known, app.trying(), picked, playing));
    column![heading(app)]
        .push(
            scrollable(container(column(lines).spacing(1)).padding(Padding::ZERO.right(GUTTER)))
                .height(Fill),
        )
        .spacing(4)
        .into()
}

/// The heading: a row that sorts, and a row that narrows.
///
/// **Every column does both, and neither row is a surprise.** It was one row
/// of choosers standing in for the headings — a column's name vanished the
/// moment somebody filtered it, three columns could be narrowed and five could
/// not, and there was no way to tell which from looking. Two rows is what a
/// table with filters has always looked like: the names stay names, and under
/// each one is the control that narrows that column.
///
/// The name is a press. It lays the table out by its column and turns it round
/// when pressed again, with the caret inked only on the column it is under.
fn heading(app: &App) -> Element<'_, Message> {
    let names = row![container(space()).width(Length::Fixed(ICON_COLUMN))]
        .extend(By::ALL.map(|by| sorter(app, by, width_of(by))))
        .spacing(COLUMN_GAP)
        .align_y(Vertical::Center);
    let filters = row![container(space()).width(Length::Fixed(ICON_COLUMN))]
        .extend(By::ALL.map(|by| narrowing(app, by, width_of(by))))
        .spacing(COLUMN_GAP)
        .align_y(Vertical::Center);
    container(column![names, filters].spacing(3))
        .padding(Padding::from([0.0, 8.0]).right(GUTTER + 8.0 + KEEP))
        .into()
}

/// How much room a column takes.
///
/// One place, asked by both heading rows and by every cell, so a column cannot
/// be laid out three widths by three functions.
const fn width_of(by: By) -> f32 {
    match by {
        By::Where => PLACE,
        By::Bank => BANK,
        By::Number => NUMBER,
        By::Name => NAME,
        By::Maker => MAKER,
        By::Category => CATEGORY,
        By::Tags => TERMS,
        By::Demos => DEMOS,
        By::Version => VERSION,
        By::About => ABOUT,
    }
}

/// One column's name, as the press that lays the table out by it.
fn sorter(app: &App, by: By, width: f32) -> Element<'_, Message> {
    let sorting = app.sorting();
    let under = sorting.by == by;
    button(
        row![
            text(by.heading())
                .size(10)
                .style(move |theme: &Theme| text::Style {
                    color: Some(if under {
                        materials(theme).metal
                    } else {
                        materials(theme).metal_low
                    }),
                }),
            caret(under, sorting.down),
        ]
        .spacing(3)
        .align_y(Vertical::Center),
    )
    .width(Length::Fixed(width))
    .padding([1, 2])
    .style(control_ui::marked)
    .on_press(Message::SortSounds(by))
    .into()
}

/// Which way round the table is laid out, on the one column it is laid out by.
///
/// Nothing at all on the others. A caret drawn dim on all nine would be nine
/// marks saying *this column could be sorted*, which is a thing somebody works
/// out by pressing one rather than a thing worth eight marks of ink.
fn caret<'a>(under: bool, down: bool) -> Element<'a, Message> {
    if !under {
        return space().width(Length::Fixed(CARET)).into();
    }
    container(
        Element::from(stencil(
            if down {
                control_ui::DOWN.screen()
            } else {
                control_ui::UP.screen()
            },
            |theme: &Theme| materials(theme).metal,
        ))
        .map(Message::Ui),
    )
    .width(Length::Fixed(CARET))
    .into()
}

/// How much room the sort mark takes beside a heading.
const CARET: f32 = 12.0;

/// What narrows one column: a chooser where its values are a set, a field
/// where they are not.
///
/// Both write the same thing — the text a cell has to carry — so the table
/// answers one question however it was asked. A chooser writes a whole word
/// and a field writes whatever somebody typed, and `Pa` finding `Pad` and
/// `Pads` is what anybody expects from a box they typed two letters into.
fn narrowing(app: &App, by: By, width: f32) -> Element<'_, Message> {
    let now = app.looking().narrowed(by).unwrap_or_default().to_owned();
    if !by.picks() {
        return text_input("", &now)
            .on_input(move |said| Message::Narrow(by, said))
            .size(10)
            .padding([1, 4])
            .width(Length::Fixed(width))
            .into();
    }
    let mut options = vec![Narrowing::everything()];
    options.extend(offered(app, by).into_iter().map(Narrowing));
    let chosen = if now.is_empty() {
        Narrowing::everything()
    } else {
        Narrowing(now)
    };
    pick_list(options, Some(chosen), move |one| {
        Message::Narrow(by, one.wanted())
    })
    .text_size(10)
    .padding([1, 4])
    .width(Length::Fixed(width))
    .style(control_ui::selector)
    .menu_style(control_ui::shortlist)
    .into()
}

/// The values one column can be narrowed to, for the four that are a set.
///
/// Read off what is actually on the table rather than written down, so a
/// category nothing is filed under is not offered and a bank nothing sits in
/// is not either. A chooser with an option that empties the table is a chooser
/// that wastes a press.
fn offered(app: &App, by: By) -> Vec<String> {
    if by == By::Version {
        return vec!["behind".to_owned(), "current".to_owned()];
    }
    let mut seen: Vec<String> = rows(app)
        .iter()
        .map(|row| row.said_in(by))
        .filter(|said| !said.is_empty())
        .collect();
    seen.sort_unstable();
    seen.dedup();
    if by == By::Where {
        // Nearest first, which is the order the column sorts in and the order
        // the three are named in everywhere else.
        seen.sort_by_key(|said| {
            Where::ALL
                .iter()
                .position(|place| place.label() == said)
                .unwrap_or(usize::MAX)
        });
    }
    seen
}

/// One option in a column's chooser.
///
/// A named type rather than a bare `String`, because a picker needs something
/// to print and what *everything* prints is a word rather than the empty
/// string a cleared filter actually is.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Narrowing(String);

impl Narrowing {
    /// The option that stops narrowing the column.
    fn everything() -> Self {
        Self(String::new())
    }

    /// The text this option narrows to, which is nothing at all for `All`.
    fn wanted(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for Narrowing {
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            into.write_str("All")
        } else {
            into.write_str(&self.0)
        }
    }
}

/// The room what a sound is like takes, which is what is left.
const ABOUT: f32 = 132.0;

/// One sound, as a row.
fn line<'a>(
    row: &Row<'a>,
    at: usize,
    known: Option<&'a Held>,
    trying: Option<&str>,
    picked: Option<&Chosen>,
    playing: Option<&str>,
) -> Element<'a, Message> {
    let banded = at % 2 == 1;
    let chosen = row.chosen();
    let category = row.category();
    let colour = known
        .zip(category)
        .and_then(|(known, category)| known.category_colour(category));
    // One lamp for two facts that almost always agree: this is the row
    // somebody has in hand. Choosing a row plays it, so the sound that is
    // sounding and the sound the toolbar is about are the same row unless
    // something else has been loaded since.
    let lit = picked == Some(&chosen)
        || matches!(row, Row::Patch(patch) if trying == Some(patch.id.as_str()));
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
        container(hearing(row, playing)).width(Length::Fixed(DEMOS)),
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
    // A press chooses the row and plays it, whichever place it came from, and
    // nothing a press does is kept: a shelf program goes to the edit buffer and
    // so does a library one. A right-press chooses it as well and opens the
    // menu on it, so there is no way to act on a row somebody has not heard.
    let played = mouse_area(
        button(face)
            .width(Length::Fill)
            .padding([5, 8])
            .style(move |theme: &Theme, status| banded_like(theme, banded, lit, status))
            .on_press(Message::ChooseSound(chosen.clone())),
    )
    .on_right_press(Message::OpenMenu(chosen));
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

/// The bank a sound sits in, in the instrument's own lettering.
///
/// A bank letter and a program number are what the front panel's display shows
/// and nothing else in this window writes, so they are written in its
/// characters: the display's own five-by-seven cell, at the pitch every other
/// drawing here shares.
///
/// **Printed on the panel and not lit on glass.** They were glass — a lit field
/// with dark dots on it, the way the screen in the middle of the panel is — and
/// sixty lit rectangles down a list is sixty backgrounds competing with the
/// rows they are in. A display is a thing that *shows* something changing; a
/// slot in a list is a thing that is *written*, like the number stencilled on
/// the case of a rack unit, so it is stencilled.
fn glass<'a>(said: Option<String>, width: f32, dots: i32) -> Element<'a, Message> {
    let Some(said) = said else {
        return container(dim("\u{2014}".to_owned(), 11.0))
            .width(Length::Fixed(width))
            .align_x(Horizontal::Center)
            .into();
    };
    // Sized to the column rather than to the word, so every cell down the
    // column is the same width rather than a row of labels that grow and shrink
    // with what is on them. A `B` and a `128` are the same slot written
    // shorter, not a smaller instrument.
    let mut screen = Screen::new(dots, LINE);
    screen.centre(0, &said, control_ui::Size::Small);
    container(
        Element::from(stencil(screen, |theme: &Theme| materials(theme).metal)).map(Message::Ui),
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

/// The colour this window spends on attention.
///
/// The instrument's own lamp, and the one colour on this surface that is not
/// the library's: the library names colours for what a sound *is*, and these
/// two facts — *this one is in the instrument*, *this one has a newer version*
/// — are not about what a sound is. One colour for both, because they are the
/// same request: look here.
const LAMP: [u8; 3] = [0xff, 0xbe, 0x3d];

/// The colour a place is drawn in.
///
/// The lamp for what is in the instrument, and nothing at all for the other
/// two, which are ground rather than figure.
const fn place_colour(place: Where) -> Option<[u8; 3]> {
    match place {
        Where::Instrument => Some(LAMP),
        Where::Machine | Where::Library => None,
    }
}

/// Which version it is, and the press that takes the newer one.
///
/// A version somebody can do nothing about is printing, and a version they can
/// is a press: where the library has published something newer, this cell *is*
/// the update, lit in the lamp, in the same press every filter on this surface
/// is drawn as. A separate Update column would have been a column that is
/// blank on every row but the few.
fn version_cell<'a>(row: &Row<'a>) -> Element<'a, Message> {
    let Some((version, latest)) = row.version() else {
        return dim("\u{2014}".to_owned(), 11.0);
    };
    if latest {
        return text(format!("v{version}"))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(materials(theme).metal_low),
            })
            .into();
    }
    let Row::Held { at, .. } = row else {
        // A patch in the library is whatever the library says it is, so there
        // is nothing here to replace it with.
        return dim(format!("v{version}"), 11.0);
    };
    hinting(
        filter(
            format!("v{version} \u{2192} v{}", row.newest()),
            Some(LAMP),
            true,
            Message::UpdateSound(*at),
        ),
        "Replace this with the newer version the library has published.",
    )
}

/// The recordings of this sound, one press each.
///
/// **A press per take, not one press for the sound.** The library lets a maker
/// upload up to four — the patch played four ways, twenty seconds each — and
/// which one you are about to hear is the thing the column has to say. So the
/// default take is `\u{25b6}` and the rest carry their variant's first letters,
/// with the whole label in the footer while the pointer is on it.
///
/// The one that is playing is lit and stops when pressed again, which is what
/// a play button in a list has always done.
fn hearing<'a>(of_row: &Row<'a>, playing: Option<&str>) -> Element<'a, Message> {
    let demos = of_row.demos();
    if demos.is_empty() {
        return space().into();
    }
    row(demos.iter().map(|demo| {
        let going = playing == Some(demo.file.as_str());
        let said = match &demo.variant {
            None => "\u{25b6}".to_owned(),
            Some(variant) => variant.chars().take(VARIANT).collect(),
        };
        hinted(
            filter(
                said,
                going.then_some(LAMP),
                going,
                Message::Hear(demo.file.clone()),
            ),
            match (&demo.variant, &demo.about) {
                (_, Some(about)) => about.clone(),
                (Some(variant), None) => format!("Hear it played {variant}."),
                (None, None) => "Hear it.".to_owned(),
            },
        )
    }))
    .spacing(3)
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

/// The room the recordings take.
///
/// Four presses, which is the library's own limit: one `\u{25b6}` for the take
/// with no name and three labelled ones. Narrower than that and the last press
/// is clipped, which is a control somebody can see and cannot read.
const DEMOS: f32 = 112.0;

/// How much of a variant's name one press carries.
///
/// Three characters, because four presses have to fit a column and `mod-wheel`
/// is not going to. The whole of it is in the footer while the pointer is on
/// the press, which is where this window says what anything under the pointer
/// is.
const VARIANT: usize = 3;

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

/// A patch's icon, on a tile of its category's colour.
///
/// **The ground is what makes it legible.** A patch's picture is seven dots
/// square and one bit deep, which is all the library has to draw with, and at
/// this window's pitch that is seventeen points: drawn as bare dots on the
/// panel, at a colour lifted a little off the metal, forty-nine of them down a
/// list read as specks.
///
/// So it is a tile. The same tint the category chip beside it stands on, the
/// same corner as everything else in this window, and the dots inked in the
/// colour that is legible *against that tint* rather than against the panel —
/// which is the one change that makes a seven-dot drawing a picture rather
/// than a smudge. It is also what the icon is for: two rows of the same
/// category now share a colour somebody sees before they read either name.
///
/// The tint is the chip's own weight, because the tile and the chip stand in
/// the same row and mean the same thing. Two shades of one category would be
/// two things to read where there is one fact.
fn drawn<'a>(icon: Option<Icon>, colour: Option<[u8; 3]>) -> Element<'a, Message> {
    // Nothing at all where there is nothing to draw and no category to draw it
    // in. An empty tile is a box that says a picture failed to load, and a row
    // whose sound nobody has published simply has no picture.
    let Some(icon) = icon else {
        return space().width(Length::Fixed(ICON_COLUMN)).into();
    };
    let mut screen = Screen::new(ICON, ICON);
    screen.blit(&icon.pixels(), 0, 0);
    container(
        Element::from(stencil(screen, move |theme: &Theme| match colour {
            Some(rgb) => control_ui::legible(of(rgb), tinted(theme, colour, CHIP), theme),
            None => materials(theme).metal_low,
        }))
        .map(Message::Ui),
    )
    .padding(2)
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(tinted(theme, colour, CHIP))),
        border: border::rounded(3),
        ..container::Style::default()
    })
    .into()
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
    container(hinting(
        button(mark(control_ui::SHELVE, true))
            .padding([4, 6])
            .style(control_ui::marked)
            .on_press(Message::ShelvePatch(id)),
        Action::Copy.about(),
    ))
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

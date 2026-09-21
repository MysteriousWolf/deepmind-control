//! Sharing a sound, drawn: the icon grid, the four things to say, and the note.
//!
//! The third of the librarian's surfaces, and the only one that makes something
//! rather than reading something. What it shares is **the sound on the screen**
//! — whatever the panel is showing, from a program read off the instrument, off
//! the shelf, or edited by hand — because that is the sound somebody has just
//! finished making.
//!
//! # It asks for four things and works out the rest
//!
//! A maker, a sentence, a licence and at least one vocabulary term. Everything
//! else comes off the 242 program bytes: the name, the category, the effects,
//! the arpeggiator, the unison count. See [`crate::publish`] for why asking for
//! any of those would be worse than useless.
//!
//! # The icon editor is forty-nine presses
//!
//! Seven by seven, one bit each, which is the whole of what a patch's picture
//! is. There is no brush, no undo and no zoom, because a grid this size does
//! not need any: a wrong dot is one press to put right, and the whole picture
//! is smaller than a word.
//!
//! It starts from the category's own drawing rather than from nothing. A blank
//! grid is a bad place to begin a picture, the repository already draws twelve
//! good ones, and somebody who wants a bass that looks like a bass but not
//! *that* bass edits from it. Left blank, nothing is written and the patch
//! falls back to its category's.

use control_ui::{materials, stencil};
use deepmind_patches::{Axis, Category};
use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
use iced::{Background, Center, Element, Fill, Length, Theme, border};

use crate::app::{App, Message, Publishing};

/// How wide one dot of the icon editor is drawn.
///
/// Big enough to press without aiming, which is what decides it: the picture
/// itself is seven dots and would be a postage stamp at the pitch this window
/// draws a mark at.
const DOT: f32 = 26.0;

/// How wide the fields beside the grid stand.
const FIELD: f32 = 340.0;

/// How far an inked dot is carried from the metal towards its category's
/// colour. The table's own, so the two agree.
const INKED: f32 = 0.55;

/// The whole of it.
pub fn view(app: &App) -> Element<'_, Message> {
    let said = app.publishing();
    let category = app.patch().program().and_then(Category::of);
    column![standing(app, category)]
        .push(
            scrollable(
                column![
                    row![grid(app, said, category), fields(said)].spacing(24),
                    terms(app, said),
                    writing(app, said, category),
                ]
                .spacing(16),
            )
            .height(Fill),
        )
        .spacing(12)
        .into()
}

/// What is about to be shared, in one line.
fn standing(app: &App, category: Option<Category>) -> Element<'_, Message> {
    let said = match app.patch().program() {
        None => "Nothing is on the screen to share. Read a sound first.".to_owned(),
        Some(program) => {
            let name = program.name().as_str().trim().to_owned();
            match category {
                Some(category) => format!(
                    "Sharing {name} \u{b7} {} \u{b7} the category is the one stored in the sound",
                    category.label()
                ),
                None => format!(
                    "Sharing {name} \u{b7} it has no category yet, and the folder it goes in is \
                     the one it calls itself. Set one on the instrument."
                ),
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

/// The icon, and the two presses that start it over.
///
/// Inked in the colour its category is drawn in everywhere else on this
/// surface, so that what somebody is drawing looks like what will appear in the
/// table beside every other sound of that kind. A picture drawn in one colour
/// and shown in another is a picture somebody has to imagine twice.
fn grid<'a>(
    app: &'a App,
    said: &'a Publishing,
    category: Option<Category>,
) -> Element<'a, Message> {
    let colour = app
        .catalogue()
        .held()
        .zip(category)
        .and_then(|(held, category)| held.category_colour(category));
    let rows = (0..7).map(move |down| {
        row((0..7).map(move |across| dot(said, across, down, colour)))
            .spacing(2)
            .into()
    });
    column![
        text("PICTURE").size(10).style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }),
        column(rows).spacing(2),
        row![
            press("From the category").on_press(Message::PublishIcon(true)),
            press("Clear").on_press(Message::PublishIcon(false)),
        ]
        .spacing(6),
    ]
    .spacing(8)
    .into()
}

/// One dot of the grid.
fn dot(
    said: &Publishing,
    across: usize,
    down: usize,
    colour: Option<[u8; 3]>,
) -> Element<'_, Message> {
    let inked = said.inked(across, down);
    button(space())
        .width(Length::Fixed(DOT))
        .height(Length::Fixed(DOT))
        .style(move |theme: &Theme, status| {
            let material = materials(theme);
            let lit = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                // The instrument's own glass: dots are inked dark on a lit
                // field, which is what the display in the middle of the panel
                // does and what this picture will be drawn on everywhere else.
                background: Some(Background::Color(if inked {
                    // The same lift the table gives an icon: carried from the
                    // metal towards the category's colour rather than set to
                    // it, because a seven-dot picture *in* `#5DB56A` is a
                    // smudge.
                    match colour {
                        Some(rgb) => control_ui::mix(material.metal, crate::sounds::of(rgb), INKED),
                        None => material.metal,
                    }
                } else if lit {
                    control_ui::mix(material.recess, material.metal, 0.3)
                } else {
                    material.recess
                })),
                border: border::rounded(2).width(1.0).color(material.recess_edge),
                ..button::Style::default()
            }
        })
        .on_press(Message::PublishDot(across, down))
        .into()
}

/// The four things nothing can work out.
fn fields(said: &Publishing) -> Element<'_, Message> {
    column![
        field("MAKER", "a person or a handle", &said.author, |text| {
            Message::PublishAuthor(text)
        }),
        field(
            "ABOUT",
            "what it sounds like, and how to play it",
            &said.about,
            Message::PublishAbout
        ),
        field(
            "LICENCE",
            "CC0-1.0, CC-BY-4.0, All rights reserved\u{2026}",
            &said.licence,
            Message::PublishLicence
        ),
        field(
            "FAMILY",
            "a collection this belongs to, if any",
            &said.collection,
            Message::PublishCollection
        ),
    ]
    .spacing(10)
    .width(Length::Fixed(FIELD))
    .into()
}

/// One labelled field.
fn field<'a>(
    label: &'static str,
    hint: &'static str,
    held: &str,
    said: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(10).style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }),
        text_input(hint, held)
            .on_input(said)
            .size(13)
            .padding([5, 8]),
    ]
    .spacing(3)
    .into()
}

/// Every term the library allows, grouped by the axis it belongs to.
///
/// Read off the catalogue's own taxonomy rather than listed here, so a
/// repository that adds a term gets it offered with nothing to change in this
/// window. Nothing at all while no catalogue is open, because a list of terms
/// this window invented is exactly what the validator would reject.
fn terms<'a>(app: &'a App, said: &'a Publishing) -> Element<'a, Message> {
    let Some(held) = app.catalogue().held() else {
        return container(
            text(
                "Open the shared patches to choose what this sound is. The vocabularies come \
                 from the library rather than from here.",
            )
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(materials(theme).metal_low),
            }),
        )
        .into();
    };
    let axes = [Axis::Genre, Axis::Mood, Axis::Timbre, Axis::Role]
        .into_iter()
        .map(|axis| {
            let key = crate::catalogue::axis_key(axis);
            let colour = held.axis_colour(axis);
            let chips = held.index().taxonomy.axis(axis).keys().map(move |term| {
                let carried = said.carries(key, term);
                chip(term.clone(), key, colour, carried)
            });
            column![
                text(key.to_uppercase())
                    .size(10)
                    .style(|theme: &Theme| text::Style {
                        color: Some(materials(theme).metal_low),
                    }),
                row(chips).spacing(4).wrap(),
            ]
            .spacing(4)
            .into()
        });
    column(axes).spacing(10).into()
}

/// One term, on or off.
///
/// The same press the table's own filters are, in the same colour: choosing a
/// term here and filtering by it there are one gesture, and a window that drew
/// them differently would be asking somebody to learn it twice.
fn chip<'a>(
    term: String,
    axis: &'static str,
    colour: Option<[u8; 3]>,
    carried: bool,
) -> Element<'a, Message> {
    let said = term.clone();
    crate::sounds::filter(
        term,
        colour,
        carried,
        Message::PublishTerm(axis.to_owned(), said),
    )
}

/// The press that writes the pair, and what is still wanted.
fn writing<'a>(
    app: &'a App,
    said: &'a Publishing,
    category: Option<Category>,
) -> Element<'a, Message> {
    let missing = said.missing();
    let ready = missing.is_empty() && category.is_some() && app.patch().program().is_some();
    let note = if let Some(where_to) = &said.wrote {
        format!(
            "Written to {where_to}. The note beside it says what to do next: fork, copy the \
             folder over a clone, commit, open a pull request."
        )
    } else if missing.is_empty() {
        "Choose a folder. What is written into it is laid out the way the repository is, so it \
         copies straight over a clone."
            .to_owned()
    } else {
        format!("Still wanted: {}.", missing.join(", "))
    };
    column![
        row![
            press("Write the files\u{2026}").on_press_maybe(ready.then_some(Message::PublishWrite)),
            container(drawn()).width(Length::Fixed(28.0)),
        ]
        .spacing(10)
        .align_y(Center),
        text(note).size(12).style(|theme: &Theme| text::Style {
            color: Some(materials(theme).metal_low),
        }),
    ]
    .spacing(8)
    .into()
}

/// The mark on the press that writes: a sound going out of this window.
fn drawn<'a>() -> Element<'a, Message> {
    Element::from(stencil(control_ui::UP.screen(), |theme: &Theme| {
        materials(theme).metal_low
    }))
    .map(Message::Ui)
}

/// A button that is not a parameter, in the instrument's own materials.
fn press(label: &str) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(label).size(13))
        .padding([5, 12])
        .style(control_ui::chrome)
}

#[cfg(test)]
mod tests {
    use crate::app::Publishing;

    #[test]
    fn a_dot_is_toggled_where_it_was_pressed() {
        // Bit 0 is leftmost, which is the library's own packing. Getting this
        // backwards would draw every icon mirrored, and a seven-dot picture is
        // symmetrical often enough that nobody would notice for a while.
        let mut said = Publishing::default();
        said.icon[2] |= 1 << 5;
        assert!(said.inked(5, 2));
        assert!(!said.inked(2, 5), "across and down are not the same axis");
        assert!(!said.inked(7, 2), "nothing outside the grid is inked");
    }
}

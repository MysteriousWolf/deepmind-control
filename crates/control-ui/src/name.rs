//! The program's name: seventeen parameters, and one word.
//!
//! The instrument stores a name as one parameter per character, which is the
//! truth about the wire and a lie about what a person is editing. Seventeen
//! faders numbered 223 to 239, each sweeping 0 to 127, is a panel nobody can
//! name a sound on: the value of `Program Name Char 4` is `115`, and what that
//! means is that the fourth letter is an `s`.
//!
//! So this is the first panel laid out by hand, and it is the only slot in the
//! editor whose control covers more than one parameter. It is the instrument's
//! own display: sixteen characters of it, in the same mono face the readouts
//! use, cut into the plate as a recess like every other control here. What a
//! keystroke costs on the wire is unchanged — a character is still one NRPN to
//! one parameter — and [`Patch::rename`] is where a word becomes those edits.
//!
//! The field is seventeen parameters wide and sixteen characters long, because
//! the seventeenth byte is the terminator and belongs to the field rather than
//! to the name. The library says which bytes those are; nothing here counts
//! them.

use deepmind_midi::param::ParamId;
use deepmind_midi::program::{NAME_PARAMETERS, ProgramName};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{column, container, text, text_input};

use crate::fader;
use crate::panel::{Message, SLOT};
use crate::style::materials;
use crate::{Confidence, Element, Patch, tint};

/// How wide the field is, in slots.
///
/// Sixteen mono characters do not fit in the 88 points a fader needs, and a
/// name clipped to `Bass Swee` is a name that lies about the sound in the
/// instrument. Two slots is the whole of it with room for the recess.
const SLOTS: f32 = 2.0;

/// The parameters the name is stored in, in the order they are printed.
///
/// The library's own list. This crate walked the offsets from `NAME_OFFSET` for
/// `NAME_LEN` bytes until `deepmind-midi` 26.2 published [`NAME_PARAMETERS`],
/// which is the same seventeen worked out where the table lives, so a library
/// that moves the field moves this with it.
#[must_use]
pub fn characters() -> &'static [ParamId] {
    &NAME_PARAMETERS
}

/// Returns whether a parameter is one of the characters of the name.
///
/// The panel asks this of every parameter it is about to draw a fader for, so
/// that the seventeen it would draw become the one field below instead.
#[must_use]
pub fn holds(parameter: ParamId) -> bool {
    characters().contains(&parameter)
}

/// Returns whether a parameter is the first character of the name.
///
/// Which is where the field is drawn, so that it takes the place in the rack
/// the name already had rather than being moved to the front of it. The
/// arrangement stays the instrument's own.
#[must_use]
pub fn begins(parameter: ParamId) -> bool {
    characters().first() == Some(&parameter)
}

/// Draws the name as the instrument's display shows it.
///
/// A slot like any other — the address above, the control, the reading, the
/// title — except that the address is a run of them and the control is a word.
pub(crate) fn field<'a, Renderer>(patch: &Patch) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim_across(characters().iter().copied());
    let name = patch.name().unwrap_or(ProgramName::EMPTY);
    let known = patch.is_known();
    column![
        text(addresses())
            .size(10)
            .font(Font::MONOSPACE)
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, Confidence::Unknown)),
            }),
        display(name, claim, known),
        text(reading(name, known))
            .size(13)
            .font(Font::MONOSPACE)
            .style(move |theme: &Theme| text::Style {
                color: Some(tint(theme, claim)),
            }),
        container(text("Name").size(11).center())
            .height(Length::Fixed(crate::panel::NAME))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(5)
    .width(Length::Fixed(SLOT * SLOTS))
    .align_x(Horizontal::Center)
    .into()
}

/// The field itself: sixteen characters, cut into the plate.
fn display<'a, Renderer>(name: ProgramName, claim: Confidence, known: bool) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // A character the name cannot hold is refused by leaving the name where it
    // is: the field draws what it is given, so a seventeenth letter or a `è`
    // never appears rather than appearing and then vanishing.
    let typed = move |text: String| Message::Rename(ProgramName::new(&text).unwrap_or(name));
    container(
        text_input("\u{2014}", name.as_str())
            .on_input_maybe(known.then_some(typed))
            .font(Font::MONOSPACE)
            .size(14)
            .padding([4, 6])
            .width(Length::Fill)
            .style(move |theme: &Theme, _status| recess(theme, claim)),
    )
    .height(Length::Fixed(fader::HEIGHT))
    .align_y(Vertical::Center)
    .into()
}

/// The addresses the name occupies, as the run they are.
///
/// One number per slot everywhere else in the rack; seventeen of them here,
/// which is a range. Taken off the parameters rather than printed from
/// [`NAME_OFFSET`], so it says what is actually drawn.
fn addresses() -> String {
    let first = characters()
        .first()
        .map_or(0, |parameter| parameter.offset());
    let last = characters()
        .last()
        .map_or(0, |parameter| parameter.offset());
    format!("{first}\u{2013}{last}")
}

/// The reading under the field: how much of the name is used.
///
/// The control already carries the value, the way a lamp and a legend do, so
/// what goes here is the thing the control cannot show: how many of the sixteen
/// characters are left before the instrument stops accepting them.
fn reading(name: ProgramName, known: bool) -> String {
    if known {
        format!("{}/{}", name.len(), ProgramName::MAX_CHARS)
    } else {
        "\u{2014}".to_owned()
    }
}

/// The style the field is drawn in: the same recess a fader's track is cut as.
///
/// The claim is in the ink rather than in a fill, because a field of text has
/// no moving part to fill: an empty display and a claimed one differ by what is
/// written in them, which is the one case where the letters are the fill.
fn recess(theme: &Theme, claim: Confidence) -> text_input::Style {
    let material = materials(theme);
    let colour = tint(theme, claim);
    text_input::Style {
        background: Background::Color(material.recess),
        border: Border {
            color: material.recess_edge,
            width: 1.0,
            radius: 2.into(),
        },
        icon: material.metal_low,
        placeholder: material.metal_low,
        value: colour,
        selection: material.recess_edge,
    }
}

#[cfg(test)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};
    use deepmind_midi::program::{NAME_LEN, ProgramName};

    use super::{begins, characters, holds};

    #[test]
    fn the_name_is_the_field_the_library_describes() {
        assert_eq!(characters().len(), NAME_LEN);
        assert!(
            characters()
                .iter()
                .all(|parameter| parameter.group() == Group::Program),
            "a character of the name is not in the program's own section"
        );
    }

    #[test]
    fn the_field_holds_one_more_byte_than_the_name_has_characters() {
        // The seventeenth byte is the terminator, and belongs to the field
        // rather than to the name.
        assert_eq!(characters().len(), ProgramName::MAX_CHARS + 1);
    }

    #[test]
    fn the_characters_are_in_the_order_they_are_printed() {
        let offsets: Vec<u8> = characters()
            .iter()
            .map(|parameter| parameter.offset())
            .collect();
        let mut sorted = offsets.clone();
        sorted.sort_unstable();

        assert_eq!(offsets, sorted);
    }

    #[test]
    fn only_the_name_is_held_and_only_its_first_character_begins_it() {
        assert!(holds(ParamId::ProgramNameChar1));
        assert!(holds(ParamId::ProgramNameChar17));
        assert!(!holds(ParamId::ProgramCategory));
        assert!(!holds(ParamId::VcfFrequency));

        assert!(begins(ParamId::ProgramNameChar1));
        assert!(!begins(ParamId::ProgramNameChar2));
        assert!(!begins(ParamId::VcfFrequency));
    }
}

//! The section bar: which panel is in front of somebody, and what backs the
//! rest of the sound while they are looking away from it.
//!
//! Two hundred and forty-two parameters are fourteen panels, and the instrument
//! does not put them on one surface either: a player presses a section and the
//! display becomes that section. So does this, and the bar is the press.
//!
//! Each tab carries its own section's claim as the same dot the legend explains,
//! which is what keeps one panel at a time from hiding the thing this editor
//! exists to say. A bar of green dots with one copper one among them is "the
//! sound is the synthesizer's, except the part I moved", read without opening
//! anything.

use deepmind_midi::param::Group;
use iced_core::alignment::Vertical;
use iced_core::{Background, Font, Theme, border, text::Renderer as TextRenderer};
use iced_widget::{button, row, text};

use crate::panel::{Message, dot};
use crate::style::materials;
use crate::{Element, Patch};

/// The sections, in the order the instrument itself lays them out.
///
/// The library's own, and not `Group::ALL`, which is alphabetical and puts the
/// effects third and the oscillators eighth. This crate read it off the offsets
/// until `deepmind-midi` 26.2 published [`Group::ORDER`], which reads it off the
/// same offsets on the side of the line that holds them: a group a later table
/// adds arrives in its right place with nothing here to edit.
#[must_use]
pub fn sections() -> &'static [Group] {
    Group::ORDER
}

/// The panel a window opens on, which is the first one the instrument lays out.
///
/// The fallback is unreachable: the library is a table of 242 parameters and
/// every one of them is in a group, so the order above has a first element. It
/// costs one line, and a window that opened on no panel at all would be worse
/// than one that opened on the wrong panel.
#[must_use]
pub fn first_section() -> Group {
    sections().first().copied().unwrap_or(Group::Vcf)
}

/// Draws the bar, with `showing` pressed in.
#[must_use]
pub fn section_bar<'a, Renderer>(patch: &Patch, showing: Group) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let tabs = sections().iter().copied().map(|group| {
        let pressed = group == showing;
        button(
            row![dot(patch.claim_of(group)), text(group.name()).size(12)]
                .spacing(6)
                .align_y(Vertical::Center),
        )
        .padding([4, 8])
        .style(move |theme: &Theme, _status| tab(theme, pressed))
        .on_press(Message::Show(group))
        .into()
    });
    row(tabs).spacing(6).wrap().into()
}

/// The style a tab is drawn in: the pressed one is the plate its rack is on.
///
/// Not a colour of its own. The section in front of somebody is the face plate
/// below it continued upwards and lit along its edge, and the rest are the panel
/// they are cut into, which is the same trick the instrument plays with a lit
/// section button.
fn tab(theme: &Theme, pressed: bool) -> button::Style {
    let material = materials(theme);
    button::Style {
        background: Some(Background::Color(if pressed {
            material.plate
        } else {
            material.panel
        })),
        text_color: if pressed {
            material.metal_high
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
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::Group;

    use super::{first_section, sections};

    #[test]
    fn the_bar_reaches_every_parameter() {
        for group in Group::ALL {
            assert!(
                sections().contains(group),
                "{group} is not on the bar, so its parameters cannot be reached"
            );
        }
        assert_eq!(
            sections().len(),
            Group::ALL.len(),
            "a section appears on the bar twice"
        );
    }

    #[test]
    fn the_bar_is_in_the_instrument_s_own_order() {
        let starts: Vec<u8> = sections()
            .iter()
            .map(|group| {
                group
                    .parameters()
                    .next()
                    .expect("a group the table produced has a parameter in it")
                    .offset()
            })
            .collect();
        let mut sorted = starts.clone();
        sorted.sort_unstable();

        assert_eq!(
            starts, sorted,
            "the bar is not in the order the instrument lays its parameters out"
        );
    }

    #[test]
    fn a_window_opens_on_the_first_panel() {
        assert_eq!(Some(first_section()), sections().first().copied());
    }
}

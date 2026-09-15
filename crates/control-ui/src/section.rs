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

/// The sections, in the order the sound passes through them.
///
/// [`Group::ALL`] is alphabetical, which is the order a generated table comes
/// out in and the order nothing on the instrument is in: it puts the effects
/// third and the oscillators eighth. This is the signal path, starting with what
/// the program is and ending with what happens to it on the way out, so that
/// moving left to right along the bar is moving forward through the voice.
pub const SECTIONS: [Group; 14] = [
    Group::Program,
    Group::Voicing,
    Group::Oscillators,
    Group::Vcf,
    Group::VcfEnvelope,
    Group::Vca,
    Group::VcaEnvelope,
    Group::Lfo1,
    Group::Lfo2,
    Group::ModEnvelope,
    Group::ModMatrix,
    Group::Arpeggiator,
    Group::ControlSequencer,
    Group::Effects,
];

/// Draws the bar, with `showing` pressed in.
#[must_use]
pub fn sections<'a, Renderer>(patch: &Patch, showing: Group) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let tabs = SECTIONS.into_iter().map(|group| {
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
mod tests {
    use deepmind_midi::param::Group;

    use super::SECTIONS;

    #[test]
    fn the_bar_reaches_every_parameter() {
        for group in Group::ALL {
            assert!(
                SECTIONS.contains(group),
                "{group} is not on the bar, so its parameters cannot be reached"
            );
        }
        assert_eq!(
            SECTIONS.len(),
            Group::ALL.len(),
            "a section appears on the bar twice, or the library grew one"
        );
    }
}

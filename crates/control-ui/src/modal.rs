//! A sheet laid over the window, and the three ways back out from under it.
//!
//! The instrument has one screen and a row of buttons that change what is on
//! it. Press `EDIT` on the filter and the display becomes the filter; press it
//! again, or press another section, and it becomes that. Nothing about the
//! front of the instrument moves while that happens: the faders are where they
//! were, the plate is where it was, and what changed is what is *over* it.
//!
//! That is a modal, and this window had a tab bar instead. Fourteen tabs above
//! a rack is a filing cabinet: it says the sections are peers of one another
//! and of the panel, when what they actually are is the detail behind one press
//! on a panel that stays where it is. So the rack comes up over the panel it was
//! opened from, the panel stays visible underneath in the shadow the sheet
//! casts, and putting it away puts somebody back exactly where they were.
//!
//! # The three ways out
//!
//! A modal that can only be closed one way is a modal somebody is stuck behind,
//! so there are three, and every one of them produces the same
//! [`Message::Close`]:
//!
//! | | |
//! | --- | --- |
//! | The panel around the sheet | [`modal`] hands the shade a press, so anywhere outside the sheet is the way out. It is why the sheet takes *almost* the window and never all of it: a modal with no margin has no outside to press |
//! | The mark on the sheet's own bar | [`SHUT`](crate::SHUT), at the end of the heading. The only one of the three that is drawn, because the other two are gestures and a gesture nobody was told about is not a way out |
//! | The escape key | Not here. A key is an event before it is a press, and this crate has no runtime to hear one in: the application listens for it and sends the same message. See `control/src/window.rs` |
//!
//! # What it is made of
//!
//! Nothing new. The sheet is [`lifted`](crate::lifted), which is the window's
//! own panel raised and lit along its edge, so that the rack on it sits on
//! exactly what it sat on before; its heading is the bar a plate on the front
//! panel prints its name in; the mark on it is stencilled the way every other
//! mark in the chrome is; and what is behind it is [`unlit`](crate::unlit),
//! which is the panel's own darkest material rather than a grey invented for a
//! dialog. A modal here is a piece of the instrument brought forward, not a
//! window from somewhere else.
//!
//! # The title is the band's cap, printed flat
//!
//! A sheet is named the way the row of tabs above the window names a section:
//! its mark and its word, in the display's own dots. It was the section's name
//! in the machine's sans with a coloured dot in front of it — the one piece of
//! this window that looked like it had been lifted out of a dialog box — and
//! the dot was a claim the display underneath already carries.
//!
//! [`page`] has no title at all, because the cap that opened it is lit directly
//! above it and a name printed twice one line apart is a name printed twice.

use deepmind_midi::param::Group;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{button, column, container, mouse_area, opaque, row, scrollable, space, stack};

use crate::Element;
use crate::badge::SHUT;
use crate::lcd::stencil;
use crate::panel::Message;
use crate::style::{lifted, marked, materials, mix, unlit};

/// How much window is left showing around a sheet.
///
/// Almost none of it, because a sheet that took half the window would be a
/// dialog and this is the section somebody asked for: the rack of the effects
/// is four engines wide and the matrix is eight routings deep, and either of
/// them in a box in the middle of the screen is the tab bar's problem again in
/// a smaller box.
///
/// Enough of it, because the margin is one of the three ways out. A hand that
/// misses the sheet has to land on something, and a border two points wide is
/// a target nobody hits on purpose. This is a band of panel wide enough to be
/// aimed at down any edge of the window.
const MARGIN: f32 = 24.0;

/// How much sheet there is around what is on it.
const WITHIN: f32 = 10.0;

/// How wide the bar a sheet scrolls on is, which is the toolkit's own width.
const BAR: f32 = 10.0;

/// How much sheet there is between what is on it and that bar.
///
/// The bar takes its own room rather than floating over the last column of the
/// rack, which is what the window's own scroll bars already do: a bar laid over
/// a rack is a bar over the last fader of every row.
const BESIDE: f32 = 6.0;

/// How much window a sheet spends on itself before anything is on it.
///
/// The margin either side, the sheet's own padding either side, and the bar it
/// scrolls on with the room beside it. Published because the window has to open
/// wide enough for what goes on a sheet: the effects are the widest thing this
/// editor draws, and a window sized for them standing on the panel is a window
/// too narrow for them standing on a sheet by exactly this much. A rack drawn
/// past the edge of its own glass is not readable at all.
#[must_use]
pub fn margins() -> f32 {
    MARGIN * 2.0 + WITHIN * 2.0 + BAR + BESIDE
}

/// How tall the bar across the top of a sheet stands.
///
/// The one a plate on the front panel prints its name in, which is where
/// somebody has just come from: the section's name was printed across the top
/// of the plate whose `EDIT` they pressed, and it is printed across the top of
/// what that press opened.
const HEAD: f32 = 26.0;

/// How far the mark on a press is carried from the panel towards the metal.
///
/// The chrome's own, so that the mark that shuts a sheet is stencilled at the
/// same weight as the marks along the header and the foot of the window.
const MARKED: f32 = 0.82;

/// How much panel there is around that mark.
const BESIDE_MARK: f32 = 5.0;

/// Lays `over` across `under`, with the window behind it in shadow.
///
/// The whole of the modal system: a stack of two, a shade between them that is
/// also the press that puts the top one away, and the sheet standing in the
/// middle of what is left with a band of window showing round it.
///
/// Every press that lands on the sheet is the sheet's. The shade is what is
/// left, so pressing the panel around it sends `dismiss`, and nothing under the
/// shade can be reached while something is over it: that is the whole
/// difference between a modal and a panel that happens to be on top.
///
/// In whatever message the two sides are already in rather than in this crate's
/// own. It is the one view here that is handed something the application has
/// drawn: what goes under a sheet is the whole window, which is a surface, a
/// header and a footer this crate knows nothing about. So the message is the
/// caller's and so is `dismiss`, which is what the application does with the
/// [`Message::Close`] the sheet's own mark sends.
#[must_use]
pub fn modal<'a, Said, Renderer>(
    under: impl Into<iced_core::Element<'a, Said, Theme, Renderer>>,
    over: impl Into<iced_core::Element<'a, Said, Theme, Renderer>>,
    dismiss: Said,
) -> iced_core::Element<'a, Said, Theme, Renderer>
where
    Said: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    stack![
        under.into(),
        // One layer rather than two: the shade is the thing the sheet stands
        // in, so the press that closes belongs to the shade and the sheet
        // swallows the presses that land on itself. A sheet on a layer of its
        // own would either be a layer that fills the window, which is a press
        // that closes nothing anywhere, or a layer sized to the sheet, which is
        // a layout this crate would have to measure for itself.
        opaque(
            mouse_area(
                container(opaque(over.into()))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(MARGIN)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .style(unlit)
            )
            .on_press(dismiss)
        )
    ]
    .into()
}

/// The frame a modal's contents are drawn on: the name, the way out, and `body`.
///
/// The body scrolls, and the heading does not. A section is as tall as it is —
/// the effects are four engines and a chain — and a sheet whose name scrolled
/// away would be a sheet with no way out at the moment somebody went looking
/// for one.
///
/// **As wide as the window and as tall as what is on it.** The width is the
/// window's because a rack is laid out across one: a rack given less wraps, and
/// a wrapped rack is a different arrangement of the instrument rather than the
/// same one in a smaller box. The height is the rack's, up to the whole window,
/// because the alternative is a plate with an acre of nothing at the bottom of
/// it whenever the section is `VCA`, which is one fader. What that means in
/// practice is that the sections worth opening a sheet for — the effects, the
/// matrix, the sequencer — do take almost the whole window, and the short ones
/// are a band across the middle of it.
#[must_use]
pub fn sheet<'a, Renderer>(
    section: Group,
    body: impl Into<Element<'a, Renderer>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let scroll = scroll(body);
    container(
        column![
            heading(section),
            // Shrink, so the sheet is as tall as its rack. A scrollable given
            // no height of its own takes its content's, up to the room it was
            // offered, and scrolls past that: which is the section that fits
            // standing at its own height and the section that does not filling
            // the window.
            scroll.height(Length::Shrink),
        ]
        .spacing(WITHIN),
    )
    .width(Length::Fill)
    .height(Length::Shrink)
    .padding(WITHIN)
    .style(lifted)
    .into()
}

/// The same rack, standing on the window rather than over it.
///
/// What a section reached from the [band of ways in](crate::ways) is drawn on.
/// Those four are not behind an `EDIT` on a plate — no plate carries them —
/// so they are not the detail behind a press on the panel, they are places of
/// their own, and the band is a row of tabs. A tab that opened a sheet would be
/// a tab that covers the surface it is part of.
///
/// **With no heading at all**, which is the whole of the difference. A sheet
/// needs one because it is over something and has to say what it is and offer
/// the way out; a page has the lit cap of the band directly above it saying
/// both. A title bar under a lit tab is the section's name printed twice, one
/// line apart, and it was drawn small and grey besides — the one piece of
/// chrome in this window that looked like it came from a different application.
///
/// It is also not lifted, padded or styled. A page has nothing under it, so it
/// is a scroll that fills, which is exactly what the front panel is and for the
/// same reason: a scrollable has to be given a bounded height by whatever holds
/// it, and a `Fill` scrollable inside a `Fill` column inside a padded container
/// is not given one. The effects rack ran off the bottom of the window and drew
/// its last two engine cases over the footer until this was a scroll and
/// nothing else.
pub fn page<'a, Renderer>(body: impl Into<Element<'a, Renderer>>) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    scroll(body).height(Length::Fill).into()
}

/// The scroll both are built on.
fn scroll<'a, Renderer>(
    body: impl Into<Element<'a, Renderer>>,
) -> iced_widget::Scrollable<'a, Message, Theme, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    scrollable(body.into()).direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(BAR).spacing(BESIDE),
    ))
}

/// The bar across the top of a sheet: what is on it, and the way out.
///
/// **The cap of the band, printed flat.** The band above the window draws a
/// section as its mark and its name in the display's own dots, and a sheet is
/// the same section opened a different way, so it is named in the same
/// lettering: the plaque is one drawing and both use it. What it
/// replaced was the section's name set in the machine's sans at fourteen
/// points with a coloured dot in front of it, which was the only text in this
/// window drawn like a dialog box, and the dot was a claim nothing else on the
/// sheet needed told twice — the display under it is drawn in the claim it is
/// under, the way every display here is.
///
/// **On the sheet and not in a box on it.** It was a recessed band with a
/// border, which is what a plate on the front panel prints its name in — and a
/// plate needs one because it stands in a row of ten plates on a dark panel.
/// A sheet is already a panel lifted off the window with its own lit edge, and
/// the rack under this is already the thing the eye lands on. A bordered band
/// inside a bordered sheet above a bordered rack is three frames deep to say
/// one word. The title is the words and the mark, standing on the sheet.
///
/// At the right-hand end is the one press. A heading with nothing at that end
/// would be a sheet whose only ways out are two gestures.
fn heading<'a, Renderer>(section: Group) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        row![
            stencil(crate::home::plaque_of(section), plate),
            space::horizontal(),
            shut(),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(HEAD))
    .into()
}

/// The ink a plaque is printed in on that band.
///
/// The chrome's own mark ink, which is what the mark beside it at the other end
/// of the bar is drawn in: one bar, one weight of printing.
fn plate(theme: &Theme) -> iced_core::Color {
    let material = materials(theme);
    mix(material.panel, material.metal, MARKED)
}

/// The press that puts the sheet away.
///
/// The chrome's own press, built the way the header and the footer build
/// theirs: no rim and no fill until a hand comes near it, a mark stencilled on
/// the panel rather than a word in a box, and the word itself in the footer
/// while the pointer is on it. A sheet is not a dialog and its close is not an
/// `OK` button.
fn shut<'a, Renderer>() -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    mouse_area(
        button(Element::from(stencil(SHUT.screen(), |theme: &Theme| {
            let material = materials(theme);
            mix(material.panel, material.metal, MARKED)
        })))
        .padding(BESIDE_MARK)
        .style(marked)
        .on_press(Message::Close),
    )
    .on_enter(Message::Hinted(Some(WAY_OUT)))
    .on_exit(Message::Hinted(None))
    .into()
}

/// What that press says about itself in the footer.
///
/// All three ways out in one sentence, because the press is where somebody
/// looking for a way out looks, and the other two are the ones they would never
/// find on their own.
const WAY_OUT: &str =
    "Close this panel. Escape, or a press on the window around it, does the same.";

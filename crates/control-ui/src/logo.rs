//! The project's own mark, drawn rather than drawn from a file.
//!
//! `docs/logo.svg` is a `DeepMind`'s front: a dark panel between two wooden end
//! cheeks, three faders cut into it with metal caps riding them, and the
//! wordmark's own horizontal slices laid across the caps. `docs/banner.svg` is
//! the same case made wide, with the project's name beside the faders in the
//! metal of one of those caps.
//!
//! The window had none of that. It had the name set in the mark's face and
//! nothing else, so the one place somebody sees this project before they see
//! the instrument looked like a heading rather than like the mark on the file
//! they downloaded. This is that mark, at the size a window header has room
//! for, drawn out of the same materials every other surface in this window is
//! drawn out of.
//!
//! # Why it is drawn and not loaded
//!
//! An `SVG` would need a renderer feature, a file beside the binary or a blob
//! inside it, and a second copy of the geometry either way. What it would buy
//! is the one thing this window does not want: a mark that ignores the theme.
//! Every surface here asks the theme what it is made of — that is what makes
//! the displays turn over on one press — and a mark that did not would be the
//! only part of the window that could not.
//!
//! # The slices
//!
//! The wordmark this project is named after cuts horizontal lines through
//! everything it touches, spaced further apart and thicker as they fall. On the
//! mark they cross the fader caps, and the cap heights in the file are chosen so
//! that a slice falls through the middle of a cap and never at its edge.
//!
//! They survive here and they do not survive on the name beside it. Measured
//! against the file's own 128-unit box, the eight run from 1 unit to 3.6, which
//! at a mark of forty-odd points is a third of a point to one and a third — thin
//! at the top of the run and solid at the bottom, which is what they are on the
//! mark itself. The same proportions against a 22-point word are every slice
//! under a point, which is not a slice: it is a smudge, and the mark is not
//! improved by being approximated. So the name is the metal without them, which
//! is what [`crate::style::wordmark`] was already for.

use iced_core::gradient::Linear;
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Border, Color, Element, Gradient, Length, Radians, Rectangle, Size, Theme, Widget,
    mouse, renderer,
};

use crate::style::{Materials, materials};

/// The box the mark is drawn in, which is the file's own.
///
/// Every number below is read off `docs/logo.svg` at this scale and divided by
/// it, so the mark in the window and the mark in the file are one drawing at
/// two sizes rather than two drawings that resemble each other.
const BOX: f32 = 128.0;

/// How far in from each side the panel starts.
const CHEEK: f32 = 13.0;

/// How wide the dark seam where the panel meets the wood is.
const SEAM: f32 = 1.4;

/// How wide the lit lip inside that seam is.
const LIP: f32 = 0.8;

/// The corner the case is rounded to.
const CORNER: f32 = 24.0;

/// Where the three fader slots stand, across the box.
const SLOTS: [f32; 3] = [29.0, 60.0, 91.0];

/// How far down a slot starts, how long it runs, and how wide it is.
const SLOT: (f32, f32, f32) = (30.0, 70.0, 8.0);

/// Where each cap sits down the box, in the order the slots are in.
///
/// Not a rhythm anybody chose: each one is where a slice crosses its slot, so
/// that the slice falls through the middle of a cap and never at its edge.
const CAPS: [f32; 3] = [81.6, 40.7, 68.2];

/// How wide a cap is, and how deep.
const CAP: (f32, f32) = (26.0, 8.6);

/// The slices, as the top of each and how thick it is.
///
/// Spaced further apart and thicker as they fall, which is the wordmark this
/// project is named after.
const SLICES: [(f32, f32); 8] = [
    (52.0, 1.0),
    (58.5, 1.3),
    (65.0, 1.6),
    (71.5, 2.0),
    (78.0, 2.4),
    (84.5, 2.8),
    (91.0, 3.2),
    (97.5, 3.6),
];

/// How much of the case's own width the mark is drawn at.
///
/// The one number here that is not the file's. A mark beside a word is set to
/// the word's own height and the file's box is square, so a header asks for a
/// height and this turns it into the box.
#[must_use]
pub fn logo<'a, Renderer>(side: f32) -> crate::Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    Element::new(Logo { side })
}

/// The case, and the three faders in it.
#[derive(Debug)]
struct Logo {
    side: f32,
}

impl Logo {
    /// Returns the rectangle a box-space rectangle lands on inside `bounds`.
    fn at(bounds: Rectangle, x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        let scale = bounds.width / BOX;
        Rectangle {
            x: bounds.x + x * scale,
            y: bounds.y + y * scale,
            width: width * scale,
            height: height * scale,
        }
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Logo
where
    Renderer: iced_core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.side), Length::Fixed(self.side))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(self.side), Length::Fixed(self.side))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let material = materials(theme);
        let scale = bounds.width / BOX;
        let unit = |units: f32| units * scale;

        // The wood the whole case is cut from, which the panel is then laid
        // over: two cheeks and a rounded corner is one rectangle and a lid, not
        // two rectangles that have to meet in the middle.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default().rounded(unit(CORNER)),
                ..renderer::Quad::default()
            },
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(core::f32::consts::FRAC_PI_2))
                    .add_stop(0.0, material.wood)
                    .add_stop(1.0, material.wood_low),
            )),
        );
        // A line of grain down each cheek. One each rather than the file's four,
        // because four lines in six points of wood is a stripe.
        for side in [CHEEK / 2.0, BOX - CHEEK / 2.0] {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Self::at(bounds, side, 0.0, LIP, BOX),
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: GRAIN,
                    ..material.grain
                }),
            );
        }
        // The panel, and the two edges it presents where it meets the wood: the
        // seam the light does not reach, and the lip inside it that catches it.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Self::at(bounds, CHEEK, 0.0, BOX - CHEEK * 2.0, BOX),
                ..renderer::Quad::default()
            },
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(core::f32::consts::PI))
                    .add_stop(0.0, material.recess_edge)
                    .add_stop(1.0, material.panel),
            )),
        );
        for (at, width, colour) in [
            (CHEEK, SEAM, material.recess),
            (BOX - CHEEK - SEAM, SEAM, material.recess),
            (CHEEK + SEAM, LIP, material.scale),
            (BOX - CHEEK - SEAM - LIP, LIP, material.scale),
        ] {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Self::at(bounds, at, 0.0, width, BOX),
                    ..renderer::Quad::default()
                },
                Background::Color(colour),
            );
        }
        Self::faders(renderer, bounds, material);
    }
}

impl Logo {
    /// The three faders, and the wordmark's slices through their caps.
    fn faders<Renderer>(renderer: &mut Renderer, bounds: Rectangle, material: Materials)
    where
        Renderer: iced_core::Renderer,
    {
        let scale = bounds.width / BOX;
        let unit = |units: f32| units * scale;
        let (top, run, across) = SLOT;
        let (wide, deep) = CAP;
        // Three slots cut into it, lit along the wall the light reaches, which
        // is the same recess every fader in this window is drawn as.
        for slot in SLOTS {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Self::at(bounds, slot, top, across, run),
                    border: Border::default().rounded(unit(across / 2.0)),
                    ..renderer::Quad::default()
                },
                Background::Color(material.recess),
            );
        }
        // The caps, in the metal a hand touches, each with the shaded edge under
        // it and the lit edge over it.
        for (slot, cap) in SLOTS.iter().zip(CAPS) {
            let left = slot + across / 2.0 - wide / 2.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Self::at(bounds, left, cap, wide, deep),
                    border: Border {
                        color: material.recess,
                        width: (unit(1.0)).max(0.5),
                        radius: unit(2.0).into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Gradient(Gradient::Linear(
                    Linear::new(Radians(core::f32::consts::PI))
                        .add_stop(0.0, material.metal_high)
                        .add_stop(1.0, material.metal_low),
                )),
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Self::at(bounds, left + 2.4, cap + 1.2, wide - 4.8, 1.0),
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: HIGHLIGHT,
                    ..material.metal_high
                }),
            );
        }
        // And the wordmark's own slices through them. Over the caps and nothing
        // else: a slice is the panel showing through, so where it crosses a cap
        // it is the panel, and where it crosses anything else it is already
        // whatever it would reveal.
        for (at, thick) in SLICES {
            for (slot, cap) in SLOTS.iter().zip(CAPS) {
                if at + thick < cap || at > cap + deep {
                    continue;
                }
                let left = slot + across / 2.0 - wide / 2.0;
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Self::at(bounds, left, at, wide, thick),
                        ..renderer::Quad::default()
                    },
                    Background::Color(material.panel),
                );
            }
        }
    }
}

/// How hard a line of grain shows on the cheek.
const GRAIN: f32 = 0.55;

/// How hard the lit edge of a cap shows on its face.
const HIGHLIGHT: f32 = 0.45;

#[cfg(test)]
mod tests {
    use super::{BOX, CAP, CAPS, SLICES, SLOT, SLOTS};

    #[test]
    fn a_slice_crosses_a_cap_through_its_middle_or_misses_it_altogether() {
        // The one thing about this drawing that is not arbitrary. The cap
        // heights in `docs/logo.svg` were chosen against the slices: two of the
        // three are cut through the middle, and the highest stands clear of the
        // run of lines entirely. A cap a couple of units off either would be a
        // cap with a slice shaving its corner, which reads as a mistake rather
        // than as a mark.
        let (_, deep) = CAP;
        let mut crossed = 0;
        for cap in CAPS {
            let crossing: Vec<(f32, f32)> = SLICES
                .into_iter()
                .filter(|(at, thick)| at + thick > cap && *at < cap + deep)
                .collect();
            let Some((at, thick)) = crossing.first().copied() else {
                continue;
            };
            assert_eq!(crossing.len(), 1, "the cap at {cap} is cut twice");
            assert!(
                at > cap + 1.0 && at + thick < cap + deep - 1.0,
                "the slice at {at} shaves the edge of the cap at {cap}"
            );
            crossed += 1;
        }
        assert_eq!(crossed, 2, "the file cuts two of the three");
    }

    #[test]
    fn nothing_in_the_mark_falls_outside_the_box_it_is_drawn_in() {
        let (top, run, across) = SLOT;
        let (wide, deep) = CAP;

        assert!(top + run < BOX);
        for slot in SLOTS {
            let left = slot + across / 2.0 - wide / 2.0;
            assert!(left > 0.0 && left + wide < BOX, "a cap runs off the case");
        }
        for cap in CAPS {
            assert!(cap > 0.0 && cap + deep < BOX);
        }
    }
}

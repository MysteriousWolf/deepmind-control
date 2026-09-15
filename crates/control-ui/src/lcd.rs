//! The display: a grid of dots, and what is drawn on it.
//!
//! A `DeepMind` has one screen, in the middle of its front panel, and it is a
//! dot matrix: a curve on it is not a curve but the dots nearest one, and the
//! gaps between them are as much of the picture as the dots are. Drawing that
//! as a smooth line on a black rectangle would be drawing some other
//! instrument's display, so this is the grid itself — one quad per lit dot, at
//! a pitch every display in the window shares.
//!
//! # One pitch, and a bigger screen is more dots
//!
//! [`PITCH`] is how far apart two dots are, everywhere. A display given more
//! room does not get bigger dots, it gets more of them, which is the difference
//! between a second screen and a magnified one: the panel's own display and the
//! strip over a plate's faders are cut from the same glass, and a plate wide
//! enough for seven faders has a display wide enough to say more.
//!
//! # Only the lit dots are drawn
//!
//! An unlit dot on the instrument is the glass, and at arm's length there is no
//! grid to see until something lights. So a screen is the glass and the dots it
//! has lit, which is both what it looks like and eight thousand quads a display
//! that nobody could see. What carries the matrix is the gap between the lit
//! ones.
//!
//! # A screen has no moving part, so the claim is its colour
//!
//! Every control in this editor carries what backs its value in the fill of the
//! thing that moves — filled for a fact, an outline for a claim, nothing at all
//! for a value nobody has read. A display has nothing that moves. It is the
//! same position the [name](crate::name) field is in, and it is answered the
//! same way: the lit dots take the claim's own colour, the control beside the
//! screen keeps the fill, and a screen drawn from anything unread is drawn
//! dark rather than in a colour nobody should trust.

use core::fmt;

use iced_core::gradient::Linear;
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Border, Element, Gradient, Length, Radians, Rectangle, Size as Area, Theme, Widget,
    mouse, renderer,
};

use crate::Confidence;
use crate::glyphs;
use crate::style::{materials, tint};

/// How far apart two dots are, in points, on every display in the window.
pub const PITCH: f32 = 2.5;

/// How much of that pitch is lit.
///
/// The rest is the gap, which is what makes a row of lit dots read as dots
/// rather than as a line.
const LIT: f32 = 1.9;

/// How much glass there is around the dots.
///
/// A display has a dead border inside its bezel, and a drawing that ran to the
/// edge of the glass would be a drawing that looked cut off.
const MARGIN: f32 = 4.0;

/// Returns how many points `dots` of the grid cover.
#[expect(
    clippy::cast_precision_loss,
    reason = "a coordinate on a screen a window has room for, which an f32 holds exactly"
)]
const fn points(dots: i32) -> f32 {
    dots as f32
}

/// Returns which dot a fraction of a run of them falls on.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a fraction of a length this screen already holds"
)]
fn nearest(fraction: f32) -> i32 {
    fraction.round() as i32
}

/// Returns how many dots fit across `points` of panel, glass and all.
///
/// What a plate asks when it is given a width: a display is as many dots as the
/// room it was given divides into, because the pitch is the same everywhere and
/// the room is not.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "a count of dots across a width a window has room for"
)]
pub fn fits(points: f32) -> i32 {
    ((points - MARGIN * 2.0) / PITCH).floor().max(0.0) as i32
}

/// Returns how much room `dots` of a display need, glass and all.
///
/// The other way round, for a plate working out how wide it has to stand to
/// have a display worth drawing on.
#[must_use]
pub const fn room(dots: i32) -> f32 {
    points(dots) * PITCH + MARGIN * 2.0
}

/// How a line is laid down.
///
/// One screen, one colour, and three things to tell apart on it: the ink is
/// what does that, the way a dashed line does it on any other drawing. It is
/// never what says how much a value is trusted — that is the colour of the
/// whole screen — so a pattern here only ever distinguishes one curve from
/// the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ink {
    /// Every dot.
    #[default]
    Solid,
    /// Three dots and two of glass.
    Dashed,
    /// One dot and two of glass.
    Dotted,
}

impl Ink {
    /// Returns whether `step` of a run is laid down.
    const fn shows(self, step: i32) -> bool {
        match self {
            Self::Solid => true,
            Self::Dashed => step.rem_euclid(5) < 3,
            Self::Dotted => step.rem_euclid(3) == 0,
        }
    }
}

/// How large a character is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    /// The cell itself: five dots by seven.
    #[default]
    Small,
    /// Every dot of it drawn as four, which is what a display does when it has
    /// one thing to say and room to say it twice as loudly.
    Large,
}

impl Size {
    /// How many dots one dot of a glyph is drawn as, across and down.
    const fn scale(self) -> i32 {
        match self {
            Self::Small => 1,
            Self::Large => 2,
        }
    }
}

/// A rectangle of a screen, in dots.
///
/// What a drawing is given rather than the whole glass: a display showing two
/// oscillators is two of these, and a curve drawn in one does not know that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// The left edge.
    pub x: i32,
    /// The top edge.
    pub y: i32,
    /// How many dots across.
    pub width: i32,
    /// How many dots down.
    pub height: i32,
}

impl Band {
    /// A band `width` by `height` dots, with its top left corner at `x`, `y`.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the band inset by `dots` on every side.
    #[must_use]
    pub const fn inset(self, dots: i32) -> Self {
        Self {
            x: self.x + dots,
            y: self.y + dots,
            width: self.width - dots * 2,
            height: self.height - dots * 2,
        }
    }

    /// Returns the `count` bands this one divides into, top to bottom, with
    /// `gutter` dots of glass between them.
    ///
    /// Empty when there is not enough height to divide, which is a display
    /// saying it is too small for what was asked of it rather than drawing
    /// lanes a dot tall.
    #[must_use]
    pub fn lanes(self, count: i32, gutter: i32) -> Vec<Self> {
        if count <= 0 {
            return Vec::new();
        }
        let height = (self.height - gutter * (count - 1)) / count;
        if height < 2 {
            return Vec::new();
        }
        (0..count)
            .map(|lane| Self {
                x: self.x,
                y: self.y + lane * (height + gutter),
                width: self.width,
                height,
            })
            .collect()
    }

    /// Returns the row a height of `fraction` of the band falls on.
    ///
    /// Nothing is its floor and one is its top, which is the way up every
    /// drawing in this crate reads and the opposite of the way a screen's rows
    /// are numbered.
    #[must_use]
    pub fn row(self, fraction: f32) -> i32 {
        let top = self.height - 1;
        self.y + top - nearest(fraction.clamp(0.0, 1.0) * points(top))
    }

    /// Returns the column `fraction` of the way across the band.
    #[must_use]
    pub fn column(self, fraction: f32) -> i32 {
        self.x + nearest(fraction.clamp(0.0, 1.0) * points(self.width - 1))
    }
}

/// What a display is showing: every dot of it, lit or not.
///
/// Built by whatever is drawing, handed to [`lcd`], and drawn once. It holds
/// dots and knows nothing about a value, a parameter or a claim: what goes on a
/// screen is the caller's business, which is why the panel can leave a
/// screen-shaped hole and let an application fill it.
#[derive(Clone, PartialEq, Eq)]
pub struct Screen {
    columns: i32,
    rows: i32,
    lit: Vec<bool>,
}

impl fmt::Debug for Screen {
    /// Its size and how much of it is lit.
    ///
    /// Eight thousand booleans printed one at a time is not a debug
    /// representation of anything.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Screen({} x {}, {} lit)",
            self.columns,
            self.rows,
            self.lit.iter().filter(|dot| **dot).count()
        )
    }
}

impl Screen {
    /// A dark screen `columns` dots across and `rows` down.
    #[must_use]
    pub fn new(columns: i32, rows: i32) -> Self {
        let columns = columns.max(0);
        let rows = rows.max(0);
        let dots = usize::try_from(columns)
            .unwrap_or_default()
            .saturating_mul(usize::try_from(rows).unwrap_or_default());
        Self {
            columns,
            rows,
            lit: vec![false; dots],
        }
    }

    /// How many dots across.
    #[must_use]
    pub const fn columns(&self) -> i32 {
        self.columns
    }

    /// How many dots down.
    #[must_use]
    pub const fn rows(&self) -> i32 {
        self.rows
    }

    /// The whole of the glass, as a band.
    #[must_use]
    pub const fn all(&self) -> Band {
        Band::new(0, 0, self.columns, self.rows)
    }

    /// Whether anything at all is lit.
    #[must_use]
    pub fn is_dark(&self) -> bool {
        !self.lit.iter().any(|dot| *dot)
    }

    /// Returns where a dot lives, when it is on the screen at all.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.columns || y >= self.rows {
            return None;
        }
        usize::try_from(y.checked_mul(self.columns)?.checked_add(x)?).ok()
    }

    /// Lights one dot. A dot off the screen is not an error; it is clipped.
    pub fn dot(&mut self, x: i32, y: i32) {
        if let Some(index) = self.index(x, y)
            && let Some(dot) = self.lit.get_mut(index)
        {
            *dot = true;
        }
    }

    /// Whether one dot is lit.
    #[must_use]
    pub fn is_lit(&self, x: i32, y: i32) -> bool {
        self.index(x, y)
            .and_then(|index| self.lit.get(index))
            .copied()
            .unwrap_or_default()
    }

    /// Draws a rule of `length` dots to the right of `x`, `y`.
    pub fn across(&mut self, x: i32, y: i32, length: i32, ink: Ink) {
        for step in 0..length {
            if ink.shows(step) {
                self.dot(x + step, y);
            }
        }
    }

    /// Draws a rule of `length` dots below `x`, `y`.
    pub fn down(&mut self, x: i32, y: i32, length: i32, ink: Ink) {
        for step in 0..length {
            if ink.shows(step) {
                self.dot(x, y + step);
            }
        }
    }

    /// Draws a line from one dot to another.
    pub fn line(&mut self, from: (i32, i32), to: (i32, i32), ink: Ink) {
        let (mut x, mut y) = from;
        let (dx, dy) = ((to.0 - x).abs(), -(to.1 - y).abs());
        let (sx, sy) = (if x < to.0 { 1 } else { -1 }, if y < to.1 { 1 } else { -1 });
        let mut error = dx + dy;
        let mut step = 0;
        loop {
            if ink.shows(step) {
                self.dot(x, y);
            }
            if x == to.0 && y == to.1 {
                return;
            }
            let doubled = error * 2;
            if doubled >= dy {
                error += dy;
                x += sx;
            }
            if doubled <= dx {
                error += dx;
                y += sy;
            }
            step += 1;
        }
    }

    /// Draws the outline of a band.
    pub fn frame(&mut self, band: Band, ink: Ink) {
        self.across(band.x, band.y, band.width, ink);
        self.across(band.x, band.y + band.height - 1, band.width, ink);
        self.down(band.x, band.y, band.height, ink);
        self.down(band.x + band.width - 1, band.y, band.height, ink);
    }

    /// Lights every dot of a band.
    pub fn fill(&mut self, band: Band) {
        for row in 0..band.height {
            self.across(band.x, band.y + row, band.width, Ink::Solid);
        }
    }

    /// Lights every other dot of a band.
    ///
    /// What a screen with one colour of light does instead of a grey: near
    /// enough to solid to read as a body, open enough to read a line drawn
    /// across it.
    pub fn wash(&mut self, band: Band) {
        for row in 0..band.height {
            for column in 0..band.width {
                if (band.x + column + band.y + row).rem_euclid(2) == 0 {
                    self.dot(band.x + column, band.y + row);
                }
            }
        }
    }

    /// Turns every dot of a band into its opposite.
    ///
    /// Which is how a display says "this line is the heading" without a second
    /// colour or a second face.
    pub fn invert(&mut self, band: Band) {
        for row in 0..band.height {
            for column in 0..band.width {
                let (x, y) = (band.x + column, band.y + row);
                if let Some(index) = self.index(x, y)
                    && let Some(dot) = self.lit.get_mut(index)
                {
                    *dot = !*dot;
                }
            }
        }
    }

    /// Writes `words` with their top left corner at `x`, `y`.
    ///
    /// Returns where the next character would start, so that a line built out
    /// of several pieces does not have to count them.
    pub fn write(&mut self, x: i32, y: i32, words: &str, size: Size) -> i32 {
        let scale = size.scale();
        let mut pen = x;
        for character in words.chars() {
            for (row, dots) in glyphs::of(character).iter().enumerate() {
                let row = i32::try_from(row).unwrap_or_default();
                for column in 0..glyphs::WIDTH {
                    if u32::from(*dots) & (1 << (glyphs::WIDTH - 1 - column)) == 0 {
                        continue;
                    }
                    for down in 0..scale {
                        for across in 0..scale {
                            self.dot(pen + column * scale + across, y + row * scale + down);
                        }
                    }
                }
            }
            pen += glyphs::ADVANCE * scale;
        }
        pen
    }

    /// Writes `words` centred across the screen.
    pub fn centre(&mut self, y: i32, words: &str, size: Size) {
        let x = (self.columns - Self::width_of(words, size)) / 2;
        self.write(x.max(0), y, words, size);
    }

    /// How many dots wide `words` are written, without the gap after the last.
    #[must_use]
    pub fn width_of(words: &str, size: Size) -> i32 {
        let characters = i32::try_from(words.chars().count()).unwrap_or(i32::MAX);
        (characters * glyphs::ADVANCE * size.scale() - glyphs::GAP * size.scale()).max(0)
    }

    /// How many dots tall a line of writing is.
    #[must_use]
    pub const fn height_of(size: Size) -> i32 {
        glyphs::HEIGHT * size.scale()
    }

    /// Draws a curve across `band`, joined column to column.
    ///
    /// `height_at` is asked for a height between nothing and the top of the
    /// band, for a position between its left edge and its right, and both are
    /// fractions: a drawing that took dots would be a drawing that had to be
    /// rewritten for a plate one fader wider.
    ///
    /// Joined rather than plotted, for the reason the envelope drawing is: a
    /// segment steeper than a dot is wide would otherwise be a column of marks
    /// with the shape missing between them.
    pub fn curve(&mut self, band: Band, ink: Ink, height_at: impl Fn(f32) -> f32) {
        if band.width <= 0 || band.height <= 0 {
            return;
        }
        let last = points((band.width - 1).max(1));
        let mut previous: Option<i32> = None;
        for column in 0..band.width {
            let row = band.row(height_at(points(column) / last));
            if ink.shows(column) {
                let from = previous.unwrap_or(row);
                for y in from.min(row)..=from.max(row) {
                    self.dot(band.x + column, y);
                }
            }
            previous = Some(row);
        }
    }

    /// Shades everything under that curve.
    ///
    /// The body of a shape, for the one curve on a screen that is the thing
    /// itself rather than a comparison with it. A quarter of the dots rather
    /// than [a half](Screen::wash): the outline is what is being read and a
    /// body as bright as its own edge is a body with no edge.
    pub fn under(&mut self, band: Band, height_at: impl Fn(f32) -> f32) {
        if band.width <= 0 || band.height <= 0 {
            return;
        }
        let last = points((band.width - 1).max(1));
        let floor = band.y + band.height - 1;
        for column in 0..band.width {
            let top = band.row(height_at(points(column) / last));
            for y in top..=floor {
                if (band.x + column + y * 2).rem_euclid(4) == 0 {
                    self.dot(band.x + column, y);
                }
            }
        }
    }

    /// Draws a mark up the whole of a band at `fraction` of the way across it.
    pub fn mark(&mut self, band: Band, fraction: f32, ink: Ink) {
        self.down(band.column(fraction), band.y, band.height, ink);
    }
}

/// Draws `screen` as the display it is, in the colour of `claim`.
///
/// The claim is the whole screen's, and it is the weakest of everything the
/// drawing read: a display that called itself the synthesizer's because half of
/// what it drew was is the one thing this editor exists not to do.
#[must_use]
pub fn lcd<'a, Renderer>(screen: Screen, claim: Confidence) -> crate::Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    Element::new(Display { screen, claim })
}

/// The glass, and the dots on it.
#[derive(Debug)]
struct Display {
    screen: Screen,
    claim: Confidence,
}

impl Display {
    /// How much room the glass takes, dots and dead border together.
    fn area(&self) -> Area<Length> {
        Area::new(
            Length::Fixed(points(self.screen.columns) * PITCH + MARGIN * 2.0),
            Length::Fixed(points(self.screen.rows) * PITCH + MARGIN * 2.0),
        )
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Display
where
    Renderer: iced_core::Renderer,
{
    fn size(&self) -> Area<Length> {
        self.area()
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let area = self.area();
        layout::atomic(limits, area.width, area.height)
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

        // The glass: the deepest recess on the panel, lit from the top the way
        // the panel itself is, because a display is a window into a lit case
        // and not a hole cut in one.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 3.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(std::f32::consts::PI))
                    .add_stop(0.0, material.glass)
                    .add_stop(1.0, material.recess),
            )),
        );

        let colour = tint(theme, self.claim);
        let inset = (PITCH - LIT) / 2.0;
        for row in 0..self.screen.rows() {
            for column in 0..self.screen.columns() {
                if !self.screen.is_lit(column, row) {
                    continue;
                }
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + MARGIN + points(column) * PITCH + inset,
                            y: bounds.y + MARGIN + points(row) * PITCH + inset,
                            width: LIT,
                            height: LIT,
                        },
                        border: Border::default().rounded(0.5),
                        ..renderer::Quad::default()
                    },
                    Background::Color(colour),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Band, Ink, Screen, Size};

    /// How many dots of a screen are lit.
    fn lit(screen: &Screen) -> usize {
        (0..screen.rows())
            .flat_map(|row| (0..screen.columns()).map(move |column| (column, row)))
            .filter(|(column, row)| screen.is_lit(*column, *row))
            .count()
    }

    #[test]
    fn a_new_screen_is_dark() {
        let screen = Screen::new(40, 20);

        assert!(screen.is_dark());
        assert_eq!(lit(&screen), 0);
    }

    #[test]
    fn a_dot_off_the_screen_is_clipped_and_not_a_panic() {
        // A drawing is written against fractions of a band, and rounding at the
        // ends of one puts a dot a row past the glass. Every screen in this
        // window would otherwise be one rounding error from taking the process
        // with it.
        let mut screen = Screen::new(8, 8);
        screen.dot(-1, 4);
        screen.dot(4, -1);
        screen.dot(8, 4);
        screen.dot(4, 8);
        screen.line((-40, -40), (80, 80), Ink::Solid);

        assert!(screen.is_lit(4, 4), "the part of the line that is on it");
        assert!(!screen.is_lit(0, 4));
    }

    #[test]
    fn a_curve_is_joined_rather_than_plotted() {
        // A fall of the whole height between two columns is a line, not two
        // marks with the shape missing between them.
        let mut screen = Screen::new(4, 16);
        screen.curve(
            screen.all(),
            Ink::Solid,
            |x| if x < 0.5 { 1.0 } else { 0.0 },
        );

        for row in 0..16 {
            assert!(
                screen.is_lit(2, row),
                "the column the drop happens in has a gap at row {row}"
            );
        }
    }

    #[test]
    fn a_curve_reaches_both_ends_of_its_band() {
        let mut screen = Screen::new(10, 10);
        screen.curve(screen.all(), Ink::Solid, |x| x);

        assert!(screen.is_lit(0, 9), "nothing at the left end");
        assert!(screen.is_lit(9, 0), "nothing at the right end");
    }

    #[test]
    fn an_ink_is_a_pattern_and_never_a_claim() {
        let dashed = (0..10).filter(|step| Ink::Dashed.shows(*step)).count();
        let dotted = (0..9).filter(|step| Ink::Dotted.shows(*step)).count();

        assert!((0..10).all(|step| Ink::Solid.shows(step)));
        assert_eq!(dashed, 6);
        assert_eq!(dotted, 3);
    }

    #[test]
    fn writing_advances_by_the_cell_and_measures_the_same() {
        let mut screen = Screen::new(64, 16);
        let pen = screen.write(0, 0, "VCF", Size::Small);

        // Three cells and the gaps between them, with no gap hanging off the
        // end: a centred word that measured its own trailing gap would sit a
        // half-cell to the left of centre.
        assert_eq!(Screen::width_of("VCF", Size::Small), 17);
        assert_eq!(pen, 18);
        assert_eq!(
            Screen::width_of("VCF", Size::Large),
            Screen::width_of("VCF", Size::Small) * 2
        );
        assert!(!screen.is_dark());
    }

    #[test]
    fn a_word_too_wide_for_the_screen_is_clipped_rather_than_wrapped() {
        // A display clips. Sixteen characters of a program name on a plate's
        // strip is what a screen that narrow has room for, and a name that
        // wrapped onto the drawing under it would be worse than a short one.
        let mut screen = Screen::new(12, 8);
        screen.write(0, 0, "a name far too long for this", Size::Small);

        assert!(!screen.is_dark());
    }

    #[test]
    fn a_wash_is_half_the_dots_and_an_inversion_is_the_other_half() {
        let band = Band::new(0, 0, 10, 10);
        let mut washed = Screen::new(10, 10);
        washed.wash(band);
        let mut inverted = washed.clone();
        inverted.invert(band);

        assert_eq!(lit(&washed), 50);
        assert_eq!(lit(&inverted), 50);
        assert_ne!(washed.is_lit(0, 0), inverted.is_lit(0, 0));
    }

    #[test]
    fn lanes_divide_a_band_and_refuse_to_when_there_is_no_room() {
        let lanes = Band::new(0, 0, 40, 20).lanes(2, 2);

        assert_eq!(lanes.len(), 2);
        assert_eq!(lanes.first().copied(), Some(Band::new(0, 0, 40, 9)));
        assert_eq!(lanes.get(1).copied(), Some(Band::new(0, 11, 40, 9)));
        assert!(
            Band::new(0, 0, 40, 6).lanes(4, 2).is_empty(),
            "four lanes in six dots is four lanes nobody can read"
        );
    }

    #[test]
    fn a_screen_says_its_size_and_how_much_of_it_is_lit() {
        let mut screen = Screen::new(6, 3);
        screen.dot(1, 1);

        assert_eq!(format!("{screen:?}"), "Screen(6 x 3, 1 lit)");
    }
}

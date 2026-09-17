//! The display: a grid of dots, and what is drawn on it.
//!
//! A `DeepMind` has one screen, in the middle of its front panel, and it is a
//! dot matrix: a curve on it is not a curve but the dots nearest one, and the
//! gaps between them are as much of the picture as the dots are. Drawing that
//! as a smooth line on a black rectangle would be drawing some other
//! instrument's display, so this is the grid itself — one quad per printed dot,
//! at a pitch every display in the window shares.
//!
//! # It is a lit panel, and the dots are dark on it
//!
//! The screen on a `DeepMind` is a backlit positive display. The glass is a
//! pale green-white and what is written on it is printed dark, which is the one
//! surface on the whole instrument that gives off light rather than catching
//! it, and the reason a photograph of the panel has one bright rectangle in the
//! middle of it. A window that drew pale dots on a dark pane would be drawing
//! the negative of the instrument it is a picture of — every other synthesizer
//! of the decade, and not this one.
//!
//! # One pitch, and a bigger screen is more dots
//!
//! [`PITCH`] is how far apart two dots are, everywhere. A display given more
//! room does not get bigger dots, it gets more of them, which is the difference
//! between a second screen and a magnified one: the panel's own display and the
//! strip over a plate's faders are cut from the same glass, and a plate wide
//! enough for seven faders has a display wide enough to say more.
//!
//! # Only the printed dots are drawn
//!
//! A dot the display has not printed is the backlight coming through, and at
//! arm's length there is no grid to see until something is written. So a screen
//! is the lit glass and the dots printed on it, which is both what it looks
//! like and eight thousand quads a display fewer than drawing the grid. What
//! carries the matrix is the glass between the printed ones.
//!
//! # A screen has no moving part, so the claim is how hard it is printed
//!
//! Every control in this editor carries what backs its value in the fill of the
//! thing that moves — filled for a fact, an outline for a claim, nothing at all
//! for a value nobody has read. A display has nothing that moves. It is the
//! same position the [name](crate::name) field is in, and the pale ground
//! answers it better than a dark one could: [`written`](crate::written) prints a fact
//! hard, a claim in copper mixed most of the way to the same black, and what
//! nobody has read barely at all. Three depths of one ink, which is an ordering
//! rather than a set of hues, so the difference survives a photograph and the
//! readers who would not see the copper. A screen with nothing read behind it
//! is left blank, which on this display means lit and empty.

use deepmind_midi::pixels::{self, Pixels};

use core::fmt;

use iced_core::gradient::Linear;
use iced_core::layout::{self, Layout};
use iced_core::widget::Tree;
use iced_core::{
    Background, Border, Color, Element, Gradient, Length, Radians, Rectangle, Size as Area, Theme,
    Widget, mouse, renderer,
};

use crate::Confidence;
use crate::glyphs;
use crate::style::{glazing, materials, written};

/// How far apart two dots are, in points, on every display in the window.
pub const PITCH: f32 = 2.5;

/// How much of that pitch the dot covers.
///
/// The rest is the glass between them, which is what makes a row of printed
/// dots read as dots rather than as a line.
const LIT: f32 = 1.9;

/// How much glass there is around the dots.
///
/// A display has a dead border inside its bezel, and a drawing that ran to the
/// edge of the glass would be a drawing that looked cut off. Two points, which
/// is a dot's width: enough that the edge is there and not so much that the
/// glass reads as a frame with a picture in it.
const MARGIN: f32 = 2.0;

/// How much surround the glass is set into.
///
/// The moulding a screen is let into rather than the dead glass inside it, and
/// the two are different things that used to be one number. This is the part
/// that catches light along its lower edge and casts a line across the top of
/// the glass, which is the whole of what makes a screen read as set *into*
/// something rather than printed on it.
const BEZEL: f32 = 2.0;

/// How hard the surround's own shadow falls across the top of the glass.
const CAST: f32 = 0.30;

/// How hard the light along its lower edge comes back off the glass.
const CATCH: f32 = 0.16;

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
    ((points - SURROUND * 2.0) / PITCH).floor().max(0.0) as i32
}

/// Returns how much room `dots` of a display need, glass and all.
///
/// The other way round, for a plate working out how wide it has to stand to
/// have a display worth drawing on.
#[must_use]
pub const fn room(dots: i32) -> f32 {
    points(dots) * PITCH + SURROUND * 2.0
}

/// How much of a display is not its dots, on one side.
///
/// The moulding and the dead glass inside it, which is what a width has to
/// allow for and what a count of dots has to be measured back out of.
const SURROUND: f32 = BEZEL + MARGIN;

/// How fast a field too small for its name scrolls, in dots a second.
///
/// Eight, which at this pitch is twenty points a second: slow enough to read a
/// ten-character name without chasing it and fast enough that a name arrives
/// rather than creeps.
const RATE: f32 = 8.0;

/// How long a scrolling field holds still at each end of its travel, in
/// seconds.
///
/// Long enough to read the beginning of a name without waiting for it to come
/// back round, which is the whole reason a label has a beginning.
const HOLD: f32 = 1.5;

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

    /// Returns the `count` bands this one divides into, left to right, with
    /// `gutter` dots of glass between them.
    ///
    /// The across twin of [`lanes`](Band::lanes), and empty on the same terms:
    /// a band with no room to divide says so rather than returning columns a
    /// dot wide.
    #[must_use]
    pub fn columns(self, count: i32, gutter: i32) -> Vec<Self> {
        if count <= 0 {
            return Vec::new();
        }
        let width = (self.width - gutter * (count - 1)) / count;
        if width < 2 {
            return Vec::new();
        }
        (0..count)
            .map(|column| Self {
                x: self.x + column * (width + gutter),
                y: self.y,
                width,
                height: self.height,
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

/// What a display is showing: every dot of it, printed or not.
///
/// Built by whatever is drawing, handed to [`lcd`], and drawn once. It holds
/// dots and knows nothing about a value, a parameter or a claim: what goes on a
/// screen is the caller's business, which is why the panel can leave a
/// screen-shaped hole and let an application fill it.
#[derive(Clone, PartialEq, Eq)]
pub struct Screen {
    columns: i32,
    rows: i32,
    inked: Vec<bool>,
}

impl fmt::Debug for Screen {
    /// Its size and how much of it is printed.
    ///
    /// Eight thousand booleans printed one at a time is not a debug
    /// representation of anything.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Screen({} x {}, {} printed)",
            self.columns,
            self.rows,
            self.inked.iter().filter(|dot| **dot).count()
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
            inked: vec![false; dots],
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

    /// Whether the glass is lit and empty.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        !self.inked.iter().any(|dot| *dot)
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
            && let Some(dot) = self.inked.get_mut(index)
        {
            *dot = true;
        }
    }

    /// Whether one dot is printed.
    #[must_use]
    pub fn is_inked(&self, x: i32, y: i32) -> bool {
        self.index(x, y)
            .and_then(|index| self.inked.get(index))
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

    /// Puts out every dot of a band.
    ///
    /// What a caller wants before it writes something that has to be read
    /// whatever is already there: a label on a drawing is the one thing on a
    /// screen that cannot be *mixed* with what it stands on, because half a
    /// letter and half a curve is neither.
    pub fn wipe(&mut self, band: Band) {
        for row in 0..band.height {
            for column in 0..band.width {
                let (x, y) = (band.x + column, band.y + row);
                if let Some(index) = self.index(x, y)
                    && let Some(dot) = self.inked.get_mut(index)
                {
                    *dot = false;
                }
            }
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
                    && let Some(dot) = self.inked.get_mut(index)
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
        self.written(x, y, words, size, None)
    }

    /// Writes `words` into a field `across` dots wide, scrolling them when they
    /// are too long for it.
    ///
    /// What a hardware display does with a name that does not fit, and what
    /// this window was doing instead was cutting the tail off: `Pitch Bend` and
    /// `BreathCtrl` are ten characters in a field cut for nine, and `Pitch Ben`
    /// is a name somebody has to already know to read. A field that scrolls
    /// says the whole thing and takes a moment over it, which is the trade
    /// every instrument with a two-line screen on it has already made.
    ///
    /// Nothing scrolls that fits: a field only moves when moving is the only
    /// way to say all of it, so a page of short names is a still page.
    ///
    /// It **holds, travels and holds**, then starts again — rather than running
    /// round and round with the tail of the name chasing its head. A name that
    /// wraps is two names on the glass at once for as long as the gap between
    /// them takes to cross, and the first thing somebody wants from a label is
    /// its beginning: this one is at the beginning for a second and a half out
    /// of every lap, at the end for as long again, and moving in between at
    /// eight dots a second.
    ///
    /// Where it has got to is a clock this module keeps rather than state a
    /// caller has to thread through, because how far a display has scrolled is
    /// not a fact about the sound. Every field in the window travels together
    /// on it, and it advances with the window's own redraws.
    pub fn marquee(&mut self, x: i32, y: i32, across: i32, words: &str, size: Size) {
        let width = Self::width_of(words, size);
        let over = width - across;
        if over <= 0 {
            self.write(x, y, words, size);
            return;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a count of dots the field holds still for"
        )]
        let hold = (RATE * HOLD) as i32;
        let gone = Self::crawl(over + hold * 2);
        let by = (gone - hold).clamp(0, over);
        self.written(x - by, y, words, size, Some((x, across)));
    }
    /// How far a scrolling field has got round its lap, in dots.
    ///
    /// Off a clock this module starts the first time anything asks, which is
    /// the one piece of state in this crate that is not a fact about the
    /// instrument: every field in the window travels together, at one rate,
    /// because they are one screen as far as a reader is concerned.
    ///
    /// It advances with the window's own redraws — every frame while a port is
    /// open, and not at all while the application is idle. A window with
    /// nothing to hear is a window with nothing to say, and a still label on
    /// one is not a label that has stopped working.
    fn crawl(lap: i32) -> i32 {
        static STARTED: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let seconds = STARTED
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f32();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "dots since the window opened, taken back round the lap"
        )]
        let gone = (seconds * RATE) as i32;
        gone.rem_euclid(lap.max(1))
    }

    /// Writes `words` at `x`, keeping only what falls inside `field`.
    ///
    /// `field` is where a scrolling name is allowed to be seen — its left edge
    /// and how wide it is — and `None` is the whole screen, which is what an
    /// ordinary write is.
    fn written(
        &mut self,
        x: i32,
        y: i32,
        words: &str,
        size: Size,
        field: Option<(i32, i32)>,
    ) -> i32 {
        let scale = size.scale();
        let mut pen = x;
        let shows = |at: i32| match field {
            Some((from, across)) => at >= from && at < from + across,
            None => true,
        };
        for character in words.chars() {
            for (row, dots) in glyphs::of(character).iter().enumerate() {
                let row = i32::try_from(row).unwrap_or_default();
                for column in 0..glyphs::WIDTH {
                    if u32::from(*dots) & (1 << (glyphs::WIDTH - 1 - column)) == 0 {
                        continue;
                    }
                    for down in 0..scale {
                        for across in 0..scale {
                            let at = pen + column * scale + across;
                            if shows(at) {
                                self.dot(at, y + row * scale + down);
                            }
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

    /// A screen cut to one line of `words` and nothing else on it.
    ///
    /// What a caller wants when the dots are the writing rather than a drawing
    /// with writing on it: the grid is as wide as the line measures and as deep
    /// as the line stands, so the box the dots come in is the word itself.
    #[must_use]
    pub fn of(words: &str, size: Size) -> Self {
        let mut screen = Self::new(Self::width_of(words, size), Self::height_of(size));
        screen.write(0, 0, words, size);
        screen
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

    /// Blits one of the library's one-bit grids, a lit pixel to a printed dot.
    ///
    /// Three things in the library are drawn on one grid at [`pixels::SIDE`] a
    /// side — an effect's mark, a modulation source's cell, and the glyph of
    /// what a parameter does — for exactly this display: one with no room to
    /// stroke anything, where which of forty-nine dots are lit is the whole of
    /// the design.
    ///
    /// Walked the way the library documents: the origin is the top left, and a
    /// pixel outside the grid answers unlit, so nothing here bounds-check it.
    /// A dot outside the screen is dropped by [`dot`](Self::dot), the same as
    /// every other drawing on it.
    pub fn blit(&mut self, pixels: &Pixels, x: i32, y: i32) {
        for down in 0..CELL {
            for across in 0..CELL {
                let (column, row) = (
                    u8::try_from(across).unwrap_or(u8::MAX),
                    u8::try_from(down).unwrap_or(u8::MAX),
                );
                if pixels.is_lit(column, row) {
                    self.dot(x + across, y + down);
                }
            }
        }
    }
}

/// How many dots one of the library's grids is, across and down.
///
/// Seven, which is [`pixels::SIDE`] and also the cell this display writes a
/// character in — the two being the same size is what lets a picture stand
/// beside a name without either of them being resampled.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "a side of the library's own grid, which is seven pixels"
)]
pub const CELL: i32 = pixels::SIDE as i32;

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
    Element::new(Display {
        screen,
        claim,
        polarity: None,
    })
}

/// Draws `screen` as dots stencilled on whatever is behind them, in `ink`.
///
/// The same grid, the same pitch and the same square dots as [`lcd`], with the
/// glass, the moulding and the light on it all left out — so what is left is
/// the printing rather than the display. A number stencilled on the case of a
/// rack unit is a dot matrix too, and one set in a typeface beside a window
/// full of screens is the one piece of writing on the page in a face nothing
/// else uses.
///
/// The ink is the caller's because this is not a display and so has no claim to
/// carry: [`lcd`] takes a [`Confidence`] and colours the whole screen with it,
/// and a marking on a case is a fact about the case. It takes a theme rather
/// than a colour for the same reason every other painted part of this window
/// does — the panel can be turned over while the window is open.
#[must_use]
pub fn stencil<'a, Renderer, Ink>(screen: Screen, ink: Ink) -> crate::Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Ink: Fn(&Theme) -> Color + 'a,
{
    Element::new(Stencil { screen, ink })
}

/// Dots, and nothing under them.
#[derive(Debug)]
struct Stencil<Ink> {
    screen: Screen,
    ink: Ink,
}

impl<Ink> Stencil<Ink> {
    /// How much room the dots take.
    ///
    /// The grid itself, with no surround: there is no glass to hold a dead
    /// border and no moulding to set it into, so the box is the printing and a
    /// caller that wants room around it says so where it puts it.
    fn area(&self) -> Area<Length> {
        Area::new(
            Length::Fixed(points(self.screen.columns()) * PITCH),
            Length::Fixed(points(self.screen.rows()) * PITCH),
        )
    }
}

impl<Message, Renderer, Ink> Widget<Message, Theme, Renderer> for Stencil<Ink>
where
    Renderer: iced_core::Renderer,
    Ink: Fn(&Theme) -> Color,
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
        print_dots(
            renderer,
            &self.screen,
            bounds.x,
            bounds.y,
            (self.ink)(theme),
        );
    }
}

/// A display the size of a character, showing the polarity it names.
///
/// What the press that turns the window's displays over wears instead of the
/// words `Negative display`. A sentence on a row of sentences said which way up
/// they would be and had to be read to say it; a screen showing itself the way
/// it is about to be says the same thing without being read, and says it in the
/// one material the press is about.
///
/// It is the only display in this window that does not take its glass from the
/// theme, for exactly that reason: it is a picture of the other way round.
#[must_use]
pub fn swatch<'a, Renderer>(negative: bool) -> crate::Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    // Seven by seven, which is the cell this instrument's display writes a
    // character in, with the lower half of it printed: the smallest drawing
    // that is obviously a screen with something on it rather than a screen.
    let mut screen = Screen::new(CHARACTER, CHARACTER);
    for row in 0..CHARACTER {
        for column in 0..CHARACTER {
            if column <= row {
                screen.dot(column, row);
            }
        }
    }
    Element::new(Display {
        screen,
        claim: Confidence::Confirmed,
        polarity: Some(negative),
    })
}

/// How many dots a character of this display's own writing stands in.
const CHARACTER: i32 = glyphs::HEIGHT;

/// The glass, and the dots on it.
#[derive(Debug)]
struct Display {
    screen: Screen,
    claim: Confidence,
    /// Which way up this one is drawn, where that is not the theme's answer.
    ///
    /// `None` everywhere but the press that turns them over, which is a picture
    /// of the polarity somebody is about to get rather than the one they have.
    polarity: Option<bool>,
}

impl Display {
    /// How much room the glass takes, dots and dead border together.
    fn area(&self) -> Area<Length> {
        Area::new(
            Length::Fixed(points(self.screen.columns) * PITCH + SURROUND * 2.0),
            Length::Fixed(points(self.screen.rows) * PITCH + SURROUND * 2.0),
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
        let mut material = materials(theme);
        if let Some(negative) = self.polarity {
            let (glass, glass_low, ink) = glazing(negative);
            material.glass = glass;
            material.glass_low = glass_low;
            material.ink = ink;
        }

        // The glass: the lit panel, brightest where the light enters it and
        // falling away across it, inside the dark bezel it is set into. It is
        // the one surface in this window that is brighter than the panel around
        // it, which is what a backlit display looks like on a dark instrument.
        // The moulding the glass is let into. Dark, with a hairline of the
        // panel's own metal along it: a screen on a piece of equipment sits in
        // a surround, and the surround is the part that catches the light in
        // the room rather than the light behind the panel.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: material.lit,
                    width: 1.0,
                    radius: 3.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(material.recess),
        );
        let glass = Rectangle {
            x: bounds.x + BEZEL,
            y: bounds.y + BEZEL,
            width: (bounds.width - BEZEL * 2.0).max(0.0),
            height: (bounds.height - BEZEL * 2.0).max(0.0),
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: glass,
                border: Border::default().rounded(1.0),
                ..renderer::Quad::default()
            },
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(std::f32::consts::PI))
                    .add_stop(0.0, material.glass)
                    .add_stop(1.0, material.glass_low),
            )),
        );
        // What the surround does to the glass under it: a line of its own shadow
        // across the top, and its own lit lower edge coming back off the foot.
        // Both land in the dead border rather than over any dot, which is what
        // the dead border is for.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    height: MARGIN,
                    ..glass
                },
                ..renderer::Quad::default()
            },
            Background::Color(Color {
                a: CAST,
                ..material.recess
            }),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    y: glass.y + glass.height - MARGIN,
                    height: MARGIN,
                    ..glass
                },
                ..renderer::Quad::default()
            },
            Background::Color(Color {
                a: CATCH,
                ..material.metal_high
            }),
        );

        // The ink is the claim's, which is the theme's answer — except on the
        // press that turns the displays over, where the whole point is that the
        // glass is the other one and the ink has to be the other one with it.
        let ink = match self.polarity {
            Some(_) => material.ink,
            None => written(theme, self.claim),
        };
        print_dots(
            renderer,
            &self.screen,
            bounds.x + SURROUND,
            bounds.y + SURROUND,
            ink,
        );
    }
}

/// Lays every printed dot of `screen` down, with its top left dot's cell at
/// `x`, `y`.
///
/// The one place a dot's size and its place in the grid are decided, because a
/// screen printed on the panel and a screen printed on glass are the same dots
/// at the same pitch — the glass is what is not the same, and it is drawn
/// before this is called or not at all.
fn print_dots<Renderer>(renderer: &mut Renderer, screen: &Screen, x: f32, y: f32, colour: Color)
where
    Renderer: iced_core::Renderer,
{
    let inset = (PITCH - LIT) / 2.0;
    for row in 0..screen.rows() {
        for column in 0..screen.columns() {
            if !screen.is_inked(column, row) {
                continue;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: x + points(column) * PITCH + inset,
                        y: y + points(row) * PITCH + inset,
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

#[cfg(test)]
mod tests {
    use super::{Band, Ink, Screen, Size};

    #[test]
    fn a_name_that_fits_its_field_does_not_move_and_one_that_does_not_stays_inside_it() {
        // Two claims and they are the whole of what a scrolling field promises.
        // A short name is a still name: a page where everything moves is a page
        // nobody can read. A long one is cut to the field by the field rather
        // than by its own tail, so a name three characters too long is three
        // characters that arrive rather than three that are lost.
        let across = Screen::width_of("123456", Size::Small);
        let mut fits = Screen::new(40, 7);
        fits.marquee(2, 0, across, "abc", Size::Small);
        let mut written = Screen::new(40, 7);
        written.write(2, 0, "abc", Size::Small);

        for row in 0..7 {
            for column in 0..40 {
                assert_eq!(
                    fits.is_inked(column, row),
                    written.is_inked(column, row),
                    "a name that fits moved at {column},{row}"
                );
            }
        }

        let mut long = Screen::new(40, 7);
        long.marquee(2, 0, across, "a name nobody has room for", Size::Small);

        assert!(!long.is_blank(), "a scrolling name is drawn at all");
        for row in 0..7 {
            for column in 0..40 {
                assert!(
                    !long.is_inked(column, row) || (2..2 + across).contains(&column),
                    "a scrolling name is outside its field at {column},{row}"
                );
            }
        }
    }

    /// How many dots of a screen are printed.
    fn inked(screen: &Screen) -> usize {
        (0..screen.rows())
            .flat_map(|row| (0..screen.columns()).map(move |column| (column, row)))
            .filter(|(column, row)| screen.is_inked(*column, *row))
            .count()
    }

    #[test]
    fn a_new_screen_is_blank() {
        let screen = Screen::new(40, 20);

        assert!(screen.is_blank());
        assert_eq!(inked(&screen), 0);
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

        assert!(screen.is_inked(4, 4), "the part of the line that is on it");
        assert!(!screen.is_inked(0, 4));
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
                screen.is_inked(2, row),
                "the column the drop happens in has a gap at row {row}"
            );
        }
    }

    #[test]
    fn a_curve_reaches_both_ends_of_its_band() {
        let mut screen = Screen::new(10, 10);
        screen.curve(screen.all(), Ink::Solid, |x| x);

        assert!(screen.is_inked(0, 9), "nothing at the left end");
        assert!(screen.is_inked(9, 0), "nothing at the right end");
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
        assert!(!screen.is_blank());
    }

    #[test]
    fn a_word_too_wide_for_the_screen_is_clipped_rather_than_wrapped() {
        // A display clips. Sixteen characters of a program name on a plate's
        // strip is what a screen that narrow has room for, and a name that
        // wrapped onto the drawing under it would be worse than a short one.
        let mut screen = Screen::new(12, 8);
        screen.write(0, 0, "a name far too long for this", Size::Small);

        assert!(!screen.is_blank());
    }

    #[test]
    fn a_wash_is_half_the_dots_and_an_inversion_is_the_other_half() {
        let band = Band::new(0, 0, 10, 10);
        let mut washed = Screen::new(10, 10);
        washed.wash(band);
        let mut inverted = washed.clone();
        inverted.invert(band);

        assert_eq!(inked(&washed), 50);
        assert_eq!(inked(&inverted), 50);
        assert_ne!(washed.is_inked(0, 0), inverted.is_inked(0, 0));
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
    fn a_screen_of_a_line_is_the_line_and_no_glass_around_it() {
        // What a stencil is cut from: the grid is the writing, so a caller that
        // centres one in a gutter is centring the digit rather than a box with
        // a digit somewhere in it.
        let screen = Screen::of("3", Size::Small);

        assert_eq!(screen.columns(), Screen::width_of("3", Size::Small));
        assert_eq!(screen.rows(), Screen::height_of(Size::Small));
        assert!(screen.is_inked(0, 0), "the glyph starts in the corner");
        assert_eq!(
            Screen::of("3", Size::Large).rows(),
            screen.rows() * 2,
            "the other size is the same cell drawn twice as large"
        );
    }

    #[test]
    fn a_screen_says_its_size_and_how_much_of_it_is_inked() {
        let mut screen = Screen::new(6, 3);
        screen.dot(1, 1);

        assert_eq!(format!("{screen:?}"), "Screen(6 x 3, 1 printed)");
    }
}

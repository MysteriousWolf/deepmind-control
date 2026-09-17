//! The marks the window's own presses wear.
//!
//! Every press in this window's chrome asks the instrument something or says
//! something about the window: ask who is there, read the edit buffer, look for
//! ports again, open one, put one down, say what is known about the cable. They
//! were words, and a row of words beside the one press that is a picture, the
//! display that shows which way up the glass is about to be, was a row where
//! one press was drawn and the rest were labelled.
//!
//! # Why they are dots
//!
//! Because everything small in this window is. A display writes its characters
//! as a five by seven cell of dots; an effect engine's case carries its number
//! in that cell; a routing's row carries its number in it; the patch bay writes
//! its names in it. A press with a line-drawn icon on it would be the only
//! small drawing here made of anything else.
//!
//! So a mark is a grid, `SIDE` square, and it is stencilled onto the panel
//! rather than lit on glass. [`crate::stencil`] is what draws it, which is the
//! same call the numbers on a case and in a matrix row go through. The
//! exception is the press that turns the displays over: that one is a display,
//! because what it is about *is* the display.
//!
//! # Why nine and not seven
//!
//! Seven is the cell a *character* stands in, and these are not characters.
//! Nine is the smallest odd grid with a middle dot, a dot either side of it and
//! a dot either side of those, which is what a circle, an arrow and a plug all
//! need before they stop being suggestions. At the pitch every display here
//! shares that is twenty-two points, which is what a press this window's height
//! has room for.
//!
//! # What they are not
//!
//! They are not a typeface and they are not anybody's icon set. Each one is the
//! smallest arrangement of dots that says the thing: a socket is the five pins
//! a `DIN` plug has, a read is an arrow coming down into a tray, an inquiry is
//! a question mark, a rescan is an arrow going round. Where a grid this size
//! has no honest answer the press keeps its word, which is why the words are
//! still there beside them.

use crate::lcd::Screen;

/// How many dots across the marks a press in the window's chrome wear are
/// drawn, and how many down.
pub const SIDE: i32 = 9;

/// How wide the two arrows beside a matrix row are drawn.
///
/// Seven, which is the cell the display writes a character in. They stand above
/// and below a numeral written in that cell, in a row as tall as one line of
/// controls: a nine-dot arrow there is an arrow taller than the number it
/// moves, and a five-dot one is nine dots of ink with gaps between them, which
/// at this pitch is a smudge rather than a direction.
pub const ARROW: i32 = 7;

/// A mark, as the rows of dots it is drawn from.
///
/// Row-major from the top, one bit per dot, the dot at the left in the bit at
/// `across - 1`. Written out rather than computed because at these sizes there
/// is nothing to compute: the drawing *is* the numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    /// The dots, one row per number.
    rows: &'static [u16],
    /// How many of each number are the drawing.
    across: i32,
}

impl Badge {
    /// A mark `across` dots wide and as many deep as it has rows.
    const fn new(rows: &'static [u16], across: i32) -> Self {
        Self { rows, across }
    }

    /// How many dots across this mark is drawn.
    #[must_use]
    pub const fn across(self) -> i32 {
        self.across
    }

    /// How many dots down it is, which is how many rows it was written as.
    #[must_use]
    pub fn down(self) -> i32 {
        i32::try_from(self.rows.len()).unwrap_or_default()
    }

    /// Returns this mark as a screen, ready to be stencilled onto the panel.
    #[must_use]
    pub fn screen(self) -> Screen {
        let mut screen = Screen::new(self.across, self.down());
        for (row, dots) in self.rows.iter().copied().enumerate() {
            let row = i32::try_from(row).unwrap_or_default();
            for column in 0..self.across {
                if dots & (1 << (self.across - 1 - column)) != 0 {
                    screen.dot(column, row);
                }
            }
        }
        screen
    }
}

/// Ask who is there: a question mark.
///
/// The one press that asks the instrument to say what it is, and a question
/// mark is what a question looks like in every grid this size ever drawn.
pub const WHO: Badge = Badge::new(
    &[
        0b0_0111_1100,
        0b0_1100_0110,
        0b0_1000_0010,
        0b0_0000_0110,
        0b0_0001_1100,
        0b0_0011_0000,
        0b0_0011_0000,
        0b0_0000_0000,
        0b0_0011_0000,
    ],
    SIDE,
);

/// Read the edit buffer: an arrow coming down into a tray.
///
/// What is being asked for arrives here, which is the direction the arrow
/// points; the tray is the line it lands on, and it is open at the top because
/// something is still coming.
pub const READ: Badge = Badge::new(
    &[
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_1001_0010,
        0b0_0101_0100,
        0b0_0011_1000,
        0b0_0001_0000,
        0b0_1111_1110,
    ],
    SIDE,
);

/// Look for ports again: a magnifier.
///
/// A ring with a handle, which is what looking for something is drawn as in
/// every grid this size ever made. A circular arrow is three quarters of a ring
/// with a stub on the end of it, which at nine dots is a broken circle with
/// specks round it rather than a thing going round. What this press does
/// is go and *look*, and a magnifier is a shape with two parts rather than a
/// shape with a gap in it.
pub const RESCAN: Badge = Badge::new(
    &[
        0b0_1110_0000,
        0b1_0001_0000,
        0b1_0001_0000,
        0b1_0001_0000,
        0b0_1110_0000,
        0b0_0001_1000,
        0b0_0000_1100,
        0b0_0000_0110,
        0b0_0000_0011,
    ],
    SIDE,
);

/// Open a port: an empty `DIN` socket.
///
/// The connector this whole application arrives through, drawn the way it is
/// stamped on the back of every instrument that has one: a ring, and the five
/// pins in the arc the standard puts them in. Empty, because the press is the
/// one that puts something in it.
///
/// The ring is two dots thick at its shoulders. A hairline circle with five
/// single dots inside it is, at this size, a dotted circle with specks in the
/// middle: the ring has to be a ring before the pins read as pins.
pub const PORT: Badge = Badge::new(
    &[
        0b0_0111_1100,
        0b0_1000_0010,
        0b1_0010_1001,
        0b1_0000_0001,
        0b1_0100_0101,
        0b1_0001_0001,
        0b0_1000_0010,
        0b0_0111_1100,
        0b0_0000_0000,
    ],
    SIDE,
);

/// Put the port down: the same socket with a plug in it.
///
/// Two presses that do the same thing to the same port would otherwise differ
/// only by the word beside them, and the words are in the footer, so the mark is
/// what has to say which of the two this is. A socket with
/// something in it is a port that is open, which is a fact about the cable
/// rather than an instruction, and it reads at a glance the way a lit lamp
/// does.
pub const PLUGGED: Badge = Badge::new(
    &[
        0b0_0111_1100,
        0b0_1000_0010,
        0b1_0011_1001,
        0b1_0111_1101,
        0b1_0111_1101,
        0b1_0011_1001,
        0b0_1000_0010,
        0b0_0111_1100,
        0b0_0000_0000,
    ],
    SIDE,
);

/// What is known about the instrument: a lower-case `i`, set as a mark.
///
/// The one press whose subject is this window rather than the instrument, and
/// the one place a letter is the honest drawing: an inquiry has no shape and
/// every reader of every interface knows this one.
pub const ABOUT: Badge = Badge::new(
    &[
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0000_0000,
        0b0_0011_0000,
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0011_1000,
    ],
    SIDE,
);

/// Send this routing out into the window: a reticle.
///
/// What the press does is put the routing over the panel and wait for somebody
/// to take hold of a control, so the mark is the thing you aim: a ring with a
/// crosshair through it and a dot in the middle is what aiming has looked like
/// for as long as anything has been aimed.
///
/// A word is the honest mark where a grid this size has no answer, which is why
/// [`ABOUT`] is still a letter, but
/// this one has an answer, and a press whose face is a picture is a press that
/// does not have to be read. The word has not gone anywhere: it is in the
/// footer, as the sentence the pointer brings up, the same as every other press
/// in this window.
///
/// The centre dot is the whole of why it is a reticle rather than a ring. A
/// ring is a thing; a ring with something in the middle of it is a thing
/// pointed at something.
pub const MAP: Badge = Badge::new(
    &[
        0b0_0001_0000,
        0b0_0001_0000,
        0b0_0011_1000,
        0b0_0100_0100,
        0b1_1101_0111,
        0b0_0100_0100,
        0b0_0011_1000,
        0b0_0001_0000,
        0b0_0001_0000,
    ],
    SIDE,
);

/// Put what is open away: a cross.
///
/// The one press whose subject is a window rather than the instrument or the
/// sound in it, and the only shape that has ever meant this. A cross is two
/// strokes corner to corner, so at nine dots it is the one mark here that
/// needs no thickening: the diagonals cross at the middle dot, which is what
/// [`SIDE`] is an odd number for.
///
/// It is the third of the three ways out of a modal and the only one that is
/// drawn. The other two are the key and the panel around the sheet, neither of
/// which is a thing on the screen, and a sheet whose only way out is a gesture
/// nobody was told about is a sheet somebody is stuck behind.
pub const SHUT: Badge = Badge::new(
    &[
        0b1_0000_0001,
        0b0_1000_0010,
        0b0_0100_0100,
        0b0_0010_1000,
        0b0_0001_0000,
        0b0_0010_1000,
        0b0_0100_0100,
        0b0_1000_0010,
        0b1_0000_0001,
    ],
    SIDE,
);

/// Move this routing up the matrix: an arrow, pointing that way.
///
/// A solid triangle, [seven dots across](ARROW) and four down, because of where
/// it stands: above a numeral written in the display's own character cell, in a
/// row as tall as one line of controls, with its twin below. What it is not is
/// an arrow with a shaft: at this size a three-dot head on a one-dot stem reads
/// as a cross, and the head has to be most of the mark before anybody sees
/// which way it points.
///
/// Nor is it the typographer's `\u{25b2}` set at nine points, whose size and
/// weight are both the face's business, which is why two of those never looked
/// like a pair.
pub const UP: Badge = Badge::new(&[0b000_1000, 0b001_1100, 0b011_1110, 0b111_1111], ARROW);

/// Move it down: the same triangle, the other way up.
///
/// Drawn rather than flipped in code, because the two are what somebody
/// compares: a pair that is visibly one mark reflected is a pair, and a pair
/// that differs by a row is a mistake nobody can see and everybody can feel.
pub const DOWN: Badge = Badge::new(&[0b111_1111, 0b011_1110, 0b001_1100, 0b000_1000], ARROW);

#[cfg(test)]
mod tests {
    use super::{ABOUT, ARROW, Badge, DOWN, MAP, PLUGGED, PORT, READ, RESCAN, SHUT, SIDE, UP, WHO};

    /// Every mark this module publishes.
    const ALL: [(&str, Badge); 10] = [
        ("who", WHO),
        ("read", READ),
        ("rescan", RESCAN),
        ("port", PORT),
        ("plugged", PLUGGED),
        ("about", ABOUT),
        ("map", MAP),
        ("shut", SHUT),
        ("up", UP),
        ("down", DOWN),
    ];

    #[test]
    fn every_mark_fits_the_grid_it_is_drawn_in() {
        // Nine bits of a sixteen-bit row, and the seven above them are not a
        // drawing: a mark with a dot up there is a mark with a dot nobody can
        // see, which is a typing mistake rather than a design.
        for (name, badge) in ALL {
            for dots in badge.rows.iter().copied() {
                assert_eq!(
                    dots >> u32::try_from(badge.across()).unwrap_or_default(),
                    0,
                    "{name} has a dot off the right-hand side of its grid"
                );
            }
        }
    }

    #[test]
    fn every_mark_has_something_drawn_in_it() {
        for (name, badge) in ALL {
            let screen = badge.screen();
            assert!(!screen.is_blank(), "{name} is an empty grid");
            assert_eq!(screen.columns(), badge.across());
            assert_eq!(screen.rows(), badge.down());
            // One of the two widths this module draws in, and nothing between
            // them: a mark at a size nothing else on the panel is drawn at is a
            // mark somebody chose rather than a mark that fits where it goes.
            assert!(
                badge.across() == SIDE || badge.across() == ARROW,
                "{name} is {} dots across",
                badge.across()
            );
        }
    }

    #[test]
    fn the_two_arrows_are_one_mark_reflected() {
        // They stand one above the other in the same column, four points
        // apart, and a pair that differ by a row is a pair somebody reads as
        // wobbling without ever seeing why.
        let up = UP.screen();
        let down = DOWN.screen();

        for row in 0..UP.down() {
            for column in 0..ARROW {
                assert_eq!(
                    up.is_inked(column, row),
                    down.is_inked(column, UP.down() - 1 - row),
                    "the arrows differ at {column},{row}"
                );
            }
        }
    }

    #[test]
    fn a_mark_is_the_rows_it_was_written_as() {
        // The top row of the socket is its five top dots, and they are where
        // the row says they are: this is the one thing that can go wrong with a
        // drawing written as numbers, and it goes wrong silently.
        let screen = PORT.screen();

        assert!(!screen.is_inked(1, 0));
        assert!((2..=6).all(|column| screen.is_inked(column, 0)));
        assert!(!screen.is_inked(7, 0));
        // And the one with a plug in it is the same socket with its middle
        // filled: a pair that differed anywhere else would be two sockets.
        let plugged = PLUGGED.screen();
        for column in 0..SIDE {
            assert_eq!(screen.is_inked(column, 0), plugged.is_inked(column, 0));
            assert_eq!(screen.is_inked(column, 7), plugged.is_inked(column, 7));
        }
        assert!(plugged.is_inked(4, 3) && !screen.is_inked(4, 3));
    }
}

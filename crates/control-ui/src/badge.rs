//! The marks the window's own presses wear.
//!
//! Every press in this window's chrome asks the instrument something or says
//! something about the window: ask who is there, read the edit buffer, look for
//! ports again, open one, put one down, say what is known about the cable. They
//! were words, and a row of words beside the one press that is a picture — the
//! display that shows which way up the glass is about to be — was a row where
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
//! rather than lit on glass — [`crate::stencil`] is what draws it, which is the
//! same call the numbers on a case and in a matrix row go through. The
//! exception is the press that turns the displays over: that one is a display,
//! because what it is about *is* the display.
//!
//! # Why nine and not seven
//!
//! Seven is the cell a *character* stands in, and these are not characters.
//! Nine is the smallest odd grid with a middle dot, a dot either side of it and
//! a dot either side of those — which is what a circle, an arrow and a plug all
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
//! has no honest answer the press keeps its word — which is why the words are
//! still there beside them.

use crate::lcd::Screen;

/// How many dots across a mark is drawn, and how many down.
pub const SIDE: i32 = 9;

/// A mark, as the nine rows of nine dots it is drawn from.
///
/// Row-major from the top, one bit per dot, the high bit at the left. Written
/// out rather than computed because at this size there is nothing to compute:
/// the drawing *is* the nine numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge([u16; SIDE as usize]);

impl Badge {
    /// Returns this mark as a screen, ready to be stencilled onto the panel.
    #[must_use]
    pub fn screen(self) -> Screen {
        let mut screen = Screen::new(SIDE, SIDE);
        for (row, dots) in self.0.into_iter().enumerate() {
            let row = i32::try_from(row).unwrap_or_default();
            for column in 0..SIDE {
                if dots & (1 << (SIDE - 1 - column)) != 0 {
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
pub const WHO: Badge = Badge([
    0b0_0111_1100,
    0b0_1100_0110,
    0b0_1000_0010,
    0b0_0000_0110,
    0b0_0001_1100,
    0b0_0011_0000,
    0b0_0011_0000,
    0b0_0000_0000,
    0b0_0011_0000,
]);

/// Read the edit buffer: an arrow coming down into a tray.
///
/// What is being asked for arrives here, which is the direction the arrow
/// points; the tray is the line it lands on, and it is open at the top because
/// something is still coming.
pub const READ: Badge = Badge([
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_1001_0010,
    0b0_0101_0100,
    0b0_0011_1000,
    0b0_0001_0000,
    0b0_1111_1110,
]);

/// Look for ports again: an arrow that goes round.
///
/// Three quarters of a ring with a head on the end of it, which is the one
/// thing a grid this size can say about starting over.
pub const RESCAN: Badge = Badge([
    0b0_0011_1000,
    0b0_1100_0110,
    0b0_1000_0011,
    0b0_1000_0000,
    0b0_1000_0000,
    0b0_1000_0011,
    0b0_1100_0110,
    0b0_0111_1100,
    0b0_0011_0000,
]);

/// Open a port: the five pins of a `DIN` socket.
///
/// The connector this whole application arrives through, drawn the way it is
/// stamped on the back of every instrument that has one: a ring, a key notch at
/// the top, and five pins in the arc the standard puts them in.
pub const PORT: Badge = Badge([
    0b0_0011_1000,
    0b0_0100_0100,
    0b0_1001_0010,
    0b0_1000_0001,
    0b0_1010_0101,
    0b0_1000_0001,
    0b0_1001_0010,
    0b0_0100_0100,
    0b0_0011_1000,
]);

/// What is known about the instrument: a lower-case `i`, set as a mark.
///
/// The one press whose subject is this window rather than the instrument, and
/// the one place a letter is the honest drawing — an inquiry has no shape and
/// every reader of every interface knows this one.
pub const ABOUT: Badge = Badge([
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0000_0000,
    0b0_0011_0000,
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0001_0000,
    0b0_0011_1000,
]);

#[cfg(test)]
mod tests {
    use super::{ABOUT, Badge, PORT, READ, RESCAN, SIDE, WHO};

    /// Every mark this module publishes.
    const ALL: [(&str, Badge); 5] = [
        ("who", WHO),
        ("read", READ),
        ("rescan", RESCAN),
        ("port", PORT),
        ("about", ABOUT),
    ];

    #[test]
    fn every_mark_fits_the_grid_it_is_drawn_in() {
        // Nine bits of a sixteen-bit row, and the seven above them are not a
        // drawing: a mark with a dot up there is a mark with a dot nobody can
        // see, which is a typing mistake rather than a design.
        for (name, badge) in ALL {
            for dots in badge.0 {
                assert_eq!(
                    dots >> u32::try_from(SIDE).unwrap_or_default(),
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
            assert_eq!(screen.columns(), SIDE);
            assert_eq!(screen.rows(), SIDE);
        }
    }

    #[test]
    fn a_mark_is_the_rows_it_was_written_as() {
        // The top row of the socket is its three top dots, and they are where
        // the row says they are: this is the one thing that can go wrong with a
        // drawing written as numbers, and it goes wrong silently.
        let screen = PORT.screen();

        assert!(!screen.is_inked(2, 0));
        assert!(screen.is_inked(3, 0) && screen.is_inked(4, 0) && screen.is_inked(5, 0));
        assert!(!screen.is_inked(6, 0));
    }
}

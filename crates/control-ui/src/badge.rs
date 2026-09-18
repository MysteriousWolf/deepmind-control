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

/// The modulation matrix: a grid of cells.
///
/// What the surface behind this press *is*: eight routings, each with a source,
/// a destination and a depth, which is a table. A fan of one source into three
/// destinations says what the matrix is for and, at nine dots, says it as a
/// trunk with a crossbar: the branches and the wire that feeds them land on the
/// same rows and what is left is a plus sign.
pub const MATRIX: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1101_1011,
        0b0_1101_1011,
        0b0_0000_0000,
        0b0_1101_1011,
        0b0_1101_1011,
        0b0_0000_0000,
        0b0_1101_1011,
        0b0_1101_1011,
    ],
    SIDE,
);

/// The effects: a unit with the signal running through it.
///
/// One box and not the four the block holds. At nine dots a box is three across
/// and its inside is a single dot, so four of them are four blobs in a line and
/// two are two; one box with room inside it is a *unit*, and the wire going in
/// one side and out the other is what says the signal passes through it.
pub const CHAIN: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0111_1100,
        0b0_0100_0100,
        0b0_0100_0100,
        0b1_1100_0111,
        0b0_0100_0100,
        0b0_0100_0100,
        0b0_0111_1100,
        0b0_0000_0000,
    ],
    SIDE,
);

/// The control sequencer: three steps, each one taller than the last.
///
/// A step sequencer's own picture. Bars of a height are a level meter and bars
/// that climb are a sequence, so the climb is the whole of the drawing; three
/// of them, because two is a comparison and four at this width is a comb.
pub const STEPS: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0000_0011,
        0b0_0000_0011,
        0b0_0001_1011,
        0b0_0001_1011,
        0b0_1101_1011,
        0b0_1101_1011,
        0b0_1101_1011,
        0b0_0000_0000,
    ],
    SIDE,
);

/// The program: a card with writing on it.
///
/// What is behind this press is the sound's name, its category and the settings
/// that are about the program rather than about the sound, which is a label on
/// a thing rather than a part of it. Two lines inside a border, because one
/// line is a box with a bar in it.
pub const PROGRAM: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1111_1110,
        0b0_1000_0010,
        0b0_1011_1010,
        0b0_1000_0010,
        0b0_1011_1010,
        0b0_1000_0010,
        0b0_1111_1110,
        0b0_0000_0000,
    ],
    SIDE,
);

/// The front panel itself: three faders, each one set differently.
///
/// The way back out of a section, and what it goes back to is a rack of faders:
/// the one drawing that is this window's first surface rather than any part of
/// it. Three tracks with a cap across each, at three heights, because three caps
/// at one height is a comb and one fader alone is a line with a lump on it.
///
/// A cap is two rows deep and not one. One row is a track crossed by a bar,
/// which at this pitch is a plus sign three times over; two is a thing sitting
/// *on* the track, which is what a fader cap is and what tells this mark from
/// the matrix's grid at a glance.
pub const PANEL: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1011_1010,
        0b0_1011_1010,
        0b0_1001_0111,
        0b0_1001_0111,
        0b1_1101_0010,
        0b1_1101_0010,
        0b0_1001_0010,
        0b0_0000_0000,
    ],
    SIDE,
);

/// Three dots: the press that puts you where the thing is changed.
///
/// It was a pencil, and before that a display, and the pencil was redrawn five
/// times: a pencil at nine dots is a diagonal band, and a diagonal band is
/// either a blob or a stroke. What finally read was two edges and a point —
/// legible, and still a drawing of an *instrument* rather than of what the
/// press does.
///
/// A pencil says *write here*, and nothing behind this press is written. What
/// is behind it is the rest of the section: the plate on the panel carries the
/// four or five controls a hand reaches for, and the press opens the twenty the
/// plate had no room for. That is the ellipsis every toolbar in every
/// application has meant by three dots since menus had them — *and more* — and
/// it is the one mark in this module that needed no invention at all.
///
/// Three dots, one apart, on the middle line, two by two each.
///
/// They were single dots, which is the honest thing for a display to draw an
/// ellipsis as and too light for a press: three specks on a cap the size of a
/// thumb is a cap that looks blank until you go looking. Two by two is a dot
/// with weight and still a dot.
///
/// Nine columns cannot hold three two-wide blocks symmetrically — six of ink
/// and two of gap is eight — so the spare column is on the left and the
/// drawing sits half a dot right of centre, which at this pitch is a point.
pub(crate) const MORE: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_1101_1011,
        0b0_1101_1011,
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A power symbol: the press that turns a thing on and off.
///
/// The broken ring with the bar through it, which is the one mark in this set
/// that nobody has to be told. It is on the arpeggiator, where the instrument
/// silkscreens `ON/OFF`.
pub(crate) const POWER: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0001_0000,
        0b0_0101_0100,
        0b0_1001_0010,
        0b0_1000_0010,
        0b0_1000_0010,
        0b0_0100_0100,
        0b0_0011_1000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A snowflake: what is sounding now goes on sounding.
///
/// The arpeggiator's `HOLD`, which keeps the notes playing after the keys are
/// let go. It was a padlock, because a latch is the thing a padlock is — and a
/// padlock on a panel is what a *locked* control looks like, which is the one
/// thing this press is not: nothing here is protected, and the arpeggio is not
/// prevented from being changed. It is frozen where it is.
///
/// So the mark is the freeze every transport and every synthesizer front panel
/// already draws, the six-point star: three axes through one middle dot, each
/// with the branch a snowflake has. Nine dots is exactly enough for it — the
/// middle row is the full width, the two diagonals fall a dot at a time, and
/// the vertical is the one column that reaches both edges.
pub(crate) const FREEZE: Badge = Badge::new(
    &[
        0b0_0001_0000,
        0b0_1001_0010,
        0b0_0101_0100,
        0b0_0011_1000,
        0b1_1111_1111,
        0b0_0011_1000,
        0b0_0101_0100,
        0b0_1001_0010,
        0b0_0001_0000,
    ],
    SIDE,
);

/// A speaker: what the high-pass filter's `BOOST` puts back.
///
/// It was a low shelf — the response curve, lifted at the bottom and flat above
/// it — and the trouble with drawing a response on this panel is that a
/// response is what half the marks here already are: a step at nine dots is a
/// square wave with a leg missing, and on a row that already carries a sawtooth
/// and a pulse it reads as a third waveform.
///
/// So it is the thing you hear rather than the curve that makes it — and seen
/// from the front, which is the half of that decision that took two goes. A
/// cone drawn from the side is a wedge with a box behind it, and a wedge is as
/// much a tweeter as a woofer. A driver seen face on is a ring with a dust cap
/// in the middle of it, and at nine dots the cap is three by three: big, round
/// and obviously moving air.
pub(crate) const SPEAKER: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0111_1100,
        0b0_1100_0110,
        0b0_1011_1010,
        0b0_1011_1010,
        0b0_1011_1010,
        0b0_1100_0110,
        0b0_0111_1100,
        0b0_0000_0000,
    ],
    SIDE,
);

/// Two rings, linked: one thing running in step with another.
///
/// `SYNC` on the oscillators, where the second is restarted by the first. It
/// kept its word until now under the rule this module is written to — where a
/// grid this size has no honest answer, the press keeps its word — and what
/// makes the word unnecessary is that the honest answer was never a picture of
/// *oscillator* sync. Every drawing of that at nine dots is either the sawtooth
/// already on the cap two along or the arrow the chrome spends on a rescan.
/// Linked rings are a picture of *sync*, which is what the press is called and
/// what it does.
pub(crate) const LOCKED: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_0110_1100,
        0b0_1001_0010,
        0b0_1001_0010,
        0b0_1001_0010,
        0b0_0110_1100,
        0b0_0000_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// The shelf: a row of sounds standing side by side, on the shelf they stand on.
///
/// The library is a bank of programs and nothing else in this window is, so the
/// mark is the thing itself rather than an idea about it. Books on a shelf,
/// which is the one arrangement at nine dots that is plainly *many of one kind
/// of thing, kept* — a grid would be the matrix's mark and a card would be the
/// program's.
pub(crate) const SHELF: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1010_1010,
        0b0_1010_1010,
        0b0_1010_1010,
        0b0_1010_1010,
        0b0_1010_1010,
        0b0_1010_1010,
        0b0_1111_1110,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A sawtooth, which is what the panel prints over one of the two presses that
/// choose an oscillator's waveform.
///
/// The hardware prints the wave itself and no word, because the wave *is* the
/// name: a ramp rising to the right and dropping straight back is a sawtooth
/// wherever it is drawn, and `SAW` set in the panel's own caps would be this
/// window explaining a symbol every synthesizer shares.
pub(crate) const SAW: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0000_0010,
        0b0_0000_0110,
        0b0_0000_1010,
        0b0_0001_0010,
        0b0_0010_0010,
        0b0_0100_0010,
        0b0_1000_0010,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A pulse with a dotted falling edge, which is the other one.
///
/// Traced off the instrument rather than invented, a dot at a time off a
/// photograph of the panel, which took three goes and is the only way this one
/// was ever going to come out right. Every other waveform on the front is drawn
/// solid; this is the one that carries a dotted line, because the `PWM` fader
/// beside the press is what moves the edge it marks.
///
/// What the instrument draws is a *narrow* pulse and then a dotted line
/// somewhere out in the low part of the wave: a tall thin `⊓` at the left, the
/// low bar running right from its falling edge, the next rising edge at the far
/// right, and a dashed vertical standing between them. The dashes are not the
/// falling edge drawn dotted — they are a second, later falling edge, which is
/// where `PWM` would move the one that is drawn. A width, shown as the two
/// places its edge can be.
///
/// It runs off both sides of the grid, which the instrument's does not have to.
/// A wave that starts and stops inside its own frame is a glyph; one that
/// enters at the bottom left and leaves at the top right is a wave with more of
/// it either side, which is what a cycle of anything is. Two dots at each
/// corner buy that.
pub(crate) const PULSE: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1110_0011,
        0b0_1010_1010,
        0b0_1010_0010,
        0b0_1010_1010,
        0b0_1010_0010,
        0b0_1010_1010,
        0b1_1011_1110,
        0b0_0000_0000,
    ],
    SIDE,
);

/// One cycle of a sine: an `LFO`.
///
/// The slowest thing on the instrument and the one whose shape is the whole
/// point of it, so the mark is the shape. Crossings at the two edges and the
/// middle, the peak and the trough a third of the way in from each end, and a
/// dot in the column of each crossing so the curve does not break into two arcs
/// where it is steepest — which is the only place nine dots cannot follow it.
pub(crate) const SINE: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0100_0000,
        0b0_1010_0000,
        0b1_0001_0000,
        0b1_0001_0001,
        0b0_0001_0001,
        0b0_0000_1010,
        0b0_0000_0100,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A low-pass response: flat, a corner, and down. The `VCF`.
///
/// The one drawing the filter's own display already makes, at a fortieth of the
/// dots: everything below the corner comes through and everything above it
/// does not. It is a response curve, which this module says elsewhere is a
/// dangerous thing to draw at this size because a response and a waveform are
/// the same few dots — and here it is safe, because nothing beside it is a
/// waveform: the sheet titles are one section each and no two of them are the
/// same kind of picture.
pub(crate) const SLOPE: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0000_0000,
        0b1_1111_0000,
        0b0_0000_1000,
        0b0_0000_0100,
        0b0_0000_0010,
        0b0_0000_0001,
        0b0_0000_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// An attack, a decay, a sustain and a release: an envelope.
///
/// All three of them — the one on the amplifier, the one on the filter and the
/// spare — because what they are is the same thing and which one is open is
/// what the word beside the mark says. The contour is the four stages in
/// order, with the rise and the fall carried by the column they leave rather
/// than broken into dots nobody would join up.
pub(crate) const CONTOUR: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0100_0000,
        0b0_1010_0000,
        0b0_1010_0000,
        0b0_1001_0000,
        0b1_0001_1100,
        0b1_0000_0010,
        0b1_0000_0001,
        0b0_0000_0000,
    ],
    SIDE,
);

/// The amplifier triangle: the `VCA`.
///
/// The symbol every schematic ever drawn uses for gain, pointing the way the
/// signal goes. It is the one solid mark in this module and the one that should
/// be: a triangle drawn as its outline at nine dots is three lines that do not
/// meet, and what it is a picture of is a block rather than a shape.
pub(crate) const AMPLIFIER: Badge = Badge::new(
    &[
        0b0_0100_0000,
        0b0_0110_0000,
        0b0_0111_0000,
        0b0_0111_1000,
        0b0_0111_1100,
        0b0_0111_1000,
        0b0_0111_0000,
        0b0_0110_0000,
        0b0_0100_0000,
    ],
    SIDE,
);

/// Three notes sounding at once: the voicing.
///
/// What the section is about is how many of the twelve voices one key takes and
/// how the rest are shared out, which is a count rather than a picture. Three
/// blocks in a triangle is the count: more than one, together, and arranged —
/// which is unison, polyphony and the order they are assigned in, the three
/// things on the plate.
pub(crate) const VOICES: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0011_1000,
        0b0_0011_1000,
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_1110_1110,
        0b0_1110_1110,
        0b0_0000_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// A run of notes climbing, twice: the arpeggiator.
///
/// A piano roll rather than a waveform, which is what an arpeggio is: the held
/// chord played one note at a time, up, and then again. Single dots and not
/// bars, because the gaps between them are the pattern — a run drawn as a solid
/// diagonal is a glissando, and the thing that makes an arpeggio an arpeggio is
/// that it is steps.
pub(crate) const ARPEGGIO: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_0010_0010,
        0b0_0000_0000,
        0b0_0100_0100,
        0b0_0000_0000,
        0b0_1000_1000,
        0b0_0000_0000,
        0b1_0001_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// Three notes sounding together: the arpeggiator's `CHORD`.
///
/// A piano roll again, which is what [`ARPEGGIO`] is: three bars, one above
/// another, at the same moment. An arpeggio is a chord played one note at a
/// time and a chord is the same notes at once, so the two marks are the same
/// picture rearranged — which is the relationship the two presses have on the
/// panel, sitting in the same row.
pub(crate) const CHORD: Badge = Badge::new(
    &[
        0b0_0000_0000,
        0b0_1111_1110,
        0b0_0000_0000,
        0b0_1111_1110,
        0b0_0000_0000,
        0b0_1111_1110,
        0b0_0000_0000,
        0b0_0000_0000,
        0b0_0000_0000,
    ],
    SIDE,
);

/// Two of those, one under the other: `POLY CHORD`.
///
/// `CHORD` latches one chord and transposes it under every key; `POLY CHORD`
/// puts a *different* chord under each. So the mark is two chords rather than
/// one, at two pitches, which is the difference and the only part of it nine
/// dots can carry. They share the middle row, where the two overlap, because
/// three bars and three bars with a gap between them is six bars.
pub(crate) const POLY_CHORD: Badge = Badge::new(
    &[
        0b1_1110_0000,
        0b0_0000_0000,
        0b1_1110_0000,
        0b0_0000_0000,
        0b1_1110_1111,
        0b0_0000_0000,
        0b0_0000_1111,
        0b0_0000_0000,
        0b0_0000_1111,
    ],
    SIDE,
);

#[cfg(test)]
mod tests {
    use super::{ABOUT, AMPLIFIER, ARPEGGIO, ARROW, Badge, CHAIN, CHORD, CONTOUR, DOWN, MAP};
    use super::{FREEZE, LOCKED, MATRIX, MORE, PANEL, PLUGGED, POLY_CHORD, PORT, POWER};
    use super::{PROGRAM, PULSE, READ, RESCAN};
    use super::{SAW, SHELF, SHUT, SIDE, SINE, SLOPE, SPEAKER, STEPS, UP, VOICES, WHO};

    /// Every mark this module publishes.
    ///
    /// All of them, and it is a list somebody has to add to: the four the band
    /// of ways in wears were written under the ten the chrome wears and were
    /// checked by nothing for it.
    const ALL: [(&str, Badge); 31] = [
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
        ("panel", PANEL),
        ("matrix", MATRIX),
        ("chain", CHAIN),
        ("steps", STEPS),
        ("program", PROGRAM),
        ("saw", SAW),
        ("pulse", PULSE),
        ("shelf", SHELF),
        ("more", MORE),
        ("power", POWER),
        ("freeze", FREEZE),
        ("speaker", SPEAKER),
        ("locked", LOCKED),
        ("sine", SINE),
        ("slope", SLOPE),
        ("contour", CONTOUR),
        ("amplifier", AMPLIFIER),
        ("voices", VOICES),
        ("arpeggio", ARPEGGIO),
        ("chord", CHORD),
        ("poly chord", POLY_CHORD),
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

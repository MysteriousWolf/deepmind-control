//! The face the displays are written in, which is the fourth one and the only
//! one that is dots.
//!
//! `style.rs` asks the machine for the three faces the window is set in, and a
//! dot-matrix display cannot ask anybody for anything: it has a grid, and a
//! letter on it is whichever dots of that grid are lit. So the glyphs are here,
//! five dots across and seven down, which is the cell every display of this
//! kind has used since they were made — small enough that two lines fit on a
//! plate's screen, and large enough that a `5` is not an `S`.
//!
//! It is not a transcription of anybody's typeface. Five by seven is a grid
//! with one obvious letter in most of its cells, these are those letters, and
//! the ones with a choice in them — the single-storey `a`, the tailless `g` —
//! were chosen to be read at three points rather than to look like anything.
//!
//! A character the table has no glyph for is drawn as the hollow box every
//! display draws for one, rather than as a space: a name with a letter missing
//! should say that a letter is missing.

/// How many dots wide a glyph is.
pub(crate) const WIDTH: i32 = 5;

/// How many dots tall.
pub(crate) const HEIGHT: i32 = 7;

/// How many dots are left between two of them.
pub(crate) const GAP: i32 = 1;

/// How far the pen moves from one character to the next.
pub(crate) const ADVANCE: i32 = WIDTH + GAP;

/// The first character the table holds.
const FIRST: char = ' ';

/// What is drawn for a character the table does not hold.
const MISSING: [u8; 7] = [
    0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111,
];

/// Returns the dots `character` is drawn as, a row at a time from the top.
///
/// Each row is five bits, the leftmost dot in the highest of them.
pub(crate) fn of(character: char) -> [u8; 7] {
    let Some(index) = (character as u32).checked_sub(FIRST as u32) else {
        return MISSING;
    };
    usize::try_from(index)
        .ok()
        .and_then(|index| GLYPHS.get(index))
        .copied()
        .unwrap_or(MISSING)
}

/// Every printable character, from the space to the tilde.
///
/// Written a row at a time so that each line of the table is the letter it
/// draws, read downwards. A glyph edited here is a glyph that can be checked by
/// looking at it, which is the only review a bitmap font gets.
#[rustfmt::skip]
const GLYPHS: [[u8; 7]; 95] = [
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // the space
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100], // !
    [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // "
    [0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010], // #
    [0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100], // $
    [0b11001, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b10011], // %
    [0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101], // &
    [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // '
    [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010], // (
    [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000], // )
    [0b00000, 0b10101, 0b01110, 0b00100, 0b01110, 0b10101, 0b00000], // *
    [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000], // +
    [0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100, 0b01000], // ,
    [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000], // -
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100], // .
    [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000], // /
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110], // 0
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], // 1
    [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111], // 2
    [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110], // 3
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010], // 4
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110], // 5
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110], // 6
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000], // 7
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110], // 8
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100], // 9
    [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000], // :
    [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b01000], // ;
    [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010], // <
    [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000], // =
    [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000], // >
    [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100], // ?
    [0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110], // @
    [0b00100, 0b01010, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001], // A
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110], // B
    [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110], // C
    [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100], // D
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111], // E
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000], // F
    [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110], // G
    [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001], // H
    [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], // I
    [0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110], // J
    [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001], // K
    [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111], // L
    [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001], // M
    [0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001], // N
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110], // O
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000], // P
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101], // Q
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001], // R
    [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110], // S
    [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100], // T
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110], // U
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100], // V
    [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001], // W
    [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001], // X
    [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100], // Y
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111], // Z
    [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110], // [
    [0b10000, 0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00001], // \
    [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110], // ]
    [0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000], // ^
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111], // _
    [0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // `
    [0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111], // a
    [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110], // b
    [0b00000, 0b00000, 0b01110, 0b10001, 0b10000, 0b10001, 0b01110], // c
    [0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111], // d
    [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110], // e
    [0b00110, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000, 0b01000], // f
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b10001, 0b01110], // g
    [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001], // h
    [0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110], // i
    [0b00010, 0b00000, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100], // j
    [0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010], // k
    [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], // l
    [0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10001], // m
    [0b00000, 0b00000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001], // n
    [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110], // o
    [0b00000, 0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000], // p
    [0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001], // q
    [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000], // r
    [0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110], // s
    [0b01000, 0b01000, 0b11110, 0b01000, 0b01000, 0b01001, 0b00110], // t
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101], // u
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100], // v
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010], // w
    [0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001], // x
    [0b00000, 0b10001, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110], // y
    [0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111], // z
    [0b00011, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00011], // {
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100], // |
    [0b11000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b11000], // }
    [0b00000, 0b00000, 0b01001, 0b10101, 0b10010, 0b00000, 0b00000], // ~
];

#[cfg(test)]
mod tests {
    use super::{GLYPHS, MISSING, WIDTH, of};

    #[test]
    fn every_printable_character_has_a_glyph() {
        for character in ' '..='~' {
            assert_ne!(
                of(character),
                MISSING,
                "{character} is printable and is drawn as the missing box"
            );
        }
    }

    #[test]
    fn anything_else_is_drawn_as_the_box_that_says_so() {
        // A name with a letter the display cannot draw should say that a letter
        // is missing rather than quietly close up around it.
        for character in ['\u{00e8}', '\u{2014}', '\u{0007}', '\u{7f}'] {
            assert_eq!(of(character), MISSING);
        }
    }

    #[test]
    fn a_glyph_fits_the_cell_it_is_drawn_in() {
        let width = u8::try_from((1_u16 << WIDTH) - 1).unwrap_or(u8::MAX);
        for (index, glyph) in GLYPHS.iter().enumerate() {
            for row in glyph {
                assert!(
                    row & !width == 0,
                    "glyph {index} has a dot outside the cell"
                );
            }
        }
    }

    #[test]
    fn the_space_is_blank_and_nothing_else_is() {
        let blank = [0_u8; 7];
        let empty: Vec<usize> = GLYPHS
            .iter()
            .enumerate()
            .filter(|(_, glyph)| **glyph == blank)
            .map(|(index, _)| index)
            .collect();

        assert_eq!(empty, vec![0], "a character other than the space is blank");
    }
}

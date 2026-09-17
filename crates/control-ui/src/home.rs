//! The front panel: what the instrument shows before anybody goes looking.
//!
//! A `DeepMind` is two rows of section plates with a screen between them.
//! Twenty-odd faders under four-letter legends, a few lit buttons under each
//! group, and a yellow `EDIT` on every section that opens that section on the
//! display. The rest of the 242 parameters are behind one of those.
//!
//! This is that panel, and it is the window's home for the same reason it is
//! the instrument's: somebody who has just plugged a synthesizer in wants to
//! see the sound, not a list of fourteen sections. The controls here are the
//! ones Behringer put a fader under, drawn from the library's table like every
//! other control in this editor, and each section's way in is the same press
//! the hardware uses.
//!
//! # Nothing here is transcribed any more
//!
//! Which parameters have a physical control, what is silkscreened over each
//! one, what a hand touches and which row it is in used to be a table in this
//! file, the only one in this repository: the library published what exists and
//! what has a CC, and nothing that said what has a fader.
//!
//! `deepmind-midi` 26.3 publishes it: [`front::sections`] is that table, on the
//! side of the split the rest of the instrument lives on, with a loader check
//! that a plate's controls belong to the group its name claims. The table here
//! is deleted and `docs/waiting.md` records the ask as answered.
//!
//! # Two sections are drawn as more than one plate
//!
//! What is left is layout, which is this window's to decide: a hand layout
//! changes the arrangement and never what a control is. [`plates`] spends that
//! twice, in both cases because a window has room the front of a synthesizer
//! does not.
//!
//! The oscillators become `OSC 1` and `OSC 2`, which is the pair of brackets
//! the instrument prints inside its own `DCO 1 & 2` plate. The envelopes become
//! one plate each, because the hardware has four envelope faders, three
//! envelopes and a button to point the one set at the other, where a window can
//! draw twelve faders and three screens. They get a row of their own, which is
//! what the instrument had no room to give them.
//!
//! Both are derived rather than written down: the oscillator's bracket is a
//! slice of the library's own parameter name, and an envelope's four faders are
//! found by asking the library for the parameters whose short names match the
//! four the section carries. What a control *is* is still the library's answer
//! in every plate.
//!
//! # The arrangement is the instrument's, and so is the livery
//!
//! The hardware's buttons are white, amber and cyan, and this panel takes the
//! amber and the cyan for the two jobs it has that need a colour: amber opens a
//! section, which is what the instrument's `EDIT` does, and cyan marks a
//! control something other than a hand can move, which is what its `MOD` does.
//! A way in is a legend silkscreened on the panel over a lit square, because
//! that is what it is on the instrument and not a word in a box.
//!
//! The row of twelve lamps over the hardware's `POLY` section is missing for a
//! different reason: it says how many voices are sounding, and nothing on a
//! MIDI port says that. A lamp that cannot be lit honestly is not drawn.

use std::sync::LazyLock;

use deepmind_midi::front::{self, PanelControl, PanelShape, Section};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{Space, button, column, container, mouse_area, responsive, row, text};

use crate::envelope;
use crate::lcd::{self, Screen};
use crate::mapping::{Mapper, Sent};
use crate::matrix;
use crate::panel::{Message, Room, readout};
use crate::scene;
use crate::style::{materials, printed, reading};
use crate::{Element, Patch};

/// How wide one lane of the panel is.
///
/// Narrower than a rack's slot, because the instrument's own panel holds
/// eighteen of them across its lower row and a rack holds seven.
const LANE: f32 = 46.0;

/// How long a fader on the panel runs.
///
/// A third of a rack fader, which is the proportion the hardware uses: two rows
/// of controls and a screen fit on the front of a synthesizer because none of
/// its faders is as tall as a rack's.
const TRAVEL: f32 = 76.0;

/// How much room a lit set of legends is given beside the faders.
///
/// Wide enough for the longest name in the set it lights: a legend clipped to
/// `Triang` is a legend that lies about which shape is lit. A set with a name
/// missing lies about more than that, which is what [`lit`] is for.
///
/// The set it lights is the LFO's seven shapes and the longest of those is
/// `Sample & Glide`, which is fourteen characters of the reading face at the
/// size a legend is set in, and the room a legend is inset by either side. A
/// name that does not fit is drawn on a second line the strip has no room for,
/// so a strip a few points too narrow is a shape with half its name missing.
const LAMPS: f32 = 104.0;

/// How many dots tall the display over a plate's faders is.
///
/// Two lines of the dot font and a little room, which is as much as a strip
/// over a row of faders can take without the faders becoming the second thing
/// on the plate.
const STRIP: i32 = 20;

/// The fewest dots across a display is worth drawing on.
///
/// A plate narrower than this stands wider than its faders need so that its
/// display has somewhere to be: `VCA` is one fader, and one fader's worth of
/// glass is not a picture of an amplifier. It is the one place a drawing
/// changes the panel's arrangement rather than the other way about, and it is
/// the same trade the modulation matrix's rows make.
const NARROWEST: i32 = 40;

/// How tall the light bar across the top of a plate stands.
///
/// Given rather than taken, like the [legend](LEGEND) over a lane and for the
/// same reason: what a heading measures is what a face measures it at, and a
/// plate whose height depended on that is a plate this file cannot put a number
/// on. It has to put a number on it, because the screen between the rows is cut
/// to the plates either side of it.
const HEAD: f32 = 20.0;

/// How much room the reading under a lane is given.
///
/// The reading is the rack's own, at the rack's own size on every surface in
/// this window, so this does not grow with the panel around it.
const READ: f32 = 18.0;

/// How wide the screen is.
///
/// Fixed, and not what is left over. The two rows reflow when a window is
/// narrower than the panel, and a screen that took the rest of its row would
/// take the whole of it and push the voicing onto a line of its own.
const SCREEN: f32 = 340.0;

/// Height of the legend printed with a control, so that lanes line up whatever
/// they hold.
///
/// Two lines of it, because the panel prints `PITCH MOD` on two.
const LEGEND: f32 = 22.0;

/// How much room a button under the faders is given.
const SWITCH: f32 = 58.0;

/// How much room a way into a section is given.
///
/// The cap it holds and no more. The cap is the rack's own and does not stretch
/// with the panel (see [`way`]), so the lane it stands in starts at its width
/// and grows around it rather than under it.
const WAY: f32 = crate::panel::CAP;

/// How much panel there is above and below the display in the panel's own hole.
const GAP: f32 = 2.0;

/// How much panel there is between two plates of a row.
const ACROSS: f32 = 8.0;

/// How much panel there is between the two rows.
const DOWN: f32 = 10.0;

/// How far apart the parts of one plate stand.
const WITHIN: f32 = 6.0;

/// How much panel there is around a plate's contents.
///
/// Named rather than typed into the one call that uses it, because the display
/// over a plate's faders has to know it: a plate's width is the room it takes
/// on the panel, and what a display fits into is the room inside that.
const PAD: f32 = 6.0;

/// How far the parts of one control stand apart: its legend, it, and its
/// reading.
const APART: f32 = 4.0;

/// How far a button stands under its legend.
///
/// Closer than a fader stands under its own, because there is no reading
/// underneath it to leave room for.
const UNDER: f32 = 2.0;

/// How far the panel is stretched to fill the window it was given.
///
/// Every dimension on this panel is written down at the size the instrument's
/// own proportions want, and a window is whatever somebody dragged it to. A
/// panel drawn at its written size in a window twice as wide is a photograph of
/// a synthesizer pinned to the top left corner of a wall, which is what this
/// panel was until now.
///
/// So the numbers below are a proportion rather than a measurement: the widest
/// row is measured at its written size, divided into the room there actually
/// is, and everything is drawn through the result. That is the lanes, the travel
/// of a fader, the buttons, the type, and the gaps between and inside the
/// plates. Scaling the gaps is the half that decides whether it looks like an
/// instrument or like a panel with its parts pushed apart.
///
/// The displays are the one thing that does not simply get bigger. A screen
/// given more room gets more dots at the same pitch, which is what
/// [`lcd`](crate::lcd) is for, so a wider window is a filter curve drawn more
/// finely rather than a magnified one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Scale(f32);

impl Scale {
    /// The panel at the size its constants are written in.
    const NATURAL: Self = Self(1.0);

    /// As far as it is stretched, whatever the window does.
    ///
    /// A window on a wide desktop would otherwise draw a fader as long as an
    /// arm and a legend that had to be read from across the room. Past this the
    /// panel stays the size it is and sits in the middle of what it was given,
    /// which is what the instrument would do on the same desk.
    const MOST: f32 = 1.75;

    /// The scale at which the panel fills `available` points of window.
    ///
    /// Never smaller than written: a narrow window wraps its rows, which is the
    /// arrangement the panel already had for one, and shrinking the type to
    /// avoid that would trade a row somebody can find for a legend nobody can
    /// read.
    fn filling(available: f32) -> Self {
        let natural = widest();
        if natural <= 0.0 || !available.is_finite() {
            return Self::NATURAL;
        }
        Self((available / natural).clamp(1.0, Self::MOST))
    }

    /// `points` of the written panel, at this scale.
    fn of(self, points: f32) -> f32 {
        points * self.0
    }
}

/// How much room the panel wants, at the size its proportions are written in.
///
/// What a window opens at, so that the instrument's own arrangement is what
/// somebody sees first: below this the rows wrap, which is still readable and
/// is no longer two rows and a screen.
#[must_use]
pub fn panel_width() -> f32 {
    widest()
}

/// How wide the widest row of the panel stands at its written size.
///
/// What [`Scale::filling`] divides the window into. Measured off the same table
/// the panel is drawn from rather than written down beside it, so a plate that
/// gains a fader widens the row here too and the panel goes on filling the
/// window it is in.
fn widest() -> f32 {
    rows()
        .iter()
        .enumerate()
        .map(|(index, plates)| row_width(plates, index == 0, Scale::NATURAL))
        .fold(0.0_f32, f32::max)
}

/// How wide the panel stands in a window `room` points across.
///
/// All of it, less the gap a row keeps at its end, and every line
/// [drawn out](Share) to that, because a front panel is a rectangle. Past the
/// width the panel is allowed to grow to it stands in the middle of the room,
/// which is where an instrument on a desk that size would be.
///
/// The two agree at the point they meet: the panel stops growing where
/// [`Scale::filling`] stops, so nothing jumps as a window is dragged past it.
/// So do this and [`panel_width`], which is the same measurement with that gap
/// still on it: a window opened at what the panel wants holds the instrument's
/// own arrangement, on one line a row.
fn span(room: f32, scale: Scale) -> f32 {
    (room.min(widest() * Scale::MOST) - scale.of(ACROSS)).max(0.0)
}

/// How tall a plate stands at `scale`, and with it the screen between the rows.
///
/// Every plate, and not every plate of a row: a panel whose plates were each as
/// tall as they happened to be is a panel with a ragged edge under every row
/// and its `EDIT` presses at five different heights, which is the one thing a
/// front panel never is.
///
/// Added up from the parts rather than written down beside them, because two of
/// those parts do not grow with the rest. A display gains dots instead of
/// getting bigger, so the strip over a plate's faders is the same height in any
/// window; and the buttons along the foot are drawn in the room the rest of
/// this editor gives a control that is not a fader, which is the rack's room
/// and not the panel's. What is left scales, and this is the sum.
fn standing(scale: Scale) -> f32 {
    scale.of(PAD * 2.0 + HEAD + WITHIN * 3.0 + LEGEND + APART + TRAVEL + APART + LEGEND + UNDER)
        + READ
        + lcd::room(STRIP)
        + crate::panel::BUTTON
}

/// How wide one row stands: its plates, the screen if it holds it, and the gaps.
fn row_width(plates: &[Plate], holds_screen: bool, scale: Scale) -> f32 {
    let across: f32 = plates.iter().map(|plate| plate.width(scale)).sum();
    let boxes = count(plates.len()) + if holds_screen { 1.0 } else { 0.0 };
    across + if holds_screen { scale.of(SCREEN) } else { 0.0 } + boxes * scale.of(ACROSS)
}

/// Counts a handful of things as a width does.
///
/// A plate holds a dozen controls at the outside, so the conversion is exact
/// and the saturating fallback is unreachable; it costs one line and keeps a
/// cast out of a layout.
fn count(of: usize) -> f32 {
    f32::from(u16::try_from(of).unwrap_or(u16::MAX))
}

/// What stands in one row of the panel: its plates, and the screen among them.
///
/// A row is not only plates, because the instrument cuts its display into the
/// top row between what a player reaches for and the voicing. The two are laid
/// out by the same arithmetic, so they are one type while it is being done.
#[derive(Debug, Clone, Copy)]
enum Standing<'a> {
    /// One of the library's sections, as a plate of the panel.
    Plate(&'a Plate),
    /// The hole the application's own display is cut into.
    Screen,
}

impl Standing<'_> {
    /// How wide it measures, before a line shares out what it has spare.
    fn width(self, scale: Scale) -> f32 {
        match self {
            Self::Plate(plate) => plate.width(scale),
            Self::Screen => scale.of(SCREEN),
        }
    }

    /// Whether a line may draw it out.
    ///
    /// The plates, and not the screen. A screen given what a row had spare
    /// would take the whole of it and push the voicing onto a line of its own,
    /// which is the same reason it is a written width and not what is left.
    const fn grows(self) -> bool {
        matches!(self, Self::Plate(_))
    }
}

/// What stands in `plates`' row, with the screen cut in where it belongs.
///
/// At the end of it. The instrument cuts its display in before the *last* plate
/// of the top row, between what a player reaches for and the voicing, and the
/// voicing is not in that row any more, so the end of the row is where the
/// display's own neighbour went.
fn standing_in(plates: &[Plate], holds_screen: bool) -> Vec<Standing<'_>> {
    let mut row: Vec<Standing<'_>> = plates.iter().map(Standing::Plate).collect();
    if holds_screen {
        row.push(Standing::Screen);
    }
    row
}

/// Breaks a row into the lines a panel `across` points wide has room for.
///
/// A window narrower than the panel gets as many lines as the row takes, which
/// is the arrangement the panel has always fallen back to and is still
/// readable: two rows and a screen is what the instrument is, and a row with
/// its end clipped off is nothing at all.
///
/// The lines are broken here rather than left to the toolkit's own wrapping for
/// the reason every line is [drawn out](Share) afterwards: a wrapped line is a
/// line, and a panel whose last three plates sat in the top left corner of an
/// empty one is exactly what a row that fills its width is not.
fn lines<'a>(row: &[Standing<'a>], across: f32, scale: Scale) -> Vec<Vec<Standing<'a>>> {
    let gap = scale.of(ACROSS);
    let mut lines: Vec<Vec<Standing<'a>>> = Vec::new();
    let mut line: Vec<Standing<'a>> = Vec::new();
    let mut taken = 0.0;
    for item in row {
        let width = item.width(scale);
        if !line.is_empty() && taken + gap + width > across {
            lines.push(core::mem::take(&mut line));
            taken = 0.0;
        }
        taken += if line.is_empty() { width } else { gap + width };
        line.push(*item);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// How a line's spare width is shared out among its plates.
///
/// The panel is as wide as its widest row and the others measure less, the
/// signal path by a plate's worth and the envelopes by two plates', so a row
/// drawn at what it measures leaves that difference as bare panel at its right
/// hand end. There is no such thing on the instrument: a row of a front panel
/// runs the whole width of the instrument, because the plates are cut to fill
/// it. A window whose lower rows stopped two thirds of the way across would be
/// a photograph of a synthesizer with the end sawn off.
///
/// So the difference is shared out in proportion to what each plate already
/// holds, which is the one way of spending it that changes no proportion: a
/// five-fader plate stays twice the width of a two-fader one, every plate of a
/// line grows by the same fraction of itself, and nothing inside any of them
/// moves except the display, which gains dots rather than magnifying the ones
/// it has. It is the same trade the narrowest plates already make for their
/// screens, spent across a row instead of on one plate.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Share {
    /// Points of panel the line has spare.
    slack: f32,
    /// What the things it may draw out measure between them, which is what the
    /// slack is shared out in proportion to.
    holding: f32,
}

impl Share {
    /// Nothing to share out: everything is drawn at what it measures.
    ///
    /// What a row that had to be broken gets. A line of a broken row is a
    /// fraction of a row, and drawing a fraction of a row out to the width of
    /// the panel is how `POLY`, which is one fader, ends up as wide as the
    /// window: filling is what a row does, and a narrow window has already lost
    /// the rows. It keeps the written arrangement instead, which is what the
    /// panel fell back to before any of this and is still readable.
    const MEASURED: Self = Self {
        slack: 0.0,
        holding: 0.0,
    };

    /// How one line fills a panel `across` points wide.
    ///
    /// A line already wider than the panel has nothing to share: it is the one
    /// the panel is measured from, or it is a single plate too wide for the
    /// window it is in, and stretching either would be inventing room.
    fn of(line: &[Standing], across: f32, scale: Scale) -> Self {
        let measured: f32 = line.iter().map(|item| item.width(scale)).sum();
        let gaps = count(line.len().saturating_sub(1)) * scale.of(ACROSS);
        Self {
            slack: (across - measured - gaps).max(0.0),
            holding: line
                .iter()
                .filter(|item| item.grows())
                .map(|item| item.width(scale))
                .sum(),
        }
    }

    /// How wide `item` is drawn, with its share of what the line had spare.
    fn width(self, item: Standing, scale: Scale) -> f32 {
        let measured = item.width(scale);
        if !item.grows() || self.holding <= 0.0 {
            return measured;
        }
        measured + self.slack * (measured / self.holding)
    }
}

/// One control of the panel: what is printed over it, and what it moves.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Control {
    /// What the panel prints over it.
    ///
    /// The hardware's own word, which is not the library's: a silkscreen has
    /// room for `KYBD` and a lane has room for about as much. Two words are
    /// printed as two lines, which is what the panel does with `PITCH MOD`.
    legend: &'static str,
    /// The parameter it moves.
    parameter: ParamId,
}

/// One group of controls, as a plate of this window's panel.
///
/// Built from the library's own [`Section`] rather than written down here. Two
/// sections are unfolded into more than one plate on the way (see [`plates`]),
/// so a plate is not always a section, which is why this is a type of its own.
#[derive(Debug, Clone)]
pub(crate) struct Plate {
    /// What is printed across the top of the plate.
    ///
    /// Owned, because two of them are not a silkscreen: an unfolded section's
    /// plates are named after what the library calls the part they hold, in the
    /// caps the rest of the panel is printed in.
    name: String,
    /// The section this plate's `EDIT` opens, which is where the rest of its
    /// parameters are.
    opens: Group,
    /// The controls the hardware puts a fader under, in the order it puts them.
    faders: Vec<Control>,
    /// The named set drawn as a strip of legends beside the faders, where the
    /// panel has one: the LFO's shape is a column of lamps on the instrument.
    lamps: Option<Control>,
    /// The controls the hardware puts in the row of buttons under the faders.
    switches: Vec<Control>,
    /// What the specification records about this plate beyond its controls.
    ///
    /// `front::Section::note`: which fader of the instrument's is missing from
    /// this section and why, or which of three envelopes the shared faders
    /// address. A sentence about the *panel* rather than about a parameter, so
    /// it is said where this window says what is under the pointer rather than
    /// printed on a plate that has no room for it.
    note: Option<&'static str>,
}

impl Plate {
    /// What is printed across the top of the plate.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Every parameter this plate puts a control under.
    pub(crate) fn parameters(&self) -> impl Iterator<Item = ParamId> {
        self.controls().map(|control| control.parameter)
    }

    /// How wide the plate stands.
    ///
    /// Worked out rather than given: a plate that took the width it was offered
    /// would be a row of one, because a row of plates is only a row while each
    /// of them is as wide as what it holds. The wider of its two rows wins, so
    /// on `HPF`, which is one fader over a button and a way in, the buttons
    /// decide it.
    fn width(&self, scale: Scale) -> f32 {
        let lanes = count(self.faders.len()) * scale.of(LANE + 2.0)
            + if self.lamps.is_some() {
                scale.of(LAMPS + 2.0)
            } else {
                0.0
            };
        let buttons = count(self.switches.len()) * scale.of(SWITCH + 4.0) + scale.of(WAY + 4.0);
        // The narrowest plate still stands wide enough for a display worth
        // drawing on, and that floor is in dots rather than points: a screen
        // does not scale, it gains columns, so the room it needs is the room
        // forty dots need whatever the window is doing.
        lanes
            .max(buttons)
            .max(lcd::room(NARROWEST) + scale.of(PAD * 2.0))
    }

    /// Every control this plate draws.
    pub(crate) fn controls(&self) -> impl Iterator<Item = Control> {
        self.faders
            .iter()
            .copied()
            .chain(self.lamps)
            .chain(self.switches.iter().copied())
    }
}

/// The panel, in the rows the instrument prints it in.
///
/// **Read off the library rather than written down here.** `deepmind-midi` 26.3
/// publishes [`front::sections`]: which parameters the instrument puts a
/// control under, what is silkscreened over each one, what a hand touches and
/// which row it is in. Until that landed this file held the one table this
/// repository transcribed, and deleting it is the whole of what
/// [deepmind-midi#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)
/// was for.
///
/// Built once, because the panel is drawn on every frame and the library's
/// table does not change between them.
static PLATES: LazyLock<Vec<Vec<Plate>>> = LazyLock::new(plates);

/// Returns the panel's rows, each a row of plates, left to right.
pub(crate) fn rows() -> &'static [Vec<Plate>] {
    &PLATES
}

/// Builds the plates out of the library's sections, unfolding two of them.
///
/// # Hand layout changes the arrangement and never what a control is
///
/// The rule the rest of this crate is written under, and this is where it is
/// spent. Two of the instrument's nine plates are drawn as more than one here,
/// in both cases for the reason the panel has a screen on every plate where the
/// hardware has one screen in the middle: a window has room the front of a
/// synthesizer does not. What a control *is* stays the library's answer in
/// both: the parameter, the legend printed over it and the shape a hand touches
/// all come from the table a single-plate panel would use.
///
/// **The oscillators become two plates.** The instrument prints `DCO 1 & 2`
/// across one plate and then prints `OSC 1` and `OSC 2` in brackets over the
/// two clusters of faders inside it, because it has one plate's width and two
/// oscillators. Those brackets are the split, promoted to a plate each: the
/// panel's own subdivision rather than an invention, and the names are a slice
/// of the library's own parameter names rather than a third table.
///
/// **The envelopes become one plate each.** The instrument has four envelope
/// faders and three envelopes, and `VCA`, `VCF` and `MOD` buttons that choose
/// which of the three those four faders address, which the library says in that
/// section's own note. A window does not have to multiplex: each envelope gets
/// the same four legends over its *own* parameters, and its own display. Three
/// envelopes sharing one screen are three panes of a strip; three plates are
/// three full drawings.
///
/// **The amplifier stands at the head of the envelope row.** `VCA` is one fader,
/// how loud the voice is, and the plate immediately after it is
/// `VCA ENVELOPE`, which is what moves that fader while a note is held. On the
/// instrument they are two rows apart because the envelopes are multiplexed
/// onto four faders in the middle of the panel; unfolded, the amplifier and its
/// own envelope are two plates that belong beside each other, and the row reads
/// as the level and the three shapes that drive levels.
///
/// **The voicing drops to the [second row](VOICING_ROW).** It is one fader and a
/// strip of lamps, how many voices a note takes and how far they are detuned,
/// which is about the voice the signal path builds rather than about the two
/// modulators and the arpeggiator it was printed beside. The instrument has it
/// in the top row because that is where its front had the room, and taking it
/// out of that row is also what lets the display stand at the end of one rather
/// than in the middle.
fn plates() -> Vec<Vec<Plate>> {
    let mut rows: Vec<Vec<Plate>> = vec![Vec::new(); usize::from(front::PANEL_ROWS)];
    // The envelopes get a row of their own rather than the one the instrument
    // prints them in, and for the same reason they are three plates at all: the
    // hardware multiplexes them onto four faders because its front is full, and
    // three unfolded plates pushed into that row would make it half again as
    // wide as the window that has to hold it. A row is what the instrument did
    // not have to give them.
    let mut unfolded: Vec<Plate> = Vec::new();
    for section in front::sections() {
        if let Some(envelopes) = envelopes(section) {
            unfolded.extend(envelopes);
            continue;
        }
        let drawn = oscillators(section).unwrap_or_else(|| {
            let all: Vec<&'static PanelControl> = section.controls().iter().collect();
            vec![whole(section, section.name(), &all)]
        });
        if let Some(row) = rows.get_mut(usize::from(section.row())) {
            row.extend(drawn);
        }
    }
    if !unfolded.is_empty() {
        rows.push(unfolded);
    }
    // And two plates stand somewhere other than the row the instrument prints
    // them in. See the note below for why each of them moves; both are the same
    // hand layout the oscillators and the envelopes already are, and neither
    // changes what a control is.
    let last = rows.len().saturating_sub(1);
    lift(&mut rows, Group::Vca, last, Beside::First);
    lift(&mut rows, Group::Voicing, VOICING_ROW, Beside::Last);
    rows.retain(|row| !row.is_empty());
    rows
}

/// Which row the voicing stands in, counting from the top.
///
/// The second. It is one fader and a row of lamps, and how many voices a note
/// takes and how far they are detuned is about the voice the row under it builds
/// rather than about the two modulators and the arpeggiator it was printed
/// beside. The instrument has it up there because that is where its front panel
/// had the room.
const VOICING_ROW: usize = 1;

/// Which end of its new row a moved plate stands at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Beside {
    /// The left-hand end, in front of what is already there.
    First,
    /// The right-hand end, after it.
    Last,
}

/// Moves the plate that opens `group` to the `to`th row, at `beside`.
///
/// Nothing happens where the library has no such section, which is the same
/// refusal every other reading of its tables makes here: a panel that lost a
/// plate because a firmware renamed a group would be a panel drawn from an
/// assumption.
fn lift(rows: &mut [Vec<Plate>], group: Group, to: usize, beside: Beside) {
    let mut moved = None;
    for row in rows.iter_mut() {
        if let Some(at) = row.iter().position(|plate| plate.opens == group) {
            moved = Some(row.remove(at));
            break;
        }
    }
    let (Some(plate), Some(row)) = (moved, rows.get_mut(to)) else {
        return;
    };
    match beside {
        Beside::First => row.insert(0, plate),
        Beside::Last => row.push(plate),
    }
}

/// Gathers `controls` of `section` onto one plate called `name`.
///
/// The shape a hand touches is the library's answer and not the byte's: `SYNC`
/// is a switch to the parameter table and a button on the panel, and an LFO's
/// shape is a column of lit legends rather than a list, which nothing about the
/// byte says.
fn whole(section: &'static Section, name: &str, controls: &[&'static PanelControl]) -> Plate {
    let of = |wanted: PanelShape| -> Vec<Control> {
        controls
            .iter()
            .filter(|control| control.shape() == wanted)
            .map(|control| Control {
                legend: control.legend(),
                parameter: control.parameter(),
            })
            .collect()
    };
    Plate {
        name: name.to_owned(),
        opens: section.group(),
        faders: of(PanelShape::Fader),
        lamps: of(PanelShape::Lamps).first().copied(),
        switches: of(PanelShape::Button),
        note: section.note(),
    }
}

/// What an oscillator's parameters are named after on this instrument.
const BRACKET: &str = "OSC ";

/// The two plates an oscillator section is drawn as, when it is one.
///
/// Found by the bracket the panel prints over each cluster, which is the prefix
/// of the library's own name for the parameter under it: `OSC 1 PWM Depth` is
/// printed under `OSC 1`. A section where fewer than two brackets appear is not
/// one and is drawn whole.
fn oscillators(section: &'static Section) -> Option<Vec<Plate>> {
    let mut brackets: Vec<&'static str> = Vec::new();
    for control in section.controls() {
        if let Some(bracket) = bracket(control)
            && !brackets.contains(&bracket)
        {
            brackets.push(bracket);
        }
    }
    if brackets.len() < 2 {
        return None;
    }
    let last = *brackets.last()?;
    Some(
        brackets
            .iter()
            .map(|wanted| {
                let mine: Vec<&'static PanelControl> = section
                    .controls()
                    .iter()
                    // A control naming no oscillator rides with the last one:
                    // the noise level and the sync that makes the second follow
                    // the first are printed at that end of the plate, and
                    // neither is about the first.
                    .filter(|control| bracket(control).unwrap_or(last) == *wanted)
                    .collect();
                whole(section, wanted, &mine)
            })
            .collect(),
    )
}

/// `OSC 1` out of `OSC 1 Pitch Mod Depth`, and nothing out of `Noise Level`.
///
/// A slice of the library's own name for the parameter, so the spelling is the
/// library's and this file holds no copy of it.
fn bracket(control: &'static PanelControl) -> Option<&'static str> {
    let name = control.parameter().name();
    let rest = name.strip_prefix(BRACKET)?;
    let digits = rest
        .find(|letter: char| !letter.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits == 0 {
        return None;
    }
    name.get(..BRACKET.len() + digits)
}

/// The plate per envelope an envelope section is drawn as, when it is one.
///
/// The section carries one envelope's parameters under four legends, and the
/// note beside it says the panel's three buttons point those four faders at
/// whichever envelope is chosen. Here each envelope gets its own four, matched
/// through the library by short name, and a plate of its own.
fn envelopes(section: &'static Section) -> Option<Vec<Plate>> {
    envelope::of(section.group())?;
    let groups: Vec<Group> = Group::ORDER
        .iter()
        .copied()
        .filter(|group| envelope::of(*group).is_some())
        .collect();
    if groups.len() < 2 {
        return None;
    }
    Some(
        groups
            .into_iter()
            .map(|group| Plate {
                // The library's own name for the group, in the caps the rest of
                // the panel is printed in. Not the one word the hardware prints
                // on the button: `VCA` beneath an `ENVELOPES` heading is
                // unambiguous, and `VCA` on a plate of its own beside the
                // amplifier's `VCA` plate is two plates with one name.
                name: group.name().to_uppercase(),
                opens: group,
                faders: section
                    .controls()
                    .iter()
                    .filter_map(|control| addressed(control, group))
                    .collect(),
                lamps: None,
                switches: Vec::new(),
                // The section's own note, which on this one is what the
                // unfolding is *about*: the instrument multiplexes three
                // envelopes onto four faders and says so here.
                note: section.note(),
            })
            .collect(),
    )
}

/// The same control on another envelope, under the same legend.
///
/// `A` over `VCA Envelope Attack Time` is `A` over `VCF Envelope Attack Time`,
/// found by the short name the library strips the group off for. `None` where
/// an envelope has no parameter answering to the same short name, which would
/// be a library that had stopped describing its three envelopes the same way
/// and is better as a missing fader than as a wrong one.
fn addressed(control: &'static PanelControl, group: Group) -> Option<Control> {
    let wanted = control.parameter().short_name();
    let parameter = group
        .parameters()
        .find(|parameter| parameter.short_name() == wanted)?;
    Some(Control {
        legend: control.legend(),
        parameter,
    })
}

/// Draws the front panel, with `screen` where the instrument's display sits.
///
/// The screen is the caller's, because what it says is the application's
/// business rather than the synthesizer's: which sound is on it, what backs
/// that, and what last happened. A plugin answers those differently from a
/// desktop window.
#[must_use]
pub fn panel<'a, Renderer>(
    patch: &'a Patch,
    firmware: Version,
    mapper: &'a Mapper,
    paint: impl Fn(&mut Screen) + 'a,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mapping = mapper.mapped();
    // What the patch's other routings already reach, read once for the whole
    // panel rather than once per control: the walk is the same answer for all
    // of them, and it is only asked for at all while a routing is being mapped.
    let reaches = mapping
        .map(|_| matrix::reaching(patch, firmware))
        .unwrap_or_default();
    responsive(move |room| {
        let sent = mapping.map(|mapping| Sent::new(mapping, &reaches));
        let scale = Scale::filling(room.width);
        // What every line of the panel is drawn out to.
        let panel_across = span(room.width, scale);
        let mut panel = column![].spacing(scale.of(DOWN));
        let mut hole = true;
        for (index, plates) in rows().iter().enumerate() {
            let standing = standing_in(plates, index == 0);
            hole &= !standing.iter().any(|item| !item.grows());
            let drawn = lines(&standing, panel_across, scale);
            // A row the window had room for fills it. A row it did not is the
            // written arrangement, wrapped, which is what a narrow window has
            // always shown.
            let broken = drawn.len() > 1;
            for line in drawn {
                let share = if broken {
                    Share::MEASURED
                } else {
                    Share::of(&line, panel_across, scale)
                };
                let mut across = row![].spacing(scale.of(ACROSS)).align_y(Vertical::Top);
                for item in line {
                    across = across.push(match item {
                        Standing::Plate(plate) => group(
                            patch,
                            plate,
                            firmware,
                            scale,
                            share.width(item, scale),
                            sent,
                        ),
                        Standing::Screen => display(patch, &paint, scale),
                    });
                }
                panel = panel.push(across);
            }
        }
        // A screen with nowhere to go still goes somewhere: a panel with no
        // rows in it loses the instrument's arrangement, not its display.
        if hole {
            panel = panel.push(display(patch, &paint, scale));
        }
        // And the panel stands in the middle of the window rather than against
        // its left edge. It only ever has room to spare when the window is
        // wider than the panel is allowed to grow, and an instrument left on a
        // desk that wide is in the middle of it and not pushed into a corner.
        container(panel)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    })
    .into()
}

/// A blank screen the size of the hole the panel leaves at its written size.
///
/// The panel paints its own, because only it knows how wide the hole came out at
/// the scale the window forced. This is what a test, or anything else with no
/// window to measure, writes on to see what a display would say.
#[must_use]
pub fn screen() -> Screen {
    blank(Scale::NATURAL)
}

/// The blank screen the hole leaves at `scale`.
fn blank(scale: Scale) -> Screen {
    Screen::new(
        lcd::fits(scale.of(SCREEN)),
        lcd::fits(standing(scale) - scale.of(GAP * 2.0)),
    )
}

/// Returns every parameter the panel puts a control under.
///
/// The way for a test to ask what the table claims without drawing it.
#[must_use]
pub fn panelled() -> Vec<ParamId> {
    rows()
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|plate| plate.controls().map(|control| control.parameter))
        .collect()
}

/// Draws one group of the panel: its name, its controls, and its way in.
///
/// How wide it is drawn is the row's business rather than the plate's (see
/// [`Share`]), because a plate's width is what makes a row fill the panel, and
/// a plate given its own measured width is a row that stops short. How tall it
/// [stands](standing) is every plate's, so that a row has one edge along the
/// bottom of it.
fn group<'a, Renderer>(
    patch: &Patch,
    plate: &'a Plate,
    firmware: Version,
    scale: Scale,
    width: f32,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mut controls = row![].spacing(scale.of(2.0)).align_y(Vertical::Top);
    for control in plate.faders.iter().copied() {
        controls = controls.push(lane(patch, control, firmware, scale, sent));
    }
    if let Some(control) = plate.lamps {
        controls = controls.push(strip(patch, control, firmware, scale, sent));
    }
    let mut buttons = row![].spacing(scale.of(4.0)).align_y(Vertical::Top);
    for control in plate.switches.iter().copied() {
        buttons = buttons.push(switch(patch, control, firmware, scale, sent));
    }
    // Every plate has a way in, and it is the press the hardware calls EDIT.
    buttons = buttons.push(way("EDIT", plate.opens, scale));
    container(
        column![
            heading(plate.name(), plate.note, scale),
            glass(patch, plate, firmware, scale, width),
            controls,
            buttons
        ]
        .spacing(scale.of(WITHIN))
        .align_x(Horizontal::Center),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(standing(scale)))
    .padding(scale.of(PAD))
    .style(|theme: &Theme| {
        let material = materials(theme);
        container::Style {
            background: Some(Background::Color(material.plate)),
            border: Border {
                color: material.recess_edge,
                width: 1.0,
                radius: 3.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// The display over a plate's faders, showing what its section is doing.
///
/// The instrument has one screen for fourteen sections and this window has one
/// for each, which is the one place the panel is deliberately not the
/// instrument: the room a window has is what the hardware did not, and a filter
/// is a shape before it is three numbers. What is drawn is
/// [the scene](crate::scene) the plate's own controls ask for, on as many dots
/// as the plate is wide.
///
/// A plate whose section this window has no drawing for keeps the room anyway,
/// so that a row of plates is a row rather than a skyline.
fn glass<'a, Renderer>(
    patch: &Patch,
    plate: &Plate,
    firmware: Version,
    scale: Scale,
    width: f32,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let columns = lcd::fits(width - scale.of(PAD * 2.0));
    scene::display(patch, plate.parameters(), firmware, columns, STRIP).unwrap_or_else(|| {
        container(Space::new())
            .height(Length::Fixed(lcd::room(STRIP)))
            .into()
    })
}

/// The bar the panel prints a group's name in.
///
/// Caps on a darker band across the top of the plate, which is how the
/// instrument prints every one of them, and how a person finds `VCF` without
/// reading the whole panel.
/// What the library records about the plate beyond its controls is said in the
/// footer while the pointer is on this bar. That is where this window already
/// says what is under the pointer, and the only place a sentence about a *plate*
/// can go without being printed on every plate that has one.
fn heading<'a, Renderer>(
    name: &'a str,
    note: Option<&'static str>,
    scale: Scale,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let bar = container(
        text(name)
            .size(scale.of(11.0))
            .font(printed())
            .style(|theme: &Theme| text::Style {
                color: Some(materials(theme).recess),
            }),
    )
    .width(Length::Fill)
    .height(Length::Fixed(scale.of(HEAD)))
    .padding([0.0, scale.of(6.0)])
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(|theme: &Theme| {
        let material = materials(theme);
        container::Style {
            // The one light band on the plate, with the name knocked out of it
            // dark. A `DeepMind` prints `ARP / SEQ`, `VCF` and `ENVELOPES` on
            // pale grey strips across the top of each group, and they are what
            // the eye follows across the panel before it reads a single legend.
            background: Some(Background::Color(material.metal_low)),
            border: Border {
                color: material.metal,
                width: 1.0,
                radius: 2.into(),
            },
            ..container::Style::default()
        }
    });
    match note {
        Some(note) => mouse_area(bar)
            .on_enter(Message::Hinted(Some(note)))
            .on_exit(Message::Hinted(None))
            .into(),
        None => bar.into(),
    }
}

/// Draws one control of the panel: what is printed over it, it, and its value.
///
/// The legend goes above, which is where the instrument prints it and not where
/// a rack's slot does: the hardware has a screen to put readings on and no room
/// under a fader, and a panel that printed its names underneath would be a
/// different object.
fn lane<'a, Renderer>(
    patch: &Patch,
    control: Control,
    firmware: Version,
    scale: Scale,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        legend(control.legend, scale),
        crate::panel::control(
            parameter,
            value,
            claim,
            firmware,
            Room::lane(scale.of(LANE), scale.of(TRAVEL)),
            sent,
        ),
        container(readout(parameter, value, claim, firmware))
            .height(Length::Fixed(READ))
            .align_y(Vertical::Center),
    ]
    .spacing(scale.of(APART))
    .width(Length::Fixed(scale.of(LANE)))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws a named set as the strip of lit legends the instrument has.
///
/// The LFO's shape is a column of lamps beside its two faders on the hardware,
/// one of them lit, and seven of them is more than a rack's slot has room to
/// light. The panel has the room, so it says so.
///
/// The room is [everything a lane holds](lit) and not the travel of the fader
/// beside it, because seven legends are taller than a panel fader is long: a
/// strip given the fader's travel lights five of the instrument's seven shapes
/// and drops `Sample & Hold` and `Sample & Glide` off the bottom of the panel,
/// where nothing says they are missing.
fn strip<'a, Renderer>(
    patch: &Patch,
    control: Control,
    firmware: Version,
    scale: Scale,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    column![
        legend(control.legend, scale),
        crate::panel::control(
            parameter,
            patch.value(parameter),
            patch.claim(parameter),
            firmware,
            Room::lamps(scale.of(LAMPS), lit(scale, named(control, firmware))),
            sent,
        ),
    ]
    .spacing(scale.of(APART))
    .width(Length::Fixed(scale.of(LAMPS)))
    .align_x(Horizontal::Center)
    .into()
}

/// How many things a control's named set names, as the library has them.
fn named(control: Control, firmware: Version) -> usize {
    control
        .parameter
        .choices_for(firmware)
        .map_or(0, <[_]>::len)
}

/// How much room a strip of lit legends naming `named` things is given.
///
/// A whole lane: the fader's travel, the gap under it and the reading it would
/// have had. A strip has no reading of its own, because the lit legend is the
/// reading, so the room a lane spends on a number is spent on the words instead
/// and the two stand the same height whatever the window is doing.
///
/// And never less than the set itself needs. A column of legends is laid out
/// into the room it is given and the ones past the end of it are drawn no lines
/// tall, so a strip too short for its own set is a panel that names five of the
/// instrument's seven LFO shapes and says nothing about the other two. A strip
/// standing a little past its lane is something somebody can see; the test
/// beside this one is what says no set on the panel needs that today.
fn lit(scale: Scale, named: usize) -> f32 {
    let lane = scale.of(TRAVEL + APART) + READ;
    lane.max(crate::panel::lit_band(named))
}

/// Draws one of the buttons under a plate's faders.
///
/// Whatever the library says the parameter is, in the room the button row has:
/// two states are a lamp, and a named set of two is the pair of legends the
/// panel prints beside its button.
fn switch<'a, Renderer>(
    patch: &Patch,
    control: Control,
    firmware: Version,
    scale: Scale,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    column![
        crate::panel::control(
            parameter,
            patch.value(parameter),
            patch.claim(parameter),
            firmware,
            Room::listed(scale.of(SWITCH)),
            sent,
        ),
        legend(control.legend, scale),
    ]
    .spacing(scale.of(UNDER))
    .width(Length::Fixed(scale.of(SWITCH)))
    .align_x(Horizontal::Center)
    .into()
}

/// What the panel prints with a control.
///
/// Over a fader, which is where the instrument prints it and where a rack's
/// slot does not: the hardware has a screen to put readings on and no room
/// under a fader. Under a button, for the same reason read the other way. A
/// `DeepMind` silkscreens `POLY` and `EDIT` on the panel *below* the cap they
/// belong to, because a finger on a button covers what is printed above it and
/// a lit cap with nothing over it is the thing the eye finds first.
fn legend<'a, Renderer>(printed_as: &'a str, scale: Scale) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(printed_as)
            .size(scale.of(9.0))
            .font(reading())
            .center()
            .style(move |theme: &Theme| text::Style {
                color: Some(materials(theme).metal_low),
            }),
    )
    .height(Length::Fixed(scale.of(LEGEND)))
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// The way into a section, which is the button the hardware calls `EDIT`.
///
/// Two things and not one. On the instrument the word is silkscreened on the
/// panel and the button it names is a blank rubber cap, lit amber the whole
/// time it is powered; a row of those along the foot of every plate is the
/// first thing anybody sees in a photograph of a `DeepMind`. So the label is
/// printed under it in the same ink as every other legend, and what is pressed
/// is the lamp.
fn way<'a, Renderer>(label: &'a str, group: Group, scale: Scale) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    column![
        // In the room a button of the row beside it is drawn in, and standing
        // in the middle of it. The cap is the rack's own, the same width and
        // height as the lamp on the next plate along, because the band along the
        // foot of a plate is one band and two sizes of button in it is a row of
        // presses that do not line up. That band is the one part of the panel
        // the window does not stretch, so neither is this.
        container(
            button(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fixed(crate::panel::CAP))
                .height(Length::Fixed(crate::panel::PRESS))
                .style(crate::style::way_in)
                .on_press(Message::Show(group)),
        )
        .height(Length::Fixed(crate::panel::BUTTON))
        .align_y(Vertical::Center),
        legend(label, scale),
    ]
    .spacing(scale.of(UNDER))
    .width(Length::Fixed(scale.of(WAY)))
    .align_x(Horizontal::Center)
    .into()
}

/// The screen, cut into the panel where the instrument's own display sits.
///
/// The glass is the display's own, and it draws the recess it is cut into the
/// way every other display in this window does, so what is left here is the hole
/// it stands in, as tall as the plates either side of it.
fn display<'a, Renderer>(
    patch: &Patch,
    paint: &impl Fn(&mut Screen),
    scale: Scale,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // Painted here rather than handed in already written, because how many dots
    // the hole holds is a question only the panel can answer: it depends on how
    // wide the window made the panel, and a screen sized anywhere else would be
    // the right picture at the wrong resolution.
    let mut screen = blank(scale);
    paint(&mut screen);
    container(crate::lcd::lcd(screen, patch.confidence()))
        .width(Length::Fixed(scale.of(SCREEN)))
        .height(Length::Fixed(standing(scale)))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([scale.of(2.0), 0.0])
        .into()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};

    use deepmind_midi::param::DEFAULT_FIRMWARE;

    use super::{ACROSS, DOWN, GAP, LEGEND, NARROWEST, PAD, Plate, SCREEN, SWITCH, Scale};
    use super::{Share, UNDER, WAY, WITHIN, rows};
    use super::{
        blank, count, lcd, lines, lit, panel_width, panelled, row_width, span, standing,
        standing_in, widest,
    };

    #[test]
    fn a_control_is_on_the_panel_once() {
        let mut panelled = panelled();
        let count = panelled.len();
        panelled.sort_unstable_by_key(|parameter| parameter.offset());
        panelled.dedup();

        assert_eq!(panelled.len(), count, "a control is on the panel twice");
    }

    #[test]
    fn every_plate_holds_parameters_of_the_section_it_opens() {
        // The one transcription in this repository, held to what the library
        // says: a plate's controls belong to the section its way in opens, so
        // a parameter moved to another group by a later library fails here
        // rather than opening a panel it is not on.
        for plate in rows().iter().flat_map(|row| row.iter()) {
            for parameter in plate.controls().map(|control| control.parameter) {
                assert_eq!(
                    parameter.group(),
                    plate.opens,
                    "{parameter} is on the {} plate and in another section",
                    plate.name
                );
            }
        }
    }

    #[test]
    fn a_legend_is_short_enough_to_be_printed_over_a_lane() {
        // What the hardware silkscreens, not what the library calls it: a lane
        // is 46 points wide, and `Keyboard Tracking` in it is a lane that lies
        // about which fader is which. Two words are two lines.
        for plate in rows().iter().flat_map(|row| row.iter()) {
            for control in plate.controls() {
                let longest = control
                    .legend
                    .split(' ')
                    .map(str::len)
                    .max()
                    .unwrap_or_default();
                assert!(
                    longest <= 6,
                    "{} is too long a word to print over a lane",
                    control.legend
                );
                assert_eq!(
                    control.legend.to_uppercase(),
                    control.legend,
                    "a panel is printed in caps"
                );
            }
        }
    }

    #[test]
    fn a_way_in_leads_to_a_section_the_editor_has() {
        for plate in rows().iter().flat_map(|row| row.iter()) {
            assert!(
                Group::ALL.contains(&plate.opens),
                "{} opens nothing",
                plate.name
            );
        }
    }

    #[test]
    fn the_panel_is_the_handful_a_player_reaches_for() {
        // Not a second rack. If this table ever grows past a fraction of the
        // instrument, the panel has stopped being the thing it is drawn from.
        let panelled = panelled();

        assert!(panelled.len() < ParamId::ALL.len() / 4, "{panelled:?}");
        assert!(panelled.len() > 20, "a front panel with nothing on it");
    }

    #[test]
    fn the_window_opens_wide_enough_for_the_arrangement() {
        // Two rows and a screen between them is the arrangement, and a window
        // narrower than the panel wraps a row rather than clipping it, which is
        // readable and is no longer the instrument's own front. So the window
        // opens at what the panel measures, and this is what says that number is
        // a window somebody could actually have.
        let wanted = panel_width();

        assert!(wanted > 0.0, "the panel measures nothing");
        assert!(
            wanted <= 1512.0,
            "the panel wants {wanted} points, which is wider than a laptop"
        );
        for (index, plates) in rows().iter().enumerate() {
            let across = row_width(plates, index == 0, Scale::NATURAL);
            assert!(
                across <= wanted,
                "row {index} is {across} across a {wanted} panel"
            );
        }
    }

    #[test]
    fn every_plate_draws_its_own_part_and_not_a_neighbours() {
        // Unfolding a section is where a drawing could be repeated: two
        // oscillator plates showing one oscillators drawing, or three envelope
        // plates showing the same three-envelope drawing, is the split costing
        // room and buying nothing.
        let drawn: Vec<crate::scene::Scene> = rows()
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|plate| crate::scene::of(plate.parameters()))
            .collect();
        let found = drawn.len();

        assert_eq!(
            found,
            rows().iter().flat_map(|row| row.iter()).count(),
            "a plate with no drawing on it"
        );
        // A dozen plates, so this is the honest check rather than the one
        // that only catches duplicates that happen to be adjacent.
        for (at, scene) in drawn.iter().enumerate() {
            assert!(
                !drawn.get(at + 1..).unwrap_or_default().contains(scene),
                "two plates draw {scene:?}"
            );
        }
    }

    #[test]
    fn no_two_plates_are_printed_with_the_same_name() {
        // Unfolding a section is where this could go wrong: the amplifier's
        // plate and the amplifier envelope's plate both answer to `VCA` on the
        // hardware, where one of them is a button under an `ENVELOPES` heading
        // and the ambiguity cannot arise.
        let mut names: Vec<&str> = rows()
            .iter()
            .flat_map(|row| row.iter())
            .map(Plate::name)
            .collect();
        let printed = names.len();
        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), printed, "two plates share a name: {names:?}");
    }

    #[test]
    fn a_wider_window_is_a_wider_panel_and_not_a_panel_in_a_corner() {
        // The whole of the point: the panel is drawn at a proportion of the
        // room it was given rather than at the size its constants are written
        // in, so a window somebody has dragged wide holds a front panel and not
        // a front panel with an empty half beside it.
        let natural = widest();

        assert!(natural > 0.0, "the panel measures nothing");
        assert_eq!(
            Scale::filling(natural),
            Scale::NATURAL,
            "a window exactly the written size is drawn at it"
        );
        let room = natural * 1.4;
        let wide = Scale::filling(room);
        let widened: f32 = rows()
            .iter()
            .enumerate()
            .map(|(index, plates)| row_width(plates, index == 0, wide))
            .fold(0.0_f32, f32::max);

        assert!(
            widened <= room,
            "{widened} points of panel in {room} of window"
        );
        // Not exactly the room, and deliberately: the plates holding one or two
        // faders are as wide as forty dots of display rather than as wide as
        // their controls, and forty dots is forty dots however large the window
        // is. What is left over is that floor, and it is a couple of points of
        // panel rather than the empty half this test exists to prevent.
        assert!(
            widened > room * 0.95,
            "{widened} points of panel left {} of window empty",
            room - widened
        );
    }

    #[test]
    fn every_row_is_drawn_out_to_the_width_of_the_panel() {
        // The complaint this exists to prevent: the signal path is a plate
        // narrower than the top row and the envelopes are two plates narrower,
        // and drawn at what they measure they leave that difference as bare
        // panel at the right hand end, which is a front panel with the end sawn
        // off.
        let panel = span(panel_width(), Scale::NATURAL);

        for (index, plates) in rows().iter().enumerate() {
            let standing = standing_in(plates, index == 0);
            let drawn = lines(&standing, panel, Scale::NATURAL);
            assert_eq!(
                drawn.len(),
                1,
                "row {index} does not fit the panel it was measured from"
            );
            for line in drawn {
                let share = Share::of(&line, panel, Scale::NATURAL);
                let across: f32 = line
                    .iter()
                    .map(|item| share.width(*item, Scale::NATURAL))
                    .sum::<f32>()
                    + count(line.len().saturating_sub(1)) * ACROSS;

                assert!(
                    (across - panel).abs() < 0.01,
                    "row {index} is {across} across a {panel} panel"
                );
            }
        }
    }

    #[test]
    fn a_window_too_narrow_for_a_row_breaks_it_into_lines_that_fit() {
        // The fallback, and the one arrangement that is not drawn out: a line
        // of a broken row is a fraction of a row, and a fraction of a row
        // filling the panel is `POLY` drawn as wide as the window. What a
        // narrow window owes somebody is every plate, in an order they can
        // read, which is what this asserts.
        let panel = span(panel_width() * 0.6, Scale::NATURAL);
        let plates = rows().first().expect("the top row");
        let standing = standing_in(plates, true);
        let drawn = lines(&standing, panel, Scale::NATURAL);

        assert!(drawn.len() > 1, "a narrow window did not break the row");
        assert_eq!(
            drawn.iter().map(Vec::len).sum::<usize>(),
            standing.len(),
            "a plate fell out of the panel when the row broke"
        );
        for line in drawn {
            let across: f32 = line
                .iter()
                .map(|item| Share::MEASURED.width(*item, Scale::NATURAL))
                .sum::<f32>()
                + count(line.len().saturating_sub(1)) * ACROSS;

            assert!(
                across <= panel + 0.01 || line.len() == 1,
                "a line of {across} in a {panel} panel"
            );
        }
    }

    #[test]
    fn sharing_out_a_row_s_spare_width_changes_no_proportion() {
        // How the difference is spent. A row whose plates grew by a fixed
        // amount each would draw `VCA`, which is one fader, as wide as `VCF`,
        // which is five: the whole arrangement is that a plate is as wide as
        // what it holds, so every plate grows by the same fraction of itself.
        let plates = rows().get(1).expect("the signal path");
        let line = standing_in(plates, false);
        let share = Share::of(&line, span(panel_width(), Scale::NATURAL), Scale::NATURAL);
        let first = *line.first().expect("a plate");
        let grown = share.width(first, Scale::NATURAL) / first.width(Scale::NATURAL);

        assert!(grown > 1.0, "the row was not drawn out at all");
        for item in line {
            let each = share.width(item, Scale::NATURAL) / item.width(Scale::NATURAL);
            assert!(
                (each - grown).abs() < 0.001,
                "something grew by {each} where the row grew by {grown}"
            );
        }
    }

    #[test]
    fn the_screen_is_the_one_thing_a_row_does_not_draw_out() {
        // It is a written width and not what is left over, because a screen
        // given what the top row had spare would take all of it and push the
        // voicing onto a line of its own.
        let plates = rows().first().expect("the top row");
        let line = standing_in(plates, true);
        let share = Share::of(
            &line,
            span(panel_width() * 2.0, Scale::NATURAL),
            Scale::NATURAL,
        );
        let screen = *line
            .iter()
            .find(|item| !item.grows())
            .expect("the top row holds the screen");

        assert!(
            (share.width(screen, Scale::NATURAL) - SCREEN).abs() < 0.01,
            "the screen was drawn out with the plates"
        );
    }

    #[test]
    fn every_lit_set_on_the_panel_fits_the_room_it_is_given() {
        // A column of legends is laid out into the room it was given and the
        // ones past the end of that room are drawn no lines tall, which is a
        // panel that quietly names five of the instrument's seven LFO shapes.
        // The room is the tightest it ever is at the written size, so this is
        // where an eighth shape in a later table fails: loudly, here, rather
        // than by dropping off the bottom of a plate.
        for plate in rows().iter().flat_map(|row| row.iter()) {
            let Some(control) = plate.lamps else {
                continue;
            };
            let named = control
                .parameter
                .choices_for(DEFAULT_FIRMWARE)
                .map_or(0, <[_]>::len);
            assert!(named > 0, "{} lights nothing", plate.name());

            let band = crate::panel::lit_band(named);
            assert!(
                (lit(Scale::NATURAL, named) - lit(Scale::NATURAL, 0)).abs() < 0.01,
                "{} names {named} things in {band} points, past the lane beside it",
                plate.name()
            );
        }
    }

    #[test]
    fn the_screen_fits_the_hole_the_plates_leave_it_at_any_size() {
        // The screen is cut to the plates either side of it, and the dots are
        // cut to the screen. Two of the things a plate is made of do not grow
        // with the window: the strip over its faders gains dots instead, and the
        // buttons along its foot are drawn in the rack's own room. So a hole
        // worked out by scaling one number would be the wrong height everywhere
        // except where that number was written.
        for scale in [
            Scale::NATURAL,
            Scale::filling(widest() * 1.4),
            Scale::filling(widest() * 8.0),
        ] {
            let hole = standing(scale) - scale.of(GAP * 2.0);
            let screen = blank(scale);

            assert!(
                lcd::room(screen.rows()) <= hole,
                "a screen of {} dots stands {} in a {hole} hole",
                screen.rows(),
                lcd::room(screen.rows())
            );
            assert!(
                lcd::room(screen.rows() + 1) > hole,
                "the hole has room for a dot the screen is not using"
            );
        }
    }

    #[test]
    fn a_plate_s_foot_holds_its_buttons_and_what_is_printed_under_them() {
        // The band along the foot of a plate is drawn in the room the rest of
        // the editor gives a control that is not a fader, and it is the one
        // part of the panel that does not grow with the window. So the cap is
        // written at a size rather than scaled, and this is what says that size
        // still fits: a cap taller than the band is a row of buttons pushed
        // through the bottom of every plate at once, and a lane narrower than a
        // cap is a button drawn over the legend of the one beside it.
        for scale in [Scale::NATURAL, Scale::filling(widest() * 8.0)] {
            let foot = crate::panel::BUTTON + scale.of(UNDER + LEGEND);
            let printed = crate::panel::PRESS + scale.of(UNDER + LEGEND);
            assert!(
                printed <= foot,
                "a cap and its legend stand {printed} in the {foot} a plate's foot has"
            );
            for (lane, name) in [(WAY, "a way in"), (SWITCH, "a switch")] {
                assert!(
                    crate::panel::CAP <= scale.of(lane),
                    "{name} is {} across, holding a cap of {}",
                    scale.of(lane),
                    crate::panel::CAP
                );
            }
        }
    }

    #[test]
    fn a_panel_stops_growing_before_a_fader_is_as_long_as_an_arm() {
        let huge = Scale::filling(widest() * 8.0);

        assert_eq!(
            huge,
            Scale(Scale::MOST),
            "a panel on a wall is still a panel"
        );
    }

    #[test]
    fn a_narrow_window_wraps_rather_than_shrinking_the_legends() {
        // The rows already wrap, which is an arrangement somebody can still
        // read. Type scaled down to avoid wrapping is a panel nobody can.
        let cramped = Scale::filling(widest() / 3.0);

        assert_eq!(cramped, Scale::NATURAL);
    }

    #[test]
    fn the_gaps_grow_with_what_they_separate() {
        // The half that is easy to miss. A panel whose plates grew and whose
        // spacings did not is a panel with its parts crowded together, and one
        // where only the spacings grew is a panel with its parts pushed apart.
        let scale = Scale::filling(widest() * 1.5);

        for points in [ACROSS, DOWN, WITHIN, PAD, GAP] {
            assert!(
                (scale.of(points) - points * 1.5).abs() < 0.01,
                "{points} points of panel did not grow with the rest"
            );
        }
    }

    #[test]
    fn a_display_fits_inside_the_plate_it_is_drawn_on() {
        // A plate's width is the room it takes on the panel and a display fits
        // into the room inside that, which is two paddings narrower. A display
        // drawn from the outer width is a display a few dots wider than the
        // plate holding it.
        for plate in rows().iter().flat_map(|row| row.iter()) {
            let inside = plate.width(Scale::NATURAL) - PAD * 2.0;
            let columns = lcd::fits(inside);

            assert!(columns >= NARROWEST, "{} has no room to draw", plate.name);
            assert!(
                lcd::room(columns) <= inside,
                "the display on {} is wider than the plate",
                plate.name
            );
        }
    }

    #[test]
    fn the_voice_the_signal_takes_is_the_second_row_and_the_envelopes_the_third() {
        let named = |row: usize| -> Vec<&str> {
            rows()
                .get(row)
                .map(|plates| plates.iter().map(Plate::name).collect())
                .unwrap_or_default()
        };

        // The top row is what a player reaches for, and the display stands at
        // the end of it now that the voicing it was cut in before has gone
        // down a row.
        assert_eq!(named(0), vec!["ARP / SEQ", "LFO 1", "LFO 2"]);
        // The signal path, left to right, with the oscillators unfolded, and the
        // voicing at the end of it, which is the voice this row builds.
        assert_eq!(named(1), vec!["OSC 1", "OSC 2", "VCF", "HPF", "POLY"]);
        // The row the instrument had no room for: the level, and the three
        // shapes that drive levels, with the one that drives *this* level
        // standing next to it.
        assert_eq!(
            named(2),
            vec!["VCA", "VCA ENVELOPE", "VCF ENVELOPE", "MOD ENVELOPE"]
        );
    }
}

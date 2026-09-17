//! The effects: four engines, and what the algorithm in one says its twelve
//! bytes are.
//!
//! Fifty-eight parameters, and fifty-six of them are `FX 1 Param 3`. That name
//! is the truth about the wire and useless on a panel: the byte is `Size` on a
//! Room Reverb, `Depth` on a Phaser, and nothing at all on a TC Deep Reverb,
//! which uses five of its twelve. A rack of fifty-eight slots under those names
//! is complete, honest and unreadable, which is what stage 3 left here and what
//! this replaces.
//!
//! `deepmind-midi` 26.2 published the table that settles it. [`Engine`] is which
//! parameters one engine addresses, [`Algorithm`] is the 35 and which byte
//! selects one on a given firmware, and [`FxSlot`] is what a slot of the loaded
//! algorithm is called. So the panel is four plates, each of them the engine's
//! own settings and then its slots under the names the algorithm gives them.
//!
//! # Nothing here is transcribed
//!
//! Which group holds the engines is [`Engine::algorithm_parameter`]'s own group,
//! not a name written down. Which slot a parameter is under the loaded algorithm
//! is [`Algorithm::slot_of`]. Which algorithm a byte selects is
//! [`Algorithm::for_value`] for the firmware a device inquiry reported, because
//! firmware 1.1 inserted Vintage Pitch rather than appending it and 33 is Rotary
//! Speaker on 1.0. A fifth engine, a thirty-sixth algorithm or a renamed slot
//! arrives with nothing in this file to edit.
//!
//! # The page is the instrument's own, and so is the grid
//!
//! 26.3 published the rest of it
//! ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)):
//! the grid the synthesizer's own FX page lays a slot out on, measured off the
//! 35 screenshots in the manual, and what the figure printed beside each
//! algorithm is made of. Six columns and two rows, and
//! [`FxSlot::position`] says which cell a slot is drawn in.
//!
//! The cell is what this draws it in. Six columns, each the same width and each
//! there whether or not a slot stands in it, spread across the whole page — so
//! the third column of the second row stands under the third column of the
//! first, and an algorithm using five slots on one row and six on the next
//! draws the five where the hardware draws them. Reading the grid as the *order*
//! of a row instead, and packing each row against the left at the width of a
//! rack's slot, is what left two rows that did not line up in the left half of
//! an empty plate.
//!
//! [`Panel::control`] is the shape: 29 of the 35 figures are rotary knobs, five
//! are faders and one is a numeric display, and a slot on a knob panel is drawn
//! as [a knob](crate::knob).
//!
//! # A case is as deep as what is in it
//!
//! Four plates on a page at four heights, rather than four at one. The grid is
//! two rows and an algorithm as short as five slots fills one of them, and
//! drawing the empty second row anyway — along with the deepest band of display
//! names any of the 35 needs — made every case the same shape at the price of
//! half of most of them being blank panel. A rack unit with nothing on its
//! lower half is a rack unit somebody looks for the missing knobs on. What is
//! still reserved is the shape of a row: every column of a row stands a
//! column's height whether or not a slot is in it, so the columns line up down
//! the plate and two engines running algorithms of one shape do come out level.
//!
//! # Four at once, and what that costs the livery
//!
//! All four engines are on the page together, in the two-by-two the four of
//! them make. One at a time behind a row of tabs was an engine you could see
//! and three you had to remember, on a page whose whole subject is what four
//! effects are doing *together* — and the chain drawn across the top of it is a
//! picture of exactly that. Each engine's own case carries the strip that says
//! what it is running, which is also the list that changes it, and how loud it
//! comes out; there is no separate header, because a header was a second place
//! saying which algorithm an engine was running and neither place was the
//! engine.
//!
//! The library publishes four measured colours per algorithm. How many of them
//! this page can afford depends on how many units stand on it. With one open at
//! a time it wore the lot — case, face and cap — because a single unit can be
//! that unit without the window becoming a shelf of other people's boxes. Four
//! stand on it now, and four liveries side by side are exactly the collage this
//! editor's whole argument is against. So the face went back to the window's
//! own plate and the cap back to the window's own metal, and what an engine
//! wears is its case and a hairline of its accent: enough to tell the reverb
//! from the distortion at arm's length, which is all four units at once can
//! afford to say.
//!
//! # Nothing is printed in a colour that cannot be read on what is under it
//!
//! Half of the 35 measured chassis are pale and half are dark, and no word on
//! this page names an ink of its own. It names the surface it lands on — the
//! face, a case, a band's strip, the recess the chain is cut into — and
//! [`ink`] and [`legend`] answer with whichever of the instrument's two can be
//! read there, measured rather than judged. A reading keeps the colour of its
//! claim, because that is what it means, and is lifted until it clears the
//! same threshold: amber stays amber and green stays green, and what moves is
//! how light it is.
//!
//! # What an effect is, and what it is doing
//!
//! 26.4 published the rest of the effects page
//! ([#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30),
//! [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31),
//! [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33)), and all
//! four answers are spent here.
//!
//! [`Algorithm::mark`] is nine drawings across the 35 — a decaying tail, a train
//! of repeats, a horn going round — and one of them stands at the head of every
//! engine's strip, where four long names previously had to be read one at a
//! time. The library publishes the strokes rather than a picture, which is what
//! lets [`mark`](crate::mark) draw the same mark in this window's own ink on a
//! cream chassis and on a black one.
//!
//! [`FxSlot::is_enable`] is how an effect is switched off, and the answer is
//! that on 32 of the 35 it is not: `FX n Type` is 35 effects with no `Off` in
//! the table, and what takes effects out of circuit is the `Bypass` mode, which
//! is the whole block of four. Three algorithms spend one of their twelve bytes
//! on a switch of their own, and where one of those reads off the strip says so
//! and the chain draws that engine as something the signal goes past. The
//! window used to have no way to know, so every engine drew its full panel and
//! said nothing.
//!
//! [`effect::response`] is what an engine is doing to a signal, and it answers
//! for two of the 35: the tap delays, whose panels are literally a time and a
//! gain per tap. Those two get the screen every other panel in this window has;
//! the other 33 get nothing, which is the honest answer rather than a gap. A
//! reverb's impulse response is its designer's, and a plausible one drawn here
//! would look like information and not be any.
//!
//! [`FxSlot::quantity`] is what a slot does to a signal as against what it is
//! called, and it fills the line under every slot that has no printed range: a
//! `Mix`, a `Feedback` and a `Pre-Delay` are all a byte `0..=255` and they do
//! three unrelated things.
//!
//! # A named slot is the same control it was
//!
//! The library says two things about a slot and they are not the same thing.
//! `FxSlot::kind` is how the *display* reads the byte, and the parameter table
//! is what the byte *is*: `Freeze` is two states on the panel and a parameter
//! that accepts 256 values on the wire, and the curve between the two is not
//! published. So the control is still [`control`] over the parameter's own
//! range, chosen by the same code from the same table as every other slot in
//! the editor, and what the algorithm supplies is the naming: the title, the
//! abbreviation the instrument's display prints, the band the slot belongs to,
//! and the two ends of the reading it shows. Hand layout changes the arrangement
//! and never what a control is, and that rule is what keeps this panel from
//! inventing a mapping the manual does not give.
//!
//! For the same reason a slot whose display shows names — `Ambience`, `Church`,
//! `Gate` — is not drawn as a list. The manual prints those names and never the
//! bytes they sit at, [`FxSlot::values`] says so, and a list that sent one of
//! them would be sending a guess. The names are printed as what the display
//! will show, and the byte stays draggable.
//!
//! Under the row the slot stands on, rather than under the whole plate. Nine
//! reverb presets set at nine points are two lines long on a case a quarter of
//! this page wide, so printing all of a plate's at its foot put the longest
//! thing on the case as far as it could get from the control it is about, with
//! the abbreviation at the head of the line the only thing saying which control
//! that was.
//!
//! # Twelve bytes, however many the algorithm uses
//!
//! An engine holds twelve whatever it is running, and
//! [`Algorithm::slots`](deepmind_midi::effect::Algorithm::slots) is as short as
//! five. The ones the algorithm has a name for are drawn as that name; the rest
//! are drawn at the end of the plate under the library's own `Fx 1 Param 6`,
//! marked as doing nothing, because a byte in the program that no panel reaches
//! is a byte the modulation matrix can still be mapped onto. An engine whose
//! algorithm this firmware's table does not name — or whose type nobody has read
//! — draws all twelve that way, which is stage 3's rack for exactly as long as
//! there is nothing better to say.

use deepmind_midi::effect::{
    self, Algorithm, Character, Colour, Control, Engine, FxSlot, Panel, Quantity, grid,
};
use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::gradient::Linear;
use iced_core::{
    Background, Border, Color, Font, Gradient, Length, Radians, Theme,
    text::Renderer as TextRenderer,
};
use iced_widget::{Space, column, container, pick_list, row, stack, text};

use crate::chain;
use crate::fader;
use crate::lcd::{self, Ink, Screen, Size};
use crate::mapping::Sent;
use crate::mark;
use crate::panel::{self, Message, Room, control, lit_rather_than_listed, modulated, shown};
use crate::style::{self, materials, reading as reading_face};
use crate::{Confidence, Element, Patch, tint};

/// How much room the ten topologies are laid out in.
///
/// The band, rather than a number: they take whatever the display over them
/// leaves after the mode beside them, which is nearly all of it. A width
/// written down here was a band half full with the rest of the page blank
/// beside it — the routing is the widest thing in the group and there is
/// nothing else to give the room to.
///
/// It is still passed, because a column has to be as wide as
/// `Parallel 1/2, parallel 3/4` before the band is divided into
/// [`SETTING_COLUMNS`] of them, and this is the least that is.
const SETTING: f32 = 500.0;

/// How many columns they stand in.
///
/// Three, which is four, four and two: read down, because the manual numbers
/// them `M-1` to `M-10` and the byte follows that order, so a column is a run
/// of consecutive topologies. Three rather than two because the band under the
/// display is wide and the page is better spent across than down — ten of them
/// four deep is a block the eye takes in, and five deep was a column as tall as
/// the plates beside it.
const SETTING_COLUMNS: usize = 3;

/// How much room one whose choices are lit rather than listed is given.
///
/// `Insert`, `Send` and `Bypass` are three short words in a column of lamps.
const LIT_SETTING: f32 = 86.0;

/// Returns how one of the group's own settings is chosen.
///
/// Both of them are lit rather than listed now: three words for the mode, and
/// the ten topologies laid out in columns. Which of the two a parameter is, is
/// asked of how many choices the library gives it rather than of its name, so a
/// firmware that adds an eleventh topology gets another lamp.
fn chosen_in(parameter: ParamId, firmware: Version) -> Room {
    if lit_rather_than_listed(parameter, firmware) {
        Room::lamps(LIT_SETTING, panel::lit_band(MODE_CHOICES))
    } else {
        Room::spread(SETTING, SETTING_COLUMNS)
    }
}

/// How many settings `FX Mode` has, which is what its column of lamps is tall.
const MODE_CHOICES: usize = 3;

/// How large the output gain is drawn on the strip.
///
/// A knob, and a small one. It was a fader lying on its side and it was the
/// widest thing on the strip by some way — a hundred and fifty points for one
/// byte, on a line where the name of the algorithm had to fit as well. A knob
/// of this size is a quarter of that, and what the difference bought is the
/// room the response picture now stands in.
const GAIN: f32 = 26.0;

/// Height of the box a slot's title is set in.
///
/// Two lines of it. A column of the grid is about a sixth of half the page, so
/// `Crossover Frequency 1` wraps and `Decay` does not, and both of them have to
/// leave the line under them in the same place. Fixed and set in the middle, so
/// a title of one line sits the same distance under its value as its units sit
/// under it rather than at one end of a gap.
const TITLE: f32 = 26.0;

/// Height of the line a slot's reading is described on.
///
/// One line, always taken whether or not there is anything to say on it, so
/// that the units of one row land on one line however many of the slots above
/// them have any.
const HINT: f32 = 12.0;

/// Height of the strip a band's name is knocked out of.
const BAND: f32 = 13.0;

/// How far a band's strip stands above the row it labels.
///
/// Close, because it belongs to that row and stands in the same block.
const UNDER_STRIP: f32 = 3.0;

/// How far apart two lines saying what a slot's display shows stand.
const SHOWN_APART: f32 = 1.0;

/// How far one of them stands under the row it belongs to.
///
/// Close: it is a line about a control on that row, and the row under it is
/// [`BETWEEN_ROWS`] away, so the gap is what says which of the two it belongs
/// to.
const UNDER_ROW: f32 = 2.0;

/// How tall one column of the grid stands.
///
/// Every part of a slot at its own height, added up: where it lives, the band
/// its control stands in, the reading, the title's two lines and the line under
/// them. Written as the sum rather than as a number so that a title given a
/// third line moves this with it.
const SLOT_COLUMN: f32 = ADDRESS + CONTROL_ROW + READOUT + TITLE + HINT + WITHIN_SLOT * 4.0;

/// How tall the line a slot's address and marks stand on is.
const ADDRESS: f32 = 12.0;

/// How tall the line its value is read on is.
const READOUT: f32 = 16.0;

/// How far apart the parts of one slot stand.
const WITHIN_SLOT: f32 = 2.0;

/// How much case a band keeps above and below what it holds.
///
/// Enough that the tinted block reads as a block. A band was a pale bar over
/// some controls and nothing said where it stopped; it is a surface its own
/// columns stand on now, and a surface needs an edge somebody can see.
const BAND_PAD: f32 = 4.0;

/// How far apart two bands of one row stand.
///
/// The gap is what separates them. Two runs touching is one strip of two words
/// rather than two bands, which is what `low` and `mid` looked like.
const BESIDE_BAND: f32 = 5.0;

/// How far one row of the grid stands from the next.
///
/// Far enough that a band's strip reads as the head of the row beneath it
/// rather than as something between two rows. It is the gap either side of the
/// strip that says which row the strip is for, so this is the wider of the two
/// by some margin.
const BETWEEN_ROWS: f32 = 12.0;

/// How big the thing a hand takes hold of is drawn on an effect plate.
///
/// Smaller than a rack's slot rather than larger, which is what changed when
/// all four engines came onto one page: this is twenty-four controls, four
/// cases and a chain on one surface, and the rack's forty have a page each.
///
/// The grid publishes the instrument's own proportion — a control 11.9 wide in
/// a column pitched 20, on a 128 point display — and that is not what decides
/// this. It measures a dot matrix drawing its own labels in dots, and it does
/// not survive a title set in a real face. What this window takes from the grid
/// is the arrangement, which is published and exact.
const BODY: f32 = 40.0;

/// How deep the band a slot's control stands in is.
///
/// The taller of the two the library asks for, so a plate of knobs and a plate
/// of faders come out the same height. An algorithm's figure decides which
/// shape a hand takes hold of, and that is a fact about the algorithm; how deep
/// the row it stands in is, is a fact about the page, and four cases on one page
/// whose rows are at different heights are four cases that do not line up.
const CONTROL_ROW: f32 = TRAVEL;

/// How far a fader on one of them runs.
///
/// Shorter than a rack's, the way the instrument's own front panel runs its
/// faders short to hold two rows of them. A knob shrinks in both directions at
/// once and a fader only along its travel, so the two come down to the same
/// weight in a column at different numbers.
const TRAVEL: f32 = 50.0;

/// How big one is drawn where the algorithm does not use the byte.
const SPARE: f32 = 22.0;

/// How much of the strip the slot's own number is given.
///
/// One digit set large, and the same room whichever of the four it is, so that
/// the names start in one place down a column of cases.
const SLOT_NUMBER: f32 = 24.0;

/// How large that numeral is printed.
///
/// A cell of the display's own grid, at the pitch every display in this window
/// shares, which comes out seventeen and a half points tall: as large as the
/// strip is deep and no larger. It stands behind the line rather than in it,
/// which is what makes it a ground; a numeral *taller* than the line is a
/// numeral with its head and its feet cut off, and a digit shaved at both ends
/// reads as a mistake rather than as a mark on a case.
///
/// [`Size::Large`] is the other thing on offer and it is twice this, which is
/// half as tall again as the strip — and a bigger dot is not on offer at all,
/// here or anywhere: a display given more room gets more dots, and so does a
/// case.
const SLOT_SIZE: Size = Size::Small;

/// How far it is carried from the case towards the case's own ink.
///
/// Further than the family's mark, because it is one character where the mark
/// is a drawing across a whole face, and a numeral at a watermark's strength is
/// a numeral nobody can read. Further again now that it is printed in dots
/// rather than drawn solid: a dot covers roughly half the cell it stands in and
/// a glyph roughly half the cells of its box, so the same ink laid down the
/// same way reads about half as hard as a solid one did.
const SLOT_INK: f32 = 0.7;

/// Draws the family's mark large and faint across an engine's face.
///
/// Nothing at all where the algorithm is one this firmware cannot name, which is
/// the same rule everything else on the plate follows: there is no family to
/// mark, and a mark chosen anyway would be a guess drawn at the size of a case.
fn hero<'a, Renderer>(patch: &Patch, engine: Engine, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let Some(algorithm) = algorithm(patch, engine, firmware) else {
        return Space::new().into();
    };
    mark::hero(algorithm.mark(), |theme: &Theme| {
        let material = materials(theme);
        style::mix(material.plate, ink(theme, On::Face), HERO_INK)
    })
}

/// Draws the grain of the case an engine is in.
///
/// A rack unit's face is brushed rather than painted flat, and the one thing a
/// window can do about that at this size is put a few lines of light across it.
/// Faint enough that it reads as a surface rather than as stripes: it is there
/// to stop four large blocks of flat colour looking like four large blocks of
/// flat colour, which is the one way a case measured off a photograph still
/// gives itself away.
fn grain<'a, Renderer>(figure: Option<(Colour, Colour)>) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme: &Theme| {
            let case = chassis(theme, figure);
            let lit = style::mix(case, ink(theme, On::Case(figure)), GRAIN_INK);
            container::Style {
                background: Some(Background::Gradient(Gradient::Linear(
                    Linear::new(Radians(std::f32::consts::PI))
                        .add_stop(0.0, lit)
                        .add_stop(0.45, case)
                        .add_stop(1.0, style::mix(case, lit, 0.6)),
                ))),
                ..container::Style::default()
            }
        })
        .into()
}

/// How far the hero mark is carried from the case towards the case's own ink.
///
/// Barely. It is a watermark: enough that the eye finds it without looking and
/// never enough that a word standing over it is harder to read, which is the
/// whole difference between a mark under a panel and a picture behind one.
const HERO_INK: f32 = 0.075;

/// How far the grain is carried the same way, and how far apart its lines run.
const GRAIN_INK: f32 = 0.05;

/// How many dots across the picture of what an engine is doing is drawn.
///
/// Fixed, like every other display on this page: the glass is the instrument's
/// own and a screen stretched to whatever width a plate came out at is a
/// picture whose proportions are an accident of the window.
const PICTURE: i32 = 72;

/// How many dots down it is.
///
/// Two thirds of what it was, which is what moving it onto the engine's own
/// strip costs and is worth paying. It used to stand at the head of the grid,
/// where it was a pale rectangle as wide as two of six columns with nothing
/// beside it and the controls it is a picture of underneath — a display given a
/// row of the plate and none of the room in it.
///
/// On the strip it has a line of its own under the name of the thing it is a
/// picture of, where nothing is competing for the room and the grid can start
/// at the top of the plate. Small, because what it draws is a train of three or
/// four taps along a time: a screen big enough to be a panel of its own would
/// be claiming to say more about the engine than four gains and four times can.
const PICTURE_ROWS: i32 = 10;

/// One of an engine's twelve bytes, and what the loaded algorithm calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lane {
    /// The parameter that addresses the byte, which is the same one whatever
    /// the engine is running.
    parameter: ParamId,
    /// What the algorithm says the byte is, when it uses it at all.
    slot: Option<&'static FxSlot>,
}

/// One row of the instrument's own FX grid: what stands in each of its columns.
///
/// As long as the grid is wide, always, and a column with nothing in it is a
/// column with nothing in it rather than a column that is not there. That is
/// the whole difference between this and a row of slots: six of them are drawn
/// whatever the algorithm uses, so the second row can be read down against the
/// first.
type Line = Vec<Option<Lane>>;

/// Returns the engines `group` holds, when it is the one that holds them.
///
/// Asked of the library rather than written down: an engine's type parameter
/// knows its own group, so the effects are wherever the parameter table puts
/// them and a group with no engine in it is every other group.
pub(crate) fn engines(group: Group) -> Option<Vec<Engine>> {
    let found: Vec<Engine> = Engine::ALL
        .into_iter()
        .filter(|engine| engine.algorithm_parameter().group() == group)
        .collect();
    (!found.is_empty()).then_some(found)
}

/// Returns the parameters this panel draws rather than leaving to the rack.
///
/// The whole group, where the group holds the engines. Four plates of fourteen
/// parameters each are not a layout a rack of two leftovers sits under: the
/// settings that are not an engine's are the first thing on the panel instead,
/// which is where a connection mode belongs and where a parameter a later
/// library adds to this group will appear.
pub(crate) fn claimed(group: Group) -> Vec<ParamId> {
    match engines(group) {
        Some(_) => group.parameters().collect(),
        None => Vec::new(),
    }
}

/// Returns the group's own settings, which are what no engine addresses.
///
/// The connection mode and whether the effects are inserted, sent or bypassed.
/// Found by subtraction rather than by name, so that this is "everything in the
/// effects that is not one of the four engines" and stays that whatever the
/// library adds.
fn settings(group: Group) -> Vec<ParamId> {
    let engines = engines(group).unwrap_or_default();
    group
        .parameters()
        .filter(|parameter| !engines.iter().any(|engine| addresses(*engine, *parameter)))
        .collect()
}

/// Returns whether `engine` addresses `parameter`: its type, its gain, or one
/// of its twelve.
fn addresses(engine: Engine, parameter: ParamId) -> bool {
    engine.algorithm_parameter() == parameter
        || engine.gain_parameter() == parameter
        || engine.slot_parameters().contains(&parameter)
}

/// Returns whether an engine's own switch has it in circuit.
///
/// `None` for the 32 algorithms that have no such switch, which is the answer
/// rather than a gap: `FX n Type` is 35 effects with no `Off` in the table, the
/// Effects group has no per-engine enable, and `FX Mode` is the whole block of
/// four at once
/// ([deepmind-midi#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30)).
/// Three algorithms spend one of their twelve bytes on it instead — Stereo
/// Imaging and Chorus D on an `ON`, the Noise Gate on a `PWR` — and
/// [`FxSlot::is_enable`] is what says which, so this window does not match on
/// those two words across 35 panels.
///
/// Which way round the switch reads is the slot's own two ends, because the
/// Noise Gate is the one that reads `ON` at the bottom of its range. The byte
/// is a byte and the panel is two states, so the half of the range it is in is
/// what decides.
pub(crate) fn switched_on(
    patch: &Patch,
    engine: Engine,
    algorithm: &'static Algorithm,
) -> Option<bool> {
    let slot = algorithm.slots.iter().find(|slot| slot.is_enable())?;
    let value = patch.value(engine.slot_parameter(slot.slot)?)?;
    let high = value > u8::MAX / 2;
    Some(if slot.min == Some("ON") { !high } else { high })
}

/// Draws the picture of what an engine is doing to a signal, where there is one.
///
/// [`effect::response`] answers for two of the 35 — the 3-Tap and the 4-Tap
/// delays, whose panels are literally a time and a gain per tap and whose times
/// are ratios of the master delay that the manual prints as fractions. Every
/// other engine gets nothing, and nothing is the honest answer: a reverb's
/// impulse response is its designer's, a compressor's knee is not published,
/// and a picture of either would look like information and not be any
/// ([deepmind-midi#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33)).
///
/// The horizontal is `Scale::Normalised` — the ratios between the taps are
/// published and the master time they are ratios *of* is a byte with no
/// published curve — so the glass carries the train and no axis.
fn picture<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    // The library reads the type byte on the newest firmware, as a program's own
    // accessors do, and this page reads it on whichever one answered the
    // inquiry. On a unit running 1.0 those are two different algorithms above
    // the one 1.1 inserted, so where they disagree there is no picture: a delay
    // drawn for an engine this firmware calls something else is the one thing a
    // screen must not do.
    let loaded = algorithm(patch, engine, firmware)?;
    let value = patch.value(engine.algorithm_parameter())?;
    if Algorithm::for_value(value, DEFAULT_FIRMWARE) != Some(loaded) {
        return None;
    }
    let response = effect::response(patch.program()?, engine)?;
    // The type as well as the twelve, because which of the twelve are taps is
    // what the type says.
    let claim = patch.claim_across(
        engine
            .slot_parameters()
            .iter()
            .copied()
            .chain([engine.algorithm_parameter()]),
    );
    let mut screen = Screen::new(PICTURE, PICTURE_ROWS);
    if !matches!(claim, Confidence::Unknown) {
        let band = screen.all();
        // The floor the taps stand on, so an engine whose taps are all silent
        // is a picture of a delay rather than an empty screen.
        screen.across(band.x, band.row(0.0), band.width, Ink::Dotted);
        screen.under(band, |x| response.at(x));
        screen.curve(band, Ink::Solid, |x| response.at(x));
    }
    Some(
        container(lcd::lcd(screen, claim))
            .width(Length::Fixed(lcd::room(PICTURE)))
            .height(Length::Fixed(lcd::room(PICTURE_ROWS)))
            .into(),
    )
}

/// Returns what `engine` is running, as far as this window knows.
///
/// `None` for a type nobody has read, and for a byte this firmware's table does
/// not name, which is what a dump from a unit running the other firmware can
/// hand over. Both of those are "there is nothing to call these twelve bytes",
/// and drawing them under the library's own names is the honest answer to it.
fn algorithm(patch: &Patch, engine: Engine, firmware: Version) -> Option<&'static Algorithm> {
    let value = patch.value(engine.algorithm_parameter())?;
    Algorithm::for_value(value, firmware)
}

/// Returns the twelve bytes of `engine` and what the loaded algorithm calls
/// each.
pub(crate) fn lanes(patch: &Patch, engine: Engine, firmware: Version) -> Vec<Lane> {
    let algorithm = algorithm(patch, engine, firmware);
    engine
        .slot_parameters()
        .iter()
        .copied()
        .map(|parameter| Lane {
            parameter,
            slot: algorithm.and_then(|algorithm| algorithm.slot_of(engine, parameter)),
        })
        .collect()
}
/// Gathers the lanes onto the grid the instrument's own FX page draws them on.
///
/// Six columns and two rows, and every slot in the column and row
/// [`FxSlot::position`] measured for it. That is the whole of the arrangement:
/// a slot is not placed by counting along a row, it is placed where the
/// instrument places it, so the third column of the second row stands under the
/// third column of the first however many slots either row happens to hold.
///
/// Returns the rows, and whatever has no place on the grid. The second is the
/// bytes the loaded algorithm does not use — as many as seven of the twelve —
/// and, if a later library ever measured a slot outside the grid it publishes,
/// that slot as well. Either way nothing is dropped: a byte this panel does not
/// draw is a byte the modulation matrix can still be mapped onto.
///
/// Everything with no place, where nothing is known about the algorithm. That
/// is stage 3's rack: twelve bytes under the library's own names, and no page
/// to lay them out by.
fn placed(lanes: &[Lane], panel: Option<&'static Panel>) -> (Vec<Line>, Vec<Lane>) {
    let columns = usize::from(grid().columns());
    let Some(panel) = panel else {
        return (Vec::new(), lanes.to_vec());
    };
    // This algorithm's own depth rather than the grid's. The grid has two rows
    // and a five-slot algorithm fills one of them, and reserving the second
    // anyway bought four cases of one height at the price of half of each case
    // being nothing: a rack unit whose lower half is blank panel is a rack unit
    // somebody would ask what is missing from. Four cases at four heights is
    // what four engines running four algorithms are.
    //
    // Deep enough for what is measured, which is [`Panel::rows`] and is checked
    // against the positions as well: a slot measured onto a row the panel does
    // not list is drawn where it was measured rather than swept into the strip
    // of bytes the algorithm does not use.
    let measured = lanes
        .iter()
        .filter_map(|lane| lane.slot)
        .map(|slot| usize::from(slot.position().row()) + 1)
        .max()
        .unwrap_or(0);
    let deep = measured.max(panel.rows().len());
    let mut rows: Vec<Line> = vec![vec![None; columns]; deep];
    let mut spare = Vec::new();
    for lane in lanes.iter().copied() {
        let cell = lane.slot.map(FxSlot::position).and_then(|at| {
            rows.get_mut(usize::from(at.row()))
                .and_then(|row| row.get_mut(usize::from(at.column())))
        });
        match cell {
            // An occupied cell means two slots measured to the same place,
            // which the specification does not contain and which this would
            // rather draw at the end than lose.
            Some(cell) if cell.is_none() => *cell = Some(lane),
            _ => spare.push(lane),
        }
    }
    (rows, spare)
}

/// Returns the runs of columns one band covers, along one row.
///
/// In column order and covering every column, so that a strip built from this
/// and the row of slots under it divide the same width the same way: a run of
/// three labelled columns is one entry three wide, and a column the library
/// labelled nothing is an entry of its own one wide, whether or not the column
/// beside it is also unlabelled. Two unlabelled slots side by side are two
/// slots and not a pair, which is the rule the clusters followed before the
/// grid replaced them, and an empty column is not a band either.
fn spans(line: &Line) -> Vec<(Option<&'static str>, usize)> {
    let mut runs: Vec<(Option<&'static str>, usize)> = Vec::new();
    for cell in line {
        let label = cell.and_then(|lane| lane.slot).and_then(|slot| slot.group);
        match runs.last_mut() {
            Some((last, run)) if *last == label && label.is_some() => *run += 1,
            _ => runs.push((label, 1)),
        }
    }
    runs
}

/// Which of the effects page's surfaces a word is printed on.
///
/// Every colour in this window is chosen for what it means, and what it lands
/// on here is not always chosen at all: an engine's case is a colour somebody
/// measured off a photograph of a rack unit, and half of the 35 are pale. So
/// nothing on this page names an ink directly. It names the surface, and
/// [`ink`] and [`legend`] work out what can be read on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum On {
    /// The face an engine's controls stand on, which is the window's own plate.
    Face,
    /// The case around them, carrying the algorithm's own measured chassis.
    Case(Option<(Colour, Colour)>),
    /// The strip a band's name is knocked out of.
    Strip,
    /// The panel the chain and the unnamed bytes are cut into.
    Recess,
}

impl On {
    /// Returns the colour of the surface itself.
    fn colour(self, theme: &Theme) -> Color {
        let material = materials(theme);
        match self {
            Self::Face => material.plate,
            Self::Case(figure) => chassis(theme, figure),
            Self::Strip => banding(theme),
            Self::Recess => material.panel,
        }
    }
}

/// Returns the ink a word on `on` is printed in.
///
/// Whichever of the instrument's two the surface can be read against. Nothing
/// on this page picks an ink itself, because half the surfaces on it are
/// measured off somebody else's rack unit and a window that printed every title
/// in the one ink it uses everywhere else would have picked the wrong one for
/// half the algorithms.
fn ink(theme: &Theme, on: On) -> Color {
    style::ink_on(on.colour(theme), theme)
}

/// The same ink, at the strength a legend is silkscreened in.
///
/// Half way back to the surface it is printed on and then lifted until it is
/// readable, which is to say: the dimmest a legend can be on this surface and
/// still be a legend. A grey chosen once for a dark window is a grey that
/// disappears on a cream one, and the tint of a value nobody has read — which
/// is what these words used to be drawn in — said of a printed word that its
/// value was unknown.
fn legend(theme: &Theme, on: On) -> Color {
    let surface = on.colour(theme);
    let full = style::ink_on(surface, theme);
    style::legible(style::mix(full, surface, 0.5), surface, theme)
}

/// Grey text at legend strength, on `on`.
fn printing<'a, Renderer>(what: String, on: On) -> iced_widget::Text<'a, Theme, Renderer>
where
    Renderer: TextRenderer,
{
    text(what).style(move |theme: &Theme| text::Style {
        color: Some(legend(theme, on)),
    })
}

/// Draws what a parameter is holding, in the colour of what backs it.
///
/// [`panel::readout`](crate::panel::readout) with the surface taken into
/// account: a claim is amber or green because of what it means, and an engine
/// wearing a cream case is a surface neither of them was chosen against. What
/// moves is how light the colour is and never which colour it is, so an amber
/// claim stays amber and the distinction the whole editor turns on survives
/// being printed on somebody else's rack unit.
fn reading<'a, Renderer>(
    parameter: ParamId,
    value: Option<u8>,
    claim: Confidence,
    firmware: Version,
    on: On,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    text(shown(parameter, value, firmware))
        .size(11)
        .font(reading_face())
        .style(move |theme: &Theme| text::Style {
            color: Some(style::legible(tint(theme, claim), on.colour(theme), theme)),
        })
        .into()
}

/// Draws the effects: the chain, what shapes it, and the four engines under it.
///
/// All four at once, in the two-by-two the four of them make. One at a time
/// behind a row of tabs was an engine you could see and three you had to
/// remember, on a page whose whole subject is what four effects are doing
/// together — and the chain drawn across the top of it is a picture of exactly
/// that. A tab row is the right shape for a section of an instrument and the
/// wrong one for four things wired to each other.
pub(crate) fn panels<'a, Renderer>(
    patch: &Patch,
    group: Group,
    firmware: Version,
    moved: &[ParamId],
    sent: Option<Sent<'_>>,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let engines = engines(group)?;
    // The picture, and under it the two settings that shape it. The routing
    // *is* the chain — which engine feeds which is the drawing, and whether the
    // four of them are inserted, sent or bypassed is the rest of the same
    // sentence — so they stand together.
    //
    // Under rather than beside, which is what changed when the glass grew wide
    // enough to name what each engine is running inside its own box. The
    // display takes the width of the block now, and a topology chosen from a
    // drop-down beside it was the one control on this page that hid nine of its
    // ten choices behind the one already taken.
    let block = row(settings(group).into_iter().map(|parameter| {
        cell(
            patch,
            parameter,
            firmware,
            chosen_in(parameter, firmware),
            parameter.short_name(),
            moved,
            On::Recess,
            sent,
        )
    }))
    .spacing(14)
    .align_y(Vertical::Top);
    let wiring = container(
        column![
            // The glass takes the band. How many dots that is, is the band's
            // question; that no fewer than [`chain::columns`] of them will do
            // is the drawing's, because below that a box cannot name what is
            // running in it.
            container(chain::display(patch, firmware)).width(Length::Fill),
            block,
        ]
        .spacing(10)
        // What the specification records about a topology beyond its graph,
        // which is where the loop taps on the two that have one. Under the
        // settings rather than beside them: it is a sentence, and a sentence
        // sharing a row with the ten topologies is a sentence taking the room
        // they are laid out in.
        .extend(chain::note(patch).map(|note| {
            container(printing(note.to_owned(), On::Recess).size(9))
                .width(Length::Fill)
                .into()
        })),
    )
    .padding(8)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let material = materials(theme);
        container::Style {
            background: Some(Background::Color(material.panel)),
            border: Border {
                color: material.recess_edge,
                width: 1.0,
                radius: 3.into(),
            },
            ..container::Style::default()
        }
    });
    let mut body = column![wiring].spacing(8);
    // Two across, in the order the instrument numbers them. `chunks` rather
    // than a pair of indexes so that a library that ever published a fifth
    // engine gets a third row rather than a panel nobody can reach.
    for pair in engines.chunks(2) {
        body = body.push(
            row(pair
                .iter()
                .map(|engine| plate(patch, *engine, firmware, moved, sent)))
            .spacing(8),
        );
    }
    Some(body.into())
}

/// Draws one row of a plate's grid: six columns, each of them a slot or empty.
///
/// Every column is the same width and is there whether or not anything stands
/// in it, which is what makes this a grid rather than six things in a row: an
/// algorithm using five slots on its first row and six on its second draws the
/// five under the first five, where the instrument draws them, instead of
/// spreading them over a row of its own.
fn line_of<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    line: &Line,
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mut cells = line.iter().map(|cell| match cell {
        Some(byte) => slot(patch, engine, *byte, firmware, moved, figure, sent),
        // A column the grid has nothing in still stands the height of one, so
        // that a row of two slots is as deep as a row of six and a plate is as
        // deep as the grid rather than as deep as what happens to be on it.
        None => Element::from(Space::new().width(Length::Fill).height(SLOT_COLUMN)),
    });
    // One run at a time, each of them a block: the strip and the columns it
    // covers stand in the same container, so a band is a thing on the plate
    // rather than a bar with some controls somewhere under it. The container
    // takes no width of its own — a run of three is three of the grid's own
    // portions — so the columns still line up down the rows, which is what the
    // grid is for.
    let runs = spans(line).into_iter().map(|(label, across)| {
        let portion = u16::try_from(across).unwrap_or(1);
        let held: Vec<Element<'a, Renderer>> = cells.by_ref().take(across).collect();
        let banded = label.is_some();
        Element::from(
            container(
                column![label.map_or_else(
                    || Element::from(Space::new().height(Length::Fixed(BAND))),
                    strip,
                )]
                .push(row(held).spacing(0))
                .spacing(UNDER_STRIP),
            )
            .width(Length::FillPortion(portion))
            .padding([BAND_PAD, 0.0])
            .style(move |theme: &Theme| {
                if banded {
                    container::Style {
                        background: Some(Background::Color(bedding(theme))),
                        border: Border {
                            color: materials(theme).recess_edge,
                            width: 1.0,
                            radius: 3.into(),
                        },
                        ..container::Style::default()
                    }
                } else {
                    container::Style::default()
                }
            }),
        )
    });
    // What the display will show where one of this row's slots shows names
    // rather than a number, printed under that row. It used to be printed
    // under the whole grid, which put `PST Preset shows Ambience, Church, …`
    // at the foot of the case with two rows of controls between it and the
    // knob it is about — and the reference at the head of it was the only
    // thing saying which knob that was.
    let shown: Vec<Element<'a, Renderer>> = displays(line).map(Element::from).collect();
    if shown.is_empty() {
        return row(runs).spacing(BESIDE_BAND).into();
    }
    column![
        row(runs).spacing(BESIDE_BAND),
        column(shown).spacing(SHOWN_APART),
    ]
    .spacing(UNDER_ROW)
    .into()
}

/// The pale strip a band's name is knocked out of.
///
/// How a `DeepMind` prints `ARP / SEQ` and `VCF` across the top of a group, and
/// the first thing the eye follows across the instrument's own panel.
fn strip<'a, Renderer>(label: &'static str) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(label)
            .size(8)
            .font(reading_face())
            .style(move |theme: &Theme| text::Style {
                color: Some(ink(theme, On::Strip)),
            }),
    )
    .width(Length::Fill)
    .height(Length::Fixed(BAND))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(banding(theme))),
        border: Border {
            color: materials(theme).metal,
            width: 1.0,
            radius: 2.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// What an algorithm's own printed figure says about how to draw its slots.
///
/// Two things, and both of them answered by the same `Option`: an engine
/// running something this firmware cannot name has no figure, and then a slot
/// is a fader on the window's own plate, which is what it always was.
///
/// The colours are not in here. Four engines stand on this page at once, and
/// four measured liveries side by side are the collage this window has always
/// refused to be — so what an engine wears is its case and a hairline of its
/// accent, and both of those belong to the plate rather than to a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Figure {
    /// What shape a control that sweeps a range takes.
    control: Option<Control>,
}

impl Figure {
    /// Returns what the algorithm's own panel is made of.
    fn of(panel: Option<&'static Panel>) -> Self {
        Self {
            control: panel.map(Panel::control),
        }
    }

    /// Returns the room one of the algorithm's slots is drawn in.
    ///
    /// A knob where the figure draws one, which is 29 of the 35. The size is
    /// the page's rather than the rack's: four engines on one surface is
    /// twenty-four controls and a chain, so a slot here is smaller than a slot
    /// in a rack of forty rather than larger, and the travel comes down with it
    /// the way the instrument's own front panel brings it down to hold two rows.
    fn room(self, spare: bool) -> Room {
        let body = if spare { SPARE } else { BODY };
        match self.control {
            Some(Control::Knob) => Room::SLOT.turned().sized(body),
            // A fader keeps the width a fader is, and gives up travel instead:
            // there is one cap in this window and it is as wide as it is.
            _ => Room::lane(fader::WIDTH, if spare { SPARE } else { TRAVEL }),
        }
    }
}

/// Draws one of an engine's twelve bytes, under the name the algorithm gives it.
///
/// The control is the same one the rest of the editor draws from the same
/// table, in the shape the algorithm's own figure uses: 29 of the 35 are knobs,
/// five are faders and one is a numeric display, and the library says which
/// without saying anything about what the byte is. A knob and a fader are the
/// same control over the same range with the same drag — see [`knob`](crate::knob)
/// — so the shape is an arrangement, which is the one thing hand layout here is
/// allowed to choose.
///
/// The display is drawn as a fader. It is the one shape this window does not
/// have and the one algorithm that wants it, and a control invented for a
/// single figure would be a worse lie than the fader that is already honest
/// about the byte underneath.
///
/// The title and the reading under it are two fixed boxes rather than one, so
/// that the units of a row land on one line whether the title above them ran to
/// one line or to two. A slot whose title wraps pushes nothing down but itself.
fn slot<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    lane: Lane,
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let title = lane
        .slot
        .map_or_else(|| within(engine, lane.parameter), |slot| slot.title);
    column![
        row![
            at(lane.parameter, On::Face),
            reference(lane.slot),
            Space::new().width(Length::Fixed(1.0)),
            modulated(
                moved.contains(&lane.parameter),
                lane.slot.is_none_or(|slot| slot.modulatable),
            ),
        ]
        .spacing(3)
        .align_y(Vertical::Center),
        container(control(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
            figure.room(false),
            sent,
        ))
        .height(Length::Fixed(CONTROL_ROW))
        .align_y(Vertical::Center),
        reading(
            lane.parameter,
            patch.value(lane.parameter),
            patch.claim(lane.parameter),
            firmware,
            On::Face,
        ),
        container(
            row![
                pictured(lane.slot),
                text(title)
                    .size(10)
                    .center()
                    .style(move |theme: &Theme| text::Style {
                        color: Some(ink(theme, On::Face)),
                    })
            ]
            .spacing(3)
            .align_y(Vertical::Center)
        )
        .height(Length::Fixed(TITLE))
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center),
        container(printing(hint(lane.slot), On::Face).size(9).center())
            .height(Length::Fixed(HINT))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    ]
    .spacing(2)
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// Draws the twelve bytes of an engine whose algorithm has no name here.
///
/// The one case left. An algorithm that *is* named leaves as many as seven of
/// its twelve unused, and those are no longer drawn: moving one does nothing a
/// player can hear, a byte the loaded algorithm does not read is not a control,
/// and a strip of seven of them under the five that do something was the
/// loudest half of a plate spent on the half that does nothing. They are still
/// in the program, still sent, and still reachable from the modulation matrix,
/// which is where a byte with no panel belongs.
///
/// What this is for is the other thing: an engine running a type byte this
/// firmware's table does not name, which is what a dump from a unit on the
/// other firmware hands over. There is no panel to lay out and no name to print
/// for any of the twelve, so all twelve go under the library's own `Param 9` —
/// stage 3's rack, for exactly as long as there is nothing better to say.
fn spare<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    lanes: &[Lane],
    firmware: Version,
    moved: &[ParamId],
    figure: Figure,
    sent: Option<Sent<'_>>,
) -> Option<Element<'a, Renderer>>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    if lanes.is_empty() {
        return None;
    }
    let bytes = lanes.iter().map(|lane| {
        column![
            row![
                at(lane.parameter, On::Recess),
                Space::new().width(Length::Fixed(1.0)),
                modulated(moved.contains(&lane.parameter), true),
            ]
            .spacing(3)
            .align_y(Vertical::Center),
            control(
                lane.parameter,
                patch.value(lane.parameter),
                patch.claim(lane.parameter),
                firmware,
                figure.room(true),
                sent,
            ),
            printing(within(engine, lane.parameter).to_owned(), On::Recess).size(9),
        ]
        .spacing(WITHIN_SLOT)
        .width(Length::Fixed(SPARE + 20.0))
        .align_x(Horizontal::Center)
        .into()
    });
    Some(
        container(row(bytes).spacing(4).wrap())
            .width(Length::Fill)
            .padding(5)
            .style(|theme: &Theme| {
                let material = materials(theme);
                container::Style {
                    background: Some(Background::Color(material.panel)),
                    border: Border {
                        color: material.recess_edge,
                        width: 1.0,
                        radius: 2.into(),
                    },
                    ..container::Style::default()
                }
            })
            .into(),
    )
}

/// Draws one engine: what it is running, and what that makes its twelve bytes.
///
/// On the grid the instrument's own FX page uses. 26.3 published it — six
/// columns and two rows, measured off the 35 screenshots in the manual, and
/// every slot's column and row on it
/// ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22))
/// — so a plate is not twelve slots wrapped into whatever width the window
/// happened to have, and it is not six things in a row either. It is the six
/// columns, each of them there whether or not a slot stands in it, spread
/// across the whole of this engine's half of the page. That is the arrangement
/// anybody who has edited an effect on the hardware already knows, and it is
/// the arrangement a second row can be read down against.
///
/// [`Row::align`] is what the library offers a host that lays a partial row out
/// some other way, and this is not one: a slot here is in the column that was
/// measured for it, so a row that does not fill the grid is already packed
/// where the page packs it and there is no room left over to place.
fn plate<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let lanes = lanes(patch, engine, firmware);
    let panel = algorithm(patch, engine, firmware).map(Algorithm::panel);
    let figure = Figure::of(panel);
    let (grid, left) = placed(&lanes, panel);
    let mut drawn = column![].spacing(BETWEEN_ROWS);
    for line in &grid {
        drawn = drawn.push(line_of(patch, engine, line, firmware, moved, figure, sent));
    }
    // The family's mark, large and faint, under the grid rather than under the
    // case: the face plate the controls stand on is opaque, so a watermark
    // behind that is a watermark nobody sees. See [`mark::hero`].
    let drawn = container(stack![drawn].push_under(hero(patch, engine, firmware)))
        .width(Length::Fill)
        .padding(7)
        .style(|theme: &Theme| {
            let material = materials(theme);
            container::Style {
                background: Some(Background::Color(material.plate)),
                border: Border {
                    color: material.recess_edge,
                    width: 1.0,
                    radius: 2.into(),
                },
                ..container::Style::default()
            }
        });
    let body = column![header(patch, engine, firmware, moved, panel, sent)]
        .push(drawn)
        // The twelve under the library's own names, for an engine running an
        // algorithm this firmware's table cannot name. That is the only case
        // left: where the algorithm *is* named, the bytes it does not use are
        // not drawn at all — see [`spare`].
        .extend(
            panel
                .is_none()
                .then(|| spare(patch, engine, &left, firmware, moved, figure, sent))
                .flatten(),
        )
        .spacing(6);
    let figure_colours = panel.map(|panel| (panel.chassis(), panel.accent()));
    // The family's mark, large and faint, under everything the case carries,
    // and the case's own grain under that. See [`mark::hero`] for why the mark
    // is there rather than on the strip.
    // `push_under` rather than a layer over the top, because a stack takes its
    // size from its base and the base has to be what is actually on the case: a
    // grain is as tall as whatever it is behind, and a stack sized from one is
    // a case with no height at all.
    container(stack![body].push_under(grain(figure_colours)))
        .padding(6)
        .width(Length::Fill)
        .clip(true)
        // As deep as what is in it, which is not the same depth as the case
        // beside it. A row is still a row of the grid — every column stands a
        // column's height whether or not a slot is in it, every run keeps the
        // room a band's strip takes, and every control stands in a band of one
        // depth whether the figure calls for a knob or a fader — so two engines
        // running algorithms of the same shape do come out level. What is no
        // longer reserved is the shape an algorithm does not have: a five-slot
        // reverb was drawing the grid's empty second row and the deepest band
        // of display names any of the 35 needs, and half of that case was
        // blank panel waiting for an algorithm that was not loaded.
        .style(move |theme: &Theme| {
            // Cut into the group's face plate rather than raised off it: the
            // plate is what the four engines are recessed into, which is the
            // same trick the section bar plays with the panel it is cut from.
            container::Style {
                background: Some(Background::Color(chassis(theme, figure_colours))),
                border: Border {
                    color: rim(theme, figure_colours),
                    width: 1.0,
                    radius: 3.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Returns the colour the case an engine is drawn in is painted.
///
/// The measured chassis of the algorithm's own figure, most of the way back to
/// the panel this window is. The library publishes four colours per algorithm —
/// [deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)
/// — and they are the colours of 35 imaginary rack units: a cream fader panel,
/// a black one, a blue-grey one.
///
/// Two of the four are spent and two are not, and which two depends on how many
/// units are on the surface. With one open at a time this page wore the lot,
/// case and face and cap, because a single unit can be that unit without the
/// window becoming a shelf of other people's boxes. Four stand on it now, and
/// four liveries side by side are exactly the collage this editor's whole
/// argument is against — so the face went back to the window's own plate and
/// the cap back to the window's own metal, and what is left is the case and a
/// hairline of the accent: enough to tell the reverb from the distortion at
/// arm's length, which is all four units at once can afford to say.
fn chassis(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    let Some((chassis, _)) = figure else {
        return material.panel;
    };
    let worn = colour(chassis);
    // As much of the case as can be worn and still be printed on. A measured
    // chassis carried half way into the panel lands, for the cream ones, in the
    // exact middle of the instrument's two inks — where the best either can do
    // is 4.45 against it, and every word on the strip is a word somebody has to
    // squint at. So the carry is not a number somebody picked: it is the most
    // of the livery that leaves the case readable, found by backing off towards
    // the panel until it is.
    //
    // Backing off darkens, because the panel is dark, so this always terminates
    // at the panel itself — which is what an engine with no figure wears.
    (0..=CARRY)
        .rev()
        .map(|step| {
            style::mix(
                material.panel,
                worn,
                f32::from(step) / f32::from(CARRY) * FULL,
            )
        })
        .find(|case| style::contrast(*case, style::ink_on(*case, theme)) >= style::READABLE)
        .unwrap_or(material.panel)
}

/// How far the measured chassis is carried into the panel, at most.
///
/// Far enough to be the colour of the thing rather than a tint on the window,
/// and not so far that four cases side by side stop being one instrument.
const FULL: f32 = 0.5;

/// How finely [`chassis`] looks for the most of it that can be printed on.
const CARRY: u8 = 20;

/// Returns the colour of the strip a band's name is knocked out of.
///
/// The instrument's own device: a pale bar across the top of a group, the same
/// one the front panel prints `ARP / SEQ` and `VCF` on. Off the plate rather
/// than off the case, because a band belongs to the grid it is printed over and
/// the grid stands on the window's own plate whatever the engine is wearing.
fn banding(theme: &Theme) -> Color {
    let material = materials(theme);
    style::mix(material.plate, material.metal_low, 0.62)
}

/// The surface the columns of one band stand on.
///
/// The strip's own colour taken most of the way back to the plate. The strip is
/// a silkscreened label and is as pale as one; what is under it is a part of
/// the plate that has been grouped, and a group of controls drawn on a surface
/// as pale as its own label is a group whose readings and addresses have to be
/// re-inked to be seen at all. A tint is enough to say *these six belong
/// together*, which is the whole job.
fn bedding(theme: &Theme) -> Color {
    let material = materials(theme);
    style::mix(material.plate, banding(theme), 0.16)
}

/// Returns the colour the plate's edge is drawn in.
///
/// The figure's accent, which the library defines as the most saturated colour
/// covering a visible share of the printed panel: the lit label strip on a
/// cream unit, the LED on a black one. A hairline of it is what says which
/// engine is which without a second word on the plate.
fn rim(theme: &Theme, figure: Option<(Colour, Colour)>) -> Color {
    let material = materials(theme);
    figure.map_or(material.recess_edge, |(_, accent)| {
        style::mix(material.recess_edge, colour(accent), 0.55)
    })
}

/// Returns a colour the library measured, as a colour this window can draw.
///
/// Three components rather than a parse, because three components are what the
/// library publishes and what a window wants; the one thing this does is put
/// them on the scale `iced` uses.
fn colour(measured: Colour) -> Color {
    let [red, green, blue] = measured.to_rgb();
    Color::from_rgb8(red, green, blue)
}

/// Draws the strip across the top of one engine's case.
///
/// Everything an engine is, in the order somebody reads it. What it is, on the
/// left: the mark of its family, the slot it is in, what it is running written
/// out, and the manual's own category after it. What to do about it, on the
/// right: the list that changes the algorithm and the fader that sets how loud
/// it comes out.
///
/// That split is the whole of the arrangement. Six things strung along one line
/// left a name squeezed between a drop-down and a fader, and the name is the
/// thing on the strip that is read rather than operated. The algorithm is still
/// the parameter's own value table under the abbreviations the instrument's
/// display prints — a list, which is to say the thing you press to change it —
/// with what those stand for written out beside it rather than substituted into
/// it, because the control is the library's and the prose is the panel's.
///
/// # One line, because the strip is not the subject
///
/// The category sat on a line of its own under the name, which made every strip
/// two lines of text tall — and the strip is a label on a case, not the thing
/// on the page anybody is reading. Four of them down a page is four times what
/// that costs. `Reverb` and `Processing` are one word each and they follow the
/// name the way a subtitle does, so they go beside it at legend weight and the
/// strip comes down to roughly two thirds of what it stood at.
///
/// This is where the page it replaced had a whole header band of its own, under
/// a row of tabs that had the name on it as well. Two places saying which
/// algorithm an engine is running, and neither of them where the engine was.
fn header<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
    panel: Option<&'static Panel>,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let on = On::Case(panel.map(|panel| (panel.chassis(), panel.accent())));
    let strip = row![
        // The gutter the slot's own number stands in, which is drawn behind the
        // strip rather than in it — see below.
        Space::new().width(Length::Fixed(SLOT_NUMBER)),
        // The list *is* the title. It was a list of the display's own
        // abbreviations with what they stand for written out beside it, which
        // is two controls' worth of room saying one thing: the name is what a
        // reader wants and the list is what a hand wants, and a list whose
        // entries are the names is both. The abbreviation has not gone
        // anywhere — it is what the chain's own glass prints in every box, and
        // what the footer says about the byte.
        container(named(patch, engine, firmware, on)).width(Length::Fill),
        // What kind of thing it is, beside what it does. One or two quiet
        // words, and nothing at all for the plain reverbs and the noise gate,
        // whose family says everything there is to say about them.
        kind(patch, engine, firmware, on),
        // Where an engine carries its own switch and that switch is off, that
        // it is out of circuit. Three of the 35 can say it and the other 32
        // cannot, which is the instrument's answer rather than this window's.
        // The manual's category went with the second line the strip used to
        // have: it is one word about a family whose mark is already on the
        // case, and the footer says it of whatever is under the pointer.
    ]
    // What the engine is doing to a signal, for the two of the 35 whose panels
    // say it outright. On the strip, between the name and the level, which is
    // the room the gain gave up by turning into a knob.
    .extend(picture(patch, engine, firmware))
    .push(output(patch, engine, firmware, moved, sent))
    .spacing(10)
    .align_y(Vertical::Center);
    // Which slot this is, set the way the family's mark is set on the case:
    // large, faint, and behind what the strip carries rather than beside it.
    // `FX 1` in a box of its own was two characters saying what the page
    // already says and a third competing with the name of the algorithm; the
    // number alone is the one thing here that has to be read without reading,
    // and a numeral standing behind the line is how a rack unit puts a channel
    // number on a case.
    //
    // It runs taller than the strip and is clipped by it, which is what stops
    // it from setting the height of a line it is only the ground of.
    stack![strip]
        .push_under(
            container(lcd::stencil(
                Screen::of(&engine.number().to_string(), SLOT_SIZE),
                move |theme: &Theme| {
                    style::mix(
                        chassis(theme, panel.map(|panel| (panel.chassis(), panel.accent()))),
                        ink(theme, on),
                        SLOT_INK,
                    )
                },
            ))
            .width(Length::Fixed(SLOT_NUMBER))
            // The height as well as the width. A stack lays its under-layers
            // out at its own size and puts them at its own origin, so a layer
            // that shrinks to its contents is a layer aligned against nothing:
            // the numeral sat against the top of the strip with the gap under
            // it, which is what `align_y` was supposed to be deciding.
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .clip(true),
        )
        .into()
}

/// Draws the list of what an engine could be running, under the names.
///
/// The same parameter, the same value table and the same bytes as the rack's
/// own control draws; what changes is which of the library's two names for an
/// algorithm each entry carries. `Algorithm::full_name` is the library's own
/// join between the two, so this is not the panel translating anything — it is
/// the panel choosing which of the published names has the room.
fn named<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    on: On,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = engine.algorithm_parameter();
    let loaded = algorithm(patch, engine, firmware);
    let live = !matches!(patch.claim(parameter), Confidence::Unknown);
    // Only the algorithms this firmware's table has a byte for. A unit running
    // the older firmware names two fewer, and an entry that cannot be selected
    // is an entry that does nothing when it is.
    let offered: Vec<&'static Algorithm> = Algorithm::all()
        .iter()
        .filter(|algorithm| algorithm.value_for(firmware).is_some())
        .collect();
    let chosen = loaded.map(Named);
    let list = pick_list(
        offered.into_iter().map(Named).collect::<Vec<_>>(),
        chosen,
        move |Named(algorithm)| match algorithm.value_for(firmware) {
            Some(value) => Message::Edit { parameter, value },
            // Unreachable: the list only offers what this firmware names.
            None => Message::Pointed(None),
        },
    )
    .text_size(13)
    .padding([3, 8])
    .width(Length::Fill)
    .style(style::selector);
    if live {
        Element::from(list)
    } else {
        // Nothing has been read, so there is nothing to change: the same list,
        // and no way to send a byte the window would then be claiming.
        Element::from(
            container(
                text(loaded.map_or("nothing has been read", |algorithm| algorithm.full_name))
                    .size(13)
                    .style(move |theme: &Theme| text::Style {
                        color: Some(ink(theme, on)),
                    }),
            )
            .padding([3, 8]),
        )
    }
}

/// Draws what kind of thing an engine is, beside its name.
///
/// [`Algorithm::characters`], published in 26.5: any number of them, in the
/// order the specification declares them, and empty for an algorithm with
/// nothing to say beyond its family. A Tel-Ray Delay is a vintage unit and a
/// lo-fi one; a Stereo Chorus is two channels and something moving inside it;
/// a Hall Reverb is a reverb and nothing else a word can add.
///
/// They are read here rather than matched on. `Vintage` is not "the name
/// contains the word vintage" — the library's own file carries a reason per
/// membership, and the reasons are the manual's name for the effect, the slots
/// it gives it, or the unit a name refers to. A window deriving that from
/// `full_name` would be right until the day it was not.
///
/// Quiet, and after the name: it is a caption on a thing that already has a
/// mark across its case and a title along its strip, and a third loud thing on
/// one row is a row with no first thing.
fn kind<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    on: On,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let said: Vec<&'static str> = algorithm(patch, engine, firmware)
        .map(|algorithm| {
            algorithm
                .characters()
                .iter()
                .filter_map(|character| word(*character))
                .collect()
        })
        .unwrap_or_default();
    if said.is_empty() {
        return Space::new().width(Length::Fixed(0.0)).into();
    }
    text(said.join(" \u{00b7} "))
        .size(10)
        .style(move |theme: &Theme| text::Style {
            color: Some(style::mix(
                chassis(theme, None),
                ink(theme, on),
                CHARACTER_INK,
            )),
        })
        .into()
}

/// How far a character's word is carried from the case towards its ink.
///
/// Not far. It is a caption, and a caption as loud as the name it captions is
/// a second title.
const CHARACTER_INK: f32 = 0.55;

/// Returns the word for one of the kinds an effect can be.
///
/// [`Character`] is marked as a set that can grow, and one this window has no
/// word for prints nothing rather than a guess — the same rule
/// [`quantity`] is read under.
fn word(character: Character) -> Option<&'static str> {
    Some(match character {
        Character::Vintage => "vintage",
        Character::Modelled => "modelled",
        Character::Stereo => "stereo",
        Character::Dual => "dual",
        Character::Multiband => "multiband",
        Character::Combined => "two in one",
        Character::LoFi => "lo-fi",
        Character::Modulated => "modulated",
        Character::Dynamic => "dynamic",
        _ => return None,
    })
}

/// An algorithm, named the way the list names it.
///
/// A wrapper because the list needs `Display`, and what it displays is the
/// library's `full_name` rather than the abbreviation its `Debug` would give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Named(&'static Algorithm);

impl core::fmt::Display for Named {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0.full_name)
    }
}

/// Draws how loud an engine comes out, cut into its case.
///
/// A reading carries its claim in its colour, and the claims were chosen
/// against this window's own dark panel: on the palest of the 35 cases both of
/// them have to be lifted so far to be read that they arrive as the same ink,
/// which would spend the one distinction this editor exists to draw on a
/// livery. So the one value on the strip sits in a recess, which is also where
/// a control belongs.
fn output<'a, Renderer>(
    patch: &Patch,
    engine: Engine,
    firmware: Version,
    moved: &[ParamId],
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let gain = engine.gain_parameter();
    container(
        row![
            column![
                printing("gain".to_owned(), On::Recess).size(9),
                modulated(moved.contains(&gain), true),
            ]
            .spacing(1)
            .align_x(Horizontal::Center),
            control(
                gain,
                patch.value(gain),
                patch.claim(gain),
                firmware,
                Room::SLOT.turned().sized(GAIN),
                sent,
            ),
            reading(
                gain,
                patch.value(gain),
                patch.claim(gain),
                firmware,
                On::Recess,
            ),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
    )
    .padding([2, 5])
    .style(|theme: &Theme| {
        let material = materials(theme);
        container::Style {
            background: Some(Background::Color(material.panel)),
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

/// Draws one parameter that is a setting rather than a slot.
///
/// A slot of the rack with its title moved off the bottom, because what these
/// stand in is a row and not a column: the address and the modulation mark
/// above, the control, and the reading under it.
#[expect(
    clippy::too_many_arguments,
    reason = "a control, what it is called, what it stands on, and what is pointed at it"
)]
fn cell<'a, Renderer>(
    patch: &Patch,
    parameter: ParamId,
    firmware: Version,
    room: Room,
    title: &'a str,
    moved: &[ParamId],
    on: On,
    sent: Option<Sent<'_>>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        row![
            at(parameter, on),
            Space::new().width(Length::Fixed(2.0)),
            modulated(moved.contains(&parameter), true),
            Space::new().width(Length::Fixed(4.0)),
            text(title)
                .size(10)
                .style(move |theme: &Theme| text::Style {
                    color: Some(ink(theme, on)),
                }),
            Space::new().width(Length::Fill),
            reading(parameter, value, claim, firmware, on),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
        control(parameter, value, claim, firmware, room, sent),
    ]
    .spacing(3)
    .width(room.across_as())
    .into()
}

/// Draws where a parameter lives, in the ink of the surface it is printed on.
///
/// The NRPN number, which is also the parameter's byte offset in a dump, so one
/// number is both its name on the wire and its address in memory.
/// [`panel::address`](crate::panel::address) is the same thing on the window's
/// own plate; this page needs it per surface, because the grey a rack prints it
/// in is a grey half the 35 chassis are.
fn at<'a, Renderer>(parameter: ParamId, on: On) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    printing(parameter.offset().to_string(), on)
        .size(9)
        .font(reading_face())
        .into()
}

/// Draws the abbreviation the instrument's display prints for a slot.
///
/// `PST`, `DCY`, `HMQ`: the word a player reads on the hardware while looking
/// at this panel, so that the two say the same thing. Nothing for a byte the
/// algorithm does not use, which has no display name because it is not on the
/// display.
fn reference<'a, Renderer>(slot: Option<&'static FxSlot>) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let name = slot.map_or("", |slot| slot.reference);
    printing(name.to_owned(), On::Face)
        .size(9)
        .font(reading_face())
        .into()
}

/// Draws the picture of what a slot does, beside its title.
///
/// [`FxSlot::glyph`], published in 26.5: every slot has one, because what a
/// slot does is the one thing the algorithm always knows about it. It is finer
/// than the [`Quantity`] the line under it is drawn from and it is for a
/// different job — a pre-delay and a decay are both a time, and a plate with
/// twelve of these on it wants the gap drawn on one and the tail on the other.
///
/// The same glyph serves every slot doing the same thing, so a `Low Cut` on a
/// reverb and one on a delay are one picture; and it is the same picture the
/// footer puts beside a program parameter doing that job, which is the whole
/// value of the glyphs being the library's.
///
/// A byte the algorithm does not use has no slot and so no picture: there is
/// nothing to draw a picture *of*.
fn pictured<'a, Renderer>(slot: Option<&'static FxSlot>) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let Some(slot) = slot else {
        return Space::new().width(Length::Fixed(0.0)).into();
    };
    let mut screen = Screen::new(lcd::CELL, lcd::CELL);
    screen.blit(slot.glyph().pixels(), 0, 0);
    lcd::stencil(screen, |theme: &Theme| ink(theme, On::Face))
}

/// The line under a slot's title: what the instrument's display reads there.
///
/// The two ends of the reading and its unit, which the manual prints, and never
/// a value worked out from the byte, which it does not: the curve between `0.1`
/// and `6.0 s` is not published, so the fader's own readout stays the byte and
/// this says what the ends of it mean. A slot showing names says how many
/// rather than which, because which one a byte shows is the part nobody has
/// published.
fn hint(slot: Option<&'static FxSlot>) -> String {
    // A byte the algorithm does not use is on a strip of its own under the
    // grid, at half the size, under the library's own name for it. There is
    // nothing left for a line here to add.
    let Some(slot) = slot else {
        return String::new();
    };
    if slot.is_selector() {
        return match slot.values.len() {
            1 => "1 setting".to_owned(),
            names => format!("{names} settings"),
        };
    }
    match (slot.min, slot.max) {
        (Some(low), Some(high)) => match slot.unit {
            Some(unit) => format!("{low}\u{2013}{high} {unit}"),
            None => format!("{low}\u{2013}{high}"),
        },
        // No printed ends, so what the library can still say is what kind of
        // quantity the byte is. It is the answer to "what does moving this do",
        // which is the question a slot with no printed range leaves open, and
        // it is the library's rather than this window reading the title.
        _ => quantity(slot.quantity())
            .map(str::to_owned)
            .unwrap_or_default(),
    }
}

/// Returns the word for what a slot does to a signal.
///
/// [`Quantity`] is a closed set of nine that the library derives from the
/// parameter rather than from the slot's title, which is the point of it: a
/// `Mix`, a `Feedback` and a `Pre-Delay` are all a byte `0..=255` and they do
/// three unrelated things. The set is marked as one that can grow, and a
/// quantity this window has no word for prints nothing rather than a guess.
fn quantity(what: Quantity) -> Option<&'static str> {
    Some(match what {
        Quantity::Time => "a time",
        Quantity::Frequency => "a frequency",
        Quantity::Gain => "a level",
        Quantity::Feedback => "how much goes back in",
        Quantity::Depth => "how much effect",
        Quantity::Position => "a place in the stereo field",
        Quantity::Shape => "the shape of the response",
        Quantity::Switch => "in or out",
        Quantity::Selection => "one of a list",
        _ => return None,
    })
}

/// The names a row's slots show on the instrument's display.
///
/// One line per slot that shows names instead of a number, printed under the
/// row rather than in the slot, because that is what they are: a reading and
/// not something to send. The manual gives the names and never the bytes they
/// sit at, so the control stays the byte and this says what the display will
/// make of it.
///
/// Under its own row rather than under the whole grid. A line naming nine
/// reverb presets is two lines long on a case this wide, and printing all of
/// them at the foot of the plate put the longest thing on it furthest from the
/// control it describes.
fn displays<'a, Renderer>(
    line: &Line,
) -> impl Iterator<Item = iced_widget::Text<'a, Theme, Renderer>>
where
    Renderer: TextRenderer<Font = Font>,
{
    shows(line)
        .into_iter()
        .map(|line| printing(line, On::Face).size(9).font(reading_face()))
}

/// What those lines say, as words.
///
/// Split out from the drawing so that a test can read them: which row a line
/// belongs under is the whole of this change, and it is not a thing an
/// `Element` will answer.
fn shows(line: &Line) -> Vec<String> {
    line.iter()
        .flatten()
        .filter_map(|lane| lane.slot)
        .filter(|slot| slot.is_selector())
        .map(|slot| {
            format!(
                "{} {} shows {}",
                slot.reference,
                slot.title,
                slot.values.join(", ")
            )
        })
        .collect()
}

/// Returns a parameter's name with the engine's own taken off the front.
///
/// The rule a slot's title follows against its group, one level down: the plate
/// says `FX 1` once, so `FX 1 Type` in it is `Type`. Case-insensitively,
/// because the parameter table writes the same engine `FX 1 Type` and `Fx 1
/// Output Gain`. A name that does not start with the engine is returned whole,
/// which is how a library that renumbers them says so.
fn within(engine: Engine, parameter: ParamId) -> &'static str {
    let name = parameter.short_name();
    let prefix = format!("{engine} ");
    if name.len() > prefix.len() && name.to_ascii_uppercase().starts_with(&prefix) {
        name.get(prefix.len()..).unwrap_or(name)
    } else {
        name
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::effect::{Algorithm, Character, Engine, SLOTS_PER_ENGINE, grid};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, ParamId};
    use deepmind_midi::program::Program;
    use deepmind_midi::sysex::inquiry::Version;

    use super::{
        FxSlot, Lane, On, claimed, engines, hint, ink, lanes, legend, placed, quantity, settings,
        shows, spans, switched_on, within,
    };
    use crate::style::{READABLE, contrast, deepmind, legible, tint};
    use crate::{Confidence, Patch};

    /// Firmware 1.0, which numbers the algorithms differently.
    const FIRMWARE_1_0: Version = Version { major: 1, minor: 0 };

    /// A sound the synthesizer described, with `engine` running `algorithm`.
    fn running(algorithm: &str) -> Patch {
        let mut patch = Patch::new();
        patch.confirm(Program::new(ProtocolVersion::V7));
        let value = Algorithm::by_name(algorithm)
            .expect("an algorithm of that name")
            .value_for(DEFAULT_FIRMWARE)
            .expect("the newest firmware offers it");
        // Not asserted as a change: a program at its floor is already running
        // the algorithm the first byte selects, and asking for the one it holds
        // is what `Patch::edit` refuses.
        patch.edit(Engine::One.algorithm_parameter(), value);
        assert_eq!(patch.value(Engine::One.algorithm_parameter()), Some(value));
        patch
    }

    /// The twelve bytes of `engine` under `algorithm`, whatever the patch says.
    fn under(algorithm: &'static Algorithm, engine: Engine) -> Vec<Lane> {
        engine
            .slot_parameters()
            .iter()
            .copied()
            .map(|parameter| Lane {
                parameter,
                slot: algorithm.slot_of(engine, parameter),
            })
            .collect()
    }

    #[test]
    fn every_byte_of_an_engine_is_drawn_once_whatever_is_loaded() {
        // The plate is laid out from the library's own grid, and the one thing
        // a layout read out of a table can do wrong is lose a row of it: a byte
        // with no control on it is a byte nobody can edit, and a byte with two
        // is a plate that disagrees with itself. Neither is visible in a
        // screenshot of the one algorithm somebody happened to load.
        for algorithm in Algorithm::all() {
            for engine in Engine::ALL {
                let lanes = under(algorithm, engine);
                let (rows, spare) = placed(&lanes, Some(algorithm.panel()));

                let mut drawn: Vec<ParamId> = rows
                    .into_iter()
                    .flatten()
                    .flatten()
                    .chain(spare)
                    .map(|lane| lane.parameter)
                    .collect();
                let count = drawn.len();
                drawn.sort_unstable_by_key(|parameter| parameter.offset());
                drawn.dedup();

                assert_eq!(
                    count, SLOTS_PER_ENGINE,
                    "{engine} running {} draws {count} of its twelve bytes",
                    algorithm.full_name
                );
                assert_eq!(
                    drawn.len(),
                    count,
                    "{engine} running {} draws a byte twice",
                    algorithm.full_name
                );
            }
        }
    }

    #[test]
    fn a_slot_stands_in_the_column_the_instrument_draws_it_in() {
        // The point of the grid, and the thing a row of clusters could not do:
        // every row is as wide as the grid, and a slot is at its own measured
        // column of it, so the two rows of a plate line up down the page.
        let columns = usize::from(grid().columns());
        for algorithm in Algorithm::all() {
            let lanes = under(algorithm, Engine::One);
            let (rows, _) = placed(&lanes, Some(algorithm.panel()));

            // As deep as this algorithm is and no deeper. The grid has two rows
            // and an algorithm that fills one of them used to draw both, which
            // is half a case of blank panel on every reverb on the page.
            assert_eq!(
                rows.len(),
                algorithm.panel().rows().len(),
                "{} does not draw its own depth",
                algorithm.full_name
            );
            assert!(
                rows.len() <= usize::from(grid().rows()),
                "{} has more rows than the grid holds",
                algorithm.full_name
            );
            for line in &rows {
                assert_eq!(
                    line.len(),
                    columns,
                    "{} has a short row",
                    algorithm.full_name
                );
                for (column, cell) in line.iter().enumerate() {
                    let Some(slot) = cell.and_then(|lane| lane.slot) else {
                        continue;
                    };
                    assert_eq!(
                        usize::from(slot.position().column()),
                        column,
                        "{} draws {} out of its column",
                        algorithm.full_name,
                        slot.title
                    );
                }
            }
        }
    }

    #[test]
    fn a_display_name_line_stands_under_the_row_its_own_slot_is_on() {
        // The line is a reading of one control, and it used to be printed at
        // the foot of the case whatever row that control was on. Every one of
        // them names a slot of the row it is under now, and every slot that
        // shows names has a line somewhere.
        for algorithm in Algorithm::all() {
            let lanes = under(algorithm, Engine::One);
            let (rows, _) = placed(&lanes, Some(algorithm.panel()));
            let mut printed = 0;
            for line in &rows {
                for said in shows(line) {
                    printed += 1;
                    let named = line
                        .iter()
                        .flatten()
                        .filter_map(|lane| lane.slot)
                        .any(|slot| said.starts_with(slot.reference));
                    assert!(
                        named,
                        "{} prints {said:?} under a row that slot is not on",
                        algorithm.full_name
                    );
                }
            }
            let selectors = lanes
                .iter()
                .filter_map(|lane| lane.slot)
                .filter(|slot| slot.is_selector())
                .count();
            assert_eq!(
                printed, selectors,
                "{} loses a display name between the grid and the rows",
                algorithm.full_name
            );
        }
    }

    #[test]
    fn the_bytes_an_algorithm_does_not_use_are_the_ones_off_the_grid() {
        // TC Deep Reverb uses five of its twelve, so seven are left for the
        // strip under the plate and the grid holds exactly the five.
        let deep = Algorithm::by_name("TC-DeepVRB").expect("the TC reverb");

        let (rows, spare) = placed(&under(deep, Engine::One), Some(deep.panel()));

        assert_eq!(spare.len(), 7);
        assert!(
            spare.iter().all(|lane| lane.slot.is_none()),
            "a named slot fell off the grid"
        );
        let on = rows.iter().flatten().flatten().count();
        assert_eq!(on, 5, "the grid holds the five the algorithm names");
    }

    #[test]
    fn nothing_known_about_the_algorithm_leaves_every_byte_off_the_grid() {
        // Stage 3's rack, which is what a dump from a firmware whose table this
        // build does not have still deserves: twelve bytes, no page to lay them
        // out by, and every one of them draggable.
        let patch = Patch::new();
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        let (rows, spare) = placed(&lanes, None);

        assert!(rows.is_empty());
        assert_eq!(spare.len(), SLOTS_PER_ENGINE);
    }

    /// Every surface a word on this page can land on.
    ///
    /// The four the page has, with the case in all 35 of its liveries: half the
    /// measured chassis are pale and half are dark, and a page that reads on
    /// one of them proves nothing about the other 34.
    fn surfaces() -> Vec<On> {
        let mut found = vec![On::Face, On::Strip, On::Recess, On::Case(None)];
        found.extend(
            Algorithm::all()
                .iter()
                .map(Algorithm::panel)
                .map(|panel| On::Case(Some((panel.chassis(), panel.accent())))),
        );
        found
    }

    #[test]
    fn every_word_can_be_read_on_every_surface_it_lands_on() {
        // The rule this page is drawn under, checked rather than eyeballed. A
        // colour that is right in a palette and unreadable on a panel is not
        // right, and a screenshot of the one algorithm somebody happened to
        // load is not a check.
        let theme = deepmind();
        for on in surfaces() {
            let surface = on.colour(&theme);
            let title = contrast(ink(&theme, on), surface);
            assert!(
                title >= READABLE,
                "a title on {on:?} is {title:.2} against it"
            );
            let printed = contrast(legend(&theme, on), surface);
            assert!(
                printed >= READABLE,
                "a legend on {on:?} is {printed:.2} against it"
            );
        }
    }

    #[test]
    fn a_reading_keeps_its_claim_and_is_still_readable() {
        // The two surfaces a value is ever printed on, and the reason the list
        // is two rather than four: a claim is amber or green because of what it
        // means, and both were chosen against this window's own dark panel. On
        // the palest of the 35 measured cases each of them has to be lifted so
        // far to be read that they arrive as the same ink — so nothing on this
        // page prints a value on a case. The one value the strip across a case
        // carries sits in a recess cut into it, which is where a control
        // belongs anyway.
        let theme = deepmind();
        for on in [On::Face, On::Recess] {
            let surface = on.colour(&theme);
            for claim in [
                Confidence::Unknown,
                Confidence::Assumed,
                Confidence::Confirmed,
            ] {
                let reading = legible(tint(&theme, claim), surface, &theme);
                let against = contrast(reading, surface);
                assert!(
                    against >= READABLE,
                    "a {claim:?} reading on {on:?} is {against:.2} against it"
                );
            }
            // And the two that carry a colour still carry it. Contrast is a
            // measure of light and says nothing about hue, so what is asserted
            // is the warmth: an assumed value is the copper of the wooden
            // cheeks and a confirmed one is green, and lifting either of them
            // until it can be read must not turn one into the other.
            let warmth = |claim: Confidence| {
                let reading = legible(tint(&theme, claim), surface, &theme);
                reading.r - reading.g
            };
            assert!(
                warmth(Confidence::Assumed) > warmth(Confidence::Confirmed),
                "on {on:?} an assumed reading came out no warmer than a confirmed one"
            );
        }
    }

    #[test]
    fn one_group_holds_the_engines_and_the_rest_are_racks() {
        let found: Vec<Group> = Group::ALL
            .iter()
            .copied()
            .filter(|group| engines(*group).is_some())
            .collect();

        assert_eq!(found, vec![Group::Effects], "found {found:?}");
        assert_eq!(engines(Group::Effects).map(|found| found.len()), Some(4));
    }

    #[test]
    fn the_panel_claims_the_whole_group_and_leaves_every_other_alone() {
        let drawn = claimed(Group::Effects);

        assert_eq!(drawn.len(), Group::Effects.parameters().count());
        for parameter in Group::Effects.parameters() {
            assert!(drawn.contains(&parameter), "{parameter} is not drawn");
        }
        assert!(claimed(Group::Vcf).is_empty());
        assert!(claimed(Group::ModMatrix).is_empty());
    }

    #[test]
    fn the_settings_are_what_no_engine_addresses() {
        let settings = settings(Group::Effects);

        // The connection mode and whether the effects are in the signal path.
        // Named here to say what the subtraction found, not to find it.
        assert_eq!(settings.len(), 2);
        for parameter in &settings {
            assert!(
                Engine::of(*parameter).is_none(),
                "{parameter} is a slot of an engine"
            );
            for engine in Engine::ALL {
                assert_ne!(*parameter, engine.algorithm_parameter());
                assert_ne!(*parameter, engine.gain_parameter());
            }
        }
    }

    #[test]
    fn every_parameter_is_either_a_setting_or_something_an_engine_addresses() {
        let settings = settings(Group::Effects);
        let engines = engines(Group::Effects).expect("the effects hold them");
        let addressed = engines.len() * (SLOTS_PER_ENGINE + 2);

        assert_eq!(
            settings.len() + addressed,
            Group::Effects.parameters().count(),
            "nothing falls between the engines and the settings"
        );
    }

    #[test]
    fn a_slot_is_named_by_the_algorithm_that_is_loaded() {
        let patch = running("RoomRev");

        let room = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(room.len(), SLOTS_PER_ENGINE);
        let third = room.get(2).copied().expect("twelve of them");
        assert_eq!(third.parameter, ParamId::Fx1Param3);
        assert_eq!(third.slot.map(|slot| slot.title), Some("Size"));
        // And the same byte is something else under another algorithm, which is
        // the whole reason this panel exists.
        let phaser = running("Phaser");
        let third = lanes(&phaser, Engine::One, DEFAULT_FIRMWARE)
            .get(2)
            .copied()
            .expect("twelve of them");
        assert_ne!(third.slot.map(|slot| slot.title), Some("Size"));
    }

    #[test]
    fn an_algorithm_that_uses_fewer_leaves_the_rest_as_the_bytes_they_are() {
        // TC Deep Reverb uses five of its twelve.
        let patch = running("TC-DeepVRB");

        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(lanes.len(), SLOTS_PER_ENGINE, "twelve bytes either way");
        let named = lanes.iter().filter(|lane| lane.slot.is_some()).count();
        assert_eq!(named, 5);
        for lane in lanes.iter().skip(named) {
            assert_eq!(lane.slot, None, "{} is not used", lane.parameter);
            assert!(
                hint(lane.slot).is_empty(),
                "the strip's heading says it once"
            );
        }
    }

    #[test]
    fn a_sound_nobody_has_read_has_nothing_to_call_its_slots() {
        let patch = Patch::new();

        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);

        assert_eq!(lanes.len(), SLOTS_PER_ENGINE);
        assert!(
            lanes.iter().all(|lane| lane.slot.is_none()),
            "a byte nobody has read is under the library's own name"
        );
    }

    #[test]
    fn an_algorithm_this_firmware_does_not_name_leaves_the_slots_as_bytes() {
        // Vintage Pitch is firmware 1.1's, inserted rather than appended, and
        // its byte selects Rotary Speaker on 1.0.
        let patch = running("Vintage Pitch");

        let newer = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let older = lanes(&patch, Engine::One, FIRMWARE_1_0);

        assert_eq!(
            newer
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            Algorithm::by_name("Vintage Pitch")
                .and_then(|algorithm| algorithm.slot(1))
                .map(|slot| slot.title)
        );
        assert_ne!(
            older
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            newer
                .first()
                .and_then(|lane| lane.slot)
                .map(|slot| slot.title),
            "the same byte is another algorithm on the older firmware"
        );
    }

    #[test]
    fn a_band_covers_the_columns_of_the_row_it_runs_along() {
        // The Midas equaliser's slots are four bands and an output, and the
        // strip over a row has to divide that row exactly the way the row
        // divides itself or a band of three stands over two and a half.
        let patch = running("MidasEQ");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let panel = Algorithm::by_name("MidasEQ").expect("the Midas").panel();

        let (rows, _) = placed(&lanes, Some(panel));

        let mut named: Vec<&str> = Vec::new();
        for line in &rows {
            let runs = spans(line);
            assert_eq!(
                runs.iter().map(|(_, across)| across).sum::<usize>(),
                line.len(),
                "the strip and the row divide different widths"
            );
            named.extend(runs.iter().filter_map(|(label, _)| *label));
        }
        // `high-mid` twice, because the band runs across the break between the
        // two rows and each row prints what stands over it.
        assert_eq!(
            named,
            vec!["low", "low-mid", "high-mid", "high-mid", "high"]
        );
    }

    #[test]
    fn two_slots_the_library_labelled_nothing_are_not_a_band() {
        let patch = running("RoomRev");
        let lanes = lanes(&patch, Engine::One, DEFAULT_FIRMWARE);
        let panel = Algorithm::by_name("RoomRev").expect("the room").panel();

        let (rows, _) = placed(&lanes, Some(panel));

        for line in &rows {
            for (label, across) in spans(line) {
                assert!(
                    label.is_some() || across == 1,
                    "unlabelled columns ran together into a band"
                );
            }
        }
    }

    #[test]
    fn a_reading_says_the_ends_the_manual_prints_and_never_the_byte_between() {
        let deep = Algorithm::by_name("TC-DeepVRB").expect("the TC reverb");

        assert_eq!(hint(deep.slot(2)), "0.1\u{2013}6.0 s");
        // A slot the display shows names on says how many, because which one a
        // byte shows is the part nobody has published.
        assert_eq!(hint(deep.slot(1)), "11 settings");
        assert!(hint(None).is_empty());
    }

    #[test]
    fn every_algorithm_reaches_a_mark_and_a_family() {
        // Nine drawings across the 35, and the strip carries whichever one the
        // algorithm's family is in. A mark with no strokes would be a badge
        // drawn as nothing at all, which on a strip that has room for it reads
        // as an engine that failed to load.
        for algorithm in Algorithm::all() {
            assert!(
                !algorithm.mark().strokes().is_empty(),
                "{} has a mark with nothing in it",
                algorithm.full_name
            );
        }
    }

    #[test]
    fn three_algorithms_can_say_they_are_out_of_circuit_and_the_rest_cannot() {
        // The answer to how an effect is switched off, which is that on 32 of
        // the 35 it is not: `FX n Type` has no `Off`, and what takes effects
        // out is `FX Mode`, which is the whole block. This window asks the
        // library rather than matching on `ON` and `PWR` across 35 panels.
        let switchable: Vec<&'static str> = Algorithm::all()
            .iter()
            .filter(|algorithm| algorithm.slots.iter().any(FxSlot::is_enable))
            .map(|algorithm| algorithm.name)
            .collect();

        assert_eq!(switchable.len(), 3, "found {switchable:?}");

        let mut patch = running("NoiseGate");
        let gate = Algorithm::by_name("NoiseGate").expect("a Noise Gate");
        let power = gate.slot(8).expect("a Power slot");
        let parameter = Engine::One
            .slot_parameter(power.slot)
            .expect("a parameter for the slot");

        // The Noise Gate is the one that reads `ON` at the bottom of its range,
        // so the byte alone would get it exactly the wrong way round. A program
        // at its floor is already at the bottom, which is why the far end is
        // asked for first.
        assert!(patch.edit(parameter, 255));
        assert_eq!(switched_on(&patch, Engine::One, gate), Some(false));
        assert!(patch.edit(parameter, 0));
        assert_eq!(switched_on(&patch, Engine::One, gate), Some(true));

        // And an algorithm with no switch of its own says so.
        let reverb = Algorithm::by_name("RoomRev").expect("a Room Reverb");
        assert_eq!(switched_on(&running("RoomRev"), Engine::One, reverb), None);
    }

    #[test]
    fn the_two_tap_delays_are_the_two_engines_with_a_picture() {
        // The whole of what the library will draw for an effect, and the point
        // of it is what it refuses: a reverb's impulse response is its
        // designer's, so an engine running one gets no screen rather than a
        // plausible one.
        let drawn: Vec<&'static str> = Algorithm::all()
            .iter()
            .filter(|algorithm| {
                let patch = running(algorithm.name);
                patch
                    .program()
                    .and_then(|program| deepmind_midi::effect::response(program, Engine::One))
                    .is_some()
            })
            .map(|algorithm| algorithm.name)
            .collect();

        assert_eq!(drawn, ["3TapDelay", "4TapDelay"], "found {drawn:?}");
    }

    #[test]
    fn a_picture_is_not_drawn_for_an_algorithm_the_two_firmwares_disagree_on() {
        // The library reads the type byte on the newest firmware and this page
        // reads it on whichever answered the inquiry. 1.1 inserted Vintage
        // Pitch rather than appending it, so a byte above that point is two
        // different algorithms, and the picture is the one drawing that cannot
        // survive being wrong about which.
        let mut disagreed = 0;
        for algorithm in Algorithm::all() {
            let Some(value) = algorithm.value_for(FIRMWARE_1_0) else {
                continue;
            };
            let mut patch = Patch::new();
            patch.confirm(Program::new(ProtocolVersion::V7));
            patch.edit(Engine::One.algorithm_parameter(), value);

            let agreed = Algorithm::for_value(value, DEFAULT_FIRMWARE) == Some(algorithm);
            let drawn: Option<crate::Element<'_, iced_widget::renderer::Renderer>> =
                super::picture(&patch, Engine::One, FIRMWARE_1_0);
            assert!(
                agreed || drawn.is_none(),
                "{} draws a picture the two firmwares disagree about",
                algorithm.name
            );
            disagreed += usize::from(!agreed);
        }
        // And the guard is not vacuous: the two firmwares really do read some
        // of the bytes as different algorithms, which is why it is there.
        assert!(disagreed > 0, "the two firmwares agree about all 35");
    }

    #[test]
    fn a_slot_with_no_printed_ends_says_what_kind_of_quantity_it_is() {
        // What the library added in 26.4, and it is the answer to the one
        // question a slot with a title and no range leaves open. Every slot of
        // the 35 says something now, which is what the line under a slot is
        // for.
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                assert!(
                    !hint(Some(slot)).is_empty(),
                    "{} {} says nothing under its title",
                    algorithm.name,
                    slot.title
                );
            }
        }
        // And a byte the algorithm does not use still says nothing, because the
        // strip it stands on has said it already.
        assert!(hint(None).is_empty());
    }

    #[test]
    fn every_character_the_library_publishes_has_a_word() {
        // The same rule the quantities are read under: the set is marked as one
        // that can grow, so a kind this window has no word for prints nothing —
        // and a kind it *does* have is one it prints rather than derives from
        // the algorithm's name.
        for character in Character::ALL {
            assert!(
                super::word(character).is_some(),
                "{character:?} has no word"
            );
        }
    }

    #[test]
    fn an_effect_says_what_kind_of_thing_it_is_from_the_library() {
        let tel_ray = Algorithm::by_name("T-RayDelay").expect("a Tel-Ray Delay");
        assert!(
            tel_ray.has(Character::Vintage),
            "not read as a vintage unit"
        );
        assert!(tel_ray.has(Character::LoFi));

        // And the ones with nothing to add print nothing rather than a blank
        // caption: the family's mark on the case has already said it.
        let hall = Algorithm::by_name("HallRev").expect("a Hall Reverb");
        assert!(!hall.has(Character::Vintage));
    }

    #[test]
    fn an_effect_wears_its_own_mark_where_the_library_draws_one() {
        // 26.5 tells a plate from a hall, which 26.4 declined to: the window
        // asks for a mark and gets the finer one where there is one, so this
        // is the library's answer arriving rather than anything drawn here.
        let plate = Algorithm::by_name("PlateRev").expect("a Plate Reverb");
        let ambient = Algorithm::by_name("AmbVerb").expect("an Ambient Reverb");

        assert!(plate.own_mark().is_some(), "a plate has no mark of its own");
        assert!(ambient.own_mark().is_none());
        assert_ne!(
            plate.mark().pixels(),
            ambient.mark().pixels(),
            "two reverbs drew one picture"
        );
    }

    #[test]
    fn every_slot_carries_the_picture_of_what_it_does() {
        // Every slot has one, which is what the library says: what a slot does
        // is the one thing the algorithm always knows about it. And the same
        // job is the same picture wherever it is met.
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                let drawn = slot.glyph().pixels();
                assert!(
                    drawn.rows().iter().any(|row| *row != 0),
                    "{}'s {} is a blank picture",
                    algorithm.name,
                    slot.title
                );
            }
        }
        let room = Algorithm::by_name("RoomRev").expect("a Room Reverb");
        let decay = room.slot(2).expect("a Decay slot").glyph();
        assert_eq!(
            Some(decay),
            ParamId::VcfEnvelopeDecayTime.glyph(),
            "a decay on a reverb and a decay on an envelope drew two pictures"
        );
    }

    #[test]
    fn every_quantity_the_library_publishes_has_a_word() {
        use deepmind_midi::effect::Quantity;

        for what in Quantity::ALL {
            assert!(quantity(what).is_some(), "{what:?} has no word");
        }
    }

    #[test]
    fn a_plate_says_the_engine_once() {
        assert_eq!(within(Engine::One, ParamId::Fx1Type), "Type");
        assert_eq!(within(Engine::One, ParamId::Fx1OutputGain), "Output Gain");
        assert_eq!(within(Engine::Four, ParamId::Fx4Param12), "Param 12");
        // A parameter that is not the engine's keeps its whole name.
        assert_eq!(
            within(Engine::One, ParamId::Fx2Type),
            ParamId::Fx2Type.short_name()
        );
    }
}

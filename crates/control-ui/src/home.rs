//! The front panel: what the instrument shows before anybody goes looking.
//!
//! A `DeepMind` is two rows of section plates with a screen between them.
//! Twenty-odd faders under four-letter legends, a few lit buttons under each
//! group, and a yellow `EDIT` on every section that opens that section on the
//! display. Everything else — 242 parameters' worth — is behind one of those.
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
//! file — the only one in this repository, kept because the library published
//! what exists and what has a CC and nothing that said what has a fader.
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
//! one plate each, because the hardware has four envelope faders and three
//! envelopes and a button to point the one set at the other — and a window can
//! simply draw twelve faders and three screens. They get a row of their own,
//! which is what the instrument had no room to give them.
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
use iced_widget::{Space, button, column, container, responsive, row, text};

use crate::envelope;
use crate::lcd::{self, Screen};
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
/// `Triang` is a legend that lies about which shape is lit.
const LAMPS: f32 = 76.0;

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

/// How tall a plate of the upper row stands.
///
/// The screen is cut to it rather than given what is left: a display that took
/// the height it was offered would be as tall as the window, and the panel
/// under it would be somewhere below the fold.
const PLATE: f32 = LEGEND + TRAVEL + 96.0 + lcd::room(STRIP) + 6.0;

/// How wide the screen is.
///
/// Fixed, and not what is left over. The two rows reflow when a window is
/// narrower than the panel, and a screen that took the rest of its row would
/// take the whole of it and push the voicing onto a line of its own.
const SCREEN: f32 = 340.0;

/// Height of the legend over a lane, so that lanes line up whatever they hold.
///
/// Two lines of it, because the panel prints `PITCH MOD` on two.
const LEGEND: f32 = 22.0;

/// How much room a button under the faders is given.
const SWITCH: f32 = 58.0;

/// How much room a way into a section is given.
const WAY: f32 = 40.0;

/// How tall the lamp under a way in stands.
///
/// A square-ish button rather than a word in a box: the instrument's are wider
/// than they are tall by about this much, and the legend is on the panel above
/// rather than inside them.
const PRESS: f32 = 15.0;

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
/// is, and everything — the lanes, the travel of a fader, the buttons, the type,
/// the gaps between plates and the gaps inside them — is drawn through the
/// result. Scaling the gaps is the half that is easy to forget and the half
/// that decides whether it looks like an instrument or like a panel with its
/// parts pushed apart.
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
/// sections are unfolded into more than one plate on the way — see [`plates`] —
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
    /// of them is as wide as what it holds. The wider of its two rows wins —
    /// `HPF` is one fader over a button and a way in, and the buttons are what
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
/// both — the parameter, the legend printed over it and the shape a hand
/// touches all come from the table a single-plate panel would use.
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
/// which of the three those four faders address — which the library says out
/// loud in that section's own note. A window does not have to multiplex: each
/// envelope gets the same four legends over its *own* parameters, and its own
/// display, which is what it was really short of. Three envelopes sharing one
/// screen were three panes of a strip; three plates are three full drawings.
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
    rows
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
                // on the button — `VCA` beneath an `ENVELOPES` heading is
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
/// The screen is the caller's: what it says is the application's business and
/// not the synthesizer's — which sound is on it, what backs that, and what the
/// last thing to happen was — and a plugin has different answers than a desktop
/// window does.
#[must_use]
pub fn panel<'a, Renderer>(
    patch: &'a Patch,
    firmware: Version,
    paint: impl Fn(&mut Screen) + 'a,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    responsive(move |room| {
        let scale = Scale::filling(room.width);
        let mut panel = column![].spacing(scale.of(DOWN));
        let mut hole = true;
        for (index, plates) in rows().iter().enumerate() {
            let mut across = row![].spacing(scale.of(ACROSS)).align_y(Vertical::Top);
            // The instrument cuts its display into the top row between what a
            // player reaches for and the voicing, which is before the last
            // plate of that row.
            let last = plates.last().map(Plate::name);
            for plate in plates {
                // The screen sits in the top row where the instrument puts it:
                // between what a player reaches for and what the voicing does.
                if index == 0 && hole && last == Some(plate.name()) {
                    hole = false;
                    across = across.push(display(patch, &paint, scale));
                }
                across = across.push(group(patch, plate, firmware, scale));
            }
            panel = panel.push(across.wrap());
        }
        // A screen with nowhere to go still goes somewhere: a row renamed out
        // from under this loses the instrument's arrangement, not its display.
        if hole {
            panel = panel.push(display(patch, &paint, scale));
        }
        panel.into()
    })
    .into()
}

/// A blank screen the size of the hole the panel leaves at its written size.
///
/// The panel paints its own now — only it knows how wide the hole came out at
/// the scale the window forced — so this is what a test, or anything else with
/// no window to measure, writes on to see what a display would say.
#[must_use]
pub fn screen() -> Screen {
    blank(Scale::NATURAL)
}

/// The blank screen the hole leaves at `scale`.
fn blank(scale: Scale) -> Screen {
    Screen::new(
        lcd::fits(scale.of(SCREEN)),
        lcd::fits(scale.of(PLATE - GAP * 2.0)),
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
fn group<'a, Renderer>(
    patch: &Patch,
    plate: &'a Plate,
    firmware: Version,
    scale: Scale,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mut controls = row![].spacing(scale.of(2.0)).align_y(Vertical::Top);
    for control in plate.faders.iter().copied() {
        controls = controls.push(lane(patch, control, firmware, scale));
    }
    if let Some(control) = plate.lamps {
        controls = controls.push(strip(patch, control, firmware, scale));
    }
    let mut buttons = row![].spacing(scale.of(4.0)).align_y(Vertical::Top);
    for control in plate.switches.iter().copied() {
        buttons = buttons.push(switch(patch, control, firmware, scale));
    }
    // Every plate has a way in, and it is the press the hardware calls EDIT.
    buttons = buttons.push(way("EDIT", plate.opens, scale));
    container(
        column![
            heading(plate.name(), scale),
            glass(patch, plate, firmware, scale),
            controls,
            buttons
        ]
        .spacing(scale.of(WITHIN))
        .align_x(Horizontal::Center),
    )
    .width(Length::Fixed(plate.width(scale)))
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
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let columns = lcd::fits(plate.width(scale) - scale.of(PAD * 2.0));
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
fn heading<'a, Renderer>(name: &'a str, scale: Scale) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(name)
            .size(scale.of(11.0))
            .font(printed())
            .style(|theme: &Theme| text::Style {
                color: Some(materials(theme).recess),
            }),
    )
    .width(Length::Fill)
    .padding([scale.of(2.0), scale.of(6.0)])
    .align_x(Horizontal::Center)
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
    })
    .into()
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
        ),
        readout(parameter, value, claim, firmware),
    ]
    .spacing(scale.of(4.0))
    .width(Length::Fixed(scale.of(LANE)))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws a named set as the strip of lit legends the instrument has.
///
/// The LFO's shape is a column of lamps beside its two faders on the hardware,
/// one of them lit, and seven of them is more than a rack's slot has room to
/// light. The panel has the room, so it says so.
fn strip<'a, Renderer>(
    patch: &Patch,
    control: Control,
    firmware: Version,
    scale: Scale,
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
            Room::lamps(scale.of(LAMPS), scale.of(TRAVEL)),
        ),
    ]
    .spacing(scale.of(4.0))
    .width(Length::Fixed(scale.of(LAMPS)))
    .align_x(Horizontal::Center)
    .into()
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
            Room::listed(scale.of(SWITCH)),
        ),
    ]
    .spacing(scale.of(2.0))
    .width(Length::Fixed(scale.of(SWITCH)))
    .align_x(Horizontal::Center)
    .into()
}

/// What the panel prints over a control.
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
/// panel and the button under it is a blank square that is lit amber the whole
/// time it is powered; a row of those along the foot of every plate is the
/// first thing anybody sees in a photograph of a `DeepMind`. So the label is
/// printed over it in the same ink as every other legend, and what is pressed
/// is the lamp.
fn way<'a, Renderer>(label: &'a str, group: Group, scale: Scale) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    column![
        legend(label, scale),
        button(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fixed(scale.of(WAY)))
            .height(Length::Fixed(scale.of(PRESS)))
            .style(crate::style::way_in)
            .on_press(Message::Show(group)),
    ]
    .spacing(scale.of(2.0))
    .width(Length::Fixed(scale.of(WAY)))
    .align_x(Horizontal::Center)
    .into()
}

/// The screen, cut into the panel where the instrument's own display sits.
///
/// The glass is the display's own — it draws the recess it is cut into, the
/// way every other display in this window does — so what is left here is the
/// hole it stands in, as tall as the plates either side of it.
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
        .height(Length::Fixed(scale.of(PLATE)))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([scale.of(2.0), 0.0])
        .into()
}

#[cfg(test)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};

    use super::{ACROSS, DOWN, GAP, NARROWEST, PAD, Plate, Scale, WITHIN, rows};
    use super::{lcd, panel_width, panelled, row_width, widest};

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
        // narrower than the panel wraps a row rather than clipping it — which
        // is readable and is no longer the instrument's own front. So the
        // window opens at what the panel measures, and this is what says that
        // number is a window somebody could actually have.
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
        let second: Vec<&str> = rows()
            .get(1)
            .map(|plates| plates.iter().map(Plate::name).collect())
            .unwrap_or_default();

        assert_eq!(
            second,
            vec!["OSC 1", "OSC 2", "VCF", "VCA", "HPF"],
            "the signal path, left to right, with the oscillators unfolded"
        );
        let third: Vec<&str> = rows()
            .get(2)
            .map(|plates| plates.iter().map(Plate::name).collect())
            .unwrap_or_default();

        assert_eq!(
            third,
            vec!["VCA ENVELOPE", "VCF ENVELOPE", "MOD ENVELOPE"],
            "and the envelopes have the row the instrument had no room for"
        );
    }
}

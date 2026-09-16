//! What a plate's own display draws.
//!
//! The instrument has one screen and this window has ten, which is the one
//! place the panel deliberately stops being the instrument. A `DeepMind` has
//! room on its front for a display and twenty-odd faders, so its display shows
//! whichever section was pressed last; a window has room for a display over
//! every plate, and the thing a player most wants to see about a filter is the
//! shape of it rather than three numbers that imply one.
//!
//! So every plate gets the drawing of its own part — the shapes the
//! oscillators are making, where the filter's corner is, the gates the
//! arpeggiator is opening — and the envelopes get the one drawing no hardware
//! could show: all three of them at once, on one screen, which is what an
//! instrument with one display and three envelope buttons cannot do.
//!
//! # A scene is found, not assigned
//!
//! Nothing here says which plate draws what. A scene names the controls it
//! needs, the plate offers the controls it holds, and the first scene that is
//! satisfied is the one drawn — so a plate that gains a fader gets the right
//! drawing and a plate the library empties draws nothing at all. The two
//! filters are the one place a parameter is named rather than a section:
//! `VCF` and `HPF` are two plates of one section, and which of them is which is
//! the frequency each has on it.
//!
//! # The shapes are the library's
//!
//! An envelope's bends, an LFO's wave, a filter's roll-off, the arpeggiator's
//! gates: every one of those is a fact about the instrument, and 26.4 publishes
//! them as functions
//! ([deepmind-midi#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32)).
//! What is left here is a sample loop — walk the columns of a band, ask
//! [`Generator::at`](deepmind_midi::generator::Generator::at) what the shape is
//! doing there, and print the dot nearest the answer. Where a byte's curve has
//! not been measured the library says so in
//! [`Scale`](deepmind_midi::generator::Scale) rather than inventing one, which
//! is the same refusal this file used to make in prose and could not test.
//!
//! # What these drawings do not claim
//!
//! **No axis is in anybody's units unless the library publishes one.** Two of
//! them are published and they are drawn as published: a filter's vertical is
//! decibels about unity, because the slope of a pole is, and an LFO's
//! horizontal is turns, because a cycle is a cycle whatever the rate byte does.
//! Everything else is [`Scale::Normalised`](deepmind_midi::generator::Scale::Normalised),
//! which says the shape is real and its axis is an ordering — so a corner is
//! drawn at the fraction of its own range the byte sits at, which is exactly
//! what the fader beside it says, and never at a frequency.
//!
//! **Nothing is drawn from a value nobody has read.** A scene's claim is the
//! weakest of everything it read, and a scene with anything unread is not
//! drawn: a picture of a filter assembled from four values the synthesizer
//! described and one this window invented is a picture of no filter at all.

use deepmind_midi::generator::{self, FILTER_UNITY, GATES_DRAWN, Gate, LfoId};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::LfoShape;
use deepmind_midi::sysex::inquiry::Version;

use crate::envelope;
use crate::lcd::{Band, Ink, Screen, Size};
use crate::{Confidence, Patch};

/// Decibels per octave the high-pass filter rolls off at.
///
/// The one number about a filter this file holds, and it is the one the library
/// declines to draw: [`generator::filter_response`] is the low-pass alone,
/// because putting the two corners on one axis needs the spacing between two
/// bytes whose curves are both unpublished. Drawn on its own plate there is no
/// spacing to invent — the corner is where the fader says it is and the slope
/// is the 6 dB per octave [`generator::filter_response`] records beside the
/// low-pass it does draw, on that same decibel vertical.
const HIGH_PASS_SLOPE: f32 = 6.0;

/// How far past its own fall a filter's horizontal reaches, as a multiple.
///
/// Read back out of the span the library publishes rather than chosen here: a
/// 24 dB per octave slope takes two octaves to fall from unity to the floor and
/// [`generator::filter_response`] draws three either side of the corner, so the
/// axis is half as wide again as the fall it has to show. That is what puts a
/// filter's knee in the middle of its picture instead of at one edge of it, and
/// a 6 dB per octave slope drawn on the same proportion fills the same glass.
/// `the_two_filters_are_drawn_to_one_proportion` holds it to what the library
/// publishes, so a library that widened its own span moves this with it.
const SPREAD: f32 = 1.5;

/// A scatter that is the same every time it is asked.
///
/// Noise on a display needs a number that looks unchosen and needs the same one
/// on every frame: a screen that reseeded when nothing had moved would crawl,
/// and a crawling drawing says the sound is doing something it is not. The
/// sampled LFO shapes wanted the same thing and no longer ask here — the
/// library publishes a fixed sequence for those, and says in the same breath
/// that it is not the instrument's stream.
const SCATTER: [f32; 16] = [
    0.62, 0.18, 0.91, 0.44, 0.07, 0.73, 0.35, 0.99, 0.26, 0.81, 0.53, 0.12, 0.68, 0.39, 0.86, 0.02,
];

/// Returns the scatter's `step`th number, between nothing and one.
fn scattered(step: i32) -> f32 {
    let index = usize::try_from(step.rem_euclid(16)).unwrap_or_default();
    SCATTER.get(index).copied().unwrap_or(0.5)
}

/// Returns where `parameter` sits in its own range, as a fraction of it.
///
/// `None` for a value nobody has read, which is what stops a drawing from
/// being assembled out of a sound that has not arrived.
fn travel(patch: &Patch, parameter: ParamId) -> Option<f32> {
    let low = f32::from(u8::try_from(parameter.min()).unwrap_or(u8::MIN));
    let high = f32::from(u8::try_from(parameter.max()).unwrap_or(u8::MAX));
    let value = f32::from(patch.value(parameter)?);
    Some(((value - low) / (high - low).max(1.0)).clamp(0.0, 1.0))
}

/// Returns whether a switch is on.
fn is_on(patch: &Patch, parameter: ParamId) -> Option<bool> {
    Some(patch.value(parameter)? != 0)
}

/// Returns what the instrument's own display calls the value a parameter holds.
fn named(patch: &Patch, parameter: ParamId, firmware: Version) -> Option<&'static str> {
    parameter.label_for(u16::from(patch.value(parameter)?), firmware)
}

/// Returns how many turns of something fit across a screen at `travel`.
///
/// Between one and `most`. A rate drawn as a count of turns says that a faster
/// setting is more of them in the same window, which is true of every rate this
/// instrument has and is as much as the manual establishes; a screen is not a
/// length of time, so nothing here is a speed. The shape being repeated is the
/// library's, and so is the count of turns its own horizontal covers — this is
/// only how many of them the glass is given.
fn turns(travel: f32, most: f32) -> f32 {
    1.0 + travel * (most - 1.0)
}

/// One plate's drawing.
///
/// Fieldless where the parameters it reads are one of a thing on this
/// instrument, and carrying what it found where they are a family: two LFOs
/// mean one drawing and two sets of parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scene {
    /// Where the low-pass corner is, how much it is resonating, and how far the
    /// envelope moves it.
    Filter,
    /// Where the high-pass corner is, and whether there is a boost under it.
    HighPass,
    /// The shapes one oscillator is making.
    ///
    /// Carries which, because the panel draws the two on plates of their own
    /// and `OSC 1` is two shapes that may both be switched on where `OSC 2` is
    /// one shape at a level with the noise scattered over it.
    Oscillator(Which),
    /// One envelope, filling the plate it is drawn on.
    ///
    /// Carries the group, because there are three of them and each has a plate.
    Envelope(Group),
    /// The amplifier's own envelope, under the level it is played at.
    Amplifier,
    /// How far apart a unison's voices are detuned, and what mode is playing.
    Voicing,
    /// The gates the arpeggiator opens.
    Arpeggiator,
    /// An LFO's shape, at its own rate.
    Lfo {
        /// Which of the library's two.
        which: LfoId,
        /// The parameter naming the shape.
        shape: ParamId,
        /// The one setting how fast it runs.
        rate: ParamId,
    },
}

/// Which of the two oscillators a drawing is of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Which {
    /// The one whose saw and pulse can both be switched on.
    First,
    /// The one that is a shape at a level, with the noise beside it.
    Second,
}

/// Returns the parameter holding one of the library's two LFO shapes.
///
/// The bridge the envelopes have between a group and an `EnvelopeId`, made the
/// same way and for the same reason: a match over the library's own enum, so a
/// third LFO is a third variant and a third variant fails to compile here
/// rather than drawing one of the two.
const fn shape_of(which: LfoId) -> ParamId {
    match which {
        LfoId::One => ParamId::Lfo1Shape,
        LfoId::Two => ParamId::Lfo2Shape,
    }
}

/// Returns which oscillator `parameter` belongs to, out of the library's own
/// name for it.
///
/// `OSC 1 PWM Depth` is the first and `OSC 2 Level` the second. A parameter of
/// the group that names neither — the noise — is the second's, which is where
/// the panel prints it and which is the plate this window puts it on.
fn which(parameter: ParamId) -> Which {
    if parameter.name().starts_with("OSC 1") {
        Which::First
    } else {
        Which::Second
    }
}

/// Returns the scene a plate holding `controls` draws, when it draws one.
pub(crate) fn of(controls: impl IntoIterator<Item = ParamId>) -> Option<Scene> {
    controls.into_iter().find_map(from)
}

/// Returns the scene one control puts on a plate.
fn from(control: ParamId) -> Option<Scene> {
    // Two plates of one section: the instrument prints `VCF` and `HPF` over
    // separate groups of faders and the library has one `Vcf` for both, so
    // which filter a plate is drawing is the frequency it has on it. Everything
    // else is a section, which is how a plate that gains a fader keeps its
    // drawing and a scene stops being a list of parameters to keep up to date.
    match control {
        ParamId::VcfFrequency => return Some(Scene::Filter),
        ParamId::VcfHighPassFrequency => return Some(Scene::HighPass),
        _ => {}
    }
    if envelope::of(control.group()).is_some() {
        return Some(Scene::Envelope(control.group()));
    }
    if let Some(which) = LfoId::ALL
        .into_iter()
        .find(|which| shape_of(*which) == control)
    {
        let rate = control
            .group()
            .parameters()
            .find(|parameter| parameter.short_name() == "Rate")?;
        return Some(Scene::Lfo {
            which,
            shape: control,
            rate,
        });
    }
    match control.group() {
        Group::Oscillators => Some(Scene::Oscillator(which(control))),
        Group::Vca => Some(Scene::Amplifier),
        Group::Voicing => Some(Scene::Voicing),
        Group::Arpeggiator => Some(Scene::Arpeggiator),
        _ => None,
    }
}

impl Scene {
    /// Every parameter this drawing reads.
    ///
    /// What the claim is taken across, and the answer to "would this screen
    /// change if that fader moved".
    pub(crate) fn parameters(self) -> Vec<ParamId> {
        match self {
            Self::Filter => vec![
                ParamId::VcfFrequency,
                ParamId::VcfResonance,
                ParamId::Vcf2PoleMode,
                ParamId::VcfEnvelopeDepth,
                ParamId::VcfEnvelopePolarity,
            ],
            Self::HighPass => vec![ParamId::VcfHighPassFrequency, ParamId::VcfBassBoost],
            Self::Oscillator(Which::First) => vec![
                ParamId::Osc1SawEnable,
                ParamId::Osc1PulseEnable,
                ParamId::Osc1PwmDepth,
            ],
            Self::Oscillator(Which::Second) => vec![ParamId::Osc2Level, ParamId::NoiseLevel],
            Self::Envelope(group) => envelope::of(group)
                .map(envelope::Envelope::parameters)
                .into_iter()
                .flatten()
                .collect(),
            Self::Amplifier => envelope::of(Group::VcaEnvelope)
                .map(envelope::Envelope::parameters)
                .into_iter()
                .flatten()
                .chain([ParamId::VcaLevel])
                .collect(),
            Self::Voicing => vec![ParamId::UnisonDetune, ParamId::PolyphonyMode],
            // Not the rate. How many steps the picture covers is the library's
            // `GATES_DRAWN` and not a byte: the gate time is published against
            // a step — 128 is half of one, which the manual states outright —
            // and what a step is worth in seconds is exactly what it does not
            // print. A rate byte stretched across the glass was this window
            // drawing an axis the instrument does not publish.
            Self::Arpeggiator => vec![ParamId::ArpOnOff, ParamId::ArpGateTime],
            Self::Lfo { shape, rate, .. } => vec![shape, rate],
        }
    }

    /// Draws the scene onto a screen `columns` by `rows` dots.
    pub(crate) fn screen(
        self,
        patch: &Patch,
        firmware: Version,
        columns: i32,
        rows: i32,
    ) -> Screen {
        let mut screen = Screen::new(columns, rows);
        match self {
            Self::Filter => filter(&mut screen, patch, firmware),
            Self::HighPass => high_pass(&mut screen, patch),
            Self::Oscillator(which) => oscillator(&mut screen, patch, which),
            Self::Envelope(group) => envelope_on(&mut screen, patch, group),
            Self::Amplifier => amplifier(&mut screen, patch),
            Self::Voicing => voicing(&mut screen, patch, firmware),
            Self::Arpeggiator => arpeggiator(&mut screen, patch),
            Self::Lfo { which, shape, rate } => {
                lfo(&mut screen, patch, firmware, which, shape, rate);
            }
        }
        screen
    }
}

/// Returns every group that is an envelope, in the instrument's own order.
#[cfg(test)]
fn envelopes() -> Vec<Group> {
    Group::ORDER
        .iter()
        .copied()
        .filter(|group| envelope::of(*group).is_some())
        .collect()
}

/// Returns how many octaves of glass a filter of `slope` is drawn across.
///
/// Enough for the slope to fall from unity to the library's own floor, and half
/// as much again — see [`SPREAD`]. Asking it of the slope rather than writing a
/// number down is what lets the high-pass, which is a quarter as steep as a
/// four-pole low-pass, be drawn on the same vertical without spending three
/// quarters of its glass on a line that has not fallen yet.
fn octaves(slope: f32) -> f32 {
    -generator::FILTER_FLOOR_DB / slope.max(1.0) * 2.0 * SPREAD
}

/// Returns how many octaves the library draws its own filter across.
///
/// `None` where the library stops publishing an octave axis for it, which is
/// the one thing that would make [`octaves`] a number this window had invented.
#[cfg(test)]
fn published(response: &generator::Generator) -> Option<f32> {
    match response.scale() {
        generator::Scale::Octaves(span) => Some(span),
        _ => None,
    }
}

/// Returns where a gain in decibels sits up a filter's screen.
///
/// The library's vertical: linear in decibels from its floor to its ceiling,
/// which is where [`FILTER_UNITY`] comes from and is the axis
/// [`generator::filter_response`] is already drawn on. The high-pass on the
/// plate beside it is drawn on the same one, so the two read against each
/// other.
fn decibels(gain: f32) -> f32 {
    let floor = generator::FILTER_FLOOR_DB;
    let ceiling = generator::FILTER_CEILING_DB;
    ((gain - floor) / (ceiling - floor)).clamp(0.0, 1.0)
}

/// Draws where the low-pass corner is and what is happening at it.
///
/// The curve is [`generator::filter_response`], which is the roll-off the pole
/// count gives and the peak the resonance byte lifts, on a vertical that is
/// decibels from the library's own floor to its own ceiling. The whole of what
/// this adds is where the corner stands: the generator draws the response about
/// its corner, because the slope of a pole is published and the frequency a
/// byte lands on is not, so the corner is put at the fraction of its own range
/// the byte sits at — the reading the fader beside it gives — and the response
/// is sampled either side of it.
///
/// The passband sits at [`FILTER_UNITY`], which is where the library puts unity
/// gain on that vertical, and the headroom above it is what a resonant peak
/// rises into. It is ruled, because a filter drawn without the level it passes
/// at is a curve with nothing to read it against.
fn filter(screen: &mut Screen, patch: &Patch, firmware: Version) {
    let band = screen.all();
    let (Some(program), Some(corner)) = (patch.program(), travel(patch, ParamId::VcfFrequency))
    else {
        return;
    };
    let response = generator::filter_response(program);
    // The level the filter passes, on the library's own axis.
    screen.across(band.x, band.row(FILTER_UNITY), band.width, Ink::Dotted);
    // The corner sits at `0.5` along the generator, so a column of the screen
    // is half a span either side of wherever the byte put it.
    screen.curve(band, Ink::Solid, |x| response.at(0.5 + x - corner));
    reach(screen, patch, corner, firmware);
}

/// Draws how far the envelope moves the corner, and which way.
///
/// A rule along the top of the screen from the corner to where the depth would
/// take it, which is the depth drawn as the fraction of the axis it is — the
/// same reading the fader beside it gives. It is not a second filter curve:
/// that the two parameters are in the same units is exactly what the manual
/// does not say, so this says how much of its own travel the depth is using and
/// which way the polarity points it, and stops there.
fn reach(screen: &mut Screen, patch: &Patch, corner: f32, firmware: Version) {
    let Some(depth) = travel(patch, ParamId::VcfEnvelopeDepth) else {
        return;
    };
    if depth <= 0.0 {
        return;
    }
    let positive = named(patch, ParamId::VcfEnvelopePolarity, firmware) != Some("Negative");
    let band = screen.all();
    let moved = if positive {
        corner + depth
    } else {
        corner - depth
    };
    let (from, to) = (corner.min(moved), corner.max(moved));
    let (left, right) = (band.column(from), band.column(to));
    screen.across(left, band.y, right - left + 1, Ink::Dotted);
    screen.down(left, band.y, 2, Ink::Solid);
    screen.down(right, band.y, 2, Ink::Solid);
}

/// Draws where the high-pass corner is, and says whether a boost is on.
///
/// On the low-pass's own vertical — decibels, with unity at [`FILTER_UNITY`] —
/// and across the low-pass's own span, so the two plates of one section are two
/// readings of one picture. The slope is the 6 dB per octave the library
/// records for it and the corner is where the fader says it is, the same pair of
/// facts the generator beside it is built out of; what the library will not do
/// is put both corners on one axis, and a plate with one filter on it is not
/// asking it to.
///
/// The boost is not in the curve. It is two states, what it lifts the low end
/// by is not published, and a shelf drawn under the corner would be this
/// window choosing a height and then drawing it as confidently as the corner
/// beside it. So the curve says where the corner is and the display prints the
/// setting, which is what the instrument's own display does with it.
fn high_pass(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let Some(corner) = travel(patch, ParamId::VcfHighPassFrequency) else {
        return;
    };
    let span = octaves(HIGH_PASS_SLOPE);
    screen.across(band.x, band.row(FILTER_UNITY), band.width, Ink::Dotted);
    screen.curve(band, Ink::Solid, |x| {
        let below = ((corner - x) * span).max(0.0);
        decibels(-HIGH_PASS_SLOPE * below)
    });
    if is_on(patch, ParamId::VcfBassBoost) == Some(true) {
        // Under the passband and past the corner, which is the one part of this
        // drawing that is always empty: a high-pass has nothing below it on the
        // side the curve has already risen on.
        screen.write(
            band.width - Screen::width_of("BOOST", Size::Small) - 1,
            band.height - Screen::height_of(Size::Small),
            "BOOST",
            Size::Small,
        );
    }
}

/// Draws the shapes one oscillator is making.
///
/// A plate each, because the instrument has two oscillators and this window has
/// a plate for each of them. They are not drawn added together on either: how
/// the saw and the pulse of the first sum, and at what weight, is not something
/// the manual gives, and a single waveform drawn out of a mix law this window
/// invented would be the most confident wrong picture on the panel. So each
/// shape is drawn as the shape it is, and what is on the screen is what is
/// switched on.
fn oscillator(screen: &mut Screen, patch: &Patch, which: Which) {
    let band = screen.all();
    match which {
        Which::First => first(screen, patch, band),
        Which::Second => second(screen, patch, band),
    }
}

/// The oscillator whose saw and pulse can both be switched on.
fn first(screen: &mut Screen, patch: &Patch, upper: Band) {
    let saw = is_on(patch, ParamId::Osc1SawEnable).unwrap_or_default();
    let pulse = is_on(patch, ParamId::Osc1PulseEnable).unwrap_or_default();
    let shapes = i32::from(saw) + i32::from(pulse);
    if shapes == 0 {
        // Neither shape switched on is an oscillator making nothing, which is
        // a flat line and not an empty lane: a display with nothing on it reads
        // as a display that has not been drawn.
        screen.across(upper.x, upper.row(0.5), upper.width, Ink::Solid);
    }
    let mut drawn = 0;
    for (on, square) in [(saw, false), (pulse, true)] {
        if !on {
            continue;
        }
        // Side by side where both are running, because they are two shapes and
        // not one: a lane divided is a lane saying so.
        let width = upper.width / shapes.max(1);
        let part = Band::new(upper.x + drawn * width, upper.y, width - 1, upper.height);
        if square {
            screen.curve(part, Ink::Solid, |x| {
                if (x * 2.0).fract() < 0.5 { 1.0 } else { 0.0 }
            });
            swing(screen, patch, part);
        } else {
            screen.curve(part, Ink::Solid, |x| (x * 2.0).fract());
        }
        drawn += 1;
    }
}

/// The oscillator that is a shape at a level, with the noise beside it.
///
/// The one oscillator on this instrument whose loudness is a parameter, so the
/// drawing is as tall as the level is. Noise is the same reading, scattered
/// rather than shaped, and it is on this plate because it is where the panel
/// prints it.
fn second(screen: &mut Screen, patch: &Patch, lower: Band) {
    if let Some(level) = travel(patch, ParamId::Osc2Level) {
        screen.curve(lower, Ink::Solid, |x| {
            0.5 + level / 2.0 * if (x * 2.0).fract() < 0.5 { 1.0 } else { -1.0 }
        });
    }
    if let Some(noise) = travel(patch, ParamId::NoiseLevel)
        && noise > 0.0
    {
        for column in 0..lower.width {
            let height = 0.5 + (scattered(column) - 0.5) * noise;
            screen.dot(lower.x + column, lower.row(height));
        }
    }
}

/// Draws how far pulse width modulation swings the edge of a pulse.
///
/// Dotted marks either side of the edge, as far from it as the depth is
/// through its own range. What a byte of depth does to a duty cycle is not
/// published, so this is the fader's reading drawn where the edge is rather
/// than a width: it says how much of its travel the modulation is using, and
/// says nothing about what percentage the pulse becomes.
fn swing(screen: &mut Screen, patch: &Patch, band: Band) {
    let Some(depth) = travel(patch, ParamId::Osc1PwmDepth) else {
        return;
    };
    if depth <= 0.0 {
        return;
    }
    for edge in [0.25 - depth * 0.2, 0.25 + depth * 0.2] {
        screen.mark(band, edge.clamp(0.0, 1.0), Ink::Dotted);
    }
}

/// Draws one envelope, filling the plate it is on.
///
/// The instrument has three envelopes, one display and a `VCA`, `VCF` and `MOD`
/// button choosing which of its four faders address — so a player comparing the
/// filter's decay with the amplifier's is comparing one with a memory of the
/// other. This window unfolds that into three plates, and this is what each of
/// them draws: its own envelope, at the size of its own screen, with the faders
/// that move it underneath.
///
/// The shape is [`generator::envelope`] and the bends are the curve bytes',
/// which is what 26.4 changed: the four faders under the drawing that used to
/// move nothing on it now move it.
fn envelope_on(screen: &mut Screen, patch: &Patch, group: Group) {
    let band = screen.all();
    // The floor, so that a plate whose envelope has not been read is still a
    // drawing of an envelope rather than an empty screen.
    screen.across(band.x, band.row(0.0), band.width, Ink::Dotted);
    let Some(shape) = envelope::shape_of(patch, group) else {
        return;
    };
    // Filled, because a plate showing one envelope has the room to say what
    // shape it is rather than only where its line runs.
    screen.under(band, |x| shape.at(x));
    screen.curve(band, Ink::Solid, |x| shape.at(x));
}

/// Draws the amplifier's envelope, under the level it is played at.
///
/// The level is the ceiling rather than a mark beside it: an envelope drawn to
/// the top of a screen on a patch whose `VCA Level` is half way up is an
/// envelope drawn at a loudness the instrument is not playing.
fn amplifier(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let (Some(shape), Some(level)) = (
        envelope::shape_of(patch, Group::VcaEnvelope),
        travel(patch, ParamId::VcaLevel),
    ) else {
        return;
    };
    screen.under(band, |x| shape.at(x) * level);
    screen.curve(band, Ink::Solid, |x| shape.at(x) * level);
    screen.across(band.x, band.row(level), band.width, Ink::Dotted);
}

/// Draws how far apart a unison's voices are, and what mode is playing.
///
/// Five marks: nothing is claimed about how many voices a mode actually
/// stacks, which is in the mode's own name and not in the detune. What the
/// drawing says is what the fader says — how far through its travel the detune
/// is — with the mode written above it in the display's own words.
fn voicing(screen: &mut Screen, patch: &Patch, firmware: Version) {
    let mut band = screen.all();
    if let Some(mode) = named(patch, ParamId::PolyphonyMode, firmware)
        && band.height > Screen::height_of(Size::Small) + 4
    {
        // Centred, because the marks under it are centred on the same middle:
        // a word against the left edge over a spread about the centre reads as
        // two drawings that happened to land on one screen.
        screen.centre(0, mode, Size::Small);
        let written = Screen::height_of(Size::Small) + 2;
        band = Band::new(band.x, band.y + written, band.width, band.height - written);
    }
    let Some(detune) = travel(patch, ParamId::UnisonDetune) else {
        return;
    };
    for voice in [-1.0, -0.5, 0.0, 0.5, 1.0_f32] {
        screen.mark(band, 0.5 + voice * detune * 0.5, Ink::Solid);
    }
}

/// Draws the gates the arpeggiator opens.
///
/// [`generator::arpeggiator_gates`], which is the one drawing on the panel
/// whose two axes are both published: the manual gives the gate time byte as a
/// fraction of a step outright — 0 is no note, 255 a full one and 128 half of
/// one — and [`GATES_DRAWN`] steps is the pattern its own illustration uses.
///
/// The rate is not in it. What a step is worth in seconds is exactly what the
/// manual does not print, so a rate byte stretched across the glass was this
/// window drawing an axis nobody published; the fader says what the rate is and
/// the screen says what the gate does to a step.
///
/// An arpeggiator that is switched off is a line: a screen still drawing gates
/// for a sound that is not arpeggiating would be the panel's one lie.
fn arpeggiator(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let Some(program) = patch.program() else {
        return;
    };
    if is_on(patch, ParamId::ArpOnOff) == Some(false) {
        screen.across(band.x, band.y + band.height - 1, band.width, Ink::Solid);
        return;
    }
    let gates: Vec<Gate> = generator::arpeggiator_gates(program).collect();
    if gates.is_empty() {
        // A gate byte of nothing is no note at all, which the library answers
        // with no gates. The step line stays, because the arpeggiator is
        // running and opening nothing is what it is doing.
        screen.across(band.x, band.y + band.height - 1, band.width, Ink::Dotted);
        return;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "the count of steps the library draws, which is four"
    )]
    let steps = GATES_DRAWN as f32;
    let open = move |x: f32| {
        let step = x * steps;
        let held = gates
            .iter()
            .any(|gate| step >= gate.start() && step < gate.end());
        if held { 1.0 } else { 0.0 }
    };
    // Filled, because a gate is a note being held and a block reads as one
    // where an outline reads as a shape.
    screen.under(band, &open);
    screen.curve(band, Ink::Solid, &open);
}

/// Draws an LFO's shape, at its own rate.
///
/// The shape is [`generator::lfo`] — the seven the value table names, including
/// the two sampled ones, whose steps used to be a scatter table in this file
/// and are now a fixed sequence the library publishes and marks as not being
/// the instrument's own stream. A byte this firmware's table does not name is
/// written out instead, because a display that drew a sine for a shape it did
/// not recognise would be inventing the sound.
///
/// The rate is how many of the library's own horizontals fit across the screen
/// and is not a speed: what a byte of `Rate` is in hertz is not published, and
/// what this says is that more of it is more cycles. One is what the library
/// publishes — a turn of a sine, and six steps of a sample and hold, which is
/// what that shape needs to show itself.
///
/// The delay is not drawn at all. `Delay / Fade` is one parameter doing two
/// things and the manual does not say where the byte stops doing one and starts
/// doing the other, so the fader says what it is and the screen says nothing.
fn lfo(
    screen: &mut Screen,
    patch: &Patch,
    firmware: Version,
    which: LfoId,
    shape: ParamId,
    rate: ParamId,
) {
    let band = screen.all();
    let (Some(program), Some(value), Some(rate)) =
        (patch.program(), patch.value(shape), travel(patch, rate))
    else {
        return;
    };
    if LfoShape::from_raw(value).is_none() {
        let name = named(patch, shape, firmware).unwrap_or("?");
        screen.write(
            0,
            band.height / 2 - Screen::height_of(Size::Small) / 2,
            name,
            Size::Small,
        );
        return;
    }
    let drawn = generator::lfo(program, which);
    let over = turns(rate, 5.0);
    screen.curve(band, Ink::Solid, |x| {
        let through = x * over;
        // The far edge is the end of the last cycle rather than the start of
        // the next one, which is where `fract` would put it.
        drawn.at(if through >= over {
            1.0
        } else {
            through.fract()
        })
    });
}

/// Draws the display a plate holding `controls` has, when it has one.
///
/// The claim is the weakest across everything the scene reads, and a scene
/// with anything unread draws nothing: the glass is still there, dark, which is
/// the answer a fader with no cap gives to the same question.
pub(crate) fn display<'a, Renderer>(
    patch: &Patch,
    controls: impl IntoIterator<Item = ParamId>,
    firmware: Version,
    columns: i32,
    rows: i32,
) -> Option<crate::Element<'a, Renderer>>
where
    Renderer: iced_core::Renderer + 'a,
{
    let scene = of(controls)?;
    let claim = patch.claim_across(scene.parameters());
    let screen = if matches!(claim, Confidence::Unknown) {
        Screen::new(columns, rows)
    } else {
        scene.screen(patch, firmware, columns, rows)
    };
    Some(crate::lcd::lcd(screen, claim))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{Scene, decibels, envelopes, of, shape_of, travel};
    use crate::home::rows;
    use crate::{Confidence, Patch};
    use deepmind_midi::generator::{FILTER_UNITY, LfoId};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, Kind, ParamId};
    use deepmind_midi::program::{LfoShape, Program};

    /// A patch the synthesizer has described.
    fn read() -> Patch {
        let mut patch = Patch::new();
        patch.confirm(Program::new(ProtocolVersion::V7));
        patch
    }

    #[test]
    fn every_plate_of_the_panel_draws_something() {
        // The panel is ten plates and each of them is a section of the
        // instrument, so a plate with no drawing is a section this window has
        // nothing to say about.
        for plate in rows().iter().flat_map(|row| row.iter()) {
            assert!(
                of(plate.parameters()).is_some(),
                "the {} plate draws nothing",
                plate.name()
            );
        }
    }

    #[test]
    fn the_two_filters_are_told_apart_by_the_frequency_on_the_plate() {
        // One section, two plates. Every other scene is found by the section a
        // control belongs to, and these two cannot be.
        assert_eq!(of([ParamId::VcfFrequency]), Some(Scene::Filter));
        assert_eq!(of([ParamId::VcfHighPassFrequency]), Some(Scene::HighPass));
        assert_eq!(
            ParamId::VcfFrequency.group(),
            ParamId::VcfHighPassFrequency.group()
        );
    }

    #[test]
    fn an_lfo_is_found_by_the_shape_parameter_the_library_names() {
        // Two LFOs, one drawing, and the scene carries which of the library's
        // two it is asking for rather than working it out again where it draws.
        for (which, shape, rate) in [
            (LfoId::One, ParamId::Lfo1Shape, ParamId::Lfo1Rate),
            (LfoId::Two, ParamId::Lfo2Shape, ParamId::Lfo2Rate),
        ] {
            assert_eq!(of([shape]), Some(Scene::Lfo { which, shape, rate }));
        }
    }

    #[test]
    fn each_lfo_plate_draws_its_own_shape() {
        // The bridge between a shape parameter and the library's own `LfoId`,
        // held to by what the answer draws: a square on one LFO leaves the
        // other one's plate drawing the sine its byte still says.
        let mut patch = read();
        assert!(patch.edit(ParamId::Lfo1Shape, LfoShape::Square.raw()));

        let square = shape_of(LfoId::One);
        assert_eq!(square, ParamId::Lfo1Shape);
        let program = patch.program().expect("a sound that has been read");
        let one = deepmind_midi::generator::lfo(program, LfoId::One);
        let two = deepmind_midi::generator::lfo(program, LfoId::Two);

        // A square is at one end or the other and a sine passes through the
        // middle, which is the cheapest way to tell the two apart.
        assert!(one.at(0.3) > 0.99 || one.at(0.3) < 0.01);
        assert!(two.at(0.3) > 0.01 && two.at(0.3) < 0.99);
    }

    #[test]
    fn every_shape_the_table_names_is_one_the_library_draws() {
        // The seven shapes of the value table, held against the seven the
        // library has a wave for. A shape it does not know is written out on
        // the glass instead of drawn, and this is the test that says which of
        // the two a firmware's table would get.
        let Kind::Enumerated(table) = ParamId::Lfo1Shape.kind() else {
            // Unreachable: a shape is one of a named set, and a library that
            // made it a sweep would have taken the names with it.
            return;
        };

        for entry in table.table_for(DEFAULT_FIRMWARE).entries {
            let raw = u8::try_from(entry.value).expect("a shape byte");
            assert!(
                LfoShape::from_raw(raw).is_some(),
                "{} is a shape the library does not draw",
                entry.name
            );
        }
    }

    #[test]
    fn each_envelope_is_its_own_screen_reading_only_its_own_eight() {
        // The whole of what unfolding the section bought. Three envelopes
        // sharing one screen were three sets of dots in the same place, and the
        // question the display is for — which of these decays first — is the
        // one an overlay answers worst. Three plates answer it by being three
        // drawings.
        assert_eq!(envelopes().len(), 3);
        for group in envelopes() {
            let parameters = Scene::Envelope(group).parameters();

            assert_eq!(parameters.len(), 8, "{group} is four times and four curves");
            assert!(
                parameters
                    .iter()
                    .all(|parameter| parameter.group() == group),
                "{group}'s screen reads another envelope's values"
            );
        }
    }

    #[test]
    fn an_envelope_plate_draws_its_own_envelope_and_not_another() {
        for group in envelopes() {
            assert_eq!(
                of(group.parameters()),
                Some(Scene::Envelope(group)),
                "{group} does not put its own drawing on its own plate"
            );
        }
    }

    #[test]
    fn a_curve_moves_the_envelope_that_is_drawn() {
        // What 26.4 changed. The four curve faders under an envelope's plate
        // used to move four bytes and nothing on the screen above them.
        let mut patch = read();
        assert!(patch.edit(ParamId::VcaEnvelopeAttackTime, 200));
        let straight = Scene::Envelope(Group::VcaEnvelope).screen(&patch, DEFAULT_FIRMWARE, 40, 20);

        assert!(patch.edit(ParamId::VcaEnvelopeAttackCurve, 255));
        let bent = Scene::Envelope(Group::VcaEnvelope).screen(&patch, DEFAULT_FIRMWARE, 40, 20);

        assert_ne!(straight, bent, "a curve byte moved nothing on the glass");
    }

    #[test]
    fn an_envelope_screen_is_a_drawing_before_anything_is_read() {
        // The floor is drawn whatever is known, because a plate about an
        // envelope with an empty screen on it reads as a plate that is broken.
        let mut screen = super::super::Screen::new(40, 20);
        super::envelope_on(&mut screen, &Patch::new(), Group::VcaEnvelope);

        assert!(!screen.is_blank(), "an envelope plate drew nothing at all");
    }

    #[test]
    fn a_filter_is_drawn_on_the_librarys_own_decibel_axis() {
        // Unity gain is where the library says it is, and it is where the
        // passband of a filter with its corner at the top of its range lands.
        assert!((decibels(0.0) - FILTER_UNITY).abs() < 0.001);
        assert!(decibels(deepmind_midi::generator::FILTER_CEILING_DB) > 0.99);
        assert!(decibels(deepmind_midi::generator::FILTER_FLOOR_DB) < 0.01);
    }

    #[test]
    fn the_two_filters_are_drawn_to_one_proportion() {
        // The one number about a filter this window still holds is how far past
        // its own fall the axis reaches, and it is not a number this window
        // chose: asked of a four-pole low-pass it gives back the span the
        // library publishes for exactly that filter. A library that widened its
        // own span fails here rather than leaving the high-pass beside it drawn
        // to the old one.
        // A program at its floor is already four-pole, which is what `0` is on
        // this instrument and why nothing here reads the byte as a count.
        let patch = read();
        let program = patch.program().expect("a sound that has been read");
        let response = super::generator::filter_response(program);

        let span = super::published(&response).expect("an octave axis");
        assert!(
            (super::octaves(24.0) - span).abs() < 0.001,
            "the library draws {span} octaves and this window would draw {}",
            super::octaves(24.0)
        );
        // And a slope a quarter as steep is drawn across four times as much, so
        // its knee lands in the same place on the glass.
        assert!((super::octaves(6.0) - span * 4.0).abs() < 0.001);
    }

    #[test]
    fn a_resonant_peak_reaches_into_the_headroom_over_the_passband() {
        // The reason a filter is drawn on a decibel axis at all: a resonance
        // has somewhere to go that is not the top of the glass, and the screen
        // says so by changing when the byte moves.
        let mut patch = read();
        assert!(patch.edit(ParamId::VcfFrequency, 128));
        let flat = Scene::Filter.screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert!(patch.edit(ParamId::VcfResonance, 255));
        let peaked = Scene::Filter.screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert_ne!(flat, peaked, "resonance moved nothing on the glass");
    }

    #[test]
    fn the_two_oscillators_are_two_drawings() {
        use super::Which;

        // `OSC 1` is two shapes that may both be switched on; `OSC 2` is one
        // shape at a level with the noise over it. One plate each, so one
        // drawing each, and neither reads the other's values.
        assert_eq!(
            of([ParamId::Osc1PwmDepth]),
            Some(Scene::Oscillator(Which::First))
        );
        assert_eq!(
            of([ParamId::Osc2Level]),
            Some(Scene::Oscillator(Which::Second))
        );
        // The noise names no oscillator and is printed at the second's end of
        // the panel, which is the plate this window puts it on.
        assert_eq!(
            of([ParamId::NoiseLevel]),
            Some(Scene::Oscillator(Which::Second))
        );

        let first = Scene::Oscillator(Which::First).parameters();
        let second = Scene::Oscillator(Which::Second).parameters();
        assert!(
            first.iter().all(|parameter| !second.contains(parameter)),
            "the two oscillator screens read the same values"
        );
    }

    #[test]
    fn the_arpeggiator_draws_the_gate_and_not_the_rate() {
        // The gate time is published against a step and the step is not
        // published against a second, so the gate is on the glass and the rate
        // is on its fader. A screen that read the rate would be claiming an
        // axis nobody prints.
        let parameters = Scene::Arpeggiator.parameters();
        assert!(parameters.contains(&ParamId::ArpGateTime));
        assert!(!parameters.contains(&ParamId::ArpRateTempo));

        let mut patch = read();
        assert!(patch.edit(ParamId::ArpOnOff, 1));
        assert!(patch.edit(ParamId::ArpGateTime, 64));
        let short = Scene::Arpeggiator.screen(&patch, DEFAULT_FIRMWARE, 40, 20);

        assert!(patch.edit(ParamId::ArpGateTime, 220));
        let held = Scene::Arpeggiator.screen(&patch, DEFAULT_FIRMWARE, 40, 20);

        assert_ne!(short, held, "the gate time moved nothing on the glass");
    }

    #[test]
    fn a_scene_is_blank_until_something_has_been_read() {
        let scene = Scene::Filter;
        let unread = Patch::new();

        assert!(
            scene.screen(&unread, DEFAULT_FIRMWARE, 40, 20).is_blank(),
            "a filter drawn from a sound nobody has read"
        );
        assert!(
            !scene.screen(&read(), DEFAULT_FIRMWARE, 40, 20).is_blank(),
            "a filter that has been read and is not drawn"
        );
    }

    #[test]
    fn a_screen_is_as_confirmed_as_the_least_of_what_it_drew() {
        // The rule the envelope drawing and the name field are both under: one
        // value this window claimed makes the whole picture a claim, because
        // nothing has heard the instrument play the sound it draws.
        let mut patch = read();
        assert_eq!(
            patch.claim_across(Scene::Filter.parameters()),
            Confidence::Confirmed
        );

        assert!(patch.edit(ParamId::VcfResonance, 90));
        assert_eq!(
            patch.claim_across(Scene::Filter.parameters()),
            Confidence::Assumed
        );
    }

    #[test]
    fn a_value_is_read_as_a_fraction_of_its_own_range() {
        let mut patch = read();

        assert!(patch.edit(ParamId::VcfFrequency, 255));
        assert_eq!(travel(&patch, ParamId::VcfFrequency), Some(1.0));
        assert!(patch.edit(ParamId::VcfFrequency, 0));
        assert_eq!(travel(&patch, ParamId::VcfFrequency), Some(0.0));
        assert_eq!(travel(&Patch::new(), ParamId::VcfFrequency), None);
    }

    #[test]
    fn a_section_with_nothing_to_draw_draws_nothing() {
        // A plate is only given a display where there is a picture to put on
        // it. The program's own settings are a name and a category, and a
        // drawing of those is a drawing of nothing.
        assert!(of([ParamId::ProgramCategory]).is_none());
        assert!(of([ParamId::Mod1Depth]).is_none());
    }
}

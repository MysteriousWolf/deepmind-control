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
//! gates: every one of those is a fact about the instrument, and 26.4 published
//! them as functions
//! ([deepmind-midi#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32)).
//! 26.5 published the four this file was still assembling for itself — the
//! high-pass's slope, what each oscillator is putting out and how its two waves
//! sum, the noise, and the LFO's fade
//! ([#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35),
//! [#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37)) — and with
//! them the three things a shape can now say about itself:
//! [`rest`](deepmind_midi::generator::Generator::rest) is the line to measure it
//! from, [`marks`](deepmind_midi::generator::Generator::marks) is where along it
//! something happens, and [`anchored`](deepmind_midi::generator::Generator::anchored)
//! says whether its left edge is a moment the instrument has
//! ([#36](https://github.com/MysteriousWolf/deepmind-midi/issues/36)).
//!
//! What is left here is a sample loop — walk the columns of a band, ask
//! [`Generator::at`](deepmind_midi::generator::Generator::at) what the shape is
//! doing there, and print the dot nearest the answer. Where a byte's curve has
//! not been measured the library says so in
//! [`Scale`](deepmind_midi::generator::Scale) rather than inventing one, which
//! is the same refusal this file used to make in prose and could not test.
//!
//! **Nothing about the instrument is written down in this file.** There was one
//! number — the high-pass's 6 dB per octave, transcribed out of a doc comment —
//! and there is not one now.
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

use deepmind_midi::generator::{
    self, FILTER_UNITY, GATES_DRAWN, Gate, Generator, LfoId, MarkKind, OscillatorId, Segment,
};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::{LfoShape, Program, VcfEnvelopePolarity};
use deepmind_midi::sysex::inquiry::Version;

use crate::envelope;
use crate::lcd::{Band, Ink, Screen, Size};
use crate::{Confidence, Patch};

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
/// Between `least` and `most`. A rate drawn as a count of turns says that a
/// faster setting is more of them in the same window, which is true of every
/// rate this instrument has and is as much as the manual establishes; a screen
/// is not a length of time, so nothing here is a speed. The shape being
/// repeated is the library's, and so is the count of turns its own horizontal
/// covers — this is only how many of them the glass is given.
fn turns(travel: f32, least: f32, most: f32) -> f32 {
    least + travel * (most - least)
}

/// Fewest turns of an LFO the glass is ever given.
///
/// Two, and never one. Every shape in the library's table starts at the bottom
/// of its range so that the seven can be drawn side by side without one looking
/// shifted, which means one turn of the sine is a hill: it leaves the floor,
/// reaches the top and comes back, and a picture of that is a bump rather than
/// something going round. The second turn is what says it repeats, and what it
/// swings about is [`centre`].
const LEAST_TURNS: f32 = 2.0;

/// Most turns of one it is given, at the top of the rate's travel.
const MOST_TURNS: f32 = 8.0;

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

impl Which {
    /// Returns the library's own name for this oscillator.
    ///
    /// The two enumerations are the same two things and this window keeps its
    /// own because a scene carries which plate it is on. Converting here rather
    /// than storing the library's keeps that one line the only place the two
    /// could ever disagree.
    const fn published(self) -> OscillatorId {
        match self {
            Self::First => OscillatorId::One,
            Self::Second => OscillatorId::Two,
        }
    }
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
            Self::Filter => filter(&mut screen, patch),
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

/// How many dots tall a tick on a published axis stands.
///
/// Two: enough to read as a mark along the foot of the glass and not enough to
/// be mistaken for anything the sound is doing. A drawing carries one curve,
/// and an axis that competed with it would be a second.
const TICK: i32 = 2;

/// Rules the axis the library publishes for a shape, where it publishes one.
///
/// [`generator::Scale`] is the honest half of a generator: two of its three
/// answers are measured — an octave either side of a filter's corner, a turn of
/// an LFO — and the third says outright that the horizontal is an ordering and
/// nothing else. So this marks the first two and draws nothing at all for the
/// third, which is the whole point of the library publishing it: a screen with
/// a scale on it that nobody measured is a screen that looks like information.
///
/// `every` is how much of the drawn width one unit of the scale covers, which
/// the caller knows and the generator does not: a window showing four turns of
/// a one-turn shape is drawing the same scale four times over.
fn ruled(screen: &mut Screen, band: Band, every: f32) {
    if every <= 0.0 || every >= 1.0 {
        return;
    }
    let mut at = every;
    while at < 1.0 {
        screen.down(
            band.column(at),
            band.y + band.height - TICK,
            TICK,
            Ink::Dotted,
        );
        at += every;
    }
}

/// Draws what the library says is at a place along a shape.
///
/// [`Generator::marks`] is the second half of what a generator knows about
/// itself: the axis says how the horizontal is divided and the marks say where
/// along it something *happens* — the corner of a filter, the end of a cycle,
/// the edge of a pulse and how far modulation swings it. Every one is a number
/// the library computed to build the shape, so a window that ruled them itself
/// would be deriving the same thing from the same bytes a second time.
///
/// `marks` are already on the glass: a caller maps the generator's own
/// positions onto the screen, which is the one thing it knows and the generator
/// does not. A filter is drawn about a corner whose byte this window read, so
/// the generator's own middle is wherever the fader put it; an LFO's horizontal
/// is the library's own repeated as many times as the rate asks for.
///
/// What is drawn is the screen's decision and not the library's, which is what
/// [`Mark`](deepmind_midi::generator::Mark) says outright: a division of the
/// axis is a tick along the foot, and the two marks that are points in a wave
/// rather than divisions of it stand up the whole band.
fn marked(screen: &mut Screen, band: Band, marks: impl Iterator<Item = (f32, MarkKind)>) {
    let mut taken: Option<i32> = None;
    for (at, kind) in marks {
        if !(0.0..=1.0).contains(&at) {
            continue;
        }
        let column = band.column(at);
        // Two marks closer than a tick is wide are one smudge. Six sampled
        // steps repeated eight times is forty-eight cycle ends across sixty
        // columns, and which of them a screen has room for is the screen's
        // business — the library says where they are and says in the same
        // breath that what to draw at one depends on the room a host has.
        if taken.is_some_and(|last| column - last < APART) {
            continue;
        }
        taken = Some(column);
        match kind {
            // Where a pulse falls and how far its modulation moves that edge.
            // Up the whole band, because it is a moment in the cycle the curve
            // is drawing and not a division of the axis under it.
            MarkKind::Width => screen.mark(band, at, Ink::Solid),
            MarkKind::Sweep => screen.mark(band, at, Ink::Dotted),
            _ => screen.down(column, band.y + band.height - TICK, TICK, Ink::Dotted),
        }
    }
}

/// Returns a shape's own marks, laid onto the glass by `place`.
fn placed(shape: &Generator, place: impl Fn(f32) -> f32) -> impl Iterator<Item = (f32, MarkKind)> {
    shape
        .marks()
        .map(move |mark| (place(mark.at()), mark.kind()))
}

/// Fewest dots apart two marks are drawn.
///
/// Three: a tick two dots wide with a dot of gap beside it is the least that
/// still reads as two marks rather than as a thicker one.
const APART: i32 = TICK + 1;

/// Writes the initial of each envelope segment where the library says it starts.
///
/// The one thing four faders under a drawing cannot say: which part of the line
/// each of them is moving. The positions are
/// [`MarkKind::Segment`](deepmind_midi::generator::MarkKind::Segment) and the
/// letters are this window's, because what to write beside a mark is the room a
/// screen has and the language it is in, which is exactly what the library
/// declines to decide.
///
/// A letter that would land on the one before it is dropped. Two boundaries
/// share a position when a time byte is zero — an attack of nothing begins and
/// ends at the left edge — and a screen cannot print two letters in one column.
fn segments(screen: &mut Screen, band: Band, shape: &Generator) {
    let tall = Screen::height_of(Size::Small);
    if band.height < tall * 2 {
        return;
    }
    let row = band.y + band.height - tall;
    let mut taken = band.x;
    for mark in shape.marks() {
        let MarkKind::Segment(segment) = mark.kind() else {
            continue;
        };
        let letter = match segment {
            Segment::Attack => "A",
            Segment::Decay => "D",
            Segment::Sustain => "S",
            Segment::Release => "R",
        };
        let at = band.column(mark.at()) + 1;
        let wide = Screen::width_of(letter, Size::Small);
        if at < taken || at + wide > band.x + band.width {
            continue;
        }
        screen.write(at, row, letter, Size::Small);
        taken = at + wide + 1;
    }
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
fn filter(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let (Some(program), Some(corner)) = (patch.program(), travel(patch, ParamId::VcfFrequency))
    else {
        return;
    };
    let response = generator::filter_response(program);
    // The level the filter passes, on the library's own axis.
    screen.across(band.x, band.row(FILTER_UNITY), band.width, Ink::Dotted);
    // And the axis itself, an octave a tick. The library publishes the span the
    // response is drawn across, so this is the one horizontal on any of these
    // plates that is measured rather than an ordering.
    if let generator::Scale::Octaves(span) = response.scale() {
        ruled(screen, band, 1.0 / span.max(1.0));
    }
    // The corner sits at `0.5` along the generator, so a column of the screen
    // is half a span either side of wherever the byte put it.
    screen.curve(band, Ink::Solid, |x| response.at(0.5 + x - corner));
    marked(screen, band, placed(&response, |at| at - 0.5 + corner));
    reach(screen, patch, corner);
}

/// Draws how far the envelope moves the corner, and which way.
///
/// A rule along the top of the screen from the corner to where the depth would
/// take it, which is the depth drawn as the fraction of the axis it is — the
/// same reading the fader beside it gives. It is not a second filter curve:
/// that the two parameters are in the same units is exactly what the manual
/// does not say, so this says how much of its own travel the depth is using and
/// which way the polarity points it, and stops there.
fn reach(screen: &mut Screen, patch: &Patch, corner: f32) {
    let Some(depth) = travel(patch, ParamId::VcfEnvelopeDepth) else {
        return;
    };
    if depth <= 0.0 {
        return;
    }
    // The library's own reading of the byte rather than the word its display
    // prints: a drawing that matched on `Negative` would point the reach the
    // wrong way on the day that word moved, and there is a type for it.
    let positive = patch.program().and_then(Program::vcf_envelope_polarity)
        != Some(VcfEnvelopePolarity::Negative);
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
/// [`generator::high_pass_response`], which 26.5 published: one pole falling to
/// the left, on the low-pass's own decibel vertical and across the low-pass's
/// own span, so the two plates of one section are two readings of one picture
/// rather than two pictures. The slope was the last number about the instrument
/// written down in this window, transcribed out of a doc comment in the library
/// and now asked of the library instead
/// ([deepmind-midi#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37)).
///
/// What the library still will not do is put both corners on one axis — their
/// spacing is two unpublished curves — and a plate with one filter on it is not
/// asking it to. The corner is where the fader says it is, exactly as on the
/// low-pass.
///
/// The boost is not in the curve. It is two states, what it lifts the low end
/// by is not published, and a shelf drawn under the corner would be this
/// window choosing a height and then drawing it as confidently as the corner
/// beside it. So the curve says where the corner is and the display prints the
/// setting, which is what the instrument's own display does with it.
fn high_pass(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let (Some(program), Some(corner)) = (
        patch.program(),
        travel(patch, ParamId::VcfHighPassFrequency),
    ) else {
        return;
    };
    let response = generator::high_pass_response(program);
    screen.across(band.x, band.row(response.rest()), band.width, Ink::Dotted);
    // The same octave a tick as the low-pass beside it, off the same accessor,
    // which is what makes the two plates two readings of one picture.
    if let generator::Scale::Octaves(span) = response.scale() {
        ruled(screen, band, 1.0 / span.max(1.0));
    }
    screen.curve(band, Ink::Solid, |x| response.at(0.5 + x - corner));
    marked(screen, band, placed(&response, |at| at - 0.5 + corner));
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

/// Draws what one oscillator is putting out.
///
/// A plate each, because the instrument has two oscillators and this window has
/// a plate for each of them. What used to be here was the window assembling a
/// shape out of six parameters and then refusing to add the two waves of `OSC 1`
/// together, because how they sum is not something the manual gives — so they
/// were drawn side by side, which is a picture of two oscillators where the
/// instrument has one.
///
/// 26.5 publishes both
/// ([deepmind-midi#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37)):
/// [`generator::oscillator`] is what the oscillator is putting out, the two
/// waves summed at equal weight, and it says in its own documentation that the
/// equal weight is the library's reading rather than the manual's. The pulse's
/// falling edge and how far modulation swings it are
/// [marks](deepmind_midi::generator::MarkKind::Width) on that shape rather than
/// arithmetic here.
fn oscillator(screen: &mut Screen, patch: &Patch, which: Which) {
    let band = screen.all();
    let Some(program) = patch.program() else {
        return;
    };
    let shape = generator::oscillator(program, which.published());
    // Where the wave sits when it is doing nothing, which the library answers
    // for every shape it draws. An oscillator with neither wave switched on is
    // a flat line along it, which is what making nothing looks like and is not
    // the same as an empty screen.
    screen.across(band.x, band.row(shape.rest()), band.width, Ink::Dotted);
    screen.curve(band, Ink::Solid, |x| shape.at(x));
    marked(screen, band, placed(&shape, |at| at));
    if which == Which::Second {
        noise(screen, patch, band);
    }
}

/// Draws the noise generator over the plate `OSC 2` is on.
///
/// [`generator::noise`], which is a scatter at the level byte's own reading —
/// the same fixed sequence on every frame, because a screen that reseeded when
/// nothing had moved would crawl and a crawling drawing says the sound is doing
/// something it is not. The sequence used to be a table in this file; it is the
/// library's now, and the library says in the same breath that it is not the
/// instrument's own stream.
///
/// Plotted rather than joined: noise is a scatter and a line drawn through it
/// is a waveform, which is the one thing it is not.
fn noise(screen: &mut Screen, patch: &Patch, band: Band) {
    let Some(program) = patch.program() else {
        return;
    };
    if travel(patch, ParamId::NoiseLevel).unwrap_or_default() <= 0.0 {
        return;
    }
    let shape = generator::noise(program);
    let last = f32::from(u16::try_from((band.width - 1).max(1)).unwrap_or(u16::MAX));
    for column in 0..band.width {
        let at = f32::from(u16::try_from(column).unwrap_or(u16::MAX)) / last;
        screen.dot(band.x + column, band.row(shape.at(at)));
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
    // drawing of an envelope rather than an empty screen. An envelope rests at
    // the bottom of its range and the library is what says so.
    screen.across(band.x, band.row(0.0), band.width, Ink::Dotted);
    let Some(shape) = envelope::shape_of(patch, group) else {
        return;
    };
    // Filled, because a plate showing one envelope has the room to say what
    // shape it is rather than only where its line runs.
    screen.under(band, |x| shape.at(x));
    screen.curve(band, Ink::Solid, |x| shape.at(x));
    segments(screen, band, &shape);
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
    segments(screen, band, &shape);
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

/// Draws an LFO's shape, at its own rate, under the fade that brings it in.
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
/// # What 26.5 moved out of here
///
/// Three things, all of them
/// [deepmind-midi#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35):
///
/// - **Where it rests.** This file assumed the middle of the band. It is
///   [`Generator::rest`] now, which is the middle for an LFO read as it swings
///   and the floor for one read unipolar, and the library is what knows which.
/// - **The slew.** `Slew Rate` was not drawn at all. It is in the shape the
///   library hands back, so the corners round and a square becomes a ramp
///   between its levels without this file doing anything.
/// - **The fade.** `Delay / Fade` was not drawn either, because one parameter
///   doing two things is not something a window should guess at. It is
///   [`generator::lfo_fade`], drawn as its own dotted line rather than
///   multiplied into the wave: the library publishes the two as two shapes and
///   what their product looks like is not a third thing it published.
///
/// And one thing it added: [`Generator::anchored`] says whether the left edge
/// of the picture is a moment the instrument has. A free-running LFO is caught
/// by a note wherever it had got to, so its first cycle is where a picture had
/// to start and not where anything begins, and the glass says so by leaving
/// that edge unruled.
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
    let over = turns(rate, LEAST_TURNS, MOST_TURNS);
    // The level it swings about, drawn before the shape so that the shape
    // crosses it rather than the other way round. An LFO is a modulation source
    // and where it sits when it is doing nothing is the one thing a picture of
    // a wave needs in order to read as a wave.
    screen.across(band.x, band.row(drawn.rest()), band.width, Ink::Dotted);
    // The fade that brings it in, where there is one to draw. A fade byte of
    // nothing is a flat line at full depth, which says nothing that the wave
    // does not already say, so it is left off rather than drawn along the top.
    let fade = generator::lfo_fade(program, which);
    if fade.at(0.0) < 1.0 {
        screen.curve(band, Ink::Dotted, |x| fade.at(x));
        marked(screen, band, placed(&fade, |at| at));
    }
    // A tick a cycle along the foot, which is the axis the library calls exact:
    // a cycle is a cycle whatever the rate byte does, so a picture can count
    // them even though nothing about this instrument can say how long one is.
    // The shape's own horizontal is repeated across the glass, so each repeat
    // is ruled where the library says its cycles end.
    marked(
        screen,
        band,
        (0..count_of(over)).flat_map(|repeat| {
            let start = f32::from(repeat) / over;
            placed(&drawn, move |at| start + at / over)
        }),
    );
    // The left edge, for an LFO whose cycle a note restarts. Nothing is ruled
    // there for a free-running one: a phase the instrument does not define is
    // not a phase a screen should draw.
    if drawn.anchored() {
        screen.down(band.x, band.y + band.height - TICK, TICK, Ink::Dotted);
    }
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

/// Returns how many whole repeats of a shape `over` fits on the glass.
///
/// A byte, because [`turns`] is bounded by [`LEAST_TURNS`] and [`MOST_TURNS`]
/// and a count that could not be held in one would be a bug in those rather
/// than a number to saturate.
fn count_of(over: f32) -> u8 {
    let whole = over.max(1.0).min(f32::from(u8::MAX));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "bounded to `1..=255` on the line above"
    )]
    let whole = whole as u8;
    whole
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
    use super::{Scene, envelopes, of, shape_of, travel};
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
    fn both_filters_rest_where_the_library_puts_unity() {
        // The high-pass used to be drawn from a slope transcribed into this
        // file and a spread read back off the low-pass, so the two plates
        // agreeing was something this window arranged. Both are the library's
        // now, so the agreement is the library's too: one vertical, with unity
        // in the same place on each, and one horizontal a plate wide.
        let patch = read();
        let program = patch.program().expect("a sound that has been read");
        let low = super::generator::filter_response(program);
        let high = super::generator::high_pass_response(program);

        assert!((low.rest() - FILTER_UNITY).abs() < 0.001);
        assert!((high.rest() - FILTER_UNITY).abs() < 0.001);
        assert_eq!(span_of(&low), span_of(&high), "two plates, two axes");
    }

    /// Returns the octaves a filter is drawn across, which both of them publish.
    fn span_of(response: &super::Generator) -> Option<u32> {
        match response.scale() {
            super::generator::Scale::Octaves(span) => Some(span.to_bits()),
            _ => None,
        }
    }

    #[test]
    fn the_high_pass_rises_the_way_a_high_pass_does() {
        // The one thing the transcribed slope was for, asked of the drawing
        // rather than of the number: the corner falls to the left and the
        // passband is above it, which is what makes it the other filter.
        let patch = read();
        let program = patch.program().expect("a sound that has been read");
        let high = super::generator::high_pass_response(program);

        assert!(high.at(0.2) < high.at(0.5), "a high-pass passing its floor");
        assert!((high.at(0.8) - FILTER_UNITY).abs() < 0.01);
    }

    #[test]
    fn an_envelope_says_which_segment_each_fader_is_moving() {
        // Four faders under a line, and until 26.5 published where the
        // segments begin there was nothing on the glass saying which part of
        // the line each of them moves.
        let patch = read();
        let shape = crate::envelope::shape_of(&patch, Group::VcaEnvelope).expect("a read envelope");
        let mut kinds = shape.marks().filter_map(|mark| match mark.kind() {
            super::MarkKind::Segment(segment) => Some(segment),
            _ => None,
        });

        assert_eq!(kinds.next(), Some(super::Segment::Attack));
        assert_eq!(kinds.next(), Some(super::Segment::Decay));
        assert_eq!(kinds.next(), Some(super::Segment::Sustain));
        assert_eq!(kinds.next(), Some(super::Segment::Release));
        assert_eq!(kinds.next(), None);

        // And a plate with room for the letters draws more than one without.
        let mut lettered = super::super::Screen::new(120, 40);
        super::envelope_on(&mut lettered, &patch, Group::VcaEnvelope);
        let mut bare = super::super::Screen::new(120, 40);
        super::envelope_on(&mut bare, &patch, Group::VcaEnvelope);
        let band = bare.all();
        super::segments(&mut bare, band, &shape);
        assert_eq!(lettered, bare, "the letters are not on the plate");
    }

    #[test]
    fn a_sampled_lfo_is_not_ruled_into_a_smudge() {
        // The library's own marks say where every cycle of a sample and hold
        // ends, and the shape covers six of them before the rate repeats it up
        // to eight times over. Which of those forty-eight a screen has room for
        // is the screen's business, so the ones that could not be told apart
        // are dropped — a plate this wide would otherwise have a tick in
        // roughly every column of its foot.
        let mut patch = read();
        assert!(patch.edit(ParamId::Lfo1Shape, LfoShape::SampleAndHold.raw()));
        assert!(patch.edit(ParamId::Lfo1Rate, 255));
        let screen = Scene::Lfo {
            which: LfoId::One,
            shape: ParamId::Lfo1Shape,
            rate: ParamId::Lfo1Rate,
        }
        .screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        // The row a tick starts on. A tick is dotted, so of its two dots only
        // one is laid down, and it is this one.
        let foot = screen.rows() - super::TICK;
        let ruled = (0..screen.columns())
            .filter(|column| screen.is_inked(*column, foot))
            .count();
        // Eight repeats of a six-cycle shape is forty-eight cycle ends, which
        // on sixty columns is a tick in four columns out of five.
        assert!(
            ruled < 24,
            "the foot of the glass carries {ruled} marks across 60 columns"
        );
        assert!(ruled > 0, "nothing was ruled at all");
    }

    #[test]
    fn an_lfos_fade_is_drawn_when_there_is_one_and_not_when_there_is_not() {
        // `Delay / Fade` used to be a fader that moved nothing on the glass.
        let mut patch = read();
        // A fade byte of nothing, which is where the instrument ships and so
        // may already be where the patch sits: what is asserted is the drawing.
        patch.edit(ParamId::Lfo1DelayFade, 0);
        let none = Scene::Lfo {
            which: LfoId::One,
            shape: ParamId::Lfo1Shape,
            rate: ParamId::Lfo1Rate,
        }
        .screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert!(patch.edit(ParamId::Lfo1DelayFade, 255));
        let faded = Scene::Lfo {
            which: LfoId::One,
            shape: ParamId::Lfo1Shape,
            rate: ParamId::Lfo1Rate,
        }
        .screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert_ne!(none, faded, "the fade moved nothing on the glass");
    }

    #[test]
    fn the_oscillators_sum_their_waves_the_way_the_library_does() {
        // OSC 1's saw and pulse used to be drawn side by side, because how
        // they sum was not something this window would guess at. It is one
        // shape now, and switching the second wave on changes it.
        let mut patch = read();
        patch.edit(ParamId::Osc1SawEnable, 1);
        patch.edit(ParamId::Osc1PulseEnable, 0);
        let alone = Scene::Oscillator(super::Which::First).screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert!(patch.edit(ParamId::Osc1PulseEnable, 1));
        let summed =
            Scene::Oscillator(super::Which::First).screen(&patch, DEFAULT_FIRMWARE, 60, 24);

        assert_ne!(alone, summed, "the second wave changed nothing");
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

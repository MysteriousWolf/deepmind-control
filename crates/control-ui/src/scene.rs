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
//! # What these drawings do not claim
//!
//! The same two refusals the envelope drawing is under, because they are the
//! same refusals the library is under.
//!
//! **No axis is in anybody's units.** A filter's corner is drawn at the
//! fraction of its own range the byte sits at, not at a frequency; a rate is
//! drawn as how many cycles fit across a screen, not as a speed. The manual
//! prints the two ends of a range and almost never the curve between them, so
//! a drawing that put a hertz on an axis would be wrong in a way nobody could
//! see. What every one of these says is *where in its travel* a value is,
//! which is exactly what the fader beside it says.
//!
//! **Nothing is drawn from a value nobody has read.** A scene's claim is the
//! weakest of everything it read, and a scene with anything unread is not
//! drawn: a picture of a filter assembled from four values the synthesizer
//! described and one this window invented is a picture of no filter at all.

use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;

use crate::envelope;
use crate::lcd::{Band, Ink, Screen, Size};
use crate::{Confidence, Patch};

/// Where a passband sits, so that a resonant peak has somewhere to go.
///
/// A filter drawn flat across the top of its screen is a filter whose
/// resonance cannot be drawn at all.
const PASSBAND: f32 = 0.55;

/// How much of a screen a two-pole fall takes to reach nothing.
///
/// Twice as much as a four-pole one, which is the whole of what the pole count
/// changes about the picture. It is not a claim about decibels: it is the one
/// relation between the two that every filter obeys, drawn on an axis that is
/// nobody's units.
const FALL: f32 = 1.1;

/// A scatter that is the same every time it is asked.
///
/// Noise on a display and a sample-and-hold's steps both need a number that
/// looks unchosen, and both need the same one on every frame: a screen that
/// reseeded when nothing had moved would crawl, and a crawling drawing says the
/// sound is doing something it is not.
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

/// Returns how many cycles of something fit across a screen at `travel`.
///
/// Between one and `most`. A rate drawn as a count of cycles says that a faster
/// setting is more of them in the same window, which is true of every rate this
/// instrument has and is as much as the manual establishes; a screen is not a
/// length of time, so nothing here is a speed.
fn cycles(travel: f32, most: f32) -> f32 {
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
    /// The shapes the two oscillators and the noise are making.
    Oscillators,
    /// All three envelopes, on one screen.
    Envelopes,
    /// The amplifier's own envelope, under the level it is played at.
    Amplifier,
    /// How far apart a unison's voices are detuned, and what mode is playing.
    Voicing,
    /// The gates the arpeggiator opens.
    Arpeggiator,
    /// An LFO's shape, at its own rate.
    Lfo {
        /// The parameter naming the shape.
        shape: ParamId,
        /// The one setting how fast it runs.
        rate: ParamId,
    },
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
        return Some(Scene::Envelopes);
    }
    if control.short_name() == "Shape" {
        let rate = control
            .group()
            .parameters()
            .find(|parameter| parameter.short_name() == "Rate")?;
        return Some(Scene::Lfo {
            shape: control,
            rate,
        });
    }
    match control.group() {
        Group::Oscillators => Some(Scene::Oscillators),
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
            Self::Oscillators => vec![
                ParamId::Osc1SawEnable,
                ParamId::Osc1PulseEnable,
                ParamId::Osc1PwmDepth,
                ParamId::Osc2Level,
                ParamId::NoiseLevel,
            ],
            Self::Envelopes => envelopes()
                .into_iter()
                .filter_map(|group| envelope::of(group).map(envelope::Envelope::parameters))
                .flatten()
                .collect(),
            Self::Amplifier => envelope::of(Group::VcaEnvelope)
                .map(envelope::Envelope::parameters)
                .into_iter()
                .flatten()
                .chain([ParamId::VcaLevel])
                .collect(),
            Self::Voicing => vec![ParamId::UnisonDetune, ParamId::PolyphonyMode],
            Self::Arpeggiator => vec![
                ParamId::ArpOnOff,
                ParamId::ArpRateTempo,
                ParamId::ArpGateTime,
            ],
            Self::Lfo { shape, rate } => vec![shape, rate],
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
            Self::Oscillators => oscillators(&mut screen, patch),
            Self::Envelopes => envelopes_on(&mut screen, patch),
            Self::Amplifier => amplifier(&mut screen, patch),
            Self::Voicing => voicing(&mut screen, patch, firmware),
            Self::Arpeggiator => arpeggiator(&mut screen, patch),
            Self::Lfo { shape, rate } => lfo(&mut screen, patch, firmware, shape, rate),
        }
        screen
    }
}

/// Returns every group that is an envelope, in the instrument's own order.
fn envelopes() -> Vec<Group> {
    Group::ORDER
        .iter()
        .copied()
        .filter(|group| envelope::of(*group).is_some())
        .collect()
}

/// Draws where the low-pass corner is and what is happening at it.
///
/// The corner sits at the fraction of its own range the byte is at, the
/// resonance is how far the peak rises out of the passband, and the fall past
/// it is twice as steep with four poles as with two. The pole count is read
/// from what the value table calls the value and never from the value: `0` is
/// `4 Pole` on this instrument, so a drawing that counted the byte would draw
/// every filter the wrong way round.
fn filter(screen: &mut Screen, patch: &Patch, firmware: Version) {
    let band = screen.all();
    let (Some(corner), Some(resonance)) = (
        travel(patch, ParamId::VcfFrequency),
        travel(patch, ParamId::VcfResonance),
    ) else {
        return;
    };
    let poles = named(patch, ParamId::Vcf2PoleMode, firmware)
        .and_then(|name| name.chars().next())
        .and_then(|digit| digit.to_digit(10))
        .unwrap_or(2);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a count of poles, which this instrument has two or four of"
    )]
    let span = FALL / poles as f32;
    let peak = PASSBAND + resonance * (1.0 - PASSBAND);
    // The rise into the corner. A resonant peak with no width is a spike a dot
    // wide, which on a screen this size is a dot.
    let shoulder = 0.08;
    screen.curve(band, Ink::Solid, |x| {
        if x <= corner {
            let into = corner - shoulder;
            if shoulder > 0.0 && x > into {
                PASSBAND + (peak - PASSBAND) * (x - into) / shoulder
            } else {
                PASSBAND
            }
        } else {
            (peak - (x - corner) * (PASSBAND / span)).max(0.0)
        }
    });
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
    let span = FALL / 2.0;
    screen.curve(band, Ink::Solid, |x| {
        if x >= corner {
            PASSBAND
        } else {
            (PASSBAND - (corner - x) * (PASSBAND / span)).max(0.0)
        }
    });
    if is_on(patch, ParamId::VcfBassBoost) == Some(true) {
        // Under the passband, which is the one part of this drawing that is
        // always flat and always empty.
        screen.write(
            band.width - Screen::width_of("BOOST", Size::Small) - 1,
            band.height - Screen::height_of(Size::Small),
            "BOOST",
            Size::Small,
        );
    }
}

/// Draws the shapes the oscillators are making.
///
/// Two lanes, because the instrument has two oscillators and they are not
/// drawn added together: how the saw and the pulse of `DCO 1` sum, and at what
/// weight, is not something the manual gives, and a single waveform drawn out
/// of a mix law this window invented would be the most confident wrong picture
/// on the panel. So each shape is drawn as the shape it is, and what is on the
/// screen is what is switched on.
fn oscillators(screen: &mut Screen, patch: &Patch) {
    let lanes = screen.all().lanes(2, 2);
    let (Some(upper), Some(lower)) = (lanes.first().copied(), lanes.get(1).copied()) else {
        return;
    };
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
    // `DCO 2` is a shape at a level, which is the one oscillator on this
    // instrument whose loudness is a parameter, so the drawing is as tall as
    // the level is. Noise is the same reading, scattered rather than shaped.
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

/// How much glass there is between two panes of one display.
const GUTTER: i32 = 3;

/// Draws all three envelopes side by side, each in a pane with its name on it.
///
/// The one thing on this panel the instrument itself cannot show. A `DeepMind`
/// has three envelopes, one display and three buttons that choose which of them
/// the four faders under it are addressing, so a player comparing the filter's
/// attack with the amplifier's is comparing one of them with their memory of
/// the other.
///
/// They are laid out rather than stacked. Three curves sharing one band and
/// told apart by a dash pattern is a drawing that has all three envelopes in it
/// and shows you none of them: on a grid of dots two lines crossing are the
/// same dots, and the question anybody asks this display — which of these
/// decays first — is the question an overlay answers worst. A pane each, named,
/// is the whole of what the room is for.
///
/// Where the room is not there, the overlay is still the right drawing, and
/// [`stacked`] is it. A plate whose display is too narrow to divide gets the
/// three curves in one band rather than nothing.
fn envelopes_on(screen: &mut Screen, patch: &Patch) {
    let groups = envelopes();
    // One pane per envelope the library has, and not three: an instrument with
    // a fourth gets a fourth pane with nothing here to edit.
    let panes = screen
        .all()
        .columns(i32::try_from(groups.len()).unwrap_or_default(), GUTTER);
    if panes.is_empty() {
        stacked(screen, patch);
        return;
    }
    let named = Screen::height_of(Size::Small);
    for (pane, group) in panes.into_iter().zip(groups) {
        screen.write(pane.x, pane.y, button(group), Size::Small);
        let under = Band::new(
            pane.x,
            pane.y + named + 1,
            pane.width,
            pane.height - named - 1,
        );
        // The floor, so that a pane whose envelope has not been read is still a
        // pane rather than a gap, and so that three curves at three heights are
        // read against three baselines rather than against each other.
        screen.across(under.x, under.row(0.0), under.width, Ink::Dotted);
        let Some(corners) = envelope::corners(patch, group) else {
            continue;
        };
        // The amplifier's is filled, because it is the one you hear. The other
        // two are the line, because a filter's envelope is a shape and not an
        // amount of anything.
        if group == Group::VcaEnvelope {
            screen.under(under, |x| corners.height_at(x));
        }
        screen.curve(under, Ink::Solid, |x| corners.height_at(x));
    }
}

/// The three envelopes in one band, for a display with no room to divide.
fn stacked(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    for (index, group) in envelopes().into_iter().enumerate() {
        let Some(corners) = envelope::corners(patch, group) else {
            continue;
        };
        let ink = match index {
            0 => Ink::Solid,
            1 => Ink::Dashed,
            _ => Ink::Dotted,
        };
        screen.curve(band, ink, |x| corners.height_at(x));
    }
}

/// What the instrument prints on the button that chooses this envelope.
///
/// The first word of the library's own name for the group, which is `VCA`,
/// `VCF` and `Mod` — the three legends the hardware prints under its own three
/// buttons. Taken off the name rather than written down beside it, so a library
/// that renames a group renames the pane.
fn button(group: Group) -> &'static str {
    group.name().split_whitespace().next().unwrap_or_default()
}

/// Draws the amplifier's envelope, under the level it is played at.
///
/// The level is the ceiling rather than a mark beside it: an envelope drawn to
/// the top of a screen on a patch whose `VCA Level` is half way up is an
/// envelope drawn at a loudness the instrument is not playing.
fn amplifier(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let (Some(corners), Some(level)) = (
        envelope::corners(patch, Group::VcaEnvelope),
        travel(patch, ParamId::VcaLevel),
    ) else {
        return;
    };
    screen.under(band, |x| corners.height_at(x) * level);
    screen.curve(band, Ink::Solid, |x| corners.height_at(x) * level);
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
        screen.write(0, 0, mode, Size::Small);
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
/// How many of them fit across the screen is the rate through its own travel,
/// and how much of each is open is the gate time through its own. An
/// arpeggiator that is switched off is a line: a screen still drawing gates
/// for a sound that is not arpeggiating would be the panel's one lie.
fn arpeggiator(screen: &mut Screen, patch: &Patch) {
    let band = screen.all();
    let (Some(rate), Some(gate)) = (
        travel(patch, ParamId::ArpRateTempo),
        travel(patch, ParamId::ArpGateTime),
    ) else {
        return;
    };
    if is_on(patch, ParamId::ArpOnOff) == Some(false) {
        screen.across(band.x, band.y + band.height - 1, band.width, Ink::Solid);
        return;
    }
    let gates = cycles(rate, 8.0);
    let open = gate.clamp(0.05, 0.95);
    screen.curve(band, Ink::Solid, |x| {
        if (x * gates).fract() < open { 1.0 } else { 0.0 }
    });
}

/// Draws an LFO's shape, at its own rate.
///
/// The shape is whichever one the value table names, drawn as that shape; a
/// name this drawing has no shape for is written out instead, because a
/// display that drew a sine for a shape it did not recognise would be inventing
/// the sound. The rate is how many cycles fit across the screen and is not a
/// speed: what a byte of `Rate` is in hertz is not published, and what this
/// says is that more of it is more cycles.
///
/// The delay is not drawn at all. `Delay / Fade` is one parameter doing two
/// things and the manual does not say where the byte stops doing one and starts
/// doing the other, so the fader says what it is and the screen says nothing.
fn lfo(screen: &mut Screen, patch: &Patch, firmware: Version, shape: ParamId, rate: ParamId) {
    let band = screen.all();
    let (Some(name), Some(rate)) = (named(patch, shape, firmware), travel(patch, rate)) else {
        return;
    };
    let over = cycles(rate, 5.0);
    let Some(drawn) = wave(name) else {
        screen.write(
            0,
            band.height / 2 - Screen::height_of(Size::Small) / 2,
            name,
            Size::Small,
        );
        return;
    };
    screen.curve(band, Ink::Solid, |x| 0.5 + drawn(x * over) / 2.0);
}

/// Returns the shape a named one is drawn as, over a run of cycles.
///
/// Given how many cycles have gone by, and answering between minus one and one.
/// Matched on the display's own name for the value, the way the envelopes are
/// found by what the library calls their parameters: a firmware that renamed
/// one draws its name instead of the wrong picture.
fn wave(name: &str) -> Option<fn(f32) -> f32> {
    Some(match name {
        "Sine" => |cycle: f32| (cycle * core::f32::consts::TAU).sin(),
        "Triangle" => |cycle: f32| {
            let through = cycle.fract();
            if through < 0.5 {
                through * 4.0 - 1.0
            } else {
                3.0 - through * 4.0
            }
        },
        "Square" => |cycle: f32| if cycle.fract() < 0.5 { 1.0 } else { -1.0 },
        "Ramp Up" => |cycle: f32| cycle.fract() * 2.0 - 1.0,
        "Ramp Down" => |cycle: f32| 1.0 - cycle.fract() * 2.0,
        // One number a cycle, which is what a sample and hold is: held to the
        // end of the cycle, or walked to the next one.
        "Sample & Hold" => |cycle: f32| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a cycle count across a screen, which is single figures"
            )]
            let step = cycle as i32;
            scattered(step) * 2.0 - 1.0
        },
        "Sample & Glide" => |cycle: f32| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a cycle count across a screen, which is single figures"
            )]
            let step = cycle as i32;
            let through = cycle.fract();
            let from = scattered(step) * 2.0 - 1.0;
            let to = scattered(step + 1) * 2.0 - 1.0;
            from + (to - from) * through
        },
        _ => return None,
    })
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
    use super::{Scene, envelopes, of, travel, wave};
    use crate::home::ROWS;
    use crate::{Confidence, Patch};
    use deepmind_midi::ids::ProtocolVersion;
    use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, Kind, ParamId};
    use deepmind_midi::program::Program;

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
        for plate in ROWS.iter().flat_map(|row| row.iter()) {
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
    fn an_lfo_is_found_by_what_the_library_calls_its_parameters() {
        // Two LFOs, one drawing. Found the way the envelopes are, so that a
        // third one a later library adds draws itself with nothing here to
        // edit.
        for (shape, rate) in [
            (ParamId::Lfo1Shape, ParamId::Lfo1Rate),
            (ParamId::Lfo2Shape, ParamId::Lfo2Rate),
        ] {
            assert_eq!(of([shape]), Some(Scene::Lfo { shape, rate }));
        }
    }

    #[test]
    fn the_envelope_screen_reads_all_three_of_them() {
        let parameters = Scene::Envelopes.parameters();

        assert_eq!(envelopes().len(), 3);
        assert_eq!(parameters.len(), 12, "four values from each of three");
        for group in [Group::VcaEnvelope, Group::VcfEnvelope, Group::ModEnvelope] {
            assert!(
                parameters
                    .iter()
                    .any(|parameter| parameter.group() == group),
                "{group} is not on the screen that draws all three"
            );
        }
    }

    #[test]
    fn the_three_envelopes_are_three_panes_and_not_three_curves_in_one() {
        // The point of the drawing. Three envelopes sharing a band are three
        // sets of dots in the same place, and the question this display is for
        // — which of these decays first — is the one an overlay answers worst.
        let mut screen = super::super::Screen::new(90, 20);
        super::envelopes_on(&mut screen, &read());

        // Each pane is named after the button the hardware chooses it with.
        for group in super::envelopes() {
            let named = super::button(group);
            assert!(!named.is_empty(), "{group:?} names no button");
            assert!(!named.contains(' '), "{named} is a name and not a legend");
        }
        assert!(!screen.is_blank(), "three panes and nothing drawn");
    }

    #[test]
    fn a_display_too_narrow_to_divide_still_draws_all_three() {
        // The fallback is the old drawing, not an empty pane: a plate whose
        // display cannot hold three panes is still a plate about envelopes.
        let mut screen = super::super::Screen::new(12, 12);
        super::envelopes_on(&mut screen, &read());

        assert!(!screen.is_blank(), "a narrow display drew nothing");
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
    fn the_seven_shapes_the_table_names_are_the_seven_that_are_drawn() {
        // Matched on the display's own words, so this is the test that says the
        // words have not moved. A shape with no drawing is written out instead,
        // which is why nothing here can fail quietly.
        let Kind::Enumerated(table) = ParamId::Lfo1Shape.kind() else {
            // Unreachable: a shape is one of a named set, and a library that
            // made it a sweep would have taken the names with it.
            return;
        };

        for entry in table.table_for(DEFAULT_FIRMWARE).entries {
            assert!(
                wave(entry.name).is_some(),
                "{} is a shape the display does not draw",
                entry.name
            );
        }
        assert!(wave("Parabola").is_none());
    }

    #[test]
    fn a_shape_stays_between_the_two_ends_of_its_swing() {
        for name in [
            "Sine",
            "Triangle",
            "Square",
            "Ramp Up",
            "Ramp Down",
            "Sample & Glide",
        ] {
            let shape = wave(name).expect("a shape the table names");
            for step in 0..200_i16 {
                let cycle = f32::from(step) / 40.0;
                let drawn = shape(cycle);
                assert!(
                    (-1.0..=1.0).contains(&drawn),
                    "{name} reaches {drawn} at {cycle}"
                );
            }
        }
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

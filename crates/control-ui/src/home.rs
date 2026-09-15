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
//! # The one thing this repository transcribes
//!
//! Which parameters have a physical control, and which of the panel's groups
//! each one sits in, is a fact about the hardware that the library does not
//! publish: its parameter table says what exists, its controller table says
//! what has a CC, and neither says what has a fader. So [`ROWS`] is a table
//! here, and it is the only one in this repository.
//!
//! It is kept as honest as a transcription can be. Every entry is a [`ParamId`]
//! rather than a name, so a parameter the library renames is a compilation
//! error rather than a wrong legend; a test asserts that each appears once and
//! that its group is the one the plate says. What a control *is* still comes
//! from the library — a sweep gets a fader, two states get a lamp, a named set
//! gets its names — so this table says where a control is and never what it is.
//! [deepmind-midi#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)
//! is the ask that would delete it, and `docs/waiting.md` is where that is
//! remembered.
//!
//! # The arrangement is the instrument's, the livery is this window's
//!
//! The hardware's buttons are white, yellow and cyan. This panel's are not,
//! because in this window a colour already means something: copper is a claim
//! this application is making and green is the synthesizer's own account of
//! itself, and a yellow `EDIT` beside them would be a fourth meaning for a
//! reader to learn. So the layout is the instrument's and the materials are the
//! ones every other panel here is drawn in.
//!
//! The row of twelve lamps over the hardware's `POLY` section is missing for a
//! different reason: it says how many voices are sounding, and nothing on a
//! MIDI port says that. A lamp that cannot be lit honestly is not drawn.

use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{Background, Border, Font, Length, Theme, text::Renderer as TextRenderer};
use iced_widget::{Space, button, column, container, row, text};

use crate::lcd::{self, Screen};
use crate::panel::{Message, Room, readout};
use crate::scene;
use crate::style::{materials, printed, reading};
use crate::{Element, Patch, fader};

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

/// How much panel there is above and below the display in the panel's own hole.
const GAP: f32 = 2.0;

/// How much panel there is around a plate's contents.
///
/// Named rather than typed into the one call that uses it, because the display
/// over a plate's faders has to know it: a plate's width is the room it takes
/// on the panel, and what a display fits into is the room inside that.
const PAD: f32 = 6.0;

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

/// One group of controls, as the panel prints it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Plate {
    /// What the panel prints across the top of the group.
    name: &'static str,
    /// The section this plate's `EDIT` opens, which is where the rest of its
    /// parameters are.
    opens: Group,
    /// The controls the hardware puts a fader under, in the order it puts them.
    faders: &'static [Control],
    /// The named set drawn as a strip of legends beside the faders, where the
    /// panel has one: the LFO's shape is a column of lamps on the instrument.
    lamps: Option<Control>,
    /// The controls the hardware puts in the row of buttons under the faders.
    switches: &'static [Control],
    /// The sections this plate's other buttons open.
    ///
    /// The hardware's `VCA`, `VCF` and `MOD` under the envelopes choose which
    /// envelope its four faders address; here they are the way into each
    /// envelope's own panel, which is where those four faders and the other
    /// five parameters both are.
    ways: &'static [(&'static str, Group)],
}

impl Plate {
    /// What the panel prints across the top of the group.
    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    /// Every parameter this plate puts a control under.
    pub(crate) fn parameters(self) -> impl Iterator<Item = ParamId> {
        self.controls().map(|control| control.parameter)
    }

    /// How wide the plate stands.
    ///
    /// Worked out rather than given: a plate that took the width it was offered
    /// would be a row of one, because a row of plates is only a row while each
    /// of them is as wide as what it holds. The wider of its two rows wins —
    /// `HPF` is one fader over a button and a way in, and the buttons are what
    /// decide it.
    fn width(self) -> f32 {
        let lanes = count(self.faders.len()) * (LANE + 2.0)
            + if self.lamps.is_some() {
                LAMPS + 2.0
            } else {
                0.0
            };
        let buttons =
            count(self.switches.len()) * (SWITCH + 4.0) + count(self.ways.len() + 1) * (WAY + 4.0);
        lanes.max(buttons).max(lcd::room(NARROWEST) + PAD * 2.0)
    }

    /// Every control this plate draws.
    pub(crate) fn controls(self) -> impl Iterator<Item = Control> {
        self.faders
            .iter()
            .copied()
            .chain(self.lamps)
            .chain(self.switches.iter().copied())
    }
}

/// The panel, in the two rows the instrument prints it in.
///
/// Row one is what a player reaches for between notes — the arpeggiator, the
/// two LFOs, the screen, and the voicing. Row two is the voice itself, left to
/// right in the order the signal takes: oscillators, filter, amplifier, and the
/// envelopes that move them.
pub(crate) const ROWS: [&[Plate]; 2] = [
    &[
        Plate {
            name: "ARP / SEQ",
            opens: Group::Arpeggiator,
            faders: &[
                Control {
                    legend: "RATE",
                    parameter: ParamId::ArpRateTempo,
                },
                Control {
                    legend: "GATE TIME",
                    parameter: ParamId::ArpGateTime,
                },
            ],
            lamps: None,
            switches: &[
                Control {
                    legend: "ON/OFF",
                    parameter: ParamId::ArpOnOff,
                },
                Control {
                    legend: "HOLD",
                    parameter: ParamId::ArpHold,
                },
            ],
            ways: &[],
        },
        Plate {
            name: "LFO 1",
            opens: Group::Lfo1,
            faders: &[
                Control {
                    legend: "RATE",
                    parameter: ParamId::Lfo1Rate,
                },
                Control {
                    legend: "DELAY TIME",
                    parameter: ParamId::Lfo1DelayFade,
                },
            ],
            lamps: Some(Control {
                legend: "SHAPE",
                parameter: ParamId::Lfo1Shape,
            }),
            switches: &[],
            ways: &[],
        },
        Plate {
            name: "LFO 2",
            opens: Group::Lfo2,
            faders: &[
                Control {
                    legend: "RATE",
                    parameter: ParamId::Lfo2Rate,
                },
                Control {
                    legend: "DELAY TIME",
                    parameter: ParamId::Lfo2DelayFade,
                },
            ],
            lamps: Some(Control {
                legend: "SHAPE",
                parameter: ParamId::Lfo2Shape,
            }),
            switches: &[],
            ways: &[],
        },
        Plate {
            name: "POLY",
            opens: Group::Voicing,
            // One fader, where the hardware has two. The other is its DATA
            // ENTRY, which edits whatever the display is showing — a window has
            // the value under the pointer instead, and does not need one.
            faders: &[Control {
                legend: "UNISON DETUNE",
                parameter: ParamId::UnisonDetune,
            }],
            lamps: None,
            switches: &[],
            ways: &[("MOD", Group::ModMatrix), ("FX", Group::Effects)],
        },
    ],
    &[
        Plate {
            name: "DCO 1 & 2",
            opens: Group::Oscillators,
            faders: &[
                Control {
                    legend: "PITCH MOD",
                    parameter: ParamId::Osc1PitchModDepth,
                },
                Control {
                    legend: "PWM",
                    parameter: ParamId::Osc1PwmDepth,
                },
                Control {
                    legend: "PITCH MOD",
                    parameter: ParamId::Osc2PitchModDepth,
                },
                Control {
                    legend: "TONE MOD",
                    parameter: ParamId::Osc2ToneModDepth,
                },
                Control {
                    legend: "PITCH",
                    parameter: ParamId::Osc2Pitch,
                },
                Control {
                    legend: "LEVEL",
                    parameter: ParamId::Osc2Level,
                },
                Control {
                    legend: "NOISE",
                    parameter: ParamId::NoiseLevel,
                },
            ],
            lamps: None,
            switches: &[Control {
                legend: "SYNC",
                parameter: ParamId::OscSyncEnable,
            }],
            ways: &[],
        },
        Plate {
            name: "VCF",
            opens: Group::Vcf,
            faders: &[
                Control {
                    legend: "FREQ",
                    parameter: ParamId::VcfFrequency,
                },
                Control {
                    legend: "RES",
                    parameter: ParamId::VcfResonance,
                },
                Control {
                    legend: "ENV",
                    parameter: ParamId::VcfEnvelopeDepth,
                },
                Control {
                    legend: "LFO",
                    parameter: ParamId::VcfLfoDepth,
                },
                Control {
                    legend: "KYBD",
                    parameter: ParamId::VcfKeyboardTracking,
                },
            ],
            lamps: None,
            switches: &[Control {
                legend: "POLES",
                parameter: ParamId::Vcf2PoleMode,
            }],
            ways: &[],
        },
        Plate {
            name: "VCA",
            opens: Group::Vca,
            faders: &[Control {
                legend: "LEVEL",
                parameter: ParamId::VcaLevel,
            }],
            lamps: None,
            switches: &[],
            ways: &[],
        },
        Plate {
            name: "HPF",
            opens: Group::Vcf,
            faders: &[Control {
                legend: "FREQ",
                parameter: ParamId::VcfHighPassFrequency,
            }],
            lamps: None,
            switches: &[Control {
                legend: "BOOST",
                parameter: ParamId::VcfBassBoost,
            }],
            ways: &[],
        },
        Plate {
            name: "ENVELOPES",
            opens: Group::VcaEnvelope,
            faders: &[
                Control {
                    legend: "A",
                    parameter: ParamId::VcaEnvelopeAttackTime,
                },
                Control {
                    legend: "D",
                    parameter: ParamId::VcaEnvelopeDecayTime,
                },
                Control {
                    legend: "S",
                    parameter: ParamId::VcaEnvelopeSustainLevel,
                },
                Control {
                    legend: "R",
                    parameter: ParamId::VcaEnvelopeReleaseTime,
                },
            ],
            lamps: None,
            switches: &[],
            ways: &[
                ("VCA", Group::VcaEnvelope),
                ("VCF", Group::VcfEnvelope),
                ("MOD", Group::ModEnvelope),
            ],
        },
    ],
];

/// Draws the front panel, with `screen` where the instrument's display sits.
///
/// The screen is the caller's: what it says is the application's business and
/// not the synthesizer's — which sound is on it, what backs that, and what the
/// last thing to happen was — and a plugin has different answers than a desktop
/// window does.
#[must_use]
pub fn panel<'a, Renderer>(
    patch: &Patch,
    firmware: Version,
    screen: Element<'a, Renderer>,
) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mut rows = column![].spacing(10);
    let mut screen = Some(screen);
    for (index, plates) in ROWS.iter().enumerate() {
        let mut across = row![].spacing(8).align_y(Vertical::Top);
        for plate in plates.iter().copied() {
            // The screen sits in the top row where the instrument puts it:
            // between what a player reaches for and what the voicing does.
            if index == 0
                && plate.name == "POLY"
                && let Some(screen) = screen.take()
            {
                across = across.push(display(screen));
            }
            across = across.push(group(patch, plate, firmware));
        }
        rows = rows.push(across.wrap());
    }
    // A screen with nowhere to go still goes somewhere: a row renamed out from
    // under this loses the instrument's arrangement, not its display.
    if let Some(screen) = screen.take() {
        rows = rows.push(display(screen));
    }
    rows.into()
}

/// A blank screen the size of the hole the panel leaves for a display.
///
/// What an application writes on and hands back to [`panel`]. The size is the
/// panel's business rather than the application's — it is the hole in the
/// instrument's own front — and how many dots are in it is a division rather
/// than a number, because every display in this window is at the same pitch.
#[must_use]
pub fn screen() -> Screen {
    Screen::new(lcd::fits(SCREEN), lcd::fits(PLATE - GAP * 2.0))
}

/// Returns every parameter the panel puts a control under.
///
/// The way for a test to ask what the table claims without drawing it.
#[must_use]
pub fn panelled() -> Vec<ParamId> {
    ROWS.iter()
        .flat_map(|row| row.iter())
        .flat_map(|plate| plate.controls().map(|control| control.parameter))
        .collect()
}

/// Draws one group of the panel: its name, its controls, and its way in.
fn group<'a, Renderer>(patch: &Patch, plate: Plate, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let mut controls = row![].spacing(2).align_y(Vertical::Top);
    for control in plate.faders.iter().copied() {
        controls = controls.push(lane(patch, control, firmware));
    }
    if let Some(control) = plate.lamps {
        controls = controls.push(strip(patch, control, firmware));
    }
    let mut buttons = row![].spacing(4).align_y(Vertical::Bottom);
    for control in plate.switches.iter().copied() {
        buttons = buttons.push(switch(patch, control, firmware));
    }
    for (label, group) in plate.ways.iter().copied() {
        buttons = buttons.push(way(label, group));
    }
    // Every plate has a way in, and it is the press the hardware calls EDIT.
    buttons = buttons.push(way("EDIT", plate.opens));
    container(
        column![
            heading(plate.name()),
            glass(patch, plate, firmware),
            controls,
            buttons
        ]
        .spacing(6)
        .align_x(Horizontal::Center),
    )
    .width(Length::Fixed(plate.width()))
    .padding(PAD)
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
fn glass<'a, Renderer>(patch: &Patch, plate: Plate, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let columns = lcd::fits(plate.width() - PAD * 2.0);
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
fn heading<'a, Renderer>(name: &'a str) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(name)
            .size(11)
            .font(printed())
            .style(|theme: &Theme| text::Style {
                color: Some(materials(theme).metal_high),
            }),
    )
    .width(Length::Fill)
    .padding([2, 6])
    .align_x(Horizontal::Center)
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
    .into()
}

/// Draws one control of the panel: what is printed over it, it, and its value.
///
/// The legend goes above, which is where the instrument prints it and not where
/// a rack's slot does: the hardware has a screen to put readings on and no room
/// under a fader, and a panel that printed its names underneath would be a
/// different object.
fn lane<'a, Renderer>(patch: &Patch, control: Control, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    let claim = patch.claim(parameter);
    let value = patch.value(parameter);
    column![
        legend(control.legend),
        crate::panel::control(parameter, value, claim, firmware, Room::lane(LANE, TRAVEL)),
        readout(parameter, value, claim, firmware),
    ]
    .spacing(4)
    .width(Length::Fixed(LANE))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws a named set as the strip of lit legends the instrument has.
///
/// The LFO's shape is a column of lamps beside its two faders on the hardware,
/// one of them lit, and seven of them is more than a rack's slot has room to
/// light. The panel has the room, so it says so.
fn strip<'a, Renderer>(patch: &Patch, control: Control, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    column![
        legend(control.legend),
        crate::panel::control(
            parameter,
            patch.value(parameter),
            patch.claim(parameter),
            firmware,
            Room::lamps(LAMPS, TRAVEL),
        ),
    ]
    .spacing(4)
    .width(Length::Fixed(LAMPS))
    .align_x(Horizontal::Center)
    .into()
}

/// Draws one of the buttons under a plate's faders.
///
/// Whatever the library says the parameter is, in the room the button row has:
/// two states are a lamp, and a named set of two is the pair of legends the
/// panel prints beside its button.
fn switch<'a, Renderer>(patch: &Patch, control: Control, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    let parameter = control.parameter;
    column![
        legend(control.legend),
        crate::panel::control(
            parameter,
            patch.value(parameter),
            patch.claim(parameter),
            firmware,
            Room::listed(SWITCH),
        ),
    ]
    .spacing(2)
    .width(Length::Fixed(SWITCH))
    .align_x(Horizontal::Center)
    .into()
}

/// What the panel prints over a control.
fn legend<'a, Renderer>(printed_as: &'a str) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        text(printed_as)
            .size(9)
            .font(reading())
            .center()
            .style(move |theme: &Theme| text::Style {
                color: Some(materials(theme).metal_low),
            }),
    )
    .height(Length::Fixed(LEGEND))
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// The way into a section, which is the button the hardware calls `EDIT`.
fn way<'a, Renderer>(label: &'a str, group: Group) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(
        button(text(label).size(9).font(reading()))
            .padding([3, 7])
            .style(crate::style::chrome)
            .on_press(Message::Show(group)),
    )
    .height(Length::Fixed(LEGEND + fader::WIDTH))
    .align_y(Vertical::Bottom)
    .into()
}

/// The screen, cut into the panel where the instrument's own display sits.
///
/// The glass is the display's own — it draws the recess it is cut into, the
/// way every other display in this window does — so what is left here is the
/// hole it stands in, as tall as the plates either side of it.
fn display<'a, Renderer>(screen: Element<'a, Renderer>) -> Element<'a, Renderer>
where
    Renderer: TextRenderer<Font = Font> + 'a,
{
    container(screen)
        .width(Length::Fixed(SCREEN))
        .height(Length::Fixed(PLATE))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([2, 0])
        .into()
}

#[cfg(test)]
mod tests {
    use deepmind_midi::param::{Group, ParamId};

    use super::{NARROWEST, PAD, ROWS, SCREEN, lcd, panelled};

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
        for plate in ROWS.iter().flat_map(|row| row.iter()) {
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
        for plate in ROWS.iter().flat_map(|row| row.iter()) {
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
        for plate in ROWS.iter().flat_map(|row| row.iter()) {
            assert!(
                Group::ALL.contains(&plate.opens),
                "{} opens nothing",
                plate.name
            );
            for (label, group) in plate.ways {
                assert!(Group::ALL.contains(group), "{label} opens nothing");
                assert!(!label.is_empty());
            }
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
    fn a_row_of_plates_fits_the_window_it_opens_in() {
        // Two rows and a screen between them is the arrangement, and a row that
        // does not fit wraps: a front panel in four rows is a list of plates.
        // The plates are as wide as what they hold, so this is what says that
        // giving every one of them a display did not cost the panel its shape.
        const ROOM: f32 = 1120.0 - 16.0 * 2.0;

        for (index, plates) in ROWS.iter().enumerate() {
            let mut across: f32 = plates.iter().map(|plate| plate.width() + 8.0).sum();
            if index == 0 {
                across += SCREEN + 8.0;
            }
            assert!(across <= ROOM, "row {index} is {across} points across");
        }
    }

    #[test]
    fn a_display_fits_inside_the_plate_it_is_drawn_on() {
        // A plate's width is the room it takes on the panel and a display fits
        // into the room inside that, which is two paddings narrower. A display
        // drawn from the outer width is a display a few dots wider than the
        // plate holding it.
        for plate in ROWS.iter().flat_map(|row| row.iter()) {
            let inside = plate.width() - PAD * 2.0;
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
    fn the_voice_the_signal_takes_is_the_second_row() {
        let second: Vec<&str> = ROWS
            .get(1)
            .map(|plates| plates.iter().map(|plate| plate.name).collect())
            .unwrap_or_default();

        assert_eq!(
            second,
            vec!["DCO 1 & 2", "VCF", "VCA", "HPF", "ENVELOPES"],
            "the lower row is the signal path, left to right"
        );
    }
}

//! The plugin itself: a MIDI output with an editor on it.

use std::sync::{Arc, Mutex};

use deepmind_midi::ids::ProtocolVersion;
use deepmind_midi::program::Program;
use nice_plug::prelude::*;

use crate::editor::{self, Shared};
use crate::outbox::Outbox;
use crate::wire;

/// How long the wire is given for one edit, in seconds.
///
/// One NRPN is twelve bytes, which is a little under four milliseconds of a MIDI
/// cable. This is a little slower than that, for the reason the desktop build's
/// is: a synthesizer that drops edits is worse than a fader that lags.
const GAP: f32 = 0.005;

/// What the plugin keeps beside its host's own parameters.
#[derive(Params, Debug)]
pub struct ControlParams {
    /// The channel the edits go out on, which is the synthesizer's own.
    ///
    /// The manual says a unit's device ID is its global MIDI channel, so this is
    /// the one number that has to agree with the instrument.
    #[id = "channel"]
    pub channel: IntParam,

    /// The sound on the window, as the bytes a program is stored as.
    ///
    /// The program and not a pointer to it: a plugin that stored "bank C,
    /// program 41" would bring back a different sound on a unit whose bank had
    /// been rewritten since. Empty until somebody has touched something, which
    /// is what keeps a plugin nobody has used from sending a program made of
    /// minimums to an instrument that was holding a sound.
    #[persist = "program"]
    pub program: Arc<Mutex<Vec<u8>>>,
}

impl Default for ControlParams {
    fn default() -> Self {
        Self {
            channel: IntParam::new("Channel", 1, IntRange::Linear { min: 1, max: 16 }),
            program: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// The `DeepMind` editor, as a plugin.
///
/// Advanced mode in the plan's words: the desktop editing surface in a plugin
/// window, whose MIDI output is the port. What it sends is NRPN, the same edits
/// the desktop build sends, and it sends nothing else: no bank, no librarian, and
/// nothing it was not asked to by a hand or by a session being brought back.
#[derive(Debug)]
pub struct DeepMindControl {
    params: Arc<ControlParams>,
    outbox: Arc<Mutex<Outbox>>,
    sample_rate: f32,
    /// How far into the next block the wire is still busy, in samples.
    busy: f32,
}

impl Default for DeepMindControl {
    fn default() -> Self {
        Self {
            params: Arc::new(ControlParams::default()),
            outbox: Arc::new(Mutex::new(Outbox::new())),
            sample_rate: 44_100.0,
            busy: 0.0,
        }
    }
}

impl DeepMindControl {
    /// The program the session was saved with, where it was saved with one.
    fn remembered(&self) -> Option<Program> {
        let bytes = self.params.program.lock().ok()?;
        Program::from_bytes(ProtocolVersion::V7, &bytes).ok()
    }
}

impl Plugin for DeepMindControl {
    const NAME: &'static str = "DeepMind Control";
    const VENDOR: &'static str = "MysteriousWolf";
    const URL: &'static str = "https://github.com/MysteriousWolf/deepmind-control";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // No audio: this is a controller. A layout with neither input nor output is
    // how a CLAP says so.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout::const_default()];

    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    type SysExMessage = ();
    type BackgroundTask = ();
    type Editor = nice_plug_iced::IcedEditor;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        editor::create(Shared {
            params: Arc::clone(&self.params),
            outbox: Arc::clone(&self.outbox),
        })
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.sample_rate = config.sample_rate;
        // A session coming back says the sound it had, so the instrument is
        // told. A plugin with nothing remembered says nothing at all.
        if let (Some(program), Ok(mut outbox)) = (self.remembered(), self.outbox.lock()) {
            outbox.set_all(&program);
        }
        true
    }

    fn reset(&mut self) {
        self.busy = 0.0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a block is a few thousand samples, far inside what an f32 holds"
        )]
        let frames = buffer.samples() as f32;
        let spacing = GAP * self.sample_rate;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the value is clamped to a channel number first"
        )]
        let channel = (self.params.channel.value().clamp(1, 16) - 1) as u8;

        // Never wait for the window: a block with nothing to say is better than
        // a block that did not finish. Whatever is waiting goes in the next one.
        let mut at = self.busy;
        if let Ok(mut outbox) = self.outbox.try_lock() {
            while at < frames {
                let Some((parameter, value)) = outbox.take() else {
                    break;
                };
                let Some(controls) = wire::controls(parameter, value) else {
                    continue;
                };
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "inside the block, which is what the loop condition says"
                )]
                let timing = at.max(0.0) as u32;
                // All four or none: an NRPN with its value missing is a number
                // selected and nothing said about it. A host with no room puts
                // the edit back to wait for the next block, and the retry sends
                // the whole twelve bytes again, which an instrument reads as one.
                let sent = controls.into_iter().all(|(cc, value)| {
                    context
                        .try_send_event(NoteEvent::MidiCC {
                            timing,
                            channel,
                            cc,
                            value,
                        })
                        .is_ok()
                });
                if !sent {
                    outbox.set(parameter, value);
                    break;
                }
                at += spacing;
            }
        }
        self.busy = (at - frames).max(0.0);

        // Edits arrive whenever a hand does, and a host that put the plugin to
        // sleep for having no input would never run the block that sends them.
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for DeepMindControl {
    const CLAP_ID: &'static str = "io.github.mysteriouswolf.deepmind-control";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Edit a Behringer DeepMind from the DAW, and have the session bring its sound back");
    const CLAP_MANUAL_URL: Option<&'static str> =
        Some("https://github.com/MysteriousWolf/deepmind-control");
    const CLAP_SUPPORT_URL: Option<&'static str> =
        Some("https://github.com/MysteriousWolf/deepmind-control/issues");
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

//! Hearing a demo: one output, one clip at a time.
//!
//! A patch in the shared library may carry up to four short recordings of
//! itself — the sound alone, twenty seconds, one per way of playing it — and
//! this is what plays one. It is the only part of this application that makes a
//! noise of its own: everything else makes a noise by asking the synthesizer
//! to.
//!
//! # Why an output can be opened at all
//!
//! Sound costs a dependency, and this one costs less than it looks. `rodio`
//! brings `cpal` for the device and `symphonia` for the decoding, and on Linux
//! `cpal` talks to ALSA — which this repository already needs, because `midir`
//! talks to ALSA too and CI has installed `libasound2-dev` since the first
//! commit that opened a port. So a Linux build needs nothing it did not already
//! need, and macOS and Windows need nothing at all.
//!
//! # It is opened late and never in a test
//!
//! Opening an output device takes a moment and claims a handle, and most of
//! what this window does never makes a sound. So there is no output until the
//! first press of a play button, and a machine with no sound card gets a
//! sentence in the status line rather than a window that would not start.
//!
//! # One clip at a time, and a press stops it
//!
//! Two demos playing at once is two sounds nobody asked to hear together.
//! Starting one stops whatever was going, and pressing the one that is playing
//! stops it — which is what a play button on a list has always done.
//!
//! The decode happens on `rodio`'s own thread. Nothing here blocks a frame:
//! the bytes are read off the disk (a demo is at most 350 KB) and handed over,
//! and the window carries on drawing.

use std::path::Path;

use rodio::stream::{DeviceSinkBuilder, MixerDeviceSink};
use rodio::{Player, stream};

/// The output, and what is playing on it.
///
/// `None` throughout on a machine with no sound card, which is a machine that
/// says so once rather than one that refuses to run.
///
/// Not `Debug`: `rodio::Player` is not, and a window's own `Debug` is worth
/// less than the derive costs to work around.
#[derive(Default)]
pub struct Audio {
    /// The device, opened on the first press and kept afterwards.
    ///
    /// Held rather than dropped because dropping it closes the stream, and a
    /// stream reopened per clip is a click at the start of every one.
    out: Option<MixerDeviceSink>,
    /// What is playing, and the clip it is playing.
    playing: Option<(String, Player)>,
}

impl std::fmt::Debug for Audio {
    /// What is playing, and nothing about the device.
    ///
    /// Written out because neither `rodio` type carries a `Debug`, and the
    /// window this sits in derives one. What a reader of that wants to know is
    /// which clip is going, not the shape of a mixer.
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        into.debug_struct("Audio")
            .field("open", &self.out.is_some())
            .field("playing", &self.playing.as_ref().map(|(file, _)| file))
            .finish()
    }
}

impl Audio {
    /// Nothing open and nothing playing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Which clip is playing, while one is.
    #[must_use]
    pub fn playing(&self) -> Option<&str> {
        self.playing.as_ref().map(|(file, _)| file.as_str())
    }

    /// Forgets a clip that has run out.
    ///
    /// Called every message, beside everything else this window drains, so a
    /// play button goes back to saying *play* by itself rather than staying lit
    /// on twenty seconds that finished while nobody was looking. There is
    /// nothing to poll: a finished sink says so.
    pub fn settle(&mut self) {
        if self.playing.as_ref().is_some_and(|(_, sink)| sink.empty()) {
            self.playing = None;
        }
    }

    /// Stops whatever is playing.
    pub fn stop(&mut self) {
        if let Some((_, sink)) = self.playing.take() {
            sink.stop();
        }
    }

    /// Plays one clip, or stops it if it is the one already playing.
    ///
    /// `file` names it for [`playing`](Self::playing) and decides that
    /// question, so it has to be the same string the button was drawn from.
    ///
    /// # Errors
    ///
    /// When there is no output device, when the file will not open, or when
    /// what is in it will not decode.
    pub fn play(&mut self, file: &str, path: &Path) -> Result<(), String> {
        self.settle();
        if self.playing() == Some(file) {
            self.stop();
            return Ok(());
        }
        self.stop();
        if self.out.is_none() {
            self.out = Some(
                DeviceSinkBuilder::open_default_sink()
                    .map_err(|error| format!("no sound output on this machine: {error}"))?,
            );
        }
        let Some(out) = self.out.as_ref() else {
            return Err("no sound output on this machine".to_owned());
        };
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let sink = stream::play(out.mixer(), std::io::Cursor::new(bytes))
            .map_err(|error| format!("that recording would not play: {error}"))?;
        self.playing = Some((file.to_owned(), sink));
        Ok(())
    }
}

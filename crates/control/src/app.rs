//! What the window holds, and what happens to it.

use control_ui::{Patch, first_section};
use deepmind_host::{Command, Event, Link, PortRef, open, ports};
use deepmind_midi::device::Event as DeviceEvent;
use deepmind_midi::param::DEFAULT_FIRMWARE;
use deepmind_midi::param::Group;
use deepmind_midi::sysex::inquiry::{Identity, Version};
use deepmind_midi::wire::Channel;

/// Everything that happens to the window.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    /// Ask the machine what ports it has.
    Rescan,
    /// Choose a port, without opening it.
    Choose(PortRef),
    /// Open the chosen port and start the device thread on it.
    Connect,
    /// Put the port down.
    Disconnect,
    /// Ask who is there, for a synthesizer that was switched on late.
    Identify,
    /// Read the edit buffer: the sound as the synthesizer describes it.
    Read,
    /// Fold in whatever the device thread has said since the last message.
    Tick,
    /// Something a view asked for.
    Ui(control_ui::Message),
}

/// The window's state.
///
/// A copy of what the device thread reported, and nothing shared with it. The
/// only thing that crosses the boundary is a [`Command`] going one way and an
/// [`Event`] coming back the other.
#[derive(Debug)]
pub struct App {
    /// What the machine offers, as of the last scan.
    ports: Vec<PortRef>,
    /// Why the port list is what it is, when that needs saying.
    trouble: Option<String>,
    /// The port the picker is sitting on.
    chosen: Option<PortRef>,
    /// The open port, and the thread that owns it.
    link: Option<Link>,
    /// Who answered the inquiry, if anybody did.
    identity: Option<Identity>,
    /// The channel edits go out on, which the inquiry settles.
    channel: Option<Channel>,
    /// The sound, as far as this window knows it.
    patch: Patch,
    /// The section the bar has pressed in.
    section: Group,
    /// The last thing worth saying, in words.
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// A window that has not looked at the machine yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            trouble: None,
            chosen: None,
            link: None,
            identity: None,
            channel: None,
            patch: Patch::new(),
            section: first_section(),
            status: "Choose a port.".to_owned(),
        }
    }

    /// A window that has, which is how the application starts.
    #[must_use]
    pub fn booted() -> Self {
        let mut app = Self::new();
        app.update(Message::Rescan);
        app
    }

    /// Returns what the last scan found.
    #[must_use]
    pub fn ports(&self) -> &[PortRef] {
        &self.ports
    }

    /// Returns why the port list looks the way it does, when it needs saying.
    #[must_use]
    pub fn trouble(&self) -> Option<&str> {
        self.trouble.as_deref()
    }

    /// Returns the port the picker is sitting on.
    #[must_use]
    pub const fn chosen(&self) -> Option<&PortRef> {
        self.chosen.as_ref()
    }

    /// Returns whether a port is open.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.link.is_some()
    }

    /// Returns who answered the inquiry, if anybody has.
    #[must_use]
    pub const fn identity(&self) -> Option<&Identity> {
        self.identity.as_ref()
    }

    /// Returns the channel edits go out on, once one is settled.
    #[must_use]
    pub const fn channel(&self) -> Option<Channel> {
        self.channel
    }

    /// Returns the firmware every value table is read for.
    ///
    /// The library's default until a synthesizer says otherwise, because
    /// firmware 1.1 renumbered three tables and something has to be assumed
    /// until one has answered. The window says which one it is assuming.
    #[must_use]
    pub fn firmware(&self) -> Version {
        self.identity
            .map_or(DEFAULT_FIRMWARE, |identity| identity.firmware)
    }

    /// Returns the sound, as far as this window knows it.
    #[must_use]
    pub const fn patch(&self) -> &Patch {
        &self.patch
    }

    /// Returns the section the bar has pressed in.
    ///
    /// Which panel somebody is looking at, which is this window's business and
    /// nothing the synthesizer is told about. It survives a port being put
    /// down, because the sound went away and the person did not.
    #[must_use]
    pub const fn section(&self) -> Group {
        self.section
    }

    /// Returns the last thing worth saying.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Takes one message, and everything the device thread has said since the
    /// last one.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Rescan => self.rescan(),
            Message::Choose(port) => self.choose(port),
            Message::Connect => self.connect(),
            Message::Disconnect => self.disconnect("Port closed."),
            Message::Identify => self.ask(Command::Identify),
            Message::Read => self.ask(Command::ReadEditBuffer),
            Message::Tick => {}
            Message::Ui(control_ui::Message::Show(section)) => self.section = section,
            Message::Ui(control_ui::Message::Edit { parameter, value }) => {
                // The view moved it, so the window already shows it there. What
                // the synthesizer is told is the same thing, once: an edit it
                // already holds is a message the wire did not need.
                if self.patch.edit(parameter, value) {
                    self.ask(Command::SetParameter { parameter, value });
                }
            }
        }
        self.drain();
    }

    /// Writes down the last thing worth saying.
    fn say(&mut self, what: impl Into<String>) {
        self.status = what.into();
    }

    /// Asks the machine what it has.
    fn rescan(&mut self) {
        match ports() {
            Ok(found) => {
                self.trouble = (found.is_empty())
                    .then(|| "No port on this machine can hold a conversation.".to_owned());
                self.ports = found;
            }
            Err(error) => {
                // A machine with no MIDI backend can still be edited against the
                // simulated synthesizer, which is the whole reason it is a port
                // rather than a test fixture.
                self.trouble = Some(format!("No MIDI backend: {error}"));
                self.ports = vec![PortRef::simulator()];
            }
        }
        if !self
            .chosen
            .as_ref()
            .is_some_and(|port| self.ports.contains(port))
        {
            self.chosen = self.ports.first().cloned();
        }
    }

    /// Sits the picker on a port, closing whatever is open if it is another one.
    fn choose(&mut self, port: PortRef) {
        if self.chosen.as_ref() == Some(&port) {
            return;
        }
        if self.link.is_some() {
            self.disconnect("Port closed.");
        }
        self.chosen = Some(port);
    }

    /// Opens the chosen port and starts the device thread on it.
    fn connect(&mut self) {
        let Some(port) = self.chosen.clone() else {
            self.say("Choose a port first.");
            return;
        };
        self.disconnect("Port closed.");
        match open(&port) {
            Ok(link) => {
                self.say(format!("Opening {port}\u{2026}"));
                self.link = Some(link);
            }
            Err(error) => self.say(format!("{port} would not open: {error}")),
        }
    }

    /// Puts the port down and forgets the sound that was on it.
    ///
    /// The next port may be another instrument, or the same one after somebody
    /// spent the afternoon at its panel, so nothing on the screen survives it.
    fn disconnect(&mut self, why: &str) {
        if let Some(link) = self.link.take() {
            link.close();
        }
        self.identity = None;
        self.channel = None;
        self.patch.forget();
        self.say(why);
    }

    /// Sends a command, if there is a thread to send it to.
    fn ask(&mut self, command: Command) {
        let Some(link) = self.link.as_ref() else {
            self.say("Nothing is open.");
            return;
        };
        if link.send(command).is_err() {
            self.disconnect("The device thread has stopped.");
        }
    }

    /// Folds in everything the device thread has said since the last look.
    ///
    /// Never blocks. What it drains was queued by a thread that is still
    /// answering the port while this runs.
    fn drain(&mut self) {
        let Some(link) = self.link.as_ref() else {
            return;
        };
        // Taken in one go: absorbing them needs the whole of `self`, and the
        // link is part of it.
        let events: Vec<Event> = link.drain().collect();
        for event in events {
            self.absorb(event);
        }
    }

    /// Takes one thing the device thread said.
    fn absorb(&mut self, event: Event) {
        match event {
            Event::Opened => self.say("Port open. Asking who is there\u{2026}"),
            Event::Identified { identity, channel } => {
                self.identity = Some(identity);
                self.channel = Some(channel);
                self.say(format!(
                    "A DeepMind answered: firmware {}, channel {}.",
                    identity.firmware,
                    channel.number()
                ));
            }
            Event::Silent => {
                self.say("Nothing answered the inquiry. Switch the synthesizer on and ask again.");
            }
            Event::Parameter {
                parameter,
                value,
                confirmed,
            } => {
                if let Ok(value) = u8::try_from(value) {
                    self.patch.report(parameter, value, confirmed);
                }
            }
            Event::Device(DeviceEvent::EditBuffer(program)) => {
                self.say(format!("Read {}.", program.name()));
                self.patch.confirm(program);
            }
            Event::Refused { command, reason } => {
                self.say(format!("{command} refused: {reason}"));
            }
            Event::Failed(error) => self.disconnect(&format!("The port failed: {error}")),
            Event::Closed => self.disconnect("The port is closed."),
            // The rest is stage 4 and stage 5: stored programs, bank progress,
            // and everything the library reports and this window does not draw
            // yet.
            _ => {}
        }
    }
}

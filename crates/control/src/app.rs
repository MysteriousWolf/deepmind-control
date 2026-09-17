//! What the window holds, and what happens to it.

use std::path::{Path, PathBuf};

use control_ui::{Mapper, Patch, first_section};
use deepmind_host::{Command, Event, Link, Outcome, PortRef, open, ports};
use deepmind_midi::device::Event as DeviceEvent;
use deepmind_midi::ids::{Bank, PROGRAMS_PER_BANK, ProgramNumber};
use deepmind_midi::param::DEFAULT_FIRMWARE;
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::ProgramName;
use deepmind_midi::sysex::inquiry::{Identity, Version};
use deepmind_midi::wire::Channel;

use crate::files;
use crate::shelf::{self, Shelf};

/// Which of the three things this window is, at the moment somebody looks at it.
///
/// The instrument's own front, one section of it, and the sounds somebody
/// keeps. They are one application looking at three things rather than three
/// windows, and which one is showing is this window's own business and nothing
/// the synthesizer is told about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum View {
    /// The front panel: the sound as the instrument itself shows it.
    ///
    /// Where the window opens, because it is where a player looks first. The
    /// handful of controls the hardware puts a fader under, and on every
    /// section the press it calls `EDIT`.
    #[default]
    Panel,
    /// One of the fourteen panels: everything behind one of those presses.
    Editor,
    /// The shelf: the sounds a file or a bank read put there.
    Library,
}

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
    /// Show the front panel, a section of it, or the librarian.
    Show(View),
    /// Turn the displays over, and back.
    Invert,
    /// Sit the bank picker on a bank, without reading it.
    ChooseBank(Bank),
    /// Read the chosen bank onto the shelf, one dump at a time.
    ReadBank,
    /// Stop what is in flight.
    Cancel,
    /// Ask for a `.syx` file and put what it holds on the shelf.
    Open,
    /// Put what a named `.syx` file holds on the shelf, with no dialog.
    ///
    /// How the application is handed a file by whatever opened it.
    OpenNamed(PathBuf),
    /// Write the sound on the screen out as one program dump.
    SavePatch,
    /// Write the whole shelf out as a pack.
    SavePack,
    /// Make the synthesizer sound like the program in this place on the shelf.
    Load(usize),
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
    /// The control the pointer is over, which the footer describes.
    ///
    /// Not part of the sound and never sent anywhere: it is where somebody is
    /// looking, which is the window's own business and is forgotten the moment
    /// they look somewhere else.
    pointed: Option<ParamId>,
    /// What the press the pointer is over says about itself.
    ///
    /// The same thing [`pointed`](Self::pointed) is, for the parts of this
    /// window that are not parameters: the marks along the header and the foot,
    /// and the two presses that move a routing up and down the matrix. A mark
    /// nine dots square has nowhere to carry a word, so the word is here while
    /// somebody is asking for it and nowhere at all while nobody is.
    hinted: Option<&'static str>,
    /// What the modulation matrix is asking the window for.
    ///
    /// Not part of the sound and never sent anywhere. Two things: the routing
    /// somebody is pointing at the window, if any, and the searchable list each
    /// of the eight destinations is chosen from — which is state because what a
    /// list of that kind remembers is what has been typed into it.
    mapper: Mapper,
    /// The sounds that are kept rather than played.
    shelf: Shelf,
    /// The bank the picker is sitting on, which is the one a read would read.
    bank: Bank,
    /// Which of the three surfaces is showing.
    view: View,
    /// Whether the displays are drawn the other way up.
    ///
    /// Not part of the sound and nothing to do with the synthesizer: it is
    /// which of the two liveries this window is painted in, the way the glass
    /// on a piece of equipment is either dark dots on a lit screen or lit dots
    /// on a dark one. A `DeepMind` ships positive, so that is where it starts.
    negative: bool,
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
            pointed: None,
            hinted: None,
            mapper: Mapper::new(DEFAULT_FIRMWARE),
            shelf: Shelf::new(),
            bank: Bank::A,
            view: View::Panel,
            negative: false,
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

    /// A window that has, holding the first file it was handed.
    ///
    /// Which is how a desktop application is started by a document rather than
    /// by its own icon. Only the first is opened, because there is one shelf.
    #[must_use]
    pub fn opening(files: &[PathBuf]) -> Self {
        let mut app = Self::booted();
        if let Some(path) = files.first() {
            app.update(Message::OpenNamed(path.clone()));
        }
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

    /// Returns the control the pointer is over, for the footer to describe.
    #[must_use]
    pub const fn pointed(&self) -> Option<ParamId> {
        self.pointed
    }

    /// Returns what the press under the pointer says about itself.
    #[must_use]
    pub const fn hinted(&self) -> Option<&'static str> {
        self.hinted
    }

    /// Returns what the modulation matrix is asking the window for.
    ///
    /// Taken by reference and not by value: the searchable lists in it are what
    /// the eight destination pickers are drawn from, and a picker draws the one
    /// the application is holding rather than a copy of it.
    #[must_use]
    pub const fn mapper(&self) -> &Mapper {
        &self.mapper
    }

    /// Makes sure the destination lists are the ones this firmware names.
    ///
    /// Called at the end of every message rather than where an inquiry lands,
    /// so that there is one place this can be forgotten rather than several.
    /// Firmware 1.1 renumbered the destinations, and a list built for 1.0 would
    /// offer the wrong names for the right bytes.
    fn settle(&mut self) {
        let firmware = self.firmware();
        self.mapper.reading(firmware);
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
    /// down, because the sound went away and the person did not. It is also
    /// what the front panel's `EDIT` sets, so the two surfaces agree on which
    /// section is open.
    #[must_use]
    pub const fn section(&self) -> Group {
        self.section
    }

    /// Returns whether the displays are drawn the other way up.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns what the librarian is holding.
    #[must_use]
    pub const fn shelf(&self) -> &Shelf {
        &self.shelf
    }

    /// Returns the bank a read would read.
    #[must_use]
    pub const fn bank(&self) -> Bank {
        self.bank
    }

    /// Returns which of the three surfaces is showing.
    #[must_use]
    pub const fn view(&self) -> View {
        self.view
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
            Message::Show(view) => self.view = view,
            Message::ChooseBank(bank) => self.bank = bank,
            Message::ReadBank => self.read_bank(),
            Message::Cancel => self.ask(Command::Cancel),
            Message::Open => self.open_file(),
            Message::OpenNamed(path) => self.open_named(&path),
            Message::SavePatch => self.save_patch(),
            Message::SavePack => self.save_pack(),
            Message::Load(index) => self.load(index),
            // A section asked for is a section opened, whichever surface asked:
            // the front panel's `EDIT` and the section bar's own tabs are the
            // same press, and the hardware answers both by putting that section
            // on the display.
            Message::Ui(control_ui::Message::Show(section)) => {
                self.section = section;
                self.view = View::Editor;
            }
            Message::Ui(control_ui::Message::Edit { parameter, value }) => {
                self.moved(parameter, value);
                // A routing pointed at the window is asking where it goes, and
                // this is the answer arriving: whichever of the two ways said
                // it — a name chosen from the searchable list, or a control
                // taken hold of somewhere else in the window — the question has
                // been answered and the mode comes down.
                if self
                    .mapper
                    .mapped()
                    .is_some_and(|mapped| mapped.destination() == parameter)
                {
                    self.mapper.map(None);
                }
            }
            // The two halves of one gesture: a drag on a lit control while a
            // routing is pointed at the window says where the routing goes and
            // how much of it arrives. Both bytes were worked out by the view
            // that knows which control the drag is on; this is where they go
            // out, and the mode stays up until the hand lets go.
            Message::Ui(control_ui::Message::Reach {
                destination,
                at,
                depth,
                by,
            }) => {
                self.moved(destination, at);
                self.moved(depth, by);
            }
            Message::Ui(control_ui::Message::Swap { one, other }) => {
                // Read both sides before either moves, or the second pair is
                // written from a parameter the first pair has already changed.
                let held: Vec<(ParamId, Option<u8>, ParamId, Option<u8>)> = one
                    .into_iter()
                    .zip(other)
                    .map(|(one, other)| {
                        (one, self.patch.value(one), other, self.patch.value(other))
                    })
                    .collect();
                for (one, was, other, is) in held {
                    if let (Some(was), Some(is)) = (was, is) {
                        self.moved(one, is);
                        self.moved(other, was);
                    }
                }
            }
            Message::Ui(control_ui::Message::Mapper(at)) => self.mapper.map(at),
            Message::Invert => self.negative = !self.negative,
            Message::Ui(control_ui::Message::Rename(name)) => self.rename(name),
            Message::Ui(control_ui::Message::Pointed(parameter)) => self.pointed = parameter,
            Message::Ui(control_ui::Message::Hinted(said)) => self.hinted = said,
        }
        self.drain();
        // After the drain, because the drain is what an inquiry's answer
        // arrives in and the firmware it reports is what the destination lists
        // have to be built for.
        self.settle();
    }

    /// Moves a parameter, and tells the synthesizer about it once.
    ///
    /// The view moved it, so the window already shows it there. What the
    /// synthesizer is told is the same thing, once: an edit it already holds is
    /// a message the wire did not need.
    fn moved(&mut self, parameter: ParamId, value: u8) {
        if self.patch.edit(parameter, value) {
            self.ask(Command::SetParameter { parameter, value });
        }
    }

    /// Names the program, one character at a time because that is how it is
    /// stored.
    ///
    /// The same rule as an edit, seventeen times over: the patch works out
    /// which characters actually moved, and only those reach the wire. Typing a
    /// letter onto the end of a name is one NRPN; the other sixteen characters
    /// are already what they should be and cost nothing.
    fn rename(&mut self, name: ProgramName) {
        for (parameter, value) in self.patch.rename(name) {
            self.ask(Command::SetParameter { parameter, value });
        }
    }

    /// Reads the chosen bank onto the shelf.
    ///
    /// One request and 128 answers, which is about twelve seconds of a MIDI
    /// cable. The shelf is cleared first: a bank read is a picture of one bank,
    /// and half of the last one left underneath it would be a picture of
    /// nothing.
    fn read_bank(&mut self) {
        if self.link.is_none() {
            self.say("Nothing is open to read a bank from.");
            return;
        }
        let bank = self.bank;
        self.shelf.begin(bank, u16::from(PROGRAMS_PER_BANK));
        self.ask(Command::ReadBank {
            bank,
            first: ProgramNumber::FIRST,
            last: ProgramNumber::LAST,
        });
        self.say(format!("Reading bank {bank}\u{2026}"));
    }

    /// Asks for a `.syx` file and puts what it holds on the shelf.
    ///
    /// The dialog blocks the window for as long as it is open, which is the one
    /// place in this application where waiting is what somebody asked for.
    fn open_file(&mut self) {
        match files::open() {
            Some(Ok((name, bytes))) => self.shelve(&name, &bytes),
            Some(Err(trouble)) => self.say(format!("That file would not open: {trouble}")),
            None => {}
        }
    }

    /// Puts what a named `.syx` file holds on the shelf, with no dialog.
    fn open_named(&mut self, path: &Path) {
        match files::read(path) {
            Ok((name, bytes)) => self.shelve(&name, &bytes),
            Err(trouble) => self.say(format!("{} would not open: {trouble}", path.display())),
        }
    }

    /// Takes a file's bytes onto the shelf, and says what was in it.
    fn shelve(&mut self, name: &str, bytes: &[u8]) {
        self.shelf.open(name.to_owned(), bytes);
        let held = self.shelf.held().len();
        let skipped = self.shelf.skipped();
        let sound = if held == 1 { "program" } else { "programs" };
        let unreadable = match skipped {
            0 => String::new(),
            1 => ", and one frame that could not be read".to_owned(),
            many => format!(", and {many} frames that could not be read"),
        };
        self.say(format!("{name}: {held} {sound}{unreadable}."));
        self.view = View::Library;
    }

    /// Writes the sound on the screen out as one program dump.
    fn save_patch(&mut self) {
        let Some(program) = self.patch.program().cloned() else {
            self.say("There is no sound to save yet.");
            return;
        };
        let suggested = format!("{}.syx", file_stem(program.name()));
        match shelf::patch_to_syx(&program) {
            Ok(bytes) => self.wrote(&suggested, &bytes),
            Err(refusal) => self.say(format!("That sound would not pack: {refusal}")),
        }
    }

    /// Writes the whole shelf out as a pack.
    fn save_pack(&mut self) {
        if self.shelf.is_empty() {
            self.say("There is nothing on the shelf to save.");
            return;
        }
        let suggested = self.shelf.file_name();
        match self.shelf.to_syx() {
            Ok(bytes) => self.wrote(&suggested, &bytes),
            Err(refusal) => self.say(format!("The shelf would not pack: {refusal}")),
        }
    }

    /// Asks where to put a file, writes it, and says what happened.
    fn wrote(&mut self, suggested: &str, bytes: &[u8]) {
        match files::save(suggested, bytes) {
            Some(Ok(name)) => self.say(format!("Wrote {name}, {} bytes.", bytes.len())),
            Some(Err(trouble)) => self.say(format!("That file would not be written: {trouble}")),
            None => {}
        }
    }

    /// Makes the synthesizer sound like one of the programs on the shelf.
    ///
    /// Sent as the difference against what the synthesizer is believed to hold,
    /// which is the device thread's arithmetic and not this window's. The window
    /// itself assumes the whole program at once, because nothing has confirmed
    /// any of it: the sound on the screen is now a claim this application is
    /// making, and reading the edit buffer back is what turns it into a fact.
    ///
    /// It works with no port open, and says so. A pack browsed with nothing
    /// plugged in is still a pack somebody is reading, and the editor is the
    /// only thing that can show them what is in it.
    fn load(&mut self, index: usize) {
        let Some(program) = self.shelf.load(index) else {
            return;
        };
        let name = program.name().as_str().trim().to_owned();
        self.patch.assume(program.clone());
        if self.link.is_some() {
            self.ask(Command::LoadProgram(Box::new(program)));
            self.say(format!("Sending {name} to the edit buffer\u{2026}"));
        } else {
            self.say(format!(
                "{name} is on the screen. Nothing is open to send it to."
            ));
        }
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
        // The shelf is not the port's. A file somebody opened is still open and
        // a bank already read is still a backup; only the transfer that was in
        // flight went away with the thread that was carrying it.
        self.shelf.finish(Outcome::Cancelled);
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
                    "A DeepMind answered: firmware {}, channel {}. Reading the sound\u{2026}",
                    identity.firmware,
                    channel.number()
                ));
                // And read the sound it is making, without being asked. Every
                // control this window draws is drawn from a value, and until a
                // dump arrives every one of those is arithmetic: the panel is
                // the editor's guess at a synthesizer that is sitting right
                // there with the answer. Asking the moment somebody answers is
                // the difference between a window that shows the instrument and
                // one that shows what it would show if it had looked.
                //
                // It costs one message and one dump, on a port that has just
                // proved it works, and the button stays for when a sound has
                // been changed at the panel since.
                self.ask(Command::ReadEditBuffer);
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
            // A stored program is not the sound the synthesizer is making, so it
            // goes on the shelf and never into the patch. The two are different
            // questions and an editor that answered one with the other would be
            // showing somebody a sound they cannot hear.
            Event::Device(DeviceEvent::Program { slot, program }) => {
                self.shelf.arrived(slot, program);
            }
            Event::Progress {
                received, expected, ..
            } => self.shelf.advance(received, expected),
            Event::Finished {
                bank,
                received,
                expected,
                outcome,
            } => {
                self.shelf.finish(outcome);
                self.say(format!("Bank {bank}: {received} of {expected}, {outcome}."));
            }
            Event::Refused { command, reason } => {
                self.say(format!("{command} refused: {reason}"));
            }
            Event::Failed(error) => self.disconnect(&format!("The port failed: {error}")),
            Event::Closed => self.disconnect("The port is closed."),
            // The rest is stage 5, and everything the library reports that this
            // window has nothing to draw with: the globals, the patterns, and
            // the frames whose offsets the manual never published.
            _ => {}
        }
    }
}

/// Returns what to call the file a sound would be saved as.
///
/// The program's own name, with anything a file system would rather not see
/// taken out of it and the instrument's padding trimmed off. A sound with
/// nothing but punctuation for a name is saved as `patch`, because a file
/// called `.syx` is a hidden file on two of the three platforms this runs on.
fn file_stem(name: ProgramName) -> String {
    let stem: String = name
        .as_str()
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim().to_owned();
    if stem.is_empty() {
        "patch".to_owned()
    } else {
        stem
    }
}

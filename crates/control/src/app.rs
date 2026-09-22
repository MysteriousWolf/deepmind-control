//! What the window holds, and what happens to it.

use std::path::{Path, PathBuf};

use control_ui::{Livery, Mapper, Patch, Way};
use deepmind_host::{Command, Event, Link, Outcome, PortRef, open, ports};
use deepmind_midi::device::Event as DeviceEvent;
use deepmind_midi::ids::{Bank, PROGRAMS_PER_BANK, ProgramNumber};
use deepmind_midi::param::DEFAULT_FIRMWARE;
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::ProgramName;
use deepmind_midi::sysex::inquiry::{Identity, Version};
use deepmind_midi::wire::Channel;

use crate::catalogue::{Catalogue, Looking};
use crate::files;
use crate::shelf::{self, Order, Shelf};

/// Which of the two things this window is, at the moment somebody looks at it.
///
/// The instrument's own front, and the sounds somebody keeps. They are one
/// application looking at two things rather than two windows, and which one is
/// showing is this window's own business and nothing the synthesizer is told
/// about.
///
/// There were three. The middle one held whichever of the fourteen sections was
/// open, reached by a bar of tabs, and it is gone: a section is not a third
/// thing this application is, it is the detail behind one press on the first,
/// so it opens as a sheet over the panel rather than instead of it. See
/// [`editing`](App::editing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum View {
    /// The front panel: the sound as the instrument itself shows it.
    ///
    /// Where the window opens, because it is where a player looks first. The
    /// handful of controls the hardware puts a fader under, and on every
    /// section the press it calls `EDIT`.
    #[default]
    Panel,
    /// The shelf: the sounds a file or a bank read put there.
    Library,
    /// One of the four sections no plate on the front panel carries.
    ///
    /// A surface and not a sheet, which is the whole difference between the
    /// band of ways in and a plate's `EDIT`. Those four are behind no press on
    /// the instrument's front, so they are not the detail behind one: they are
    /// places, the band is a row of tabs, and a tab that covered the surface it
    /// is part of would be a modal wearing a tab's clothes.
    Section(Group),
}

/// Where a sound is.
///
/// The three places this window can see one, which is a fact about the sound
/// and so belongs in a column beside it rather than in a tab above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Where {
    /// On this machine: opened from a `.syx` file.
    Machine,
    /// On the synthesizer: read out of one of its banks.
    Instrument,
    /// In the shared library, published by somebody.
    Library,
}

/// The sound a row names, as something that outlives the drawing.
///
/// A table row is rebuilt every frame, so what is chosen cannot be a reference
/// to one. A place on the shelf or a patch's id are both stable across a
/// redraw and across a sort, which is what a selection has to survive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chosen {
    /// The program at this place on the shelf.
    Held(usize),
    /// The shared patch with this id.
    Patch(String),
}

/// Everything that can be done to a sound.
///
/// **One verb set, used in three places**: the toolbar along the top, the menu
/// a right-press opens, and this file. A window whose menu and toolbar called
/// the same thing two things would be a window somebody has to learn twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Hear it now. Nothing is kept: the edit buffer is the sound in front of
    /// somebody rather than one of the instrument's 1024.
    Play,
    /// Hear it and go to the panel, which is where it is changed.
    Edit,
    /// Onto this machine's shelf.
    Shelve,
    /// Into the instrument's memory, at a slot somebody chooses.
    Store,
    /// Describe it and write the files a pull request is made of.
    Share,
    /// Write it out as a `.syx` on its own.
    Export,
    /// Replace it with the newer version the library has published.
    Update,
}

impl Action {
    /// Every one, in the order they are offered.
    ///
    /// Hearing it first, because that is what somebody came to the list to do;
    /// then the two that move it somewhere; then the two that send it out; then
    /// the one that only sometimes applies.
    pub const ALL: [Self; 7] = [
        Self::Play,
        Self::Edit,
        Self::Shelve,
        Self::Store,
        Self::Share,
        Self::Export,
        Self::Update,
    ];

    /// The three that stand on the toolbar.
    ///
    /// **The ones that move a sound, and nothing else.** All seven are in the
    /// menu a right-press opens; only these three earn the width along the
    /// top, because the other four are either rare (`Share\u{2026}`,
    /// `Export\u{2026}`), already a press somewhere else (`Update` is the
    /// version cell, and `Update all n` beside the count), or already what a
    /// press on the row itself does (`Play`).
    ///
    /// A toolbar of everything is a toolbar nobody reads.
    pub const TOOLBAR: [Self; 3] = [Self::Edit, Self::Shelve, Self::Store];

    /// What it is called, everywhere it is offered.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Edit => "Edit",
            Self::Shelve => "Shelve",
            Self::Store => "Store\u{2026}",
            Self::Share => "Share\u{2026}",
            Self::Export => "Export\u{2026}",
            Self::Update => "Update",
        }
    }

    /// What it does, for the footer while the pointer is on it.
    #[must_use]
    pub const fn about(self) -> &'static str {
        match self {
            Self::Play => "Send it to the edit buffer. Nothing is stored and nothing is kept.",
            Self::Edit => "Send it to the edit buffer and go to the front panel.",
            Self::Shelve => "Put it on this machine's shelf, where a file or a bank read puts one.",
            Self::Store => "Write it into the instrument's memory, at a slot you choose.",
            Self::Share => "Describe it and write the files a pull request is made of.",
            Self::Export => "Write it out as a `.syx` file of its own.",
            Self::Update => "Replace it with the newer version the library has published.",
        }
    }
}

/// Which column the table is laid out by.
///
/// Named after the column heading it belongs to, because that is where
/// somebody presses to choose it and a name that did not match the heading
/// would be a name only this file knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum By {
    /// Nearest first: what is in the instrument, then the machine, then the
    /// library. The order the table opens in.
    #[default]
    Where,
    /// The bank a sound sits in, for the two places that have banks.
    Bank,
    /// The number within that bank.
    Number,
    /// What it is called.
    Name,
    /// Who made it.
    Maker,
    /// What it calls itself.
    Category,
    /// The vocabulary terms it carries.
    Tags,
    /// Which version it is, and whether a newer one is published.
    Version,
    /// What it sounds like, in the maker's words.
    About,
}

impl By {
    /// Every column, in the order they are drawn.
    ///
    /// **All of them sort and all of them narrow.** There is no column that is
    /// only printing: a table where three headings did something and five did
    /// nothing was a table somebody had to learn the exceptions to.
    pub const ALL: [Self; 9] = [
        Self::Where,
        Self::Bank,
        Self::Number,
        Self::Name,
        Self::Maker,
        Self::Category,
        Self::Tags,
        Self::Version,
        Self::About,
    ];

    /// What the column is headed.
    #[must_use]
    pub const fn heading(self) -> &'static str {
        match self {
            Self::Where => "WHERE",
            Self::Bank => "BANK",
            Self::Number => "No.",
            Self::Name => "NAME",
            Self::Maker => "MAKER",
            Self::Category => "CATEGORY",
            Self::Tags => "TAGS",
            Self::Version => "VERSION",
            Self::About => "ABOUT",
        }
    }

    /// Whether this column's values are a set somebody can be offered.
    ///
    /// Four are: where a sound is, which bank, what it calls itself, and
    /// whether it is behind. Those get a picker. The rest are open — a name, a
    /// maker, a term, a sentence — and a picker of every maker in a library is
    /// a list nobody can use, so those get a field to type in.
    #[must_use]
    pub const fn picks(self) -> bool {
        matches!(
            self,
            Self::Where | Self::Bank | Self::Category | Self::Version
        )
    }

    /// Whether the largest value belongs at the top when this column is first
    /// pressed.
    ///
    /// Only the version column, because the question somebody sorts it to ask
    /// is *what needs updating*, and that is the exception rather than the
    /// rule.
    #[must_use]
    pub const fn newest_first(self) -> bool {
        matches!(self, Self::Version)
    }
}

/// Which way round the table is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sorting {
    /// The column it is laid out by.
    pub by: By,
    /// Whether the largest is at the top.
    pub down: bool,
}

impl Where {
    /// The three, in the order somebody reads them: nearest first.
    pub const ALL: [Self; 3] = [Self::Instrument, Self::Machine, Self::Library];

    /// What the column prints.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Machine => "Machine",
            Self::Instrument => "Synth",
            Self::Library => "Library",
        }
    }

    /// How near to hand a sound in this place is.
    ///
    /// What the `WHERE` column sorts by, because *nearness* is what somebody
    /// means when they sort it: in the instrument, then on this machine, then
    /// published. Sorting the printed word would give `Library`, `Machine`,
    /// `Synth`, which is alphabetical order and an answer to nothing.
    #[must_use]
    pub const fn nearness(self) -> u8 {
        match self {
            Self::Instrument => 0,
            Self::Machine => 1,
            Self::Library => 2,
        }
    }
}

/// What a person is writing about a sound they are about to share.
///
/// Everything the repository's `.toml` asks for that this application cannot
/// work out from the program bytes. What it *can* work out is not here and is
/// never typed: the name, the category, the effects, the arpeggiator and the
/// rest all come off the 242 bytes when the pair is written, which is the same
/// rule the index is built under.
#[derive(Debug, Clone, Default)]
pub struct Publishing {
    /// A person or a handle. Not an email address.
    pub author: String,
    /// What it sounds like and how to play it.
    pub about: String,
    /// An SPDX identifier, or `All rights reserved`.
    pub licence: String,
    /// The collection folder it belongs in, where it belongs in one.
    pub collection: String,
    /// The 7x7 picture, packed a row to a byte with bit 0 leftmost.
    pub icon: [u8; 7],
    /// The vocabulary terms chosen, as the axis they came from and the term.
    pub terms: Vec<(String, String)>,
    /// Where the pair was last written, once it has been.
    pub wrote: Option<String>,
}

impl Publishing {
    /// Whether a dot of the icon is inked.
    #[must_use]
    pub fn inked(&self, across: usize, down: usize) -> bool {
        across < 7
            && self
                .icon
                .get(down)
                .is_some_and(|row| row & (1 << across) != 0)
    }

    /// Whether a term has been chosen.
    #[must_use]
    pub fn carries(&self, axis: &str, term: &str) -> bool {
        self.terms
            .iter()
            .any(|(held, said)| held == axis && said == term)
    }

    /// Everything the repository insists on, or what is missing.
    ///
    /// `name` and the category come off the program, so what a person can
    /// leave out is the rest: a maker, a sentence, a licence, and at least one
    /// term from one of the four vocabularies. The same five the repository's
    /// own validator asks for, checked here so that somebody finds out before
    /// they open a pull request rather than after.
    #[must_use]
    pub fn missing(&self) -> Vec<&'static str> {
        let mut wanted = Vec::new();
        if self.author.trim().is_empty() {
            wanted.push("a maker");
        }
        if self.about.trim().is_empty() {
            wanted.push("a sentence about it");
        }
        if self.licence.trim().is_empty() {
            wanted.push("a licence");
        }
        if self.terms.is_empty() {
            wanted.push("at least one term");
        }
        wanted
    }
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
    /// Show the front panel or the librarian.
    Show(View),
    /// Turn the displays over, and back.
    Invert,
    /// Wear the other `DeepMind`'s front, and back.
    ///
    /// A `12` prints its section names in white on the bare panel and a `12X`
    /// knocks them out of filled banners. Both are the instrument; this is
    /// which of the two the window is drawn as.
    Wear,
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
    /// Narrow the shelf to the programs these words are anywhere in.
    Search(String),
    /// Draw the shelf the other way round.
    SortBy(Order),
    /// Fetch the newest release of the shared patches.
    FetchPatches,
    /// Ask for a folder and read a checkout of the shared patches out of it.
    OpenPatches,
    /// Narrow the shared patches to the ones these words are anywhere in.
    FindPatch(String),
    /// Narrow one column to the rows whose cell carries this text.
    ///
    /// One message for every column, and an empty string clears it. A message
    /// per column was five messages that did one thing five ways, and a column
    /// added later would have needed a sixth.
    Narrow(By, String),
    /// Lay the table out by this column, or turn it round if it already is.
    SortSounds(By),
    /// Put every column's chooser back to showing everything.
    ShowEverything,
    /// Choose a sound, and play it.
    ChooseSound(Chosen),
    /// Choose a sound and open the menu on it.
    OpenMenu(Chosen),
    /// Put the menu away.
    CloseMenu,
    /// Put the share sheet away, keeping what was typed into it.
    CloseSharing,
    /// Remember where the pointer is over the table.
    PointerAt(iced::Point),
    /// Do something to the sound that is chosen.
    Act(Action),
    /// Replace the program at this place on the shelf with the newest
    /// published version of it.
    UpdateSound(usize),
    /// Replace every one the library has published a newer version of.
    UpdateEverything,
    /// Put one shared patch, by its id, on the shelf.
    ShelvePatch(String),
    /// Play one shared patch without keeping it anywhere.
    AuditionPatch(String),
    /// Say who made the sound about to be shared.
    PublishAuthor(String),
    /// Say what it sounds like.
    PublishAbout(String),
    /// Say what it is licensed under.
    PublishLicence(String),
    /// Say which collection it belongs to.
    PublishCollection(String),
    /// Turn one vocabulary term on or off.
    PublishTerm(String, String),
    /// Turn one dot of the icon on or off.
    PublishDot(usize, usize),
    /// Start the icon again, from nothing or from the category's own.
    PublishIcon(bool),
    /// Write the pair out into a folder somebody chooses.
    PublishWrite,
    /// Put every shared patch the search left on the shelf.
    ShelveShowing,
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
    /// The sounds other people have published, once any are held.
    catalogue: Catalogue,
    /// What is being asked of the shared shelf.
    looking: Looking,
    /// What is being written about a sound about to be shared.
    publishing: Publishing,
    /// Which way round the table is laid out.
    sorting: Sorting,
    /// The sound a press chose, where one is chosen.
    picked: Option<Chosen>,
    /// The sound the share sheet is open on, while it is open.
    ///
    /// **A sheet over the table rather than a surface beside it.** Sharing is
    /// something done *to one sound*, so it belongs where that sound is: a tab
    /// meant leaving the list to describe a row, coming back, and having no way
    /// to tell which row had been described. What is held here is the row, not
    /// the program, for the reason [`Chosen`] exists at all.
    sharing: Option<Chosen>,
    /// Where the menu a right-press opened is standing, while one is open.
    menu: Option<iced::Point>,
    /// Where the pointer last was over the table.
    ///
    /// Tracked only so that a menu opens where the press was. Not part of the
    /// sound and never sent anywhere.
    pointer: iced::Point,
    /// The shared patch being tried, where one is.
    ///
    /// Not part of the sound and never written anywhere. A patch being
    /// auditioned is in the instrument's edit buffer and nowhere else — not on
    /// the shelf, not in its memory — so this is the only record that it is
    /// what is sounding, and it is forgotten the moment anything else is
    /// loaded or read.
    trying: Option<String>,
    /// The section open over the window, where one is.
    editing: Option<Group>,
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
    /// of the eight destinations is chosen from, which is state because what a
    /// list of that kind remembers is what has been typed into it.
    mapper: Mapper,
    /// The sounds that are kept rather than played.
    shelf: Shelf,
    /// The bank the picker is sitting on, which is the one a read would read.
    bank: Bank,
    /// Which of the two surfaces is showing.
    view: View,
    /// Whether the displays are drawn the other way up.
    ///
    /// Not part of the sound and nothing to do with the synthesizer: it is
    /// which of the two liveries this window is painted in, the way the glass
    /// on a piece of equipment is either dark dots on a lit screen or lit dots
    /// on a dark one. A `DeepMind` ships positive, so that is where it starts.
    negative: bool,
    /// Which `DeepMind`'s front the panel is wearing.
    ///
    /// Not a fact about the sound and nothing the synthesizer is told: a `12`
    /// and a `12X` have the same 242 parameters and two different silkscreens,
    /// so this is which of the two the window is drawn as and it survives a
    /// port being put down.
    livery: Livery,
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
            editing: None,
            pointed: None,
            hinted: None,
            mapper: Mapper::new(DEFAULT_FIRMWARE),
            shelf: Shelf::new(),
            catalogue: Catalogue::new(),
            looking: Looking::default(),
            publishing: Publishing::default(),
            sorting: Sorting::default(),
            picked: None,
            sharing: None,
            menu: None,
            pointer: iced::Point::ORIGIN,
            trying: None,
            bank: Bank::A,
            view: View::Panel,
            negative: false,
            livery: Livery::default(),
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

    /// Puts an already-open port in the window's hand.
    ///
    /// What choosing a port and pressing the connection does, for a caller that
    /// opened the port itself. The previews do, because the one thing
    /// [`open`] cannot hand back is the simulated unit's own front panel, and a
    /// picture of the editor holding a sound needs somebody to have turned the
    /// knobs. The picker is sat on the port as well, because a window whose
    /// picker is empty while its display is full is a window in a state nobody
    /// can reach.
    #[cfg(feature = "previews")]
    pub fn attach(&mut self, port: PortRef, link: Link) {
        self.say(format!("Opening {port}\u{2026}"));
        self.chosen = Some(port);
        self.link = Some(link);
    }

    /// Puts a pack on the shelf, with no file chooser and no disk.
    ///
    /// What opening a `.syx` file does, for a caller that already has the
    /// bytes. The previews do: a picture of the librarian with nothing on the
    /// shelf is a picture of the two presses that fill one, and what the
    /// surface is *for* — the grid, the search, the orders and what a program
    /// calls itself — is only on the screen once something is on it.
    #[cfg(feature = "previews")]
    pub fn hold(&mut self, name: &str, bytes: &[u8]) {
        self.shelve(name, bytes);
    }

    /// Reads a checkout of the shared patches, with no folder chooser.
    ///
    /// What [`Message::OpenPatches`] does, for a caller that already knows
    /// where the checkout is. The previews do: the table's whole subject is
    /// every sound *wherever it is*, and a picture taken with nothing but a
    /// shelf in it is a picture of a third of the columns.
    ///
    /// # Errors
    ///
    /// Whatever the folder was not a patch library for.
    #[cfg(feature = "previews")]
    pub fn read_patches(&mut self, root: &Path) -> Result<(), String> {
        self.catalogue.open(root)
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
        // The download says what it has done on the same tick the device thread
        // does, and for the same reason: neither of them is waited on.
        self.catalogue.settle();
    }

    /// The sounds other people have published.
    #[must_use]
    pub const fn catalogue(&self) -> &Catalogue {
        &self.catalogue
    }

    /// What is being asked of the shared shelf.
    #[must_use]
    pub const fn looking(&self) -> &Looking {
        &self.looking
    }

    /// Which way round the table is laid out.
    #[must_use]
    pub const fn sorting(&self) -> Sorting {
        self.sorting
    }

    /// The sound a press chose, where one is chosen.
    #[must_use]
    pub const fn picked(&self) -> Option<&Chosen> {
        self.picked.as_ref()
    }

    /// Where the menu is standing, while one is open.
    #[must_use]
    pub const fn menu(&self) -> Option<iced::Point> {
        self.menu
    }

    /// The sound the share sheet is open on, while it is open.
    #[must_use]
    pub const fn sharing(&self) -> Option<&Chosen> {
        self.sharing.as_ref()
    }

    /// The program the share sheet is open on, while it is open.
    #[must_use]
    pub fn shared(&self) -> Option<deepmind_midi::program::Program> {
        self.program_of(self.sharing.as_ref()?)
    }

    /// Whether an action can be done to what is chosen.
    ///
    /// **The one place that decides**, so the toolbar and the menu grey out the
    /// same presses: a window where a menu offered what its toolbar refused
    /// would be a window that disagrees with itself.
    #[must_use]
    pub fn can(&self, action: Action) -> bool {
        let Some(picked) = &self.picked else {
            return false;
        };
        match action {
            // Anything can be heard, put on the panel, written out or
            // described, wherever it is.
            Action::Play | Action::Edit | Action::Export | Action::Share => true,
            // Only something that is not already here.
            Action::Shelve => matches!(picked, Chosen::Patch(_)) && self.catalogue.held().is_some(),
            // **Not yet, and not because of this window.** The protocol writes
            // a stored program by sending a program dump *to* the instrument —
            // the library's own note says a preset pack is exactly that — but
            // `Device` publishes no call for it: every `request_*` asks for
            // something and `edit` moves one parameter. That is
            // `deepmind-midi#49`, and `docs/waiting.md` says what this window
            // does meanwhile: the verb is drawn and refused, with the reason
            // said out loud, rather than left off the list as though nobody
            // had thought of it.
            Action::Store => false,
            // Only where the library has published a newer version than the one
            // this is.
            Action::Update => self.newer().is_some(),
        }
    }

    /// The newest published version of what is chosen, when it is not already
    /// the newest.
    #[must_use]
    fn newer(&self) -> Option<deepmind_midi::program::Program> {
        let Chosen::Held(at) = self.picked.as_ref()? else {
            return None;
        };
        self.newer_at(*at)
    }

    /// The newest published version of what sits at this place on the shelf.
    ///
    /// **Matched on the sound rather than on the name.** A fingerprint is the
    /// identity of what the 242 bytes make, so a program somebody renamed is
    /// still recognised and a program somebody edited is a different sound and
    /// is not. `None` where nothing was published, where nothing matched, and
    /// where what is here is already the newest — three different facts with
    /// the same answer, because the press they decide is the same press.
    #[must_use]
    fn newer_at(&self, at: usize) -> Option<deepmind_midi::program::Program> {
        let held = self.catalogue.held()?;
        let program = &self.shelf.held().get(at)?.program;
        let found = held.matching(program)?;
        (!found.latest)
            .then(|| held.program(found.patch).ok())
            .flatten()
    }

    /// How many sounds on the shelf the library has since published a newer
    /// version of.
    ///
    /// Counted rather than collected, and without decoding any of them: the
    /// index already says which version each fingerprint is newest at, so this
    /// is a walk of the shelf against a map and costs nothing to ask every
    /// frame. What it decides is whether the press that updates all of them is
    /// drawn at all.
    #[must_use]
    pub fn stale(&self) -> usize {
        let Some(held) = self.catalogue.held() else {
            return 0;
        };
        self.shelf
            .held()
            .iter()
            .filter(|one| {
                held.matching(&one.program)
                    .is_some_and(|found| !found.latest)
            })
            .count()
    }

    /// The shared patch being tried, where one is.
    #[must_use]
    pub fn trying(&self) -> Option<&str> {
        self.trying.as_deref()
    }

    /// What is being written about a sound about to be shared.
    #[must_use]
    pub const fn publishing(&self) -> &Publishing {
        &self.publishing
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

    /// Returns the section open over the window, where one is.
    ///
    /// `None` is a window with nothing over it, which is where it opens: the
    /// front panel is the instrument and a sheet is what one of its `EDIT`
    /// presses put in front of it.
    ///
    /// Which panel somebody is looking at is this window's business and nothing
    /// the synthesizer is told about. It survives a port being put down,
    /// because the sound went away and the person did not.
    #[must_use]
    pub const fn editing(&self) -> Option<Group> {
        self.editing
    }

    /// Returns which cap of the band of ways in is lit.
    ///
    /// Where the window *is*, in the one vocabulary the band is drawn from: the
    /// shelf, a section on a sheet, or the panel with nothing over it. A
    /// section's sheet opened from a plate's own `EDIT` lights nothing in the
    /// band, because the band carries only the four sections no plate does, and
    /// that is the comparison rather than a rule written here.
    #[must_use]
    pub const fn showing(&self) -> Way {
        match (self.view, self.editing) {
            (View::Library, _) => Way::Library,
            // A section, either way it is being shown: the surface a tab
            // opened, or the sheet a plate's `EDIT` laid over the panel. The
            // second lights nothing, because the band carries only the four
            // sections no plate does and a sheet is never one of them — which
            // is the band's own list deciding rather than a rule written here,
            // and is why the two are one arm.
            (View::Section(section), _) | (View::Panel, Some(section)) => Way::Section(section),
            (View::Panel, None) => Way::Panel,
        }
    }

    /// Returns whether the displays are drawn the other way up.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns which `DeepMind`'s front the panel is wearing.
    #[must_use]
    pub const fn livery(&self) -> Livery {
        self.livery
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

    /// Returns which of the two surfaces is showing.
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
            // Both of these are about what is on the screen and not about what
            // is on the shelf, so neither says anything to the synthesizer and
            // neither touches what a save would write.
            Message::Search(words) => self.shelf.search(words),
            Message::SortBy(order) => self.shelf.sort_by(order),
            Message::Invert => self.negative = !self.negative,
            Message::Wear => {
                self.livery = match self.livery {
                    Livery::Plain => Livery::Banners,
                    Livery::Banners => Livery::Plain,
                };
            }
            Message::ShelveShowing => self.shelve_showing(),
            Message::FetchPatches
            | Message::OpenPatches
            | Message::FindPatch(_)
            | Message::Narrow(..)
            | Message::SortSounds(_)
            | Message::ShowEverything
            | Message::ChooseSound(_)
            | Message::OpenMenu(_)
            | Message::CloseMenu
            | Message::CloseSharing
            | Message::PointerAt(_)
            | Message::Act(_)
            | Message::UpdateSound(_)
            | Message::UpdateEverything
            | Message::ShelvePatch(_)
            | Message::AuditionPatch(_) => self.looking_at(message),
            Message::PublishAuthor(_)
            | Message::PublishAbout(_)
            | Message::PublishLicence(_)
            | Message::PublishCollection(_)
            | Message::PublishTerm(..)
            | Message::PublishDot(..)
            | Message::PublishIcon(_)
            | Message::PublishWrite => self.writing(message),
            Message::Ui(asked) => self.asked(asked),
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

    /// Starts fetching the newest release of the shared patches.
    fn fetch_patches(&mut self) {
        let Some(into) = crate::catalogue::cache() else {
            self.say("This machine has nowhere to keep a downloaded catalogue.".to_owned());
            return;
        };
        self.catalogue.fetch(into);
    }

    /// Reads a checkout of the shared patches out of a folder somebody chose.
    fn open_patches(&mut self) {
        let Some(root) = files::folder() else {
            return;
        };
        match self.catalogue.open(&root) {
            Ok(()) => {
                let held = self.catalogue.held().map_or(0, |held| held.patches().len());
                self.say(format!(
                    "{held} shared patches, read off {}.",
                    root.display()
                ));
            }
            Err(trouble) => self.say(format!("That folder is not a patch library: {trouble}")),
        }
    }

    /// Everything the drawing asked for.
    ///
    /// The view layer speaks one message type and this window speaks another,
    /// so every press on a fader, a cap or a sheet's own bar arrives here.
    /// Gathered into one place because they are one boundary: what crosses it
    /// is the view saying *somebody did this*, and what happens about it is
    /// always this file's business rather than the drawing's.
    fn asked(&mut self, message: control_ui::Message) {
        match message {
            // A section asked for is a section opened, which is what the
            // instrument's own `EDIT` does: the display becomes that section
            // and the front of the synthesizer does not move. Here the sheet
            // comes up over the panel the press is on.
            //
            // One at a time. A second sheet over the first would be a window
            // nobody can find the bottom of, so asking for a section while one
            // is open is the same press the hardware's second `EDIT` is: the
            // sheet becomes the other section.
            // A plate's `EDIT`: the section as a sheet over the front panel,
            // which is what the press does on the instrument. Asking for one
            // from anywhere else brings the panel back under it, because that
            // is what the sheet is laid over.
            control_ui::Message::Show(section) => {
                self.view = View::Panel;
                self.editing = Some(section);
            }
            // A cap of the band: the section as the surface itself. Whatever
            // sheet was over the panel comes down with it, the way it does for
            // the shelf, because a sheet belongs to the surface it was opened
            // from.
            control_ui::Message::Open(section) => {
                self.view = View::Section(section);
                self.editing = None;
            }
            // The shelf, which is the one cap of the band that is a surface
            // rather than a section. Whatever sheet was over the panel comes
            // down with it: a sound's filter is not open while somebody is
            // looking at a list of sounds.
            control_ui::Message::Shelf => {
                self.view = View::Library;
                self.editing = None;
            }
            // The way back out from under a sheet: the mark on its own bar, a
            // press on the panel around it, or the escape key. It says nothing
            // about which surface is underneath, because it is not a place to
            // go — escape from the shelf leaves somebody on the shelf.
            control_ui::Message::Close => self.editing = None,
            // The first cap of the band, which *is* a place to go: the front of
            // the instrument, with nothing over it, from wherever somebody was.
            control_ui::Message::Front => {
                self.view = View::Panel;
                self.editing = None;
            }
            control_ui::Message::Edit { parameter, value } => {
                self.moved(parameter, value);
                // A routing pointed at the window is asking where it goes, and
                // this is the answer arriving. Whichever of the two ways said
                // it, a name chosen from the searchable list or a control taken
                // hold of somewhere else in the window, the question has been
                // answered and the mode comes down.
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
            control_ui::Message::Reach {
                destination,
                at,
                depth,
                by,
            } => {
                self.moved(destination, at);
                self.moved(depth, by);
            }
            control_ui::Message::Swap { one, other } => {
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
            control_ui::Message::Mapper(at) => self.mapper.map(at),
            control_ui::Message::Rename(name) => self.rename(name),
            control_ui::Message::Pointed(parameter) => self.pointed = parameter,
            control_ui::Message::Hinted(said) => self.hinted = said,
        }
    }

    /// Everything somebody does to the list of sounds.
    ///
    /// One conversation, like [`writing`](Self::writing): what is being looked
    /// for, where it is being looked for, and what happens to the one that was
    /// pressed. None of it touches the sound on the screen.
    fn looking_at(&mut self, message: Message) {
        match message {
            Message::FetchPatches => self.fetch_patches(),
            Message::OpenPatches => self.open_patches(),
            Message::FindPatch(words) => self.looking.find = words,
            Message::Narrow(column, text) => self.looking.narrow(column, text),
            Message::SortSounds(by) => self.sort_sounds(by),
            Message::ShowEverything => self.looking = Looking::default(),
            Message::ChooseSound(chosen) => self.choose_sound(chosen),
            Message::OpenMenu(chosen) => {
                self.choose_sound(chosen);
                self.menu = Some(self.pointer);
            }
            Message::CloseMenu => self.menu = None,
            Message::CloseSharing => self.sharing = None,
            Message::PointerAt(at) => self.pointer = at,
            Message::Act(action) => self.act(action),
            Message::UpdateSound(at) => self.update_at(at),
            Message::UpdateEverything => self.update_all(),
            Message::ShelvePatch(id) => self.shelve_patch(&id),
            Message::AuditionPatch(id) => self.audition(&id),
            _ => {}
        }
    }

    /// Everything somebody is writing about a sound they are about to share.
    ///
    /// Gathered into one arm because they are one conversation: eight messages
    /// that all move the same half-written record and none of which the rest of
    /// this window has any opinion about.
    fn writing(&mut self, message: Message) {
        match message {
            Message::PublishAuthor(said) => self.publishing.author = said,
            Message::PublishAbout(said) => self.publishing.about = said,
            Message::PublishLicence(said) => self.publishing.licence = said,
            Message::PublishCollection(said) => self.publishing.collection = said,
            Message::PublishTerm(axis, term) => self.publish_term(&axis, &term),
            Message::PublishDot(across, down) => self.publish_dot(across, down),
            Message::PublishIcon(from_category) => self.publish_icon(from_category),
            Message::PublishWrite => self.publish_write(),
            _ => {}
        }
    }

    /// Lays the table out by a column, or turns it round if it already is.
    ///
    /// Pressing the heading somebody is already under means *the other way*,
    /// which is what every table anybody has used does, and going back to a
    /// column always starts from whichever end that column is usually read
    /// from rather than remembering which way round it was left.
    fn sort_sounds(&mut self, by: By) {
        if self.sorting.by == by {
            self.sorting.down = !self.sorting.down;
        } else {
            self.sorting = Sorting {
                by,
                down: by.newest_first(),
            };
        }
    }

    /// Turns one vocabulary term on or off.
    fn publish_term(&mut self, axis: &str, term: &str) {
        if let Some(at) = self
            .publishing
            .terms
            .iter()
            .position(|(held, said)| held == axis && said == term)
        {
            drop(self.publishing.terms.remove(at));
        } else {
            self.publishing
                .terms
                .push((axis.to_owned(), term.to_owned()));
        }
    }

    /// Turns one dot of the icon on or off.
    fn publish_dot(&mut self, across: usize, down: usize) {
        if across >= 7 {
            return;
        }
        if let Some(row) = self.publishing.icon.get_mut(down) {
            *row ^= 1 << across;
        }
    }

    /// Starts the icon again: blank, or from the category's own drawing.
    ///
    /// The category's, because a blank grid is a bad place to start a picture
    /// and the repository already draws twelve good ones. Somebody who wants a
    /// bass that looks like a bass but not *that* bass edits from it rather
    /// than from nothing.
    fn publish_icon(&mut self, from_category: bool) {
        self.publishing.icon = match (from_category, self.sharing.clone()) {
            (true, Some(chosen)) => self.category_icon(&chosen),
            _ => [0; 7],
        };
    }

    /// Opens the share sheet on a sound, filled in with whatever is known.
    ///
    /// **Filled in and not blank.** A sound the library has already published
    /// comes with a maker, a sentence, a licence, a vocabulary and a drawing,
    /// and a sheet that asked for all five again would be asking somebody to
    /// retype a file they are about to send a change to. So the sheet opens on
    /// what the library holds and the person edits it, which is what sharing a
    /// second version of something actually is.
    ///
    /// A sound nothing has published opens on the category's own drawing and
    /// nothing else, because there is nothing else to know.
    fn start_sharing(&mut self, chosen: &Chosen) {
        self.publishing = self.described(chosen);
        self.sharing = Some(chosen.clone());
    }

    /// What the library already says about a sound, as a half-written record.
    fn described(&self, chosen: &Chosen) -> Publishing {
        let known = self.catalogue.held().and_then(|held| match chosen {
            Chosen::Patch(id) => held.index().get(id),
            Chosen::Held(at) => held
                .matching(&self.shelf.held().get(*at)?.program)
                .map(|found| found.patch),
        });
        let Some(patch) = known else {
            return Publishing {
                icon: self.category_icon(chosen),
                ..Publishing::default()
            };
        };
        let mut terms = Vec::new();
        for (axis, said) in [
            ("genre", &patch.genre),
            ("mood", &patch.mood),
            ("timbre", &patch.timbre),
            ("role", &patch.role),
        ] {
            terms.extend(said.iter().map(|term| (axis.to_owned(), term.clone())));
        }
        Publishing {
            author: patch.author.clone(),
            about: patch.about.clone(),
            licence: patch.licence.clone(),
            collection: patch.collection.clone().unwrap_or_default(),
            // The patch's own drawing where it has one, and its category's
            // where the index filled that in for it: writing a category's icon
            // back out as the patch's would be claiming a picture nobody drew.
            icon: if patch.own_icon {
                *patch.icon.rows()
            } else {
                [0; 7]
            },
            terms,
            wrote: None,
        }
    }

    /// The drawing a sound's category is published under, where there is one.
    fn category_icon(&self, chosen: &Chosen) -> [u8; 7] {
        let blank = [0; 7];
        let Some(held) = self.catalogue.held() else {
            return blank;
        };
        let program = match chosen {
            Chosen::Held(at) => match self.shelf.held().get(*at) {
                Some(one) => one.program.clone(),
                None => return blank,
            },
            Chosen::Patch(id) => match held.index().get(id).map(|patch| held.program(patch)) {
                Some(Ok(program)) => program,
                _ => return blank,
            },
        };
        deepmind_patches::Category::of(&program)
            .and_then(|category| held.category_icon(category))
            .map_or(blank, |icon| *icon.rows())
    }

    /// Writes the `.syx` and `.toml` pair into a folder somebody chooses.
    ///
    /// Laid out the way the repository wants it — `presets/<Category>/` — so
    /// that the folder this writes can be copied straight over a checkout and
    /// the result is a pull request. Beside them goes a short note saying
    /// exactly that, because a person who has just drawn an icon should not
    /// have to go and read a contributing guide to find out what the next four
    /// steps are.
    fn publish_write(&mut self) {
        let Some(program) = self
            .sharing
            .as_ref()
            .and_then(|chosen| self.program_of(chosen))
        else {
            self.say("There is no sound to share.".to_owned());
            return;
        };
        let Some(category) = deepmind_patches::Category::of(&program) else {
            self.say(
                "Set a category on the instrument first: the folder a patch goes in is the one \
                 it calls itself."
                    .to_owned(),
            );
            return;
        };
        let missing = self.publishing.missing();
        if !missing.is_empty() {
            self.say(format!("Still wanted: {}.", missing.join(", ")));
            return;
        }
        let Some(root) = files::folder() else {
            return;
        };
        match crate::publish::write(&root, &program, category, &self.publishing) {
            Ok(where_to) => {
                self.publishing.wrote = Some(where_to.clone());
                self.say(format!("Written to {where_to}."));
            }
            Err(trouble) => self.say(format!("That would not be written: {trouble}")),
        }
    }

    /// Chooses a sound, and plays it.
    ///
    /// Both at once, because playing costs nothing: the edit buffer is the
    /// sound in front of somebody rather than one of the instrument's 1024, so
    /// a press that both selects and sounds is a press with no downside.
    /// Selecting without hearing would mean two presses to do the one thing
    /// everybody came here for.
    fn choose_sound(&mut self, chosen: Chosen) {
        self.picked = Some(chosen.clone());
        self.menu = None;
        match chosen {
            Chosen::Held(at) => self.load(at),
            Chosen::Patch(id) => self.audition(&id),
        }
    }

    /// Does one thing to the sound that is chosen.
    ///
    /// The single place every verb is carried out, whichever of the three
    /// surfaces asked for it. Nothing happens for an action
    /// [`can`](Self::can) refuses, so a press that slipped through a disabled
    /// button is a press that does nothing rather than one that does something
    /// surprising.
    fn act(&mut self, action: Action) {
        self.menu = None;
        if !self.can(action) {
            return;
        }
        let Some(chosen) = self.picked.clone() else {
            return;
        };
        match action {
            Action::Play => self.choose_sound(chosen),
            Action::Edit => {
                self.choose_sound(chosen);
                self.view = View::Panel;
                self.editing = None;
            }
            Action::Shelve => {
                if let Chosen::Patch(id) = chosen {
                    self.shelve_patch(&id);
                }
            }
            Action::Store => self.say(
                "Writing into the instrument's memory needs a call deepmind-midi does not \
                 publish yet: deepmind-midi#49. See docs/waiting.md."
                    .to_owned(),
            ),
            Action::Share => self.start_sharing(&chosen),
            Action::Export => self.export_one(),
            Action::Update => {
                if let Chosen::Held(at) = chosen {
                    self.update_at(at);
                }
            }
        }
    }

    /// Writes the chosen sound out as a `.syx` of its own.
    fn export_one(&mut self) {
        let Some(program) = self.chosen_program() else {
            return;
        };
        let name = program.name().as_str().trim().to_owned();
        match crate::shelf::patch_to_syx(&program) {
            Ok(bytes) => match files::save(&name, &bytes) {
                Some(Ok(where_to)) => self.say(format!("Written to {where_to}.")),
                Some(Err(trouble)) => self.say(format!("That would not be written: {trouble}")),
                None => {}
            },
            Err(trouble) => self.say(format!("{name} would not be packed: {trouble}")),
        }
    }

    /// Replaces one program with the newest published version of it.
    ///
    /// Only what is on the shelf, and only in place: the slot it sits in is the
    /// slot it keeps, because a librarian that moved a program while updating
    /// it would be a librarian rearranging a bank nobody asked it to.
    ///
    /// **Nothing reaches the instrument.** What changes is the shelf, which is
    /// this machine's copy; the synthesizer keeps what it is holding until
    /// somebody plays the row or stores it.
    fn update_at(&mut self, at: usize) {
        let Some(newest) = self.newer_at(at) else {
            return;
        };
        let name = newest.name().as_str().trim().to_owned();
        if self.shelf.replace(at, newest) {
            self.say(format!("{name} is the published version now."));
        }
    }

    /// Replaces every one the library has published a newer version of.
    ///
    /// Read first and written after, all of it, rather than one at a time:
    /// what is newest is worked out against the shelf as it stands, so a run
    /// that replaced one program and then asked about the next would be asking
    /// about a shelf it had already changed.
    ///
    /// One sentence at the end and not one per sound. Somebody who pressed this
    /// asked about the shelf, not about each of the eleven.
    fn update_all(&mut self) {
        let newest: Vec<(usize, deepmind_midi::program::Program)> = (0..self.shelf.held().len())
            .filter_map(|at| self.newer_at(at).map(|program| (at, program)))
            .collect();
        if newest.is_empty() {
            self.say("Everything on the shelf is the published version already.".to_owned());
            return;
        }
        let mut done = 0_usize;
        for (at, program) in newest {
            if self.shelf.replace(at, program) {
                done = done.saturating_add(1);
            }
        }
        let sound = if done == 1 { "sound is" } else { "sounds are" };
        self.say(format!("{done} {sound} the published version now."));
    }

    /// The program the chosen sound holds, wherever it is.
    fn chosen_program(&self) -> Option<deepmind_midi::program::Program> {
        self.program_of(self.picked.as_ref()?)
    }

    /// The program one row holds, wherever it is.
    fn program_of(&self, chosen: &Chosen) -> Option<deepmind_midi::program::Program> {
        match chosen {
            Chosen::Held(at) => self.shelf.held().get(*at).map(|held| held.program.clone()),
            Chosen::Patch(id) => {
                let held = self.catalogue.held()?;
                held.program(held.index().get(id)?).ok()
            }
        }
    }

    /// Plays one shared patch without keeping it anywhere.
    ///
    /// **The instrument already makes this safe.** `LoadProgram` writes the
    /// edit buffer, which is the sound in front of somebody rather than any of
    /// the 1024 stored programs, so nothing is overwritten and nothing has to
    /// be put back: turning the instrument's own program knob is what undoes
    /// it. So auditioning is not a mode with a way out of it, it is simply
    /// loading without keeping.
    ///
    /// Which is the whole of what makes a computer worth having plugged in
    /// here. An instrument holds 1024 sounds and a disk holds as many as
    /// somebody has; a patch that can be heard without being stored is a patch
    /// that does not have to displace one of the 1024 to be tried, so the
    /// library on the machine is playable memory rather than an archive.
    fn audition(&mut self, id: &str) {
        let Some(held) = self.catalogue.held() else {
            return;
        };
        let Some(patch) = held.index().get(id) else {
            return;
        };
        match held.program(patch) {
            Ok(program) => {
                self.trying = Some(id.to_owned());
                let name = program.name().as_str().trim().to_owned();
                let maker = patch.author.clone();
                self.patch.assume(program.clone());
                // The shelf is deliberately untouched, and so is its `loaded`:
                // what is sounding did not come off it, and a librarian that
                // lit a row nobody had loaded would be lying about where this
                // sound is.
                if self.link.is_some() {
                    self.ask(Command::LoadProgram(Box::new(program)));
                    self.say(format!("Trying {name} by {maker}. Nothing is kept."));
                } else {
                    self.say(format!(
                        "{name} by {maker} is on the screen. Nothing is open to send it to."
                    ));
                }
            }
            Err(trouble) => self.say(format!("{} would not open: {trouble}", patch.name)),
        }
    }

    /// Puts one shared patch on the shelf.
    fn shelve_patch(&mut self, id: &str) {
        let Some(held) = self.catalogue.held() else {
            return;
        };
        let Some(patch) = held.index().get(id) else {
            return;
        };
        match held.program(patch) {
            Ok(program) => {
                let name = format!("{} - {}", patch.name, patch.author);
                match crate::shelf::patch_to_syx(&program) {
                    Ok(bytes) => self.shelve(&name, &bytes),
                    Err(trouble) => self.say(format!("{name} would not be written: {trouble}")),
                }
            }
            Err(trouble) => self.say(format!("{} would not open: {trouble}", patch.name)),
        }
    }

    /// Puts every shared patch the search left on the shelf, in index order.
    ///
    /// A filtered catalogue becomes a pack, which is the one thing a `.syx`
    /// file is for: the twelve pads somebody searched for go onto the shelf as
    /// twelve programs and out of the librarian as one file, in the order the
    /// catalogue holds them.
    fn shelve_showing(&mut self) {
        let Some(held) = self.catalogue.held() else {
            return;
        };
        let showing = held.showing(&self.looking);
        if showing.is_empty() {
            self.say("Nothing to put on the shelf.".to_owned());
            return;
        }
        let mut bytes = Vec::new();
        let mut missed = 0_usize;
        for patch in &showing {
            match held.program(patch).and_then(|program| {
                crate::shelf::patch_to_syx(&program).map_err(|error| error.to_string())
            }) {
                Ok(one) => bytes.extend_from_slice(&one),
                Err(_) => missed = missed.saturating_add(1),
            }
        }
        if bytes.is_empty() {
            self.say("None of those could be read.".to_owned());
            return;
        }
        let name = match self.looking.narrowed(By::Category) {
            Some(category) => category.to_owned(),
            None => "Shared patches".to_owned(),
        };
        self.shelve(&name, &bytes);
        if missed > 0 {
            self.say(format!("{missed} of those could not be read."));
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
        // Whatever was being tried is not what is sounding any more.
        self.trying = None;
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

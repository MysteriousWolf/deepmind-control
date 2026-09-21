//! What the librarian is holding, and where it came from.
//!
//! A shelf is the programs a `.syx` file turned out to contain, or the ones a
//! bank read off the synthesizer produced. It is the librarian's whole model:
//! everything the view draws and everything the file writer writes is here, and
//! nothing in it opens a file or talks to a port.
//!
//! # A slot is what the dump said, not where the shelf put it
//!
//! A stored program dump names the bank and program it came from, and an edit
//! buffer dump names nothing, because the edit buffer is where a sound is played
//! rather than where one is kept. Both are worth having on a shelf, so a
//! [`Held`] carries `Option<Slot>` and the one that names nothing is drawn as
//! the one that names nothing. It is also how a patch this application wrote
//! reads back: one sound, belonging nowhere in particular, which is exactly what
//! it was when it was saved.
//!
//! # Reading a file is lenient
//!
//! A file is untrusted input and the library walks it leniently: bytes between
//! frames are skipped, and a frame that does not parse is handed back as the
//! error it failed with so that the walk can go on. One bad dump in a pack is
//! one missing program and not a refused file, so the count of what could not be
//! read is kept and said out loud rather than thrown away.

use core::fmt;

use deepmind_host::Outcome;
use deepmind_midi::ids::{Bank, DeviceId, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::Program;
use deepmind_midi::sysex::inquiry::Version;
use deepmind_midi::syx::{self, File, Writer};

/// The device ID every file this application writes is addressed to.
///
/// Broadcast, because a file is traded and a device ID is local. A pack
/// addressed to unit 3 is a pack that the fourth `DeepMind` in the world loads
/// and nobody else's does, which is not a property a librarian should hand
/// somebody by accident.
const FOR_ANY_UNIT: DeviceId = DeviceId::Broadcast;

/// Where the programs on a shelf came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A `.syx` file, under the name it has on this machine.
    File(String),
    /// A bank read off the synthesizer.
    Bank(Bank),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(name) => f.write_str(name),
            Self::Bank(bank) => write!(f, "bank {bank}, off the synthesizer"),
        }
    }
}

/// The order the programs on a shelf are laid out in.
///
/// Slot order is what a bank *is*, and it is where this starts, because a
/// librarian that reordered a pack would be describing its own list rather than
/// the instrument's memory. The other two are for the other question somebody
/// brings to a shelf of 128 sounds: not "what is in B12" but "where did that
/// pad go".
///
/// Nothing here rewrites the shelf. It is which way round the same programs are
/// drawn, so what a save writes is what was opened whatever this is sitting on
/// — the order a pack is written in is the order its slots are in, and a pack
/// written out in alphabetical order would be a pack that loads into the wrong
/// slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order {
    /// Where the instrument keeps them, which is how they arrived.
    #[default]
    Slot,
    /// By name, for the sound somebody remembers the name of.
    Name,
    /// By the category the program calls itself, and by name inside it.
    ///
    /// The instrument's own word for what a sound is, out of the library's
    /// table for the firmware that answered. A program whose category this
    /// firmware has no name for sorts last rather than first: a run of
    /// unnamed things at the head of a list is a list that looks broken.
    Category,
}

impl Order {
    /// What the press that chooses this order is printed with.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Slot => "Slot",
            Self::Name => "Name",
            Self::Category => "Category",
        }
    }
}

/// The three of them, in the order they are offered.
pub const ORDERS: [Order; 3] = [Order::Slot, Order::Name, Order::Category];

/// One program on the shelf, and where it says it belongs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The slot the dump named, absent for an edit buffer dump.
    pub slot: Option<Slot>,
    /// The program itself.
    pub program: Program,
}

impl Held {
    /// Returns how the slot is written on the front panel, or a dash for a
    /// program that names none.
    #[must_use]
    pub fn address(&self) -> String {
        self.slot
            .map_or_else(|| "\u{2014}".to_owned(), |slot| slot.to_string())
    }

    /// Returns the program's name, with the padding the instrument stores taken
    /// off.
    #[must_use]
    pub fn name(&self) -> String {
        self.program.name().as_str().trim().to_owned()
    }

    /// Returns what the program calls itself, where this firmware has a word
    /// for it.
    ///
    /// The library's own table, read for the firmware that answered, which is
    /// the rule every other name in this window is drawn under. A program
    /// sitting on a value no table names has no category rather than a made-up
    /// one, and is drawn as having none.
    #[must_use]
    pub fn category(&self, firmware: Version) -> Option<&'static str> {
        let value = self.program.get(ParamId::ProgramCategory);
        ParamId::ProgramCategory.label_for(u16::from(value), firmware)
    }

    /// Returns whether `words` are anywhere in what this program says about
    /// itself.
    ///
    /// Its name, the slot it names and its category, folded to one case, which
    /// is the whole of what a shelf knows about a sound without opening it.
    /// Typing `b1` finds `B1` and `B10` through `B12`, and typing `pad` finds
    /// every pad by name and every program whose category is `Pad`, which are
    /// two different questions with one answer somebody is happy with.
    ///
    /// Empty words match everything: a search box nobody has typed in is not a
    /// filter.
    #[must_use]
    pub fn matches(&self, words: &str, firmware: Version) -> bool {
        if words.is_empty() {
            return true;
        }
        let words = words.to_lowercase();
        self.name().to_lowercase().contains(&words)
            || self.address().to_lowercase().contains(&words)
            || self
                .category(firmware)
                .is_some_and(|category| category.to_lowercase().contains(&words))
    }
}

/// A bank read on its way in.
///
/// One request and up to 128 answers, about twelve seconds of a MIDI cable. The
/// device thread reports each dump as it lands and stays answerable throughout,
/// which is what makes this a progress bar rather than a freeze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transfer {
    /// Bank being read.
    pub bank: Bank,
    /// Dumps that have arrived.
    pub received: u16,
    /// Dumps the run asked for.
    pub expected: u16,
}

impl Transfer {
    /// Returns how much of the run has arrived, between zero and one.
    ///
    /// A run that asked for nothing is finished, which is the honest answer to
    /// a division this would otherwise have to refuse.
    #[must_use]
    pub fn fraction(self) -> f32 {
        if self.expected == 0 {
            return 1.0;
        }
        f32::from(self.received) / f32::from(self.expected)
    }
}

/// The programs the librarian has in front of it.
///
/// Filled by opening a file or by reading a bank, and emptied by doing either
/// again. Nothing here sends anything or touches a disk: a shelf is changed by
/// what happened, and what happens next is the application's business.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shelf {
    /// Where what is on the shelf came from.
    source: Option<Source>,
    /// What is on it, in the order a file gave them or in slot order.
    held: Vec<Held>,
    /// Frames the library could not read, which are said out loud rather than
    /// dropped quietly.
    skipped: usize,
    /// Which one was last sent to the edit buffer.
    loaded: Option<usize>,
    /// A bank read in flight.
    transfer: Option<Transfer>,
    /// What has been typed into the search field.
    ///
    /// Browsing state rather than a fact about the sounds: it decides which of
    /// them are drawn and nothing else. Nothing that leaves this shelf — a save,
    /// a load, the count of what it holds — reads it, because a librarian that
    /// wrote out what was on the screen rather than what is on the shelf would
    /// turn a search into a deletion.
    query: String,
    /// Which way round they are drawn.
    order: Order,
}

impl Shelf {
    /// An empty shelf.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            source: None,
            held: Vec::new(),
            skipped: 0,
            loaded: None,
            transfer: None,
            query: String::new(),
            order: Order::Slot,
        }
    }

    /// Returns where what is on the shelf came from.
    #[must_use]
    pub const fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    /// Returns what is on the shelf.
    #[must_use]
    pub fn held(&self) -> &[Held] {
        &self.held
    }

    /// Returns how many frames of the last file could not be read.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.skipped
    }

    /// Returns which program was last sent to the edit buffer.
    #[must_use]
    pub const fn loaded(&self) -> Option<usize> {
        self.loaded
    }

    /// Returns the bank read in flight, if one is.
    #[must_use]
    pub const fn transfer(&self) -> Option<Transfer> {
        self.transfer
    }

    /// Returns whether there is nothing on the shelf.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    /// Returns what has been typed into the search field.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Narrows what is drawn to the programs `words` are anywhere in.
    pub fn search(&mut self, words: String) {
        self.query = words;
    }

    /// Returns which way round the shelf is drawn.
    #[must_use]
    pub const fn order(&self) -> Order {
        self.order
    }

    /// Draws it the other way round.
    pub fn sort_by(&mut self, order: Order) {
        self.order = order;
    }

    /// Returns what a search and an order leave to draw, each with the place it
    /// holds on the shelf.
    ///
    /// The place travels with the program because it is what everything else
    /// says a program *is*: a load names one by where it sits on the shelf, and
    /// a list that renumbered its own rows would send the wrong sound to the
    /// synthesizer the first time somebody sorted it by name.
    ///
    /// The shelf itself does not move. This is a view of it, built when it is
    /// drawn and thrown away after, which is what keeps a save writing slots in
    /// slot order however the screen is sorted.
    #[must_use]
    pub fn showing(&self, firmware: Version) -> Vec<(usize, &Held)> {
        let mut showing: Vec<(usize, &Held)> = self
            .held
            .iter()
            .enumerate()
            .filter(|(_, held)| held.matches(&self.query, firmware))
            .collect();
        match self.order {
            // Already in it: a bank arrives in slot order and a file is held in
            // the order the file gave it, which for a pack is the same thing.
            Order::Slot => {}
            Order::Name => showing.sort_by_key(|(_, held)| held.name().to_lowercase()),
            // A program this firmware has no word for sorts last rather than
            // first, which is what the tilde is: the last printable character,
            // so an unnamed category sorts after every named one without a
            // second comparison written out.
            Order::Category => showing.sort_by_key(|(_, held)| {
                (
                    held.category(firmware)
                        .map_or_else(|| "\u{7e}".to_owned(), str::to_lowercase),
                    held.name().to_lowercase(),
                )
            }),
        }
        showing
    }

    /// Puts the programs a `.syx` file holds on the shelf, under its name.
    ///
    /// Everything the library can read, in the order the file gives them: a pack
    /// is already in slot order and a file somebody assembled by hand is in the
    /// order they assembled it, neither of which this is entitled to improve on.
    /// Frames it cannot read are counted rather than refused.
    pub fn open(&mut self, name: String, bytes: &[u8]) {
        let mut held = Vec::new();
        let mut skipped = 0_usize;
        for entry in File::new(bytes).programs() {
            match entry {
                Ok(entry) => held.push(Held {
                    slot: entry.slot,
                    program: entry.program,
                }),
                Err(_) => skipped = skipped.saturating_add(1),
            }
        }
        *self = Self {
            source: Some(Source::File(name)),
            held,
            skipped,
            loaded: None,
            transfer: None,
            // The words were about the pack that was on the shelf, and it is
            // not on it any more. A filter that survived the file would be a
            // window that opened a pack of 128 and showed three of them, with
            // the reason five lines up the page.
            query: String::new(),
            // Which way round somebody likes to read a shelf is about them
            // rather than about what is on it, so it survives.
            order: self.order,
        };
    }

    /// Clears the shelf and starts a bank read onto it.
    pub fn begin(&mut self, bank: Bank, expected: u16) {
        *self = Self {
            source: Some(Source::Bank(bank)),
            held: Vec::new(),
            skipped: 0,
            loaded: None,
            transfer: Some(Transfer {
                bank,
                received: 0,
                expected,
            }),
            query: String::new(),
            order: self.order,
        };
    }

    /// Takes one stored program dump, wherever it arrived from.
    ///
    /// Kept in slot order rather than in arrival order, because a bank is read
    /// in order and a shelf that draws it out of order would be describing the
    /// transfer instead of the instrument. A slot that arrives twice keeps the
    /// second one, as loading a pack into a unit would.
    pub fn arrived(&mut self, slot: Slot, program: Program) {
        let held = Held {
            slot: Some(slot),
            program,
        };
        match self
            .held
            .iter()
            .position(|other| other.slot.is_some_and(|other| other >= slot))
        {
            Some(at) if self.held.get(at).and_then(|other| other.slot) == Some(slot) => {
                if let Some(there) = self.held.get_mut(at) {
                    *there = held;
                }
            }
            Some(at) => self.held.insert(at, held),
            None => self.held.push(held),
        }
    }

    /// Notes how far the bank read has got.
    ///
    /// Ignored when nothing is in flight: a progress report for a run that has
    /// already ended is a late message and not a new transfer.
    pub fn advance(&mut self, received: u16, expected: u16) {
        if let Some(transfer) = self.transfer.as_mut() {
            transfer.received = received;
            transfer.expected = expected;
        }
    }

    /// Ends the bank read, keeping whatever arrived.
    ///
    /// A run that was called off or went quiet leaves a partial shelf, which is
    /// worth more than nothing and is what the source line says it is.
    pub fn finish(&mut self, _outcome: Outcome) {
        self.transfer = None;
    }

    /// Takes one program off the shelf to be played, and remembers which.
    ///
    /// The program is cloned rather than moved: what is on a shelf stays on it,
    /// and a librarian that emptied a slot every time somebody auditioned it
    /// would be a librarian nobody trusted with a backup.
    pub fn load(&mut self, index: usize) -> Option<Program> {
        let program = self.held.get(index).map(|held| held.program.clone())?;
        self.loaded = Some(index);
        Some(program)
    }

    /// Puts a different program in one of the shelf's places.
    ///
    /// In place, keeping the slot: updating a sound to a newer version of
    /// itself is not moving it, and a librarian that rearranged a bank while
    /// updating one of its programs would be a librarian nobody could hand a
    /// backup to. Returns whether there was a place to put it.
    pub fn replace(&mut self, index: usize, program: Program) -> bool {
        match self.held.get_mut(index) {
            Some(held) => {
                held.program = program;
                true
            }
            None => false,
        }
    }

    /// Writes the shelf out as a `.syx` file.
    ///
    /// One dump per program, in the order they are held, which is the whole
    /// format: there is no header, no index and no trailer, so a pack read out
    /// of a synthesizer and written here is a file any other editor can read.
    /// A program that names a slot is written as the stored dump for that slot
    /// and one that names none is written as an edit buffer dump, so what went
    /// on the shelf is what comes off it.
    ///
    /// # Errors
    ///
    /// Returns whatever the library refused to pack.
    pub fn to_syx(&self) -> deepmind_midi::Result<Vec<u8>> {
        let mut out = vec![0; self.held.len().saturating_mul(syx::MAX_PROGRAM_FRAME_LEN)];
        let mut writer = Writer::new(&mut out, FOR_ANY_UNIT);
        for held in &self.held {
            match held.slot {
                Some(slot) => writer.push_program(slot.bank, slot.number, &held.program)?,
                None => writer.push_edit_buffer(&held.program)?,
            };
        }
        let written = writer.finish();
        out.truncate(written);
        Ok(out)
    }

    /// Returns what to call the file this shelf would be saved as.
    #[must_use]
    pub fn file_name(&self) -> String {
        match self.source.as_ref() {
            Some(Source::File(name)) => name.clone(),
            Some(Source::Bank(bank)) => format!("bank-{}.syx", bank.letter().to_lowercase()),
            None => "pack.syx".to_owned(),
        }
    }
}

/// Writes one sound out as a `.syx` file.
///
/// An edit buffer dump, because a sound taken out of the edit buffer belongs
/// nowhere in particular and a file that claimed a slot for it would be
/// inventing one. It reads back onto a shelf as the program it is, and loads
/// into any `DeepMind` without storing itself anywhere, which is the whole of
/// what this application is entitled to do to a synthesizer.
///
/// # Errors
///
/// Returns whatever the library refused to pack.
pub fn patch_to_syx(program: &Program) -> deepmind_midi::Result<Vec<u8>> {
    let mut out = vec![0; syx::MAX_PROGRAM_FRAME_LEN];
    let mut writer = Writer::new(&mut out, FOR_ANY_UNIT);
    writer.push_edit_buffer(program)?;
    let written = writer.finish();
    out.truncate(written);
    Ok(out)
}

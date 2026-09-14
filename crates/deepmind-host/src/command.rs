//! What the interface asks the device thread to do.

use core::fmt;

use deepmind_midi::ids::{Bank, ProgramNumber, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::Program;

/// One thing to do, sent to the device thread.
///
/// This vocabulary is this crate's and not the library's. The library's
/// [`Device`](deepmind_midi::Device) never leaves the thread that owns it, so
/// what crosses the channel is an intention, and what comes back is a
/// [`Event`](crate::Event).
///
/// Nothing here writes a program into the synthesizer, because the manual
/// describes no message that does. The librarian's output is a file, and storing
/// a sound into a slot is done at the panel with the instrument's own WRITE.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Command {
    /// Ask who is there, which is the only thing that reports firmware.
    ///
    /// Sent once by the thread itself as soon as the port is open; sending it
    /// again is how a host asks a synthesizer that was switched on later.
    Identify,
    /// Read the edit buffer: the program as it currently sounds.
    ///
    /// This is what turns an assumed program into a confirmed one, and is asked
    /// for after a load, on reconnect and when the user asks. Never on a timer.
    ReadEditBuffer,
    /// Read one stored program.
    ReadProgram(Slot),
    /// Read a run of stored programs, answered one dump at a time.
    ///
    /// A whole bank is 128 dumps and about thirty-five kilobytes, which is
    /// twelve seconds of a MIDI cable. The thread reports
    /// [`Event::Progress`](crate::Event::Progress) as they arrive and stays
    /// answerable throughout, so the wait is a progress bar rather than a
    /// freeze.
    ReadBank {
        /// Bank to read.
        bank: Bank,
        /// First program of the run.
        first: ProgramNumber,
        /// Last program of the run, whose dump ends it.
        last: ProgramNumber,
    },
    /// Put one parameter somewhere.
    ///
    /// Coalesced rather than queued: the thread keeps one pending value per
    /// parameter and sends the newest at a fixed rate, so a knob drag costs what
    /// the wire can carry rather than what the mouse produced. The last value of
    /// a drag is always sent, because dropping that one is a desynchronised
    /// instrument.
    SetParameter {
        /// Parameter to move.
        parameter: ParamId,
        /// Where to move it, as the program byte it is stored as.
        value: u8,
    },
    /// Make the synthesizer sound like this program.
    ///
    /// Sent as the difference against what the synthesizer is believed to hold,
    /// never as all 242 parameters: the whole program is 2904 bytes, a little
    /// under a second of solid MIDI, and a patch change that takes a second is
    /// one the player notices.
    LoadProgram(Box<Program>),
    /// Stop what is in flight.
    ///
    /// Ends a bank read and drops the parameter edits that have not gone yet.
    /// What already went stays where it went: the synthesizer has it, and this
    /// crate has no message that would take it back. Dumps the unit is already
    /// sending keep arriving as ordinary events, because nothing in the protocol
    /// interrupts a unit mid-transfer.
    Cancel,
    /// Put the port down and end the thread.
    ///
    /// Dropping the [`Link`](crate::Link) does this too.
    Stop,
}

impl Command {
    /// Returns which command this is, without its payload.
    ///
    /// What a refusal names, since a refused
    /// [`LoadProgram`](Command::LoadProgram) should not carry 242 bytes back
    /// across the channel to say so.
    #[must_use]
    pub const fn kind(&self) -> CommandKind {
        match self {
            Self::Identify => CommandKind::Identify,
            Self::ReadEditBuffer => CommandKind::ReadEditBuffer,
            Self::ReadProgram(_) => CommandKind::ReadProgram,
            Self::ReadBank { .. } => CommandKind::ReadBank,
            Self::SetParameter { .. } => CommandKind::SetParameter,
            Self::LoadProgram(_) => CommandKind::LoadProgram,
            Self::Cancel => CommandKind::Cancel,
            Self::Stop => CommandKind::Stop,
        }
    }
}

/// A [`Command`] with its payload left off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CommandKind {
    /// [`Command::Identify`].
    Identify,
    /// [`Command::ReadEditBuffer`].
    ReadEditBuffer,
    /// [`Command::ReadProgram`].
    ReadProgram,
    /// [`Command::ReadBank`].
    ReadBank,
    /// [`Command::SetParameter`].
    SetParameter,
    /// [`Command::LoadProgram`].
    LoadProgram,
    /// [`Command::Cancel`].
    Cancel,
    /// [`Command::Stop`].
    Stop,
}

impl fmt::Display for CommandKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identify => f.write_str("device inquiry"),
            Self::ReadEditBuffer => f.write_str("edit buffer read"),
            Self::ReadProgram => f.write_str("program read"),
            Self::ReadBank => f.write_str("bank read"),
            Self::SetParameter => f.write_str("parameter edit"),
            Self::LoadProgram => f.write_str("program load"),
            Self::Cancel => f.write_str("cancel"),
            Self::Stop => f.write_str("stop"),
        }
    }
}

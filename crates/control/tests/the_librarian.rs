//! The librarian: a shelf filled from a file or from a synthesizer, browsed,
//! and played one program at a time.
//!
//! The property a librarian is for is that a patch which leaves this application
//! loses nothing on the way out, so most of what is below is a round trip: bytes
//! in, programs on a shelf, bytes out, and the same programs back. The only file
//! format is the protocol's, which is what makes that testable at all — there is
//! no project format to lose anything in.
//!
//! The rest is the two claims the window makes about a shelf. That a program
//! taken off it is assumed and not confirmed, because nothing has heard the
//! synthesizer play it. And that a bank read is a progress bar rather than a
//! freeze, which is only true if the window keeps drawing while it runs.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use control::{App, Message, Shelf, Source, View};
use control_ui::Confidence;
use deepmind_host::PortRef;
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::syx::bank_to_vec;

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long it waits for a request that nothing is going to answer.
///
/// The device thread times a bank run by the gap between its dumps, and the
/// default is two seconds. A simulated unit with nothing stored answers a bank
/// request with silence, which is the same silence a switched-off synthesizer
/// gives and is exactly the path a progress bar has to survive.
const SILENCE: Duration = Duration::from_secs(8);

/// Ticks the window until `ready`, which is what a person staring at it does.
fn until<F: Fn(&App) -> bool>(app: &mut App, patience: Duration, what: &str, ready: F) {
    let deadline = Instant::now() + patience;
    while Instant::now() < deadline {
        app.update(Message::Tick);
        if ready(app) {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!(
        "no {what} in {patience:?}; the window says: {}",
        app.status()
    );
}

/// A program with a name on it and one parameter moved somewhere recognisable.
fn sound(name: &str, cutoff: u8) -> Program {
    let mut program = Program::new(ProtocolVersion::V7);
    program.set_name(ProgramName::new(name).expect("a name the instrument can hold"));
    program.set_clamped(ParamId::VcfFrequency, cutoff);
    program
}

/// A pack of `count` programs, as a synthesizer would dump bank A.
fn pack(count: usize) -> Vec<u8> {
    let programs: Vec<Program> = (0..count)
        .map(|index| {
            sound(
                &format!("Patch {index}"),
                u8::try_from(index).unwrap_or(u8::MAX),
            )
        })
        .collect();
    bank_to_vec(
        DeviceId::Broadcast,
        Bank::A,
        ProgramNumber::FIRST,
        &programs,
    )
    .expect("a bank that fits in a bank")
}

/// Writes a pack somewhere this machine will let a test read it back.
///
/// The one route onto a shelf that does not go through a file chooser, which is
/// also how the application is handed a document by whatever opened it.
fn pack_on_disk(name: &str, count: usize) -> PathBuf {
    let path = std::env::temp_dir().join(name);
    fs::write(&path, pack(count)).expect("a file this machine will write");
    path
}

/// A window with a pack on the shelf and no port open.
fn browsing(name: &str, count: usize) -> App {
    let path = pack_on_disk(name, count);
    let mut app = App::new();
    app.update(Message::OpenNamed(path));
    app
}

#[test]
fn a_pack_is_every_program_it_holds() {
    let mut shelf = Shelf::new();
    shelf.open("factory.syx".to_owned(), &pack(128));

    assert_eq!(shelf.held().len(), 128, "a bank is 128 programs");
    assert_eq!(shelf.skipped(), 0, "nothing in it was unreadable");
    assert_eq!(
        shelf.source(),
        Some(&Source::File("factory.syx".to_owned()))
    );

    let first = shelf.held().first().expect("the first program");
    assert_eq!(first.slot, Some(Slot::new(Bank::A, ProgramNumber::FIRST)));
    assert_eq!(first.name(), "Patch 0");
    assert_eq!(first.address(), "A1", "the address the front panel shows");

    let last = shelf.held().last().expect("the last program");
    assert_eq!(last.slot, Some(Slot::new(Bank::A, ProgramNumber::LAST)));
    assert_eq!(last.name(), "Patch 127");
}

#[test]
fn a_shelf_written_out_reads_back_the_same() {
    let mut shelf = Shelf::new();
    shelf.open("factory.syx".to_owned(), &pack(16));
    let written = shelf.to_syx().expect("a shelf the library can pack");

    let mut again = Shelf::new();
    again.open("written.syx".to_owned(), &written);

    assert_eq!(
        again.held().len(),
        shelf.held().len(),
        "every program survived the trip out and back"
    );
    for (before, after) in shelf.held().iter().zip(again.held()) {
        assert_eq!(before.slot, after.slot, "and kept the slot it names");
        assert_eq!(
            before.program, after.program,
            "and every one of its 242 values"
        );
    }
}

#[test]
fn a_sound_saved_on_its_own_belongs_nowhere_in_particular() {
    let program = sound("Bass Sweep", 200);
    let bytes = control::patch_to_syx(&program).expect("a sound the library can pack");

    let mut shelf = Shelf::new();
    shelf.open("bass-sweep.syx".to_owned(), &bytes);

    let held = shelf.held().first().expect("the sound that was saved");
    assert_eq!(shelf.held().len(), 1, "one sound is one dump");
    assert_eq!(
        held.slot, None,
        "an edit buffer dump names no slot, because the edit buffer is not one"
    );
    assert_eq!(held.address(), "\u{2014}", "and is drawn as naming none");
    assert_eq!(held.program, program, "and is the sound that was saved");
}

#[test]
fn one_bad_frame_costs_one_program_and_not_the_file() {
    let mut bytes = pack(4);
    // Somewhere inside the second dump, which the library will refuse and walk
    // past rather than give up on.
    let middle = bytes.len() / 4 + 8;
    if let Some(byte) = bytes.get_mut(middle) {
        *byte = 0x7f;
    }

    let mut shelf = Shelf::new();
    shelf.open("bent.syx".to_owned(), &bytes);

    assert!(
        shelf.held().len() < 4,
        "the bent dump did not read as a program"
    );
    assert!(
        !shelf.is_empty(),
        "and the other dumps in the file were not lost with it"
    );
}

#[test]
fn a_bank_arrives_in_slot_order_however_it_arrives() {
    let mut shelf = Shelf::new();
    let b = Bank::from_letter('B').expect("bank B");
    shelf.begin(b, 3);
    for number in [2_u8, 0, 1] {
        let slot = Slot::new(
            b,
            ProgramNumber::new(number).expect("a number inside a bank"),
        );
        shelf.arrived(slot, sound(&format!("Out of order {number}"), number));
    }

    let order: Vec<String> = shelf.held().iter().map(control::Held::address).collect();
    assert_eq!(
        order,
        ["B1", "B2", "B3"],
        "in the order the panel shows them"
    );
}

#[test]
fn a_slot_that_arrives_twice_keeps_the_second_one() {
    let slot = Slot::new(Bank::A, ProgramNumber::FIRST);
    let mut shelf = Shelf::new();
    shelf.begin(Bank::A, 1);
    shelf.arrived(slot, sound("First", 10));
    shelf.arrived(slot, sound("Second", 20));

    assert_eq!(shelf.held().len(), 1, "one slot holds one program");
    assert_eq!(
        shelf.held().first().expect("the program in it").name(),
        "Second",
        "and it is the one that arrived last, as loading a pack would leave it"
    );
}

#[test]
fn a_program_taken_off_the_shelf_is_assumed_and_not_confirmed() {
    let mut app = browsing("load-me.syx", 8);
    app.update(Message::Load(3));

    assert_eq!(
        app.patch()
            .name()
            .map(|name| name.as_str().trim().to_owned()),
        Some("Patch 3".to_owned()),
        "the sound on the screen is the one that was picked"
    );
    assert_eq!(
        app.patch().confidence(),
        Confidence::Assumed,
        "and nothing has heard a synthesizer play it"
    );
    assert_eq!(
        app.patch().claim(ParamId::VcfFrequency),
        Confidence::Assumed,
        "which is true of every parameter in it and not only of the sound as a whole"
    );
    assert_eq!(app.shelf().loaded(), Some(3), "and the shelf says which");
}

#[test]
fn a_pack_can_be_read_with_nothing_plugged_in() {
    let app = browsing("browse-me.syx", 8);
    assert!(!app.is_connected(), "no port, and no need of one");
    assert_eq!(app.shelf().held().len(), 8);
    assert_eq!(app.view(), View::Library, "opening a file shows it");
}

#[test]
fn a_bank_read_is_a_progress_bar_and_not_a_freeze() {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    until(&mut app, PATIENCE, "an answer to the inquiry", |app| {
        app.identity().is_some()
    });

    app.update(Message::ReadBank);
    let transfer = app.shelf().transfer().expect("a transfer in flight");
    assert_eq!(transfer.bank, Bank::A);
    assert_eq!(transfer.expected, 128, "a whole bank was asked for");
    assert_eq!(transfer.received, 0, "and none of it has arrived yet");

    // The unit this is reading has nothing stored, which is the silence a
    // switched-off synthesizer gives. The window keeps drawing through it and
    // the run ends by itself rather than hanging.
    until(&mut app, SILENCE, "the run to end", |app| {
        app.shelf().transfer().is_none()
    });
    assert!(
        app.status().contains("0 of 128"),
        "and says how far it got: {}",
        app.status()
    );
}

#[test]
fn a_bank_read_can_be_called_off() {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    until(&mut app, PATIENCE, "an answer to the inquiry", |app| {
        app.identity().is_some()
    });

    app.update(Message::ReadBank);
    assert!(app.shelf().transfer().is_some(), "a transfer in flight");

    app.update(Message::Cancel);
    until(&mut app, PATIENCE, "the run to be called off", |app| {
        app.shelf().transfer().is_none()
    });
    assert!(
        app.status().contains("cancelled"),
        "and says so: {}",
        app.status()
    );
}

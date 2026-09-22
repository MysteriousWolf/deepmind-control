//! The librarian: a shelf filled from a file or from a synthesizer, browsed,
//! and played one program at a time.
//!
//! The property a librarian is for is that a patch which leaves this application
//! loses nothing on the way out, so most of what is below is a round trip: bytes
//! in, programs on a shelf, bytes out, and the same programs back. The only file
//! format is the protocol's, which is what makes that testable at all: there is
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

use control::{App, Message, Order, Shelf, Source, View};
use control_ui::Confidence;
use deepmind_host::PortRef;
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::{DEFAULT_FIRMWARE, ParamId};
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
    shelf.begin(b, 3, "09:12".to_owned());
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
    shelf.begin(Bank::A, 1, "09:12".to_owned());
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
fn a_search_narrows_what_is_drawn_and_never_what_is_held() {
    // The property the whole thing turns on. A shelf is what a save writes out,
    // so a filter that reached the shelf would be a filter that deleted 125
    // programs the first time somebody looked for one of the other three.
    let mut shelf = Shelf::new();
    shelf.open("factory.syx".to_owned(), &pack(16));
    let whole = shelf.to_syx().expect("a shelf the library can pack");

    shelf.search("patch 1".to_owned());

    assert_eq!(shelf.held().len(), 16, "the shelf still holds all of them");
    assert_eq!(
        shelf.showing(DEFAULT_FIRMWARE).len(),
        7,
        "and draws `Patch 1` and `Patch 10` through `Patch 15`"
    );
    assert_eq!(
        shelf.to_syx().expect("a shelf the library can pack"),
        whole,
        "and writes out the pack that was opened rather than the screen"
    );
}

#[test]
fn a_search_finds_the_slot_as_well_as_the_name() {
    // Two questions with one field, because they are the same question asked by
    // somebody who remembers a different thing about the same sound.
    let mut shelf = Shelf::new();
    shelf.open("factory.syx".to_owned(), &pack(128));

    shelf.search("b".to_owned());
    assert!(
        shelf.showing(DEFAULT_FIRMWARE).is_empty(),
        "nothing in bank A is in bank B, and nothing is called it"
    );

    shelf.search("a12".to_owned());
    let found: Vec<String> = shelf
        .showing(DEFAULT_FIRMWARE)
        .into_iter()
        .map(|(_, held)| held.address())
        .collect();
    assert_eq!(
        found,
        [
            "A12", "A120", "A121", "A122", "A123", "A124", "A125", "A126", "A127", "A128"
        ],
        "the slot typed, and every slot it is the beginning of"
    );
}

#[test]
fn a_shelf_sorted_by_name_still_loads_the_sound_that_was_pressed() {
    // What the place beside each program on a sorted list is for. The row is in
    // one order and the shelf is in another, and a load names where the sound
    // sits on the shelf: a list that numbered its own rows would send the wrong
    // sound the first time anybody sorted one.
    let mut app = browsing("sort-me.syx", 16);
    app.update(Message::SortBy(Order::Name));

    let showing = app.shelf().showing(app.firmware());
    let (index, held) = showing.get(2).copied().expect("a third row");
    assert_eq!(
        held.name(),
        "Patch 10",
        "`Patch 0`, `Patch 1` and then `Patch 10`, which is what sorting by name is"
    );
    assert_eq!(index, 10, "and it is the eleventh program on the shelf");

    app.update(Message::Load(index));
    assert_eq!(
        app.patch()
            .name()
            .map(|name| name.as_str().trim().to_owned()),
        Some("Patch 10".to_owned()),
        "and that is the sound the synthesizer was given"
    );
}

#[test]
fn a_new_pack_arrives_without_the_last_one_s_search_on_it() {
    // A filter is about what is on the shelf, and what is on the shelf has just
    // been replaced. Keeping it would open a pack of 128 showing three, with
    // the reason for it in a field somebody typed in five minutes ago.
    let mut shelf = Shelf::new();
    shelf.open("first.syx".to_owned(), &pack(8));
    shelf.search("patch 3".to_owned());
    shelf.sort_by(Order::Name);

    shelf.open("second.syx".to_owned(), &pack(8));

    assert_eq!(shelf.query(), "", "the words were about the other pack");
    assert_eq!(
        shelf.showing(DEFAULT_FIRMWARE).len(),
        8,
        "so all of this one is on the screen"
    );
    assert_eq!(
        shelf.order(),
        Order::Name,
        "and which way round somebody reads a shelf is about them, not about it"
    );
}

#[test]
fn a_program_says_what_it_calls_itself() {
    // The library's own table, for the firmware that answered, which is the
    // rule every other name in this window is drawn under.
    let mut program = sound("Warm Pad", 90);
    let category = program.set_clamped(ParamId::ProgramCategory, 1);
    let named = ParamId::ProgramCategory.label_for(u16::from(category), DEFAULT_FIRMWARE);

    let mut shelf = Shelf::new();
    shelf.open(
        "pad.syx".to_owned(),
        &control::patch_to_syx(&program).expect("a sound the library can pack"),
    );
    let held = shelf.held().first().expect("the sound that was saved");

    assert_eq!(
        held.category(DEFAULT_FIRMWARE),
        named,
        "what the shelf says it is, is what the library says it is"
    );
    if let Some(named) = named {
        shelf.search(named.to_owned());
        assert_eq!(
            shelf.showing(DEFAULT_FIRMWARE).len(),
            1,
            "and typing it finds the sound"
        );
    }
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
    // Connecting now includes reading the sound the synthesizer is making, so
    // a bank read starts from a window that has already been answered twice.
    // Starting one before that dump lands means its reply arrives after the
    // cancel and takes the status line the cancel had just written.
    until(&mut app, PATIENCE, "an answer and the sound", |app| {
        app.identity().is_some() && app.patch().confidence().is_confirmed()
    });

    app.update(Message::ReadAll);
    let transfer = app.shelf().transfer().expect("a transfer in flight");
    assert_eq!(transfer.bank, Bank::A, "starting from the first bank");
    assert_eq!(
        transfer.expected, 1024,
        "every bank was asked for, not one: eight of a hundred and twenty-eight"
    );
    assert_eq!(transfer.received, 0, "and none of it has arrived yet");

    // The unit this is reading has nothing stored, which is the silence a
    // switched-off synthesizer gives. The window keeps drawing through it and
    // the run ends by itself rather than hanging.
    until(&mut app, SILENCE, "the run to end", |app| {
        app.shelf().transfer().is_none()
    });
    assert!(
        app.status().contains("0 programs"),
        "and says how far it got: {}",
        app.status()
    );
}

#[test]
fn a_bank_read_can_be_called_off() {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    // Connecting now includes reading the sound the synthesizer is making, so
    // a bank read starts from a window that has already been answered twice.
    // Starting one before that dump lands means its reply arrives after the
    // cancel and takes the status line the cancel had just written.
    until(&mut app, PATIENCE, "an answer and the sound", |app| {
        app.identity().is_some() && app.patch().confidence().is_confirmed()
    });

    app.update(Message::ReadAll);
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

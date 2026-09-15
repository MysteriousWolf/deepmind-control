//! The effects, driven against the library's simulated synthesizer.
//!
//! The panel that needs the library most. `FX 1 Param 3` is `Size` on a Room
//! Reverb and `Depth` on a Phaser, and which of the two a window is drawing
//! depends on a byte somewhere else in the same program — read for the firmware
//! that answered the inquiry, because firmware 1.1 inserted an algorithm rather
//! than appending one.
//!
//! What has to stay true across that is what this asserts. An algorithm chosen
//! here is the algorithm the synthesizer reports; every one the firmware offers
//! can be selected and comes back as itself; and an engine still holds twelve
//! bytes whatever it is running, so the seven a TC Deep Reverb does not use are
//! still reachable, still editable and still survive the wire. A panel that
//! quietly stopped sending them would be a panel that loses part of a sound the
//! moment somebody changes an effect.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::thread;
use std::time::{Duration, Instant};

use control::{App, Message};
use control_ui::Confidence;
use deepmind_host::PortRef;
use deepmind_midi::effect::{Algorithm, Engine, SLOTS_PER_ENGINE};
use deepmind_midi::param::{Group, Kind, ParamId};
use deepmind_midi::sysex::inquiry::Version;

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long it waits for a dump behind the edits an engine costs.
///
/// Fourteen parameters at the host crate's five milliseconds apiece, and a read
/// waits for the edits it would otherwise contradict. This is that, with room
/// for a loaded machine.
const ROUND_TRIP: Duration = Duration::from_secs(10);

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

/// A window with the sound read into it, showing the effects.
fn read() -> App {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    assert!(app.is_connected(), "{}", app.status());
    until(&mut app, PATIENCE, "answer to the inquiry", |app| {
        app.identity().is_some()
    });
    app.update(Message::Read);
    until(&mut app, PATIENCE, "edit buffer", |app| {
        app.patch().is_known()
    });
    app.update(Message::Ui(control_ui::Message::Show(Group::Effects)));
    app
}

/// Moves a parameter the way the panel does.
fn edit(app: &mut App, parameter: ParamId, value: u8) {
    app.update(Message::Ui(control_ui::Message::Edit { parameter, value }));
}

/// The byte that selects `name` on the firmware this window is reading for.
fn algorithm(app: &App, name: &str) -> u8 {
    Algorithm::by_name(name)
        .expect("an algorithm of that name")
        .value_for(app.firmware())
        .expect("this firmware offers it")
}

/// Every byte the `FX Type` table names, for the firmware that answered.
fn offered(firmware: Version) -> Vec<u8> {
    let Kind::Enumerated(table) = Engine::One.algorithm_parameter().kind() else {
        panic!("an engine's type is chosen from a named set")
    };
    table
        .table_for(firmware)
        .entries
        .iter()
        .map(|entry| u8::try_from(entry.value).expect("35 algorithms fit in a byte"))
        .collect()
}

#[test]
fn an_algorithm_chosen_here_is_the_one_the_synthesizer_reports() {
    let mut app = read();
    let room = algorithm(&app, "RoomRev");

    edit(&mut app, Engine::One.algorithm_parameter(), room);
    assert_eq!(
        app.patch().value(Engine::One.algorithm_parameter()),
        Some(room)
    );
    assert_eq!(
        app.patch().claim(Engine::One.algorithm_parameter()),
        Confidence::Assumed,
        "the window put it there and nothing has confirmed it"
    );

    app.update(Message::Read);
    until(&mut app, ROUND_TRIP, "dump behind the edit", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    assert_eq!(
        app.patch().value(Engine::One.algorithm_parameter()),
        Some(room),
        "the engine came back running something else"
    );
    // And the byte that came back names the panel this window draws.
    assert_eq!(
        Algorithm::for_value(room, app.firmware()).map(|algorithm| algorithm.full_name),
        Some("Room Reverb")
    );
}

#[test]
fn every_algorithm_this_firmware_offers_can_be_selected() {
    let mut app = read();
    let offered = offered(app.firmware());

    for value in offered.iter().copied() {
        edit(&mut app, Engine::Two.algorithm_parameter(), value);

        assert_eq!(
            app.patch().value(Engine::Two.algorithm_parameter()),
            Some(value),
            "FX 2 would not run the algorithm {value} selects"
        );
        assert!(
            Algorithm::for_value(value, app.firmware()).is_some(),
            "{value} is in the type table and has no panel"
        );
    }
    assert_eq!(offered.len(), Algorithm::all().len());
}

#[test]
fn an_engine_holds_twelve_bytes_whatever_it_is_running() {
    let mut app = read();
    // TC Deep Reverb uses five of its twelve, and the other seven are still in
    // the program, still addressable from the modulation matrix, and still this
    // window's to send.
    let deep = algorithm(&app, "TC-DeepVRB");
    edit(&mut app, Engine::Three.algorithm_parameter(), deep);
    let unused = Algorithm::for_value(deep, app.firmware()).expect("a panel for it");
    assert_eq!(unused.slots.len(), 5);

    let asked: Vec<(ParamId, u8)> = Engine::Three
        .slot_parameters()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, parameter)| {
            let value = u8::try_from(index + 1).expect("twelve fits in a byte");
            edit(&mut app, parameter, value);
            assert_eq!(app.patch().value(parameter), Some(value), "{parameter}");
            (parameter, value)
        })
        .collect();
    assert_eq!(asked.len(), SLOTS_PER_ENGINE);

    app.update(Message::Read);
    until(&mut app, ROUND_TRIP, "dump behind the edits", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    for (parameter, value) in asked {
        assert_eq!(
            app.patch().value(parameter),
            Some(value),
            "{parameter} came back as something else, and no panel names it"
        );
    }
}

#[test]
fn the_effects_are_as_confirmed_as_the_least_confirmed_of_them() {
    let mut app = read();
    assert_eq!(app.patch().claim_of(Group::Effects), Confidence::Confirmed);

    // One slot of one engine, in a panel of fifty-eight parameters.
    edit(&mut app, ParamId::Fx4Param12, 64);

    assert_eq!(app.patch().claim_of(Group::Effects), Confidence::Assumed);
    assert_eq!(app.patch().claim_of(Group::Vcf), Confidence::Confirmed);
}

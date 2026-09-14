//! The window's state machine, driven against the library's simulated
//! synthesizer.
//!
//! No window and no hardware: what these prove is the part of stage 2 that is
//! not pixels. A port is chosen and opened, a synthesizer answers it, the edit
//! buffer comes back and becomes the sound on the screen, an edit moves one
//! parameter and says so, and reading again turns the claim back into a fact.
//!
//! The simulator is in the port list like any other choice, which is what makes
//! all of this runnable on a machine with no MIDI backend at all.

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
use deepmind_midi::param::ParamId;

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// Ticks the window until `ready`, which is what a person staring at it does.
fn until<F: Fn(&App) -> bool>(app: &mut App, what: &str, ready: F) {
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        app.update(Message::Tick);
        if ready(app) {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!(
        "no {what} in {PATIENCE:?}; the window says: {}",
        app.status()
    );
}

/// A window with the simulated synthesizer open on it.
fn connected() -> App {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    assert!(app.is_connected(), "{}", app.status());
    until(&mut app, "answer to the inquiry", |app| {
        app.identity().is_some()
    });
    app
}

/// A window with the sound read into it.
fn read() -> App {
    let mut app = connected();
    app.update(Message::Read);
    until(&mut app, "edit buffer", |app| app.patch().is_known());
    app
}

#[test]
fn the_simulator_is_a_port_you_can_choose() {
    let mut app = App::new();

    app.update(Message::Rescan);

    assert!(
        app.ports().contains(&PortRef::simulator()),
        "the simulated synthesizer is a choice like any other: {:?}",
        app.ports()
    );
    // Whether this machine has a MIDI backend at all is not this test's
    // business, and the window says so either way.
    assert!(app.chosen().is_some(), "something is picked to start with");
}

#[test]
fn a_port_that_answers_settles_the_firmware() {
    let app = connected();

    let identity = app.identity().expect("an identity");
    assert!(identity.firmware.is(1, 1), "{:?}", identity.firmware);
    assert_eq!(app.firmware(), identity.firmware);
    assert_eq!(
        app.channel().map(deepmind_midi::wire::Channel::number),
        Some(1)
    );
}

#[test]
fn nothing_is_editable_before_the_edit_buffer_is_read() {
    let app = connected();

    assert!(!app.patch().is_known());
    assert_eq!(app.patch().confidence(), Confidence::Unknown);
    assert_eq!(
        app.patch().claim(ParamId::VcfFrequency),
        Confidence::Unknown
    );
}

#[test]
fn the_edit_buffer_becomes_the_sound_on_the_screen() {
    let app = read();

    assert_eq!(app.patch().confidence(), Confidence::Confirmed);
    assert_eq!(
        app.patch()
            .name()
            .map(|name| name.as_str().trim().to_owned()),
        Some("Simulator".to_owned())
    );
    assert_eq!(
        app.patch().claim(ParamId::VcfFrequency),
        Confidence::Confirmed
    );
}

#[test]
fn an_edit_is_assumed_until_the_synthesizer_is_asked_again() {
    let mut app = read();

    app.update(Message::Ui(control_ui::Message::Edit {
        parameter: ParamId::VcfFrequency,
        value: 200,
    }));

    // The knob is where it was put, and the window says who put it there.
    assert_eq!(app.patch().value(ParamId::VcfFrequency), Some(200));
    assert_eq!(
        app.patch().claim(ParamId::VcfFrequency),
        Confidence::Assumed
    );
    // Nothing else moved, so nothing else lost its confirmation.
    assert_eq!(
        app.patch().claim(ParamId::VcfResonance),
        Confidence::Confirmed
    );

    app.update(Message::Read);
    until(&mut app, "second read", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    // The edit went out, the dump came back, and the claim is a fact.
    assert_eq!(app.patch().value(ParamId::VcfFrequency), Some(200));
    assert_eq!(
        app.patch().claim(ParamId::VcfFrequency),
        Confidence::Confirmed
    );
}

#[test]
fn closing_the_port_forgets_the_sound() {
    let mut app = read();

    app.update(Message::Disconnect);

    assert!(!app.is_connected());
    assert!(app.identity().is_none());
    // The next port may be another instrument, or the same one after somebody
    // spent the afternoon at its panel.
    assert!(!app.patch().is_known());
}

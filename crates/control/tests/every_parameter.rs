//! Every parameter the instrument has, driven against the library's simulated
//! synthesizer.
//!
//! Stage 2 proved one group end to end. What this proves is that there is
//! nothing special about that group: all 242 parameters are reachable from the
//! section bar, all of them move, and all of them survive the round trip out to
//! a synthesizer and back as the values that were sent.
//!
//! Drawn is not the same as editable, and a panel generated from the library's
//! table is exactly where a parameter with an awkward range hides: `Program
//! Transpose` runs 80 to 176 rather than from zero, an enumerated parameter
//! accepts only what its table names, and either of those turns into a control
//! that silently refuses the value somebody asked for. So the ranges come from
//! the library here too, and the assertion is that what went out came back.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::thread;
use std::time::{Duration, Instant};

use control::{App, Message};
use control_ui::{Confidence, first_section, sections};
use deepmind_host::PortRef;
use deepmind_midi::param::{Group, ParamId};

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long it waits for a dump behind 242 coalesced edits.
///
/// The host crate lets one parameter out every five milliseconds and a read
/// waits for the edits it would otherwise contradict, so a round trip over the
/// whole program is a second and a quarter of wire before the request is even
/// sent. This is that, with room for a loaded machine.
const ROUND_TRIP: Duration = Duration::from_secs(30);

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

/// A window with the sound read into it.
fn read() -> App {
    let mut app = App::new();
    app.update(Message::Choose(PortRef::simulator()));
    app.update(Message::Connect);
    assert!(app.is_connected(), "{}", app.status());
    until(&mut app, PATIENCE, "answer to the inquiry", |app| {
        app.identity().is_some()
    });
    // Connecting reads the sound: the window asks for the edit buffer the
    // moment a synthesizer answers the inquiry, so a second read here would put
    // a second dump on the wire. That matters beyond being wasteful: a test
    // that edits and then reads is asserting that the dump it waited for went
    // out *behind* its edit, and a stray earlier dump landing after the edit
    // carries the value from before it.
    until(&mut app, PATIENCE, "the sound the connection read", |app| {
        app.patch().is_known()
    });
    app
}

/// Somewhere else in a parameter's range than where it is now.
///
/// An end of it, so that the value is one the library accepts whatever the
/// parameter's range turns out to be, and the end it is not already sitting at,
/// so that asking for it is asking for a change.
fn elsewhere(app: &App, parameter: ParamId) -> u8 {
    let low = u8::try_from(parameter.min()).expect("a parameter's floor is a program byte");
    let high = u8::try_from(parameter.max()).expect("a parameter's ceiling is a program byte");
    let now = app
        .patch()
        .value(parameter)
        .expect("a sound has been read, so every parameter has a value");
    if now == high { low } else { high }
}

#[test]
fn every_parameter_is_editable() {
    let mut app = read();

    for parameter in ParamId::ALL.iter().copied() {
        let target = elsewhere(&app, parameter);
        app.update(Message::Ui(control_ui::Message::Edit {
            parameter,
            value: target,
        }));

        assert_eq!(
            app.patch().value(parameter),
            Some(target),
            "{parameter} would not move to {target}"
        );
        assert_eq!(
            app.patch().claim(parameter),
            Confidence::Assumed,
            "{parameter} moved without saying who moved it"
        );
    }
}

#[test]
fn every_parameter_survives_the_wire() {
    let mut app = read();

    // What the whole program was asked to become, parameter by parameter.
    let asked: Vec<(ParamId, u8)> = ParamId::ALL
        .iter()
        .copied()
        .map(|parameter| {
            let target = elsewhere(&app, parameter);
            app.update(Message::Ui(control_ui::Message::Edit {
                parameter,
                value: target,
            }));
            (parameter, target)
        })
        .collect();

    // "Put it there, then tell me where it is" has one honest order, and the
    // host crate is what keeps it: this read goes out behind 242 edits rather
    // than through them.
    app.update(Message::Read);
    until(&mut app, ROUND_TRIP, "dump behind the edits", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    for (parameter, target) in asked {
        assert_eq!(
            app.patch().value(parameter),
            Some(target),
            "{parameter} came back as something else"
        );
        assert_eq!(
            app.patch().claim(parameter),
            Confidence::Confirmed,
            "{parameter} came back unconfirmed"
        );
    }
}

#[test]
fn every_section_holds_parameters_and_the_bar_holds_every_section() {
    let mut app = read();

    for section in sections().iter().copied() {
        app.update(Message::Ui(control_ui::Message::Show(section)));

        assert_eq!(app.section(), section);
        assert!(
            section.parameters().next().is_some(),
            "{section} is an empty panel"
        );
        // Nothing has been touched, so every panel is still the synthesizer's
        // own account of itself.
        assert_eq!(app.patch().claim_of(section), Confidence::Confirmed);
    }
    assert_eq!(sections().len(), Group::ALL.len());
}

#[test]
fn a_window_opens_on_a_panel_the_instrument_has() {
    let app = read();

    assert_eq!(app.section(), first_section());
    assert!(Group::ALL.contains(&app.section()));
}

#[test]
fn the_panel_somebody_is_looking_at_outlives_the_port() {
    let mut app = read();
    app.update(Message::Ui(control_ui::Message::Show(Group::Effects)));

    app.update(Message::Disconnect);

    // The sound went away and the person did not.
    assert!(!app.patch().is_known());
    assert_eq!(app.section(), Group::Effects);
    assert_eq!(app.patch().claim_of(Group::Effects), Confidence::Unknown);
}

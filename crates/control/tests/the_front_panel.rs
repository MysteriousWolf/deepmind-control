//! The front panel, driven against the library's simulated synthesizer.
//!
//! The window opens on the instrument's own front: two rows of section plates,
//! the handful of controls Behringer put a fader under, and on every plate the
//! press the hardware calls `EDIT`. What has to stay true across that is what
//! this asserts.
//!
//! Every control on the panel moves the parameter it names and survives the
//! wire, because a panel whose faders were drawn from a stale table would be a
//! panel that edits the wrong sound. A way in opens the section it says,
//! because the whole point of a home screen is that everything else is one
//! press behind it. And the panel is what a window opens on, because that is
//! where a player looks first.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::thread;
use std::time::{Duration, Instant};

use control::{App, Message, View};
use control_ui::{Confidence, panelled};
use deepmind_host::PortRef;
use deepmind_midi::param::{Group, ParamId};

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long it waits for a dump behind the edits the panel costs.
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

/// A window with the sound read into it, showing the panel.
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
    app
}

/// Somewhere else in a parameter's range than where it is now.
fn elsewhere(app: &App, parameter: ParamId) -> u8 {
    let low = u8::try_from(parameter.min()).expect("a floor is a program byte");
    let high = u8::try_from(parameter.max()).expect("a ceiling is a program byte");
    let now = app.patch().value(parameter).expect("a sound has been read");
    if now == high { low } else { high }
}

#[test]
fn a_window_opens_on_the_front_panel() {
    let app = App::new();

    assert_eq!(app.view(), View::Panel, "a window opens somewhere else");
}

#[test]
fn every_control_on_the_panel_is_one_the_instrument_has() {
    for parameter in panelled() {
        assert!(
            ParamId::ALL.contains(&parameter),
            "{parameter} is on the panel and not on the instrument"
        );
    }
    // The panel is a handful and not a second rack: it is what the hardware
    // puts a fader under, and the other two hundred are behind an `EDIT`.
    assert!(panelled().len() < ParamId::ALL.len() / 4);
}

#[test]
fn every_control_on_the_panel_edits_the_parameter_it_names() {
    let mut app = read();

    let asked: Vec<(ParamId, u8)> = panelled()
        .into_iter()
        .map(|parameter| {
            let target = elsewhere(&app, parameter);
            app.update(Message::Ui(control_ui::Message::Edit {
                parameter,
                value: target,
            }));
            assert_eq!(
                app.patch().value(parameter),
                Some(target),
                "{parameter} would not move from the panel"
            );
            (parameter, target)
        })
        .collect();

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
    }
}

#[test]
fn a_way_in_opens_the_section_it_says() {
    let mut app = read();
    assert_eq!(app.view(), View::Panel);

    // What the panel's `EDIT` sends, and what the section bar sends: one press,
    // one answer, whichever surface asked.
    app.update(Message::Ui(control_ui::Message::Show(Group::Vcf)));

    assert_eq!(app.view(), View::Editor, "the section did not open");
    assert_eq!(app.section(), Group::Vcf);

    app.update(Message::Show(View::Panel));
    assert_eq!(app.view(), View::Panel, "the way back is one press too");
    assert_eq!(app.section(), Group::Vcf, "and it remembers where it was");
}

#[test]
fn the_panel_outlives_the_port() {
    let mut app = read();

    app.update(Message::Disconnect);

    // The sound went away and the person did not: the panel is still the panel,
    // with nothing on it rather than nothing of it.
    assert_eq!(app.view(), View::Panel);
    assert!(!app.patch().is_known());
    for parameter in panelled() {
        assert_eq!(app.patch().claim(parameter), Confidence::Unknown);
    }
}

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
use control_ui::{Confidence, panelled, sections, ways_in};
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
    assert_eq!(app.editing(), None, "a window opens with nothing over it");

    // What the panel's `EDIT` sends: the section comes up over the panel, and
    // the panel is still the surface underneath it.
    app.update(Message::Ui(control_ui::Message::Show(Group::Vcf)));

    assert_eq!(app.editing(), Some(Group::Vcf), "the section did not open");
    assert_eq!(app.view(), View::Panel, "the panel went away under it");

    // And the way back out, which the mark on the sheet, the panel around it
    // and the escape key all send.
    app.update(Message::Ui(control_ui::Message::Close));
    assert_eq!(app.editing(), None, "the sheet would not come off");
    assert_eq!(app.view(), View::Panel);
}

#[test]
fn a_second_way_in_swaps_the_sheet_rather_than_stacking_one() {
    let mut app = read();

    app.update(Message::Ui(control_ui::Message::Show(Group::Vcf)));
    app.update(Message::Ui(control_ui::Message::Show(Group::Lfo1)));

    // One press on the instrument's second `EDIT` is the display becoming the
    // second section, not a second display. One close puts all of it away.
    assert_eq!(app.editing(), Some(Group::Lfo1));
    app.update(Message::Ui(control_ui::Message::Close));
    assert_eq!(app.editing(), None, "something was left underneath");
}

#[test]
fn closing_nothing_is_nothing() {
    let mut app = read();

    // The escape key is heard while a sheet is open and nothing stops it
    // arriving after one has gone. It is not a way back to the panel from the
    // shelf: what it puts away is what is over the window.
    app.update(Message::Show(View::Library));
    app.update(Message::Ui(control_ui::Message::Close));

    assert_eq!(app.view(), View::Library);
    assert_eq!(app.editing(), None);
}

#[test]
fn the_sections_with_no_way_in_are_the_four_that_are_written_down() {
    let unreachable: Vec<&'static str> = sections()
        .iter()
        .filter(|section| !ways_in().contains(section))
        .map(|section| section.name())
        .collect();

    // The panel is the library's table of what the instrument puts a fader
    // under, so a section with no fader anywhere has no plate and no `EDIT`.
    // Four of the fourteen are in that position, and since the section bar was
    // taken away there is nothing else that reaches them.
    //
    // This is not an assertion that the state of affairs is right. It is an
    // assertion that it has not moved: `docs/todo.md` is where the four are
    // listed and where what to do about them is still open, and a fifth
    // arriving, or one of these quietly gaining a way in, has to break
    // something rather than be noticed a release later.
    assert_eq!(
        unreachable,
        ["Mod Matrix", "Control Sequencer", "Effects", "Program"],
        "the sections with no way in have changed; docs/todo.md says which four they were"
    );
}

#[test]
fn every_way_in_opens_a_section_the_instrument_has() {
    for section in ways_in() {
        assert!(
            sections().contains(&section),
            "a plate opens {section}, which the instrument does not have"
        );
    }
    // Two plates open the oscillators and two open the filter, and a section
    // that appeared twice in this list would be one this window could open two
    // different sheets onto.
    let mut once = ways_in();
    once.sort_unstable_by_key(|section| section.name());
    once.dedup();
    assert_eq!(once.len(), ways_in().len(), "a section has two ways in");
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

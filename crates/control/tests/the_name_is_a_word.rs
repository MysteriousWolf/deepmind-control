//! The program's name, driven against the library's simulated synthesizer.
//!
//! The first panel laid out by hand, and the first control that covers more
//! than one parameter: the instrument stores a name one character to a
//! parameter, and a person editing it is editing a word. What has to stay true
//! across that is what this asserts. The word reaches the synthesizer as the
//! characters it is made of and comes back as the same word; a keystroke costs
//! the one parameter it moved rather than all seventeen; and nothing about the
//! wire changed, because a name is still `Program Name Char 1` to `17` on it.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::thread;
use std::time::{Duration, Instant};

use control::{App, Message};
use control_ui::{Confidence, name_characters};
use deepmind_host::PortRef;
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::ProgramName;

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long it waits for a dump behind the edits a name costs.
///
/// Sixteen characters at the host crate's five milliseconds apiece, and a read
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

/// A name, or the failure of the test that asked for one.
fn named(text: &str) -> ProgramName {
    ProgramName::new(text).expect("a name of printable ASCII within sixteen characters")
}

/// Types a name into the window, the way the field does.
fn rename(app: &mut App, name: ProgramName) {
    app.update(Message::Ui(control_ui::Message::Rename(name)));
}

#[test]
fn a_name_typed_here_is_the_name_the_synthesizer_reports() {
    let mut app = read();
    let name = named("Bass Sweep");

    rename(&mut app, name);
    assert_eq!(app.patch().name(), Some(name));

    // "Put it there, then tell me where it is" has one honest order, and the
    // host crate is what keeps it: this read goes out behind the characters.
    app.update(Message::Read);
    until(&mut app, ROUND_TRIP, "dump behind the name", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    assert_eq!(app.patch().name(), Some(name));
}

#[test]
fn a_name_is_stored_one_character_to_a_parameter() {
    let mut app = read();

    rename(&mut app, named("Bass"));

    // The word on the screen and the bytes on the wire are the same thing seen
    // from two ends, and this is the end the protocol sees.
    for (parameter, expected) in name_characters().iter().copied().zip(b"Bass\0".iter()) {
        assert_eq!(
            app.patch().value(parameter),
            Some(*expected),
            "{parameter} does not hold the character the name put in it"
        );
    }
}

#[test]
fn a_typed_letter_leaves_the_rest_of_the_name_confirmed() {
    let mut app = read();
    rename(&mut app, named("Bass"));
    app.update(Message::Read);
    until(&mut app, ROUND_TRIP, "dump behind the name", |app| {
        app.patch().confidence() == Confidence::Confirmed
    });

    rename(&mut app, named("Bassy"));

    // One keystroke is one character, and the four before it are still exactly
    // as the synthesizer described them.
    assert_eq!(
        app.patch().claim(ParamId::ProgramNameChar5),
        Confidence::Assumed
    );
    for parameter in name_characters().iter().copied().take(4) {
        assert_eq!(
            app.patch().claim(parameter),
            Confidence::Confirmed,
            "{parameter} was rewritten by a keystroke that did not touch it"
        );
    }
}

#[test]
fn the_name_a_dump_carries_is_the_synthesizer_s_own() {
    let app = read();

    assert_eq!(
        app.patch().claim_across(name_characters().iter().copied()),
        Confidence::Confirmed
    );
    // A panel is as confirmed as its least confirmed parameter, and the name is
    // seventeen of the program section's nineteen.
    assert_eq!(app.patch().claim_of(Group::Program), Confidence::Confirmed);
}

#[test]
fn a_sound_nobody_has_read_cannot_be_named() {
    let mut app = App::new();

    rename(&mut app, named("Bass"));

    assert_eq!(app.patch().name(), None);
    assert!(!app.patch().is_known());
}

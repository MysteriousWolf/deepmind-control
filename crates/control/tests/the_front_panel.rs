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
use control_ui::{Confidence, Livery, Way, band, panelled, sections, unplated, ways_in};
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
fn every_section_the_instrument_has_has_a_way_in() {
    let unreachable: Vec<&'static str> = sections()
        .iter()
        .filter(|section| !ways_in().contains(section))
        .map(|section| section.name())
        .collect();

    // What `docs/todo.md` carried as its one open hole until the panel grew a
    // last band for it. Ten sections are opened by the `EDIT` on a plate, which
    // is the library's table of what the instrument puts a fader under; the
    // other four have no fader anywhere and are opened by the row of caps under
    // the rack. Between them that is all fourteen, and a fifteenth arriving in
    // a later firmware lands in the row by subtraction rather than by anybody
    // writing it down.
    assert_eq!(
        unreachable,
        Vec::<&'static str>::new(),
        "a section has no way in; the panel's last band is supposed to be the one that catches them"
    );
}

/// Which caps of the band are lit while `showing` is what the window has up.
///
/// The band's own rule, asked the way it is drawn: a cap is lit when what it
/// puts on the screen is what is already on it.
fn lit(app: &App) -> Vec<Way> {
    let showing = app.showing();
    band().into_iter().filter(|cap| *cap == showing).collect()
}

#[test]
fn the_band_lights_exactly_the_cap_for_what_is_open() {
    let mut app = read();

    // A window with nothing over it is a window looking at the front panel, and
    // the front panel has a cap of its own: the row says where somebody is
    // before it says where they can go.
    assert_eq!(app.editing(), None);
    assert_eq!(lit(&app), vec![Way::Panel], "the way home is not lit");

    // And each section the band carries lights its own cap and nothing else,
    // which is the difference between a row of tabs and a row of buttons.
    for section in unplated() {
        app.update(Message::Ui(control_ui::Message::Show(section)));
        assert_eq!(app.editing(), Some(section));
        assert_eq!(
            lit(&app),
            vec![Way::Section(section)],
            "{section} is open and its cap is not the lit one"
        );
    }
}

#[test]
fn a_section_with_a_plate_lights_nothing_in_the_band() {
    let mut app = read();

    // The ten sections a plate opens are not in the band, so a sheet of one of
    // them leaves every cap unlit — including the way home, because a sheet is
    // up and the front panel is not what somebody is looking at. Lighting the
    // way home there would be the row claiming a place nobody is in.
    app.update(Message::Ui(control_ui::Message::Show(Group::Vcf)));

    assert!(
        !band().contains(&Way::Section(Group::Vcf)),
        "VCF has a plate"
    );
    assert_eq!(lit(&app), Vec::new(), "something in the band lit");

    // The way home still works from there, which is why it is a cap and not a
    // fifth section: it is the one press in the band that means anything while
    // a plate's own sheet is up.
    app.update(Message::Ui(control_ui::Message::Close));
    assert_eq!(app.editing(), None);
    assert_eq!(lit(&app), vec![Way::Panel]);
}

#[test]
fn the_window_opens_as_a_twelve_and_can_wear_a_twelve_x() {
    // Two liveries, both the instrument: a `12` prints its section names in
    // white on the bare panel and a `12X` knocks them out of filled banners.
    // The plainer one is what the window opens as, because it is what most
    // `DeepMind`s in the world are, and the press in the footer is the other.
    let mut app = read();

    assert_eq!(app.livery(), Livery::Plain, "the window opens as a 12X");

    app.update(Message::Wear);
    assert_eq!(app.livery(), Livery::Banners);
    app.update(Message::Wear);
    assert_eq!(app.livery(), Livery::Plain, "the press does not go back");

    // And it is a fact about the window rather than about the sound: putting
    // the port down leaves somebody looking at the same instrument.
    app.update(Message::Wear);
    app.update(Message::Disconnect);
    assert_eq!(
        app.livery(),
        Livery::Banners,
        "the livery went with the port"
    );
}

#[test]
fn the_shelf_is_a_cap_of_the_band_like_any_other() {
    // The switch between the two surfaces is gone: the library is where this
    // window can be, the same way a section is, so it is a press in the same
    // row and it lights by the same comparison.
    let mut app = read();

    app.update(Message::Ui(control_ui::Message::Shelf));
    assert_eq!(lit(&app), vec![Way::Library], "the shelf's cap is not lit");
    assert_eq!(
        app.editing(),
        None,
        "a sheet is still up over a window showing the shelf"
    );

    // And a section reached from the shelf brings the panel back under it,
    // because a section is a sheet over the panel and nothing else.
    app.update(Message::Ui(control_ui::Message::Show(Group::ModMatrix)));
    assert_eq!(lit(&app), vec![Way::Section(Group::ModMatrix)]);
    assert_eq!(app.showing(), Way::Section(Group::ModMatrix));
}

#[test]
fn the_band_is_the_way_home_and_then_what_no_plate_carries() {
    let caps = band();

    assert_eq!(
        caps.first(),
        Some(&Way::Panel),
        "the way home is not the first cap of the band"
    );
    assert_eq!(
        caps.last(),
        Some(&Way::Library),
        "the shelf is not the last cap of the band"
    );
    assert_eq!(
        caps.into_iter()
            .filter_map(|cap| match cap {
                Way::Section(section) => Some(section),
                Way::Panel | Way::Library => None,
            })
            .collect::<Vec<Group>>(),
        unplated(),
        "the band and the sections with no plate have come apart"
    );
}

#[test]
fn the_last_band_carries_exactly_what_no_plate_does() {
    let plated: Vec<Group> = ways_in()
        .into_iter()
        .filter(|section| !unplated().contains(section))
        .collect();

    for section in unplated() {
        assert!(
            !plated.contains(&section),
            "{section} is opened twice: by a plate and by the row under the rack"
        );
    }
    assert_eq!(
        plated.len() + unplated().len(),
        ways_in().len(),
        "a way in is neither a plate's nor the row's"
    );
    // And the row is the four the instrument has no fader for, which is the
    // list `docs/todo.md` used to say had no way in at all.
    let named: Vec<&'static str> = unplated().iter().map(|section| section.name()).collect();
    assert_eq!(
        named,
        ["Mod Matrix", "Control Sequencer", "Effects", "Program"],
        "the sections with no plate have changed"
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

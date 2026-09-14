//! The device thread, driven against the library's simulated synthesizer.
//!
//! No hardware, no ports, and no timing assumptions beyond "this should have
//! happened by now". What these prove is that this crate drives the protocol the
//! way the library expects to be driven: that a port which answers is adopted,
//! that an edit reaches the other end, that a patch load is a difference, and
//! that twelve seconds of bank transfer is something with progress and an end.
//!
//! What they cannot prove is that the manual is right. Both ends of the
//! conversation are generated from the same specification.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::time::Duration;

use deepmind_host::{Command, Event, Link, Options, Outcome, Pack, open_simulator};
use deepmind_midi::device::Event as DeviceEvent;
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::wire::Channel;

/// How long a test waits for something that should take microseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// Waits for the first event `matches` accepts, and returns everything up to and
/// including it.
fn until<F: Fn(&Event) -> bool>(link: &Link, matches: F) -> Vec<Event> {
    let mut seen = Vec::new();
    while let Some(event) = link.wait(PATIENCE) {
        let found = matches(&event);
        seen.push(event);
        if found {
            return seen;
        }
    }
    panic!("nothing matched in {PATIENCE:?}; saw {seen:#?}");
}

/// A program with a name, to tell one from another.
fn named(name: &str) -> Program {
    let mut program = Program::new(ProtocolVersion::V7);
    program.set_name(ProgramName::new(name).expect("a program name"));
    program
}

/// Slot `number` of bank A.
fn slot(number: u8) -> Slot {
    Slot::new(
        Bank::A,
        ProgramNumber::new(number).expect("a program number"),
    )
}

/// A unit with `count` programs in the front of bank A.
fn pack(count: u8) -> Pack {
    let mut pack = Pack::empty();
    for number in 0..count {
        pack.insert(slot(number), named(&format!("Preset {number}")));
    }
    pack
}

/// The edit buffer, as the synthesizer reports it.
fn read_edit_buffer(link: &Link) -> Program {
    link.send(Command::ReadEditBuffer).expect("the thread");
    let events = until(link, |event| {
        matches!(event, Event::Device(DeviceEvent::EditBuffer(_)))
    });
    match events.last() {
        Some(Event::Device(DeviceEvent::EditBuffer(program))) => program.clone(),
        other => panic!("the edit buffer, not {other:?}"),
    }
}

#[test]
fn a_port_that_answers_says_who_it_is() {
    let link = open_simulator(Pack::empty(), Options::default());

    let events = until(&link, |event| matches!(event, Event::Identified { .. }));

    assert!(
        matches!(events.first(), Some(Event::Opened)),
        "the port opening is the first thing that happens: {events:#?}"
    );
    let Some(Event::Identified { identity, channel }) = events.last() else {
        panic!("an identity: {events:#?}");
    };
    // The inquiry is broadcast, and the unit answers with the device ID it
    // holds, which the manual says is also its global MIDI channel.
    assert_eq!(identity.device, DeviceId::Unit(0));
    assert_eq!(*channel, Channel::ONE);
    assert!(identity.firmware.is(1, 1), "{:?}", identity.firmware);
}

#[test]
fn the_edit_buffer_comes_back_as_a_program() {
    let link = open_simulator(Pack::empty(), Options::default());

    assert_eq!(read_edit_buffer(&link).name().as_str(), "Simulator");
}

#[test]
fn an_edit_reaches_the_synthesizer() {
    let link = open_simulator(Pack::empty(), Options::default());
    let before = read_edit_buffer(&link);
    assert_ne!(before.get(ParamId::Lfo1Rate), 64);

    link.send(Command::SetParameter {
        parameter: ParamId::Lfo1Rate,
        value: 64,
    })
    .expect("the thread");

    assert_eq!(read_edit_buffer(&link).get(ParamId::Lfo1Rate), 64);
}

#[test]
fn a_drag_sends_its_last_value_and_not_its_middle() {
    let options = Options {
        // Slow enough that sixty values cannot each get their own message in the
        // time this test takes, which is the whole point of coalescing them.
        edit_interval_ms: 50,
        ..Options::default()
    };
    let link = open_simulator(Pack::empty(), options);
    read_edit_buffer(&link);

    for value in 0..=60 {
        link.send(Command::SetParameter {
            parameter: ParamId::Lfo1Rate,
            value,
        })
        .expect("the thread");
    }

    assert_eq!(read_edit_buffer(&link).get(ParamId::Lfo1Rate), 60);
}

#[test]
fn an_edit_before_anything_is_known_is_refused() {
    let link = open_simulator(Pack::empty(), Options::default());
    until(&link, |event| matches!(event, Event::Identified { .. }));

    link.send(Command::SetParameter {
        parameter: ParamId::Lfo1Rate,
        value: 64,
    })
    .expect("the thread");

    // An edit is a difference, and there is nothing to take a difference
    // against until a dump has arrived.
    let events = until(&link, |event| matches!(event, Event::Refused { .. }));
    let Some(Event::Refused { reason, .. }) = events.last() else {
        panic!("a refusal: {events:#?}");
    };
    assert!(
        matches!(reason, deepmind_midi::Error::ProgramNotKnown),
        "{reason:?}"
    );
}

#[test]
fn loading_a_program_sends_the_difference() {
    let link = open_simulator(Pack::empty(), Options::default());
    let mut target = read_edit_buffer(&link);
    target.set(ParamId::Lfo1Rate, 12).expect("in range");
    target.set(ParamId::VcfFrequency, 34).expect("in range");

    link.send(Command::LoadProgram(Box::new(target)))
        .expect("the thread");

    let loaded = read_edit_buffer(&link);
    assert_eq!(loaded.get(ParamId::Lfo1Rate), 12);
    assert_eq!(loaded.get(ParamId::VcfFrequency), 34);
    // The name is not a parameter and no NRPN carries one, so the unit keeps
    // showing what was in its edit buffer.
    assert_eq!(loaded.name().as_str(), "Simulator");
}

#[test]
fn a_bank_read_reports_every_dump_and_then_ends() {
    let link = open_simulator(pack(4), Options::default());
    until(&link, |event| matches!(event, Event::Identified { .. }));

    link.send(Command::ReadBank {
        bank: Bank::A,
        first: ProgramNumber::FIRST,
        last: ProgramNumber::new(3).expect("a program number"),
    })
    .expect("the thread");

    let events = until(&link, |event| matches!(event, Event::Finished { .. }));

    let progress: Vec<u16> = events
        .iter()
        .filter_map(|event| match event {
            Event::Progress { received, .. } => Some(*received),
            _ => None,
        })
        .collect();
    assert_eq!(progress, [0, 1, 2, 3], "{events:#?}");

    let dumps = events
        .iter()
        .filter(|event| matches!(event, Event::Device(DeviceEvent::Program { .. })))
        .count();
    assert_eq!(dumps, 4);

    let Some(Event::Finished {
        received,
        expected,
        outcome,
        ..
    }) = events.last()
    else {
        panic!("an ending: {events:#?}");
    };
    assert_eq!((*received, *expected), (4, 4));
    assert_eq!(*outcome, Outcome::Complete);
}

#[test]
fn a_bank_read_can_be_called_off() {
    // One slot filled and a run over two: the unit answers the first and has
    // nothing to say about the second, so the transfer is still open.
    let link = open_simulator(pack(1), Options::default());

    link.send(Command::ReadBank {
        bank: Bank::A,
        first: ProgramNumber::FIRST,
        last: ProgramNumber::new(1).expect("a program number"),
    })
    .expect("the thread");
    until(&link, |event| {
        matches!(event, Event::Progress { received: 1, .. })
    });

    link.send(Command::Cancel).expect("the thread");

    let events = until(&link, |event| matches!(event, Event::Finished { .. }));
    let Some(Event::Finished {
        received, outcome, ..
    }) = events.last()
    else {
        panic!("an ending: {events:#?}");
    };
    assert_eq!(*received, 1);
    assert_eq!(*outcome, Outcome::Cancelled);
}

#[test]
fn a_unit_that_stops_partway_ends_the_transfer() {
    let options = Options {
        // The library times a run by the gap between its dumps, so this is how
        // long a stalled transfer waits and not how long the whole one may take.
        timeout_ms: 200,
        ..Options::default()
    };
    let link = open_simulator(pack(1), options);

    link.send(Command::ReadBank {
        bank: Bank::A,
        first: ProgramNumber::FIRST,
        last: ProgramNumber::new(1).expect("a program number"),
    })
    .expect("the thread");

    let events = until(&link, |event| matches!(event, Event::Finished { .. }));
    let Some(Event::Finished {
        received, outcome, ..
    }) = events.last()
    else {
        panic!("an ending: {events:#?}");
    };
    assert_eq!(*received, 1);
    assert_eq!(*outcome, Outcome::TimedOut);
}

#[test]
fn a_stored_program_is_reported_and_not_tracked() {
    let link = open_simulator(pack(1), Options::default());
    let playing = read_edit_buffer(&link);

    link.send(Command::ReadProgram(slot(0)))
        .expect("the thread");

    let events = until(&link, |event| {
        matches!(event, Event::Device(DeviceEvent::Program { .. }))
    });
    let Some(Event::Device(DeviceEvent::Program { slot: where_, .. })) = events.last() else {
        panic!("a stored program: {events:#?}");
    };
    assert_eq!(*where_, slot(0));
    // A stored program is not the sound the synthesizer is making.
    assert_eq!(read_edit_buffer(&link).name(), playing.name());
}

#[test]
fn dropping_the_link_stops_the_thread() {
    let link = open_simulator(Pack::empty(), Options::default());
    until(&link, |event| matches!(event, Event::Identified { .. }));
    assert!(link.is_open());

    link.close();
}

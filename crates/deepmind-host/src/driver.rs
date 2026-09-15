//! The loop that owns the port, the device and the twelve seconds.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use deepmind_midi::device::{Device, Event as DeviceEvent, Request};
use deepmind_midi::ids::{Bank, DeviceId};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::Program;
use deepmind_midi::sysex::inquiry::Identity;
use deepmind_midi::transport::{StdClock, Transport};
use deepmind_midi::wire::Channel;

use crate::Options;
use crate::backend::Backend;
use crate::command::{Command, CommandKind};
use crate::edits::Edits;
use crate::error::PortError;
use crate::event::{Event, Outcome};

/// The transport this crate drives: the library's defaults, over our port.
type Wire = Transport<Backend, StdClock>;

/// A bank read in flight.
#[derive(Debug, Clone, Copy)]
struct Transfer {
    bank: Bank,
    first: u8,
    last: u8,
    received: u16,
    expected: u16,
}

impl Transfer {
    /// Returns whether a dump that arrived belongs to this run.
    const fn covers(self, bank: Bank, number: u8) -> bool {
        bank.index() == self.bank.index() && self.first <= number && number <= self.last
    }
}

/// The device thread.
///
/// It owns the `Device`, the port and nothing else. The interface holds a copy
/// of the state it builds out of the events this publishes, and there is no lock
/// between them.
#[derive(Debug)]
pub(crate) struct Driver {
    wire: Wire,
    commands: Receiver<Command>,
    events: Sender<Event>,
    edits: Edits,
    /// Commands that are waiting for their turn. See [`Driver::handle`].
    deferred: VecDeque<Command>,
    /// Whether the handshake has settled, one way or the other.
    ready: bool,
    transfer: Option<Transfer>,
    options: Options,
}

impl Driver {
    /// Builds the loop around a port that is already open.
    pub(crate) fn new(
        backend: Backend,
        options: Options,
        commands: Receiver<Command>,
        events: Sender<Event>,
    ) -> Self {
        Self {
            wire: wire(backend, DeviceId::Broadcast, Channel::ONE, options),
            commands,
            events,
            edits: Edits::new(options.edit_interval_ms),
            deferred: VecDeque::new(),
            ready: false,
            transfer: None,
            options,
        }
    }

    /// Reads, feeds, ticks and sends until the interface says to stop.
    ///
    /// Nothing in here waits for an answer. The blocking helpers the library
    /// offers are the right shape for a command-line tool and the wrong shape
    /// for a window: they wait, and a waiting thread cannot report progress or
    /// be cancelled.
    pub(crate) fn run(mut self) {
        self.publish(Event::Opened);
        // A port that answers has a DeepMind on it, which is the only test of
        // one worth making.
        self.ask(CommandKind::Identify, Device::request_identity);

        let idle = Duration::from_millis(self.options.poll_ms);
        loop {
            let mut worked = false;

            loop {
                match self.commands.try_recv() {
                    Ok(Command::Stop) | Err(TryRecvError::Disconnected) => {
                        self.publish(Event::Closed);
                        return;
                    }
                    Ok(command) => {
                        self.handle(command);
                        worked = true;
                    }
                    Err(TryRecvError::Empty) => break,
                }
            }

            match self.wire.pump() {
                Ok(read) => worked |= read > 0,
                Err(error) => {
                    let failure = error.into_port().unwrap_or(PortError::Gone);
                    self.publish(Event::Failed(failure));
                    self.publish(Event::Closed);
                    return;
                }
            }

            while let Some(event) = self.wire.poll_event() {
                self.report(event);
                worked = true;
            }

            worked |= self.push_edit();
            worked |= self.release();

            if !worked {
                thread::sleep(idle);
            }
        }
    }

    /// Takes one command, now or later.
    ///
    /// Two things make a command wait. Until the handshake has settled there is
    /// no telling who is on the port or which channel to use, so everything
    /// waits for it. And a read asked for while edits are still going out waits
    /// for them, because a dump that arrives before them would confirm the
    /// values they are replacing: "put it there, then tell me where it is" has
    /// only one honest order.
    ///
    /// A parameter edit never waits behind other edits. That is what the
    /// coalescing is for: the newest value replaces the pending one instead of
    /// queueing behind it, and a drag that queued would grow without end.
    fn handle(&mut self, command: Command) {
        match command {
            // What is in flight is called off wherever the rest of it is.
            Command::Cancel => {
                self.cancel();
                return;
            }
            // Ended by the loop, which is the only thing that can end it.
            Command::Stop => return,
            _ => {}
        }
        let waiting = !self.edits.is_empty() || !self.deferred.is_empty();
        if !self.ready || (waiting && !matches!(command, Command::SetParameter { .. })) {
            self.deferred.push_back(command);
            return;
        }
        self.issue(command);
    }

    /// Issues what was held back, once what it was waiting for has happened.
    ///
    /// Returns whether anything went out.
    fn release(&mut self) -> bool {
        let mut issued = false;
        while self.ready && self.edits.is_empty() {
            let Some(command) = self.deferred.pop_front() else {
                break;
            };
            self.issue(command);
            issued = true;
        }
        issued
    }

    /// Carries out one command.
    fn issue(&mut self, command: Command) {
        match command {
            Command::Identify => self.ask(CommandKind::Identify, Device::request_identity),
            Command::ReadEditBuffer => {
                self.ask(CommandKind::ReadEditBuffer, Device::request_edit_buffer);
            }
            Command::ReadProgram(slot) => {
                self.ask(CommandKind::ReadProgram, |device| {
                    device.request_program(slot)
                });
            }
            Command::ReadBank { bank, first, last } => {
                match self.wire.device_mut().request_bank(bank, first, last) {
                    Ok(()) => {
                        let expected = u16::from(last.get().saturating_sub(first.get())) + 1;
                        self.transfer = Some(Transfer {
                            bank,
                            first: first.get(),
                            last: last.get(),
                            received: 0,
                            expected,
                        });
                        self.publish(Event::Progress {
                            bank,
                            received: 0,
                            expected,
                        });
                    }
                    Err(reason) => self.publish(Event::Refused {
                        command: CommandKind::ReadBank,
                        reason,
                    }),
                }
            }
            Command::SetParameter { parameter, value } => self.edits.set(parameter, value),
            Command::LoadProgram(program) => self.load(&program),
            // Both are answered by `handle`, which never passes them on.
            Command::Cancel | Command::Stop => {}
        }
    }

    /// Queues a request, and reports a library that would not take it.
    fn ask<F>(&mut self, command: CommandKind, request: F)
    where
        F: FnOnce(&mut Device) -> deepmind_midi::Result<()>,
    {
        if let Err(reason) = request(self.wire.device_mut()) {
            self.publish(Event::Refused { command, reason });
        }
    }

    /// Turns a program into the edits that reach it.
    ///
    /// The difference and not the program: sending all 242 parameters is 2904
    /// bytes, a little under a second of solid MIDI, and a patch change that
    /// takes a second is one the player notices. The edits go through the same
    /// coalescing as a knob, so a load that is still going out when the next one
    /// arrives costs the wire one patch change rather than two.
    fn load(&mut self, target: &Program) {
        let Some(current) = self.wire.device().program().value() else {
            self.publish(Event::Refused {
                command: CommandKind::LoadProgram,
                reason: deepmind_midi::Error::ProgramNotKnown,
            });
            return;
        };
        let changes: Vec<(ParamId, u8)> = current.changes(target).collect();
        for (parameter, value) in changes {
            self.edits.set(parameter, value);
        }
    }

    /// Calls off what has not happened yet.
    ///
    /// The edits that have not gone, the commands waiting behind them, and the
    /// transfer in flight. What already went stays where it went.
    fn cancel(&mut self) {
        self.edits.clear();
        self.deferred.clear();
        if let Some(transfer) = self.transfer.take() {
            self.publish(Event::Finished {
                bank: transfer.bank,
                received: transfer.received,
                expected: transfer.expected,
                outcome: Outcome::Cancelled,
            });
        }
    }

    /// Sends one pending parameter, if the rate allows one now.
    ///
    /// Returns whether anything went out.
    fn push_edit(&mut self) -> bool {
        if self.edits.is_empty() || self.wire.device().room() == 0 {
            return false;
        }
        let Some((parameter, value)) = self.edits.take_due(Instant::now()) else {
            return false;
        };
        // One parameter per call, so the diff the library takes is this edit and
        // nothing else, and a value the synthesizer already holds sends nothing.
        let sent = self.wire.device_mut().edit(|program| {
            let _ = program.set(parameter, value);
        });
        if let Err(reason) = sent {
            self.publish(Event::Refused {
                command: CommandKind::SetParameter,
                reason,
            });
        }
        true
    }

    /// Publishes what the synthesizer said, and what it means for a transfer.
    fn report(&mut self, event: DeviceEvent) {
        match &event {
            DeviceEvent::Identity(identity) => {
                let identity = *identity;
                self.publish(Event::Device(event));
                self.adopt(identity);
                return;
            }
            DeviceEvent::Parameter { parameter, value } => {
                let (parameter, value) = (*parameter, *value);
                // The library has already applied the report by the time it is
                // polled, so what it now says about the sound it tracks is what
                // it made of this message: an NRPN carries the whole value and
                // leaves a confirmed program confirmed, and a control change
                // carries seven bits and leaves it assumed. Two reports arriving
                // in the same pump are read after both, so a coarse one makes
                // the exact one before it look coarse too, which errs towards
                // claiming less than is known.
                self.publish(Event::Parameter {
                    parameter,
                    value,
                    confirmed: self.wire.device().program().is_confirmed(),
                });
                return;
            }
            DeviceEvent::Program { slot, .. } => {
                let (bank, number) = (slot.bank, slot.number.get());
                self.publish(Event::Device(event));
                self.advance(bank, number);
                return;
            }
            DeviceEvent::Timeout(Request::Identity) => {
                self.publish(Event::Device(event));
                // Nothing answered, so nothing is known about who is there and
                // the device goes on addressing the whole port. The interface is
                // told, and whatever it asks for next is still attempted: a
                // synthesizer that was switched on late is a likelier
                // explanation than a host that should give up.
                self.ready = true;
                self.publish(Event::Silent);
                return;
            }
            DeviceEvent::Timeout(Request::Bank { .. }) => {
                self.publish(Event::Device(event));
                self.end(Outcome::TimedOut);
                return;
            }
            _ => {}
        }
        self.publish(Event::Device(event));
    }

    /// Counts a dump towards the transfer it belongs to.
    fn advance(&mut self, bank: Bank, number: u8) {
        let Some(transfer) = self.transfer.as_mut() else {
            return;
        };
        if !(*transfer).covers(bank, number) {
            return;
        }
        transfer.received = transfer.received.saturating_add(1);
        let (received, expected) = (transfer.received, transfer.expected);
        if received >= expected {
            self.end(Outcome::Complete);
        } else {
            self.publish(Event::Progress {
                bank,
                received,
                expected,
            });
        }
    }

    /// Ends the transfer in flight, if there is one.
    fn end(&mut self, outcome: Outcome) {
        if let Some(transfer) = self.transfer.take() {
            self.publish(Event::Finished {
                bank: transfer.bank,
                received: transfer.received,
                expected: transfer.expected,
                outcome,
            });
        }
    }

    /// Rebuilds the device around the identity that answered.
    ///
    /// One message settles who to address, which channel to send NRPN on, and
    /// which value tables are true. Rebuilding costs nothing and is more honest
    /// than mutating a device that was addressing the whole port a moment ago.
    fn adopt(&mut self, identity: Identity) {
        let channel = match identity.device {
            DeviceId::Unit(id) => Channel::new(id).unwrap_or(Channel::ONE),
            DeviceId::Broadcast => Channel::ONE,
        };
        let addressed = self.wire.device().id() == identity.device
            && self.wire.device().channel().index() == channel.index();
        // A rebuilt device is a device with nothing outstanding, so this happens
        // when the address actually changes rather than on every inquiry. Asking
        // again is then free, which is what makes it the way to look for a
        // synthesizer that was switched on late.
        if !addressed {
            let spent = wire(
                Backend::Spent,
                DeviceId::Broadcast,
                Channel::ONE,
                self.options,
            );
            let (_, port, clock) = core::mem::replace(&mut self.wire, spent).into_parts();
            self.wire = Transport::new(
                Device::new(identity.device)
                    .with_channel(channel)
                    .with_timeout(self.options.timeout_ms),
                port,
                clock,
            );
        }
        self.ready = true;
        self.publish(Event::Identified { identity, channel });
    }

    /// Hands an event to the interface, if it is still listening.
    fn publish(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

/// Builds a transport around a port.
fn wire(port: Backend, id: DeviceId, channel: Channel, options: Options) -> Wire {
    Transport::new(
        Device::new(id)
            .with_channel(channel)
            .with_timeout(options.timeout_ms),
        port,
        StdClock::new(),
    )
}

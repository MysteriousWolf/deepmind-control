//! What the window has asked for and the host has not been told yet.

use std::collections::VecDeque;

use deepmind_midi::param::{PARAMETER_COUNT, ParamId};
use deepmind_midi::program::Program;

/// The edits waiting to leave as MIDI, one pending value per parameter.
///
/// The same idea as the queue the desktop build keeps in `deepmind-host`, and
/// for the same reason: one NRPN is twelve bytes, a drag at sixty frames a second
/// makes sixty of them, and every one but the last is obsolete by the time it
/// could be sent. The newest value of each parameter is kept, and a parameter
/// that is already waiting keeps its place in the line, so a drag does not push
/// everything else back.
///
/// The window pushes and the audio thread takes. Neither allocates after
/// construction: the line has room for every parameter at once, which is the
/// most it can ever hold.
#[derive(Debug)]
pub struct Outbox {
    /// Newest value asked for, by parameter offset.
    values: [Option<u8>; PARAMETER_COUNT],
    /// Parameters waiting to go, oldest first. A parameter appears once.
    order: VecDeque<ParamId>,
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Outbox {
    /// An empty outbox.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: [None; PARAMETER_COUNT],
            order: VecDeque::with_capacity(PARAMETER_COUNT),
        }
    }

    /// Asks for a parameter to be set to a value.
    pub fn set(&mut self, parameter: ParamId, value: u8) {
        let Some(slot) = self.values.get_mut(usize::from(parameter.offset())) else {
            return;
        };
        if slot.is_none() {
            self.order.push_back(parameter);
        }
        *slot = Some(value);
    }

    /// Asks for every parameter of a program, which is the whole sound.
    pub fn set_all(&mut self, program: &Program) {
        for parameter in ParamId::ALL.iter().copied() {
            self.set(parameter, program.get(parameter));
        }
    }

    /// Takes the parameter that has been waiting longest, with the value it
    /// should go out with.
    pub fn take(&mut self) -> Option<(ParamId, u8)> {
        let parameter = self.order.pop_front()?;
        let value = self
            .values
            .get_mut(usize::from(parameter.offset()))
            .and_then(Option::take)?;
        Some((parameter, value))
    }

    /// Returns whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Returns how many parameters are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.order.len()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::ids::ProtocolVersion;

    use super::*;

    fn first_two() -> (ParamId, ParamId) {
        let mut all = ParamId::ALL.iter().copied();
        (
            all.next().expect("there are parameters"),
            all.next().expect("there are at least two"),
        )
    }

    #[test]
    fn the_newest_value_replaces_the_one_waiting() {
        let (one, _) = first_two();
        let mut outbox = Outbox::new();
        outbox.set(one, 3);
        outbox.set(one, 9);
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox.take(), Some((one, 9)));
        assert!(outbox.is_empty());
    }

    #[test]
    fn a_drag_keeps_its_place_in_the_line() {
        let (one, other) = first_two();
        let mut outbox = Outbox::new();
        outbox.set(one, 1);
        outbox.set(other, 2);
        outbox.set(one, 3);
        assert_eq!(outbox.take(), Some((one, 3)));
        assert_eq!(outbox.take(), Some((other, 2)));
        assert_eq!(outbox.take(), None);
    }

    #[test]
    fn a_parameter_can_wait_again_once_it_has_gone() {
        let (one, _) = first_two();
        let mut outbox = Outbox::new();
        outbox.set(one, 1);
        assert_eq!(outbox.take(), Some((one, 1)));
        outbox.set(one, 2);
        assert_eq!(outbox.take(), Some((one, 2)));
    }

    #[test]
    fn a_program_is_every_parameter_once() {
        let program = Program::new(ProtocolVersion::V7);
        let mut outbox = Outbox::new();
        outbox.set_all(&program);
        outbox.set_all(&program);
        assert_eq!(outbox.len(), PARAMETER_COUNT);
        let mut taken = 0;
        while let Some((parameter, value)) = outbox.take() {
            assert_eq!(value, program.get(parameter));
            taken += 1;
        }
        assert_eq!(taken, PARAMETER_COUNT);
    }
}

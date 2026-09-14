//! One pending value per parameter, sent at a fixed rate.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use deepmind_midi::param::{PARAMETER_COUNT, ParamId};

/// What has been asked for and not sent yet.
///
/// One NRPN is four control changes, twelve bytes, about four milliseconds. A
/// knob drag at 60 fps produces sixty of those a second for as long as the mouse
/// is down, and every one of them but the last is obsolete when it is sent. So
/// this keeps the newest value of each parameter and hands them out no faster
/// than the wire can take them.
///
/// Dropping the intermediate values of a drag is free. Dropping the last one is
/// a desynchronised instrument, so a parameter stays here until it has gone out.
///
/// This lives in the host crate rather than in the views because the plugin and
/// the desktop build have the same problem, and the views should not know that a
/// wire has a speed.
#[derive(Debug)]
pub(crate) struct Edits {
    /// Newest value asked for, by parameter offset.
    values: [Option<u8>; PARAMETER_COUNT],
    /// Offsets waiting to go, oldest first. A parameter appears once.
    order: VecDeque<u8>,
    /// How long between one parameter going out and the next.
    interval: Duration,
    /// When the next one may go. `None` means now.
    next: Option<Instant>,
}

impl Edits {
    /// Builds a queue that lets one parameter out every `interval_ms`.
    pub(crate) const fn new(interval_ms: u64) -> Self {
        Self {
            values: [None; PARAMETER_COUNT],
            order: VecDeque::new(),
            interval: Duration::from_millis(interval_ms),
            next: None,
        }
    }

    /// Puts a parameter where the interface most recently asked for.
    ///
    /// Replaces whatever that parameter was waiting to be set to, and keeps its
    /// place in the queue: a drag does not push everything else back.
    pub(crate) fn set(&mut self, parameter: ParamId, value: u8) {
        let offset = parameter.offset();
        let Some(slot) = self.values.get_mut(usize::from(offset)) else {
            return;
        };
        if slot.is_none() {
            self.order.push_back(offset);
        }
        *slot = Some(value);
    }

    /// Drops everything that has not gone yet.
    ///
    /// What already went stays where it went. There is no message in the
    /// protocol that would take it back.
    pub(crate) fn clear(&mut self) {
        for slot in &mut self.values {
            *slot = None;
        }
        self.order.clear();
    }

    /// Returns whether anything is waiting.
    pub(crate) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Takes the next parameter to send, when the rate allows one.
    ///
    /// Returns `None` when nothing is waiting, and when the last one went too
    /// recently.
    pub(crate) fn take_due(&mut self, now: Instant) -> Option<(ParamId, u8)> {
        if self.order.is_empty() {
            self.next = None;
            return None;
        }
        if self.next.is_some_and(|next| now < next) {
            return None;
        }
        while let Some(offset) = self.order.pop_front() {
            let Some(slot) = self.values.get_mut(usize::from(offset)) else {
                continue;
            };
            let Some(value) = slot.take() else {
                continue;
            };
            let Ok(parameter) = ParamId::from_offset(offset) else {
                continue;
            };
            self.next = Some(now + self.interval);
            return Some((parameter, value));
        }
        self.next = None;
        None
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_costs_one_message_per_interval_and_keeps_its_last_value() {
        let mut edits = Edits::new(10);
        let start = Instant::now();
        for value in 0..=60 {
            edits.set(ParamId::Lfo1Rate, value);
        }

        let (parameter, value) = edits.take_due(start).expect("the newest value");
        assert_eq!(parameter, ParamId::Lfo1Rate);
        assert_eq!(value, 60);
        assert!(edits.is_empty());
    }

    #[test]
    fn the_rate_holds_the_next_one_back() {
        let mut edits = Edits::new(10);
        let start = Instant::now();
        edits.set(ParamId::Lfo1Rate, 1);
        edits.set(ParamId::Lfo1Shape, 2);

        assert!(edits.take_due(start).is_some());
        assert!(edits.take_due(start).is_none());
        assert!(edits.take_due(start + Duration::from_millis(10)).is_some());
    }

    #[test]
    fn a_cleared_queue_sends_nothing() {
        let mut edits = Edits::new(0);
        edits.set(ParamId::Lfo1Rate, 1);
        edits.clear();
        assert!(edits.is_empty());
        assert!(edits.take_due(Instant::now()).is_none());
    }
}

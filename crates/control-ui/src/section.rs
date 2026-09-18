//! The sections, and the order the instrument keeps them in.
//!
//! Two hundred and forty-two parameters are fourteen sections, and the
//! instrument does not put them on one surface either: a player presses `EDIT`
//! on a plate and the display becomes that section. So does this, and what the
//! press opens is a [modal](crate::modal) over the panel it was pressed on.
//!
//! There was a bar of fourteen tabs here, above the rack, and it is gone with
//! the surface it sat on. What is left is the list itself, which is the one
//! thing about the sections this crate ever knew: their order.

use deepmind_midi::param::Group;

/// The sections, in the order the instrument itself lays them out.
///
/// The library's own, and not `Group::ALL`, which is alphabetical and puts the
/// effects third and the oscillators eighth. This crate read it off the offsets
/// until `deepmind-midi` 26.2 published [`Group::ORDER`], which reads it off the
/// same offsets on the side of the line that holds them: a group a later table
/// adds arrives in its right place with nothing here to edit.
#[must_use]
pub fn sections() -> &'static [Group] {
    Group::ORDER
}

/// What this window prints a section as.
///
/// The library's own name for it, with one exception, and the exception is why
/// this function exists rather than a call to [`Group::name`] everywhere.
///
/// # The one word this repository spells differently
///
/// `Group::ControlSequencer` is what the manual's NRPN table calls the section,
/// and the front of the instrument does not call it that: it prints `ARP / SEQ`
/// on the plate the arpeggiator and the sequencer share. `CONTROL SEQUENCER` is
/// the longest name of the fourteen, it is a quarter of the band of ways in on
/// its own, and the word doing the work in it is the last one. So the window
/// prints `SEQUENCER`.
///
/// Nothing else is renamed and nothing here is a second table: this is the
/// library's string unless the group is that one, so a section a later library
/// adds is printed under whatever the library calls it.
#[must_use]
pub fn name(group: Group) -> &'static str {
    match group {
        Group::ControlSequencer => SEQUENCER,
        other => other.name(),
    }
}

/// What the window prints instead of `Control Sequencer`.
///
/// Written here rather than sliced off the library's own string, because a
/// slice would be this file quietly depending on the library spelling it as two
/// words with the useful one second.
const SEQUENCER: &str = "Sequencer";

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::Group;

    use super::sections;

    #[test]
    fn the_list_holds_every_parameter() {
        for group in Group::ALL {
            assert!(
                sections().contains(group),
                "{group} is not in the list, so its parameters cannot be reached"
            );
        }
        assert_eq!(
            sections().len(),
            Group::ALL.len(),
            "a section appears in the list twice"
        );
    }

    #[test]
    fn the_list_is_in_the_instrument_s_own_order() {
        let starts: Vec<u8> = sections()
            .iter()
            .map(|group| {
                group
                    .parameters()
                    .next()
                    .expect("a group the table produced has a parameter in it")
                    .offset()
            })
            .collect();
        let mut sorted = starts.clone();
        sorted.sort_unstable();

        assert_eq!(
            starts, sorted,
            "the list is not in the order the instrument lays its parameters out"
        );
    }
}

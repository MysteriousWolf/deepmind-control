//! Mapping a routing onto a control by taking hold of it.
//!
//! The modulation matrix has two hard columns and they are hard for opposite
//! reasons. A destination is one of 133 names in a list, and finding `VCF
//! Envelope Attack` in it means knowing what the abbreviation is before you go
//! looking; a depth is a byte about a centre, and knowing that `+38` is the
//! right amount means having already heard it.
//!
//! The answer to both is the same gesture, and it is the one an editor has that
//! a front panel does not: map the routing onto the window, go to the control
//! you meant, and take hold of it. Where it lands is the destination. How far
//! you dragged it is the depth.
//!
//! # What is mapped, and what that is not
//!
//! [`Mapping`] is one routing — the three parameters the matrix reads as a
//! sentence — carried to every control in the window while somebody is
//! choosing. It is small and it is [`Copy`], because it reaches every control
//! the panel draws and a control is drawn a great many times.
//!
//! It is not an edit. Nothing about a routing being mapped changes what any
//! parameter holds: while it is up, a control lights if the matrix can reach it
//! and is inert if it cannot, the sections still open, the panel still scrolls,
//! and the one thing that does not happen is the sound changing. That is the
//! whole of the mode.
//!
//! # Which destination names a control
//!
//! Not a table here. A destination is a value of the parameter's own value
//! table and `ValueEntry::parameters` is what that value moves, published by
//! `deepmind-midi` 26.2 for exactly this — so [`Mapping::names`] reads the join
//! backwards: the entries whose parameters include the control somebody took
//! hold of, narrowest first.
//!
//! Narrowest first is the whole of the choice. `All Attack` moves three
//! envelopes and `VCF Attack` moves one, and somebody who took hold of the
//! filter envelope's attack fader meant the filter's. A destination that moves
//! nothing a program parameter addresses — the pitch a key is playing, the
//! amplitude of a voice — names no control, so no control lights for it, which
//! is the honest answer rather than a gap.
//!
//! # What a drag is worth, and the one thing it assumes
//!
//! **A drag is read as if full depth moved the control over its whole range.**
//! Dragging `VCF Frequency` a third of the way up asks for a third of the
//! depth. That is an assumption, it is the only one on this page, and it is
//! marked here because the manual does not publish the law: what `Mod 1 Depth`
//! at `+64` does to the bytes at the other end of the routing is nowhere in it,
//! and the library refuses to guess as firmly as this repository does. It is
//! asked for in
//! [deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38)
//! and recorded in [`docs/waiting.md`](https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/waiting.md).
//!
//! Until it is answered the gesture is a way of *saying* an amount rather than
//! a promise about what will be heard, which is what a depth fader already was:
//! the fader that the drag moves is the same fader, holding the same byte, and
//! the drag is a second way to put a number in it.

use deepmind_midi::param::{Group, Kind, ParamId, Shape};
use deepmind_midi::sysex::inquiry::Version;
use iced_widget::combo_box;

use crate::matrix;
use crate::panel::{Choice, choices};

/// A routing the modulation matrix has mapped onto the window.
///
/// The three parameters of one row, carried to every control the window draws
/// while somebody chooses where the routing goes. [`Copy`] and three words
/// wide, because it reaches every control on a panel of forty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mapping {
    /// What the library calls the routing, which is the row's own heading:
    /// `Mod 3`.
    label: &'static str,
    /// The parameter that says where the routing goes.
    destination: ParamId,
    /// The parameter that says how much of it arrives.
    depth: ParamId,
}

impl Mapping {
    /// A routing mapped onto the window.
    pub(crate) const fn new(label: &'static str, destination: ParamId, depth: ParamId) -> Self {
        Self {
            label,
            destination,
            depth,
        }
    }

    /// Returns what the library calls this routing.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.label
    }

    /// Returns the parameter that says where the routing goes.
    #[must_use]
    pub const fn destination(self) -> ParamId {
        self.destination
    }

    /// Returns the parameter that says how much of it arrives.
    #[must_use]
    pub const fn depth(self) -> ParamId {
        self.depth
    }

    /// Returns the destination value that names `at`, if one does.
    ///
    /// The value table's own join, read backwards. `ValueEntry::parameters`
    /// says which program parameters a destination moves, so the destination
    /// for a control is the entry that lists it — and where several do, the one
    /// that lists the fewest, because `All Attack` moves three envelopes and
    /// `VCF Attack` moves one and somebody who took hold of the filter's attack
    /// meant the filter's.
    ///
    /// `None` for every control the matrix cannot reach, which is most of them:
    /// 130 destinations against 242 parameters, and a good many of the 130 name
    /// something no program parameter addresses at all.
    #[must_use]
    pub fn names(self, at: ParamId, firmware: Version) -> Option<u8> {
        let Kind::Enumerated(table) = self.destination.kind() else {
            return None;
        };
        table
            .table_for(firmware)
            .entries
            .iter()
            .filter(|entry| entry.parameters.contains(&at))
            .min_by_key(|entry| (entry.parameters.len(), entry.value))
            .and_then(|entry| u8::try_from(entry.value).ok())
    }

    /// Returns whether the matrix can be pointed at `at` at all.
    #[must_use]
    pub fn reaches(self, at: ParamId, firmware: Version) -> bool {
        self.names(at, firmware).is_some()
    }

    /// Returns the depth a drag from `from` to `to` on `at` asks for.
    ///
    /// The travel as a fraction of the control's own range, laid onto the
    /// depth's own range about its own centre. Dragging a control a third of
    /// the way up asks for a third of the depth upwards; dragging it down asks
    /// for the same amount the other way, which on the ten bipolar parameters
    /// whose range is not symmetric is not the same number of bytes.
    ///
    /// **What full depth is worth is assumed**, and it is the one assumption on
    /// this page: see the module's own note and
    /// [deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38).
    #[must_use]
    pub fn depth_of(self, at: ParamId, from: u8, to: u8) -> u8 {
        let span = f32::from(at.max().saturating_sub(at.min())).max(1.0);
        let travelled = (f32::from(to) - f32::from(from)) / span;
        let low = f32::from(self.depth.min());
        let high = f32::from(self.depth.max());
        let asked = match self.depth.shape() {
            Shape::Bipolar { centre } => {
                let centre = f32::from(centre);
                // The two halves are measured separately because they are not
                // always the same size: ten of the forty-five bipolar
                // parameters run one further below their centre than above it.
                let reach = if travelled < 0.0 {
                    centre - low
                } else {
                    high - centre
                };
                centre + travelled * reach
            }
            // A depth a later library makes unipolar has no direction to give,
            // so the drag's size is all of it and which way it went is nothing.
            _ => low + travelled.abs() * (high - low),
        };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to the parameter's own range, which is a byte either end"
        )]
        let byte = asked.clamp(low, high).round() as u8;
        byte
    }
}

/// What one routing the patch already holds does to one control.
///
/// The other seven rows, carried to every control while the eighth is being
/// mapped. Somebody choosing where a routing goes is choosing against what is
/// already there — a second routing onto the same filter corner is a thing
/// people do on purpose and a thing people do by accident, and the difference
/// is whether they could see the first one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// The control it lands on.
    at: ParamId,
    /// What the library calls the routing that lands there: `Mod 3`.
    label: &'static str,
    /// The parameter that holds how much of it arrives.
    ///
    /// The parameter and not just its byte, because how far a depth reaches is
    /// read against that depth's own range and centre — which is the library's
    /// to say, the same as everywhere else in this window.
    of: ParamId,
    /// The byte that parameter holds.
    depth: u8,
}

impl Reach {
    /// A routing that lands on `at`, with `depth` in `of`.
    pub(crate) const fn new(at: ParamId, label: &'static str, of: ParamId, depth: u8) -> Self {
        Self {
            at,
            label,
            of,
            depth,
        }
    }

    /// Returns the control this one lands on.
    #[must_use]
    pub const fn at(self) -> ParamId {
        self.at
    }

    /// Returns what the library calls the routing.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.label
    }

    /// Returns how far it can push a control sitting at `value`, as two
    /// fractions of that control's own travel.
    ///
    /// Low end first, both between nothing and one, and equal where the depth
    /// is at its centre — a routing with no depth in it moves nothing, which is
    /// a band of no width rather than no band.
    ///
    /// **What full depth is worth is assumed**, exactly as it is for a drag:
    /// full depth is taken to move the control over the whole of its range.
    /// The manual does not print the law and the library refuses to guess, so
    /// this band is the same claim the drag makes, drawn instead of typed. It
    /// is asked for in
    /// [deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38).
    ///
    /// Which *way* it swings is assumed too, and it is the second thing: a
    /// routing from an LFO swings a control about where it sits and one from an
    /// envelope rides up from it, and what a source does with a depth is not
    /// published either. So the band runs from where the control sits to as far
    /// as the depth reaches in the direction the depth's own sign gives, which
    /// is what the fader holding it already says out loud.
    #[must_use]
    pub fn swing(self, value: u8) -> (f32, f32) {
        let span = f32::from(self.at.max().saturating_sub(self.at.min())).max(1.0);
        let sits = ((f32::from(value) - f32::from(self.at.min())) / span).clamp(0.0, 1.0);
        let depth = f32::from(self.depth);
        let low = f32::from(self.of.min());
        let high = f32::from(self.of.max());
        let reaches = match self.of.shape() {
            Shape::Bipolar { centre } => {
                let centre = f32::from(centre);
                if depth < centre {
                    -(centre - depth) / (centre - low).max(1.0)
                } else {
                    (depth - centre) / (high - centre).max(1.0)
                }
            }
            // A depth a later library makes unipolar has no direction in it, so
            // all of it is upwards — the same reading the drag takes.
            _ => (depth - low) / (high - low).max(1.0),
        };
        let lands = (sits + reaches).clamp(0.0, 1.0);
        (sits.min(lands), sits.max(lands))
    }
}

/// What the modulation matrix has sent out into the window.
///
/// The routing being mapped, and what the patch's other routings already reach
/// — the second of those only matters while the first is up, which is why they
/// travel together rather than as two arguments every control has to carry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sent<'a> {
    mapping: Mapping,
    reaches: &'a [Reach],
}

impl<'a> Sent<'a> {
    /// The routing being mapped, against what is already there.
    pub(crate) const fn new(mapping: Mapping, reaches: &'a [Reach]) -> Self {
        Self { mapping, reaches }
    }

    /// Returns the routing being mapped.
    pub(crate) const fn mapping(self) -> Mapping {
        self.mapping
    }

    /// Returns what already lands on `at`, in the order the matrix reads the
    /// routings.
    pub(crate) fn already(self, at: ParamId) -> impl Iterator<Item = Reach> + 'a {
        self.reaches
            .iter()
            .copied()
            .filter(move |reach| reach.at() == at)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::param::{DEFAULT_FIRMWARE, ParamId, Shape};

    use super::Mapping;

    /// The first routing, which is the one every test here maps.
    fn first() -> Mapping {
        Mapping::new("Mod 1", ParamId::Mod1Destination, ParamId::Mod1Depth)
    }

    #[test]
    fn a_control_the_matrix_can_reach_names_the_destination_that_reaches_it() {
        let mapped = first();
        let byte = mapped
            .names(ParamId::VcfFrequency, DEFAULT_FIRMWARE)
            .expect("the filter's corner is a modulation destination");

        // And the join agrees in the direction the rest of the window reads it.
        let deepmind_midi::param::Kind::Enumerated(table) = ParamId::Mod1Destination.kind() else {
            unreachable!("a destination is chosen from a table")
        };
        assert!(
            table
                .table_for(DEFAULT_FIRMWARE)
                .parameters_of(u16::from(byte))
                .contains(&ParamId::VcfFrequency)
        );
    }

    #[test]
    fn a_destination_that_moves_the_fewest_is_the_one_a_control_names() {
        // `All Attack` and `VCF Attack` both move the filter envelope's attack,
        // and somebody who took hold of the filter envelope's attack meant the
        // filter's. Asked of the table rather than of the two names: whichever
        // entries list the parameter, the narrowest is the answer.
        let mapped = first();
        let at = ParamId::VcfEnvelopeAttackTime;
        let byte = mapped
            .names(at, DEFAULT_FIRMWARE)
            .expect("the filter envelope's attack is a destination");
        let deepmind_midi::param::Kind::Enumerated(table) = ParamId::Mod1Destination.kind() else {
            unreachable!("a destination is chosen from a table")
        };
        let table = table.table_for(DEFAULT_FIRMWARE);
        let narrowest = table
            .entries
            .iter()
            .filter(|entry| entry.parameters.contains(&at))
            .map(|entry| entry.parameters.len())
            .min()
            .expect("something reaches it");

        assert_eq!(table.parameters_of(u16::from(byte)).len(), narrowest);
    }

    #[test]
    fn what_a_routing_chooses_with_is_not_something_it_can_be_pointed_at() {
        // Where a routing goes and what it comes from are not values anything
        // modulates, so neither lights while the matrix is mapping. How much
        // arrives is: the instrument's own table has a destination for every
        // one of the eight depths, which is what makes a routing that moves
        // another routing's depth possible — so that one lights, and the table
        // is what says so rather than a rule written here.
        let mapped = first();

        assert!(!mapped.reaches(ParamId::Mod1Source, DEFAULT_FIRMWARE));
        assert!(!mapped.reaches(ParamId::Mod1Destination, DEFAULT_FIRMWARE));
        assert!(mapped.reaches(ParamId::Mod1Depth, DEFAULT_FIRMWARE));
    }

    #[test]
    fn every_routing_gets_a_list_of_its_own() {
        use deepmind_midi::param::Group;

        use super::Mapper;

        let mapper = Mapper::new(DEFAULT_FIRMWARE);
        let routings = crate::matrix::of(Group::ModMatrix).expect("the matrix is one");

        for routing in &routings {
            assert!(
                mapper.list(routing.destination()).is_some(),
                "{:?} has nothing to be searched in",
                routing.destination()
            );
        }
        // One each rather than one shared: what a list of this kind remembers
        // is what has been typed into it, and eight rows sharing that would be
        // eight rows narrowing together.
        for pair in routings.windows(2) {
            let [first, second] = pair else { continue };
            let one = mapper.list(first.destination()).expect("a list");
            let other = mapper.list(second.destination()).expect("a list");
            assert!(
                !std::ptr::eq(one, other),
                "two routings are searched in the same list"
            );
        }
    }

    #[test]
    fn nothing_is_mapped_until_something_is() {
        use super::Mapper;

        let mut mapper = Mapper::new(DEFAULT_FIRMWARE);
        assert!(mapper.mapped().is_none());

        mapper.map(Some(first()));
        assert_eq!(mapper.mapped(), Some(first()));

        mapper.map(None);
        assert!(mapper.mapped().is_none());
    }

    #[test]
    fn a_routing_with_no_depth_in_it_sweeps_nothing() {
        use super::Reach;

        let Shape::Bipolar { centre } = ParamId::Mod1Depth.shape() else {
            unreachable!("a depth is read about its centre")
        };
        let at = ParamId::VcfFrequency;
        let reach = Reach::new(
            at,
            "Mod 1",
            ParamId::Mod1Depth,
            u8::try_from(centre).expect("a byte"),
        );
        let (from, to) = reach.swing(64);

        assert!(
            (from - to).abs() < f32::EPSILON,
            "a depth at its centre swept {from}..{to}"
        );
    }

    #[test]
    fn a_routing_at_full_depth_sweeps_the_rest_of_the_control() {
        use super::Reach;

        // The whole of the assumption, from the other end: full depth is taken
        // to move the control over all of its range, so a routing at full depth
        // onto a control sitting at the bottom of its own range reaches the
        // top of it — and one onto a control already at the top has nowhere
        // left to go and says so.
        let at = ParamId::VcfFrequency;
        let full = u8::try_from(ParamId::Mod1Depth.max()).expect("a byte");
        let reach = Reach::new(at, "Mod 1", ParamId::Mod1Depth, full);
        let low = u8::try_from(at.min()).expect("a byte");
        let high = u8::try_from(at.max()).expect("a byte");

        assert_eq!(reach.swing(low), (0.0, 1.0));
        assert_eq!(reach.swing(high), (1.0, 1.0));
    }

    #[test]
    fn a_routing_below_the_centre_sweeps_downwards() {
        use super::Reach;

        // Which way a depth swings is the depth's own sign, and a band that
        // drew a negative depth as an upward sweep would be a band saying the
        // opposite of the number under the fader that holds it.
        let at = ParamId::VcfFrequency;
        let none = u8::try_from(ParamId::Mod1Depth.min()).expect("a byte");
        let reach = Reach::new(at, "Mod 1", ParamId::Mod1Depth, none);
        let middle = u8::try_from(u16::midpoint(at.min(), at.max())).expect("a byte");
        let (from, to) = reach.swing(middle);

        assert!(from < to, "nothing was swept");
        assert!(
            to <= 0.51,
            "a depth at its floor reached up to {to} from the middle"
        );
    }

    #[test]
    fn a_drag_that_went_nowhere_asks_for_no_depth() {
        let mapped = first();
        let Shape::Bipolar { centre } = ParamId::Mod1Depth.shape() else {
            unreachable!("a depth is read about its centre")
        };
        let centre = u8::try_from(centre).expect("a byte");

        assert_eq!(mapped.depth_of(ParamId::VcfFrequency, 100, 100), centre);
    }

    #[test]
    fn a_drag_to_the_top_of_a_control_asks_for_all_of_the_depth() {
        let mapped = first();
        let at = ParamId::VcfFrequency;
        let low = u8::try_from(at.min()).expect("a byte");
        let high = u8::try_from(at.max()).expect("a byte");

        assert_eq!(
            mapped.depth_of(at, low, high),
            u8::try_from(ParamId::Mod1Depth.max()).expect("a byte")
        );
        assert_eq!(
            mapped.depth_of(at, high, low),
            u8::try_from(ParamId::Mod1Depth.min()).expect("a byte")
        );
    }

    #[test]
    fn half_a_range_asks_for_half_the_depth() {
        let mapped = first();
        let at = ParamId::VcfFrequency;
        let Shape::Bipolar { centre } = ParamId::Mod1Depth.shape() else {
            unreachable!("a depth is read about its centre")
        };
        let high = ParamId::Mod1Depth.max();
        let half = f32::from(high - centre) / 2.0;

        let asked = mapped.depth_of(
            at,
            u8::try_from(at.min()).expect("a byte"),
            u8::try_from(at.min() + (at.max() - at.min()) / 2).expect("a byte"),
        );

        assert!(
            f32::from(u16::from(asked) - centre) - half < 1.0,
            "half the travel asked for {asked} rather than half the depth"
        );
    }
}

/// What the modulation matrix is asking the window for, and what it asks with.
///
/// The application owns one and hands it to every surface, because a routing
/// mapped onto the window is mapped onto all of it: the front panel, the fourteen
/// racks and the effects page are one instrument and the matrix can reach
/// controls on each of them.
///
/// Two things, and they are the two ways of answering the same question. The
/// [`Mapping`] routing is the one being mapped, if any. The lists are how a
/// destination is chosen without mapping onto anything — one searchable list per
/// routing, so that eight rows can be typed into without sharing a caret.
///
/// One per routing rather than one shared, because what a list of this kind
/// remembers is what has been typed into it, and eight rows sharing that would
/// be eight rows narrowing together.
#[derive(Debug)]
pub struct Mapper {
    /// The routing being mapped onto the window, if one is.
    mapped: Option<Mapping>,
    /// The firmware the lists were built for, so that an inquiry that changes
    /// it rebuilds them: firmware 1.1 renumbered the destinations, and a list
    /// built for the other one would offer the wrong names for the right bytes.
    firmware: Version,
    /// The searchable list each routing's destination is chosen from.
    lists: Vec<(ParamId, combo_box::State<Choice>)>,
}

impl Mapper {
    /// The lists a window opens with, built for `firmware`'s own tables.
    #[must_use]
    pub fn new(firmware: Version) -> Self {
        Self {
            mapped: None,
            firmware,
            lists: built(firmware),
        }
    }

    /// Rebuilds the lists if the firmware that answered is not the one they
    /// were built for.
    ///
    /// Called where the window redraws rather than where an inquiry lands, so
    /// that there is one place this can be forgotten rather than several.
    pub fn reading(&mut self, firmware: Version) {
        if self.firmware != firmware {
            self.firmware = firmware;
            self.lists = built(firmware);
        }
    }

    /// Returns the routing mapped onto the window, if one is.
    #[must_use]
    pub const fn mapped(&self) -> Option<Mapping> {
        self.mapped
    }

    /// Maps a routing onto the window, or stops mapping.
    pub const fn map(&mut self, at: Option<Mapping>) {
        self.mapped = at;
    }

    /// Returns the searchable list a routing's destination is chosen from.
    pub(crate) fn list(&self, destination: ParamId) -> Option<&combo_box::State<Choice>> {
        self.lists
            .iter()
            .find(|(parameter, _)| *parameter == destination)
            .map(|(_, list)| list)
    }
}

/// One searchable list per routing, in the order the matrix reads them.
///
/// The names are the destination parameter's own value table, asked of the
/// library for the firmware that answered, which is the same call the rack
/// makes when it draws that parameter as a list. A routing whose destination
/// the table does not name every value of gets no list, and its row keeps the
/// control the library says the parameter is.
fn built(firmware: Version) -> Vec<(ParamId, combo_box::State<Choice>)> {
    Group::ORDER
        .iter()
        .copied()
        .filter_map(matrix::of)
        .flatten()
        .filter_map(|routing| {
            let destination = routing.destination();
            let options = choices(destination, firmware, None)?;
            Some((destination, combo_box::State::new(options)))
        })
        .collect()
}

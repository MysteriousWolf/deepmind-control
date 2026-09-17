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
//! [`Mapping`] is one routing, the three parameters the matrix reads as a
//! sentence, carried to every control in the window while somebody is choosing.
//! It is small and it is [`Copy`], because it reaches every control
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
//! Not a walk here either. `ValueEntry::parameters` says what a destination
//! moves, and 26.5 publishes the join read backwards as
//! `ValueTable::values_naming`, the destinations that reach a control narrowest
//! first, so [`Mapping::names`] is one call and a `next`.
//!
//! Narrowest first used to be this window's ordering, and an ordering is a
//! judgement about the instrument however honest the slices under it are:
//! `All Attack` moves three envelopes and `VCF Attack` moves one, and saying
//! that somebody who took hold of the filter envelope's attack fader meant the
//! filter's is a claim about what the matrix is for. It belonged on the other
//! side of the split and it is there now
//! ([deepmind-midi#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39)),
//! down to what happens when two destinations move the same number of
//! parameters, which the library answers by putting them in value order and
//! saying outright that nothing makes one of them the narrower.
//!
//! A destination that moves nothing a program parameter addresses, such as the
//! pitch a key is playing or the amplitude of a voice, names no control, so no
//! control lights for it. That is the honest answer rather than a gap.
//!
//! # What a drag is worth, and the one thing it still assumes
//!
//! **A drag is read as if full depth moved the control over its whole range.**
//! Dragging `VCF Frequency` a third of the way up asks for a third of the
//! depth. That is an assumption and it is the last one on this page.
//!
//! What changed in 26.5 is where it is asked.
//! [`ParamId::modulation_reach`] is the accessor
//! ([deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38)),
//! and it answers `None` for every pair today, because the manual prints the
//! depth's own range and nothing relating a depth to what it does at the end of
//! the routing, and nobody has measured it. So the number is still the whole
//! range; it is now a fallback behind a question this window asks, in one
//! place ([`reach_of`]), rather than a fraction written into two of them. The
//! day a measurement lands in the library the fallback stops being reached and
//! nothing here changes.
//!
//! Until then the gesture is a way of *saying* an amount rather than a promise
//! about what will be heard, which is what a depth fader already was: the fader
//! that the drag moves is the same fader, holding the same byte, and the drag
//! is a second way to put a number in it.

use deepmind_midi::param::{Group, Kind, ParamId, Shape, Swing};
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
    /// `ValueTable::values_naming`, which is the library's own join read
    /// backwards and ordered narrowest first, and `next` is what it says to
    /// take for one destination. The walk and the ranking used to be here; see
    /// the module's note on why an ordering is not a window's to make.
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
            .values_naming(at)
            .next()
            .and_then(|value| u8::try_from(value).ok())
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
    /// **What full depth is worth is asked of the library and assumed where it
    /// has no answer**, which is everywhere today: see `reach_of` and the
    /// module's own note.
    #[must_use]
    pub fn depth_of(self, at: ParamId, from: u8, to: u8) -> u8 {
        let span = f32::from(at.max().saturating_sub(at.min())).max(1.0);
        // How far the control was dragged, in depths rather than in its own
        // travel: a depth that covered half the range would need twice as much
        // of itself to move the control the same distance.
        let travelled =
            (f32::from(to) - f32::from(from)) / span / reach_of(self.depth, at).max(f32::EPSILON);
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

/// Returns how much of `at`'s own range a full `depth` covers.
///
/// [`ParamId::modulation_reach`] where the library establishes it, and the
/// whole range where it does not. It does not, on any pair, today: the manual
/// prints what a `Mod n Depth` byte ranges over and never what that does at the
/// other end of a routing, and the library will not guess
/// ([deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38)).
///
/// The fallback is written once, here, and both things that need it go through
/// it: the drag that puts a number in a depth, and the band that says how far
/// the depths already there can push a control. An assumption in two places is
/// an assumption that gets corrected in one of them.
fn reach_of(depth: ParamId, at: ParamId) -> f32 {
    depth.modulation_reach(at).unwrap_or(1.0)
}

/// What one routing the patch already holds does to one control.
///
/// The other seven rows, carried to every control while the eighth is being
/// mapped. Somebody choosing where a routing goes is choosing against what is
/// already there: a second routing onto the same filter corner is a thing
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
    /// read against that depth's own range and centre, which is the library's to
    /// say, the same as everywhere else in this window.
    of: ParamId,
    /// The byte that parameter holds.
    depth: u8,
    /// Which way the source at the other end of the routing moves things.
    ///
    /// `ValueTable::swing_of`, read once where the routing is: it is a fact
    /// about the source and the same for every routing that uses it, so it is
    /// carried rather than looked up per control. `None` for a source the
    /// specification does not settle and for `Off`, which moves nothing.
    swings: Option<Swing>,
}

impl Reach {
    /// A routing that lands on `at`, with `depth` in `of`, from a source that
    /// `swings` the way the library says it does.
    pub(crate) const fn new(
        at: ParamId,
        label: &'static str,
        of: ParamId,
        depth: u8,
        swings: Option<Swing>,
    ) -> Self {
        Self {
            at,
            label,
            of,
            depth,
            swings,
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
    /// Low end first, both between nothing and one, and equal where the depth is
    /// at its centre: a routing with no depth in it moves nothing, which is a
    /// band of no width rather than no band.
    ///
    /// # Which way it swings
    ///
    /// The source's, which 26.5 publishes as `Swing`
    /// ([deepmind-midi#41](https://github.com/MysteriousWolf/deepmind-midi/issues/41)).
    /// This window used to take the depth's own sign as the whole answer, which
    /// is right for a wheel and for an envelope and wrong for an LFO: an LFO at
    /// depth swings a control up *and* down about where it sits, and a band
    /// drawn one way from it said a filter could only ever open.
    ///
    /// So a `Centred` source gets a band either side of where the control sits
    /// and a `Rising` one gets a band from it, and the depth's sign is what it
    /// always was on the second, which is which way round the routing applies
    /// the source. A source the specification does not settle, which is `Off`
    /// and the note number, whose zero is printed nowhere, keeps the old
    /// reading, because a band from where the control sits is the narrower
    /// claim of the two.
    ///
    /// # What full depth is worth
    ///
    /// Asked of `reach_of`, the same as the drag asks, and assumed to be the
    /// whole range until the library has an answer. Which way a source swings
    /// and how far a depth reaches are two facts on two parameters, and only
    /// the first of them has landed.
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
            // all of it is upwards, which is the reading the drag takes.
            _ => (depth - low) / (high - low).max(1.0),
        } * reach_of(self.of, self.at);
        if self.swings == Some(Swing::Centred) {
            // Either side, and the sign says nothing: an inverted LFO reaches
            // exactly as far as an upright one and reaches it both ways.
            let reaches = reaches.abs();
            return (
                (sits - reaches).clamp(0.0, 1.0),
                (sits + reaches).clamp(0.0, 1.0),
            );
        }
        let lands = (sits + reaches).clamp(0.0, 1.0);
        (sits.min(lands), sits.max(lands))
    }
}

/// What the modulation matrix has sent out into the window.
///
/// The routing being mapped, and what the patch's other routings already reach.
/// The second only matters while the first is up, which is why they travel
/// together rather than as two arguments every control has to carry.
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
    use deepmind_midi::param::{DEFAULT_FIRMWARE, ParamId, Shape, Swing};

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
        // another routing's depth possible, so that one lights and the table is
        // what says so rather than a rule written here.
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
            Some(Swing::Rising),
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
        // onto a control sitting at the bottom of its own range reaches the top
        // of it, and one onto a control already at the top has nowhere left to
        // go and says so.
        let at = ParamId::VcfFrequency;
        let full = u8::try_from(ParamId::Mod1Depth.max()).expect("a byte");
        let reach = Reach::new(at, "Mod 1", ParamId::Mod1Depth, full, Some(Swing::Rising));
        let low = u8::try_from(at.min()).expect("a byte");
        let high = u8::try_from(at.max()).expect("a byte");

        assert_eq!(reach.swing(low), (0.0, 1.0));
        assert_eq!(reach.swing(high), (1.0, 1.0));
    }

    #[test]
    fn a_routing_below_the_centre_sweeps_downwards() {
        use super::Reach;

        // For a source that rides up from where it rests, which way a depth
        // swings is the depth's own sign: a band that drew a negative depth as
        // an upward sweep would say the opposite of the number under the fader
        // that holds it.
        let at = ParamId::VcfFrequency;
        let none = u8::try_from(ParamId::Mod1Depth.min()).expect("a byte");
        let reach = Reach::new(at, "Mod 1", ParamId::Mod1Depth, none, Some(Swing::Rising));
        let middle = u8::try_from(u16::midpoint(at.min(), at.max())).expect("a byte");
        let (from, to) = reach.swing(middle);

        assert!(from < to, "nothing was swept");
        assert!(
            to <= 0.51,
            "a depth at its floor reached up to {to} from the middle"
        );
    }

    #[test]
    fn a_centred_source_swings_a_control_both_ways() {
        use super::Reach;

        // The whole of what 26.5 answered here: an LFO at depth moves a
        // control up and down about where it sits, so a band drawn one way
        // from it said a filter could only ever open. The depth's sign says
        // which way round the routing applies the source and nothing about how
        // far it reaches, so an inverted LFO draws the same band.
        let at = ParamId::VcfFrequency;
        let middle = u8::try_from(u16::midpoint(at.min(), at.max())).expect("a byte");
        let floor = u8::try_from(ParamId::Mod1Depth.min()).expect("a byte");
        let ceiling = u8::try_from(ParamId::Mod1Depth.max()).expect("a byte");

        let down = Reach::new(at, "Mod 1", ParamId::Mod1Depth, floor, Some(Swing::Centred));
        // Where the control sits, which is where a rising source's band would
        // start and where a centred one's is centred.
        let sits = f32::from(middle) / f32::from(u8::try_from(at.max()).expect("a byte"));
        let (from, to) = down.swing(middle);
        assert!(from < sits, "an LFO reached nothing below the control");
        assert!(to > sits, "an LFO reached nothing above the control");

        let up = Reach::new(
            at,
            "Mod 1",
            ParamId::Mod1Depth,
            ceiling,
            Some(Swing::Centred),
        );
        let (rose, fell) = up.swing(middle);
        assert!((rose - from).abs() < 0.02, "inverting it moved the band");
        assert!((fell - to).abs() < 0.02, "inverting it moved the band");

        // And a rising source is the band it always was.
        let rising = Reach::new(
            at,
            "Mod 1",
            ParamId::Mod1Depth,
            ceiling,
            Some(Swing::Rising),
        );
        let (starts, _) = rising.swing(middle);
        assert!(
            (starts - sits).abs() < f32::EPSILON,
            "a rising source's band left the control at {starts}"
        );
    }

    #[test]
    fn a_control_the_library_has_no_law_for_is_swept_over_its_whole_range() {
        // The assumption behind both the drag and the band, asked the way the
        // window asks it. The library answers `None` for every pair today, so
        // this is the fallback; the day it answers a fraction, this test is
        // what says the window stopped assuming.
        let reach = ParamId::Mod1Depth.modulation_reach(ParamId::VcfFrequency);

        assert_eq!(
            reach, None,
            "the library has an answer and nothing reads it"
        );
        assert!(
            (super::reach_of(ParamId::Mod1Depth, ParamId::VcfFrequency) - 1.0).abs() < f32::EPSILON
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
/// destination is chosen without mapping onto anything: one searchable list per
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
    /// The searchable list each end of each routing is chosen from.
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
    pub(crate) fn list(&self, end: ParamId) -> Option<&combo_box::State<Choice>> {
        self.lists
            .iter()
            .find(|(parameter, _)| *parameter == end)
            .map(|(_, list)| list)
    }
}

/// One searchable list per end of every routing, in the order the matrix reads
/// them.
///
/// Both ends, because the two columns are one control drawn twice. A source is
/// one of 24 names and a destination one of 133, which is a difference in how
/// far somebody scrolls and in nothing else. A row answering the short list with
/// a picker and the long one with a field is a row wearing two controls for one
/// question.
///
/// The names are the parameter's own value table, asked of the library for the
/// firmware that answered, which is the same call the rack makes when it draws
/// that parameter as a list. An end whose table does not name every value gets
/// no list, and its row keeps the control the library says the parameter is.
fn built(firmware: Version) -> Vec<(ParamId, combo_box::State<Choice>)> {
    Group::ORDER
        .iter()
        .copied()
        .filter_map(matrix::of)
        .flatten()
        .flat_map(|routing| [routing.source(), routing.destination()])
        .filter_map(|end| {
            let options = choices(end, firmware, None)?;
            Some((end, combo_box::State::new(options)))
        })
        .collect()
}

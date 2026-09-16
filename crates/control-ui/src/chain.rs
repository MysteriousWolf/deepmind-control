//! The chain: what the effects block does with the four engines it holds.
//!
//! `FX Routing` is one byte and ten topologies, and the value table names them
//! the way the manual does — `Parallel 1/2, parallel 3/4`. That sentence is
//! what a person reads and it is not a picture: four engines in a line, two
//! pairs side by side and a loop around the fourth are three different
//! instruments to play, and a list that names them makes somebody hold the
//! difference in their head.
//!
//! `deepmind-midi` 26.3 publishes the same ten as edge lists, which is what
//! [deepmind-midi#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23)
//! was asked for: [`Routing::feeds`] is what reaches an engine, [`Routing::output`]
//! is which engines are summed to leave the block, [`Routing::is_feedback`]
//! says when one of them is in a loop, and [`Mode`] says what each of the three
//! `FX Mode` settings does to the two paths through the instrument. So this is
//! the topology drawn rather than named, out of the library's own graph.
//!
//! # Nothing here knows a topology by name
//!
//! Which engine the input reaches, what feeds what, where the loop taps and
//! what is summed at the end are all asked of the routing the byte selects. A
//! picture assembled by matching `Serial 1-2-3-4` against a string would be
//! this repository holding an eleventh copy of the table, wrong on the day a
//! firmware renumbers one — which is the whole reason the library was asked for
//! the graph instead of the names.
//!
//! Where an engine stands is this file's, because the library publishes what is
//! wired to what and not where to draw it. Columns are how far an engine is
//! from the input, worked out from the edges themselves, and an edge that runs
//! backwards through those columns is the loop: the two topologies the library
//! flags as feedback are the two that have one, and the picture agrees with the
//! flag rather than being told by it.
//!
//! # It is drawn on the instrument's own glass
//!
//! A dot matrix at the pitch every display in this window shares, over the two
//! settings it is a picture of, which is the arrangement the front panel's
//! plates already have. A `DeepMind` cannot show this — its own FX page is a
//! list — and that is the same reason the panel's plates carry drawings the
//! hardware has no room for.

use deepmind_midi::effect::{ENGINE_COUNT, Engine, Mode, Routing, Source};
use deepmind_midi::sysex::inquiry::Version;
use iced_core::Length;
use iced_widget::{container, responsive};

use crate::lcd::{self, Band, Ink, Screen, Size};
use crate::{Confidence, Element, Patch};

/// How many dots tall the glass is with nothing read.
///
/// One row of engines' worth, so that the panel does not jump when a sound
/// arrives on a topology that stacks two.
const BLANK: i32 = HEADING + PLATE + SPARE;

/// How many dots tall one engine's plate is drawn.
///
/// A line of the display's own face with a dot of glass above and below it
/// inside the box. Written rather than shared out, because a box as tall as the
/// room it was given is a box with one word adrift in the middle of it: what a
/// column of four needs is what a column of one should look like.
const PLATE: i32 = 13;

/// How many dots the line at the top of the glass takes, gap and all.
const HEADING: i32 = Screen::height_of(Size::Small) + 4;

/// How many dots the lane under the graph takes, where there is something in it.
///
/// The loop and the analog path are both drawn there, which is the one thing
/// they have in common: they are the two ways a signal goes somewhere other
/// than along the chain. A topology with neither does not pay for the lane —
/// see [`Chain::under`] — because a picture with a band of empty glass under it
/// is a display saying there is something to look at.
const UNDER: i32 = 11;

/// How much glass is left under a graph with nothing in that lane.
const SPARE: i32 = 2;

/// How much glass the rails at either end are given.
///
/// Enough for `IN` and `OUT` and the rule that leads away from each.
const IN: i32 = 16;

/// The same at the far end, where the word is a letter longer.
const OUT: i32 = 22;

/// How many dots of glass stand between two columns of engines.
const BETWEEN: i32 = 9;

/// How many dots of glass stand between two engines of one column.
const APART: i32 = 2;

/// What the effects block is doing, as far as anything has been read.
///
/// Everything the drawing needs and nothing that borrows the patch: the picture
/// is laid out once the window knows how wide the glass is, and a closure that
/// held onto the sound would tie this display to the frame it was built in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Chain {
    /// The topology the `FX Routing` byte selects.
    routing: &'static Routing,
    /// What the `FX Mode` byte does to the two paths, where it names one.
    mode: Option<&'static Mode>,
    /// What each engine is running, as the instrument's display abbreviates it.
    running: [Option<&'static str>; ENGINE_COUNT],
}

impl Chain {
    /// Returns what is wired, when the byte that says so has been read.
    ///
    /// `None` for a routing nobody has read and for a byte the table does not
    /// name, which are the same answer: there is no chain to draw, and a
    /// picture of one would be this window inventing a topology.
    fn read(patch: &Patch, firmware: Version) -> Option<Self> {
        let routing = Routing::for_value(patch.value(Routing::parameter())?)?;
        let mut running = [None; ENGINE_COUNT];
        for engine in Engine::ALL {
            let loaded = patch
                .value(engine.algorithm_parameter())
                .and_then(|value| deepmind_midi::effect::Algorithm::for_value(value, firmware))
                .map(|algorithm| algorithm.name);
            if let Some(slot) = running.get_mut(usize::from(engine.index())) {
                *slot = loaded;
            }
        }
        Some(Self {
            routing,
            mode: patch.value(Mode::parameter()).and_then(Mode::for_value),
            running,
        })
    }

    /// Returns how far each engine stands from the block's input.
    ///
    /// The input's own engines are at nothing, and everything else is one past
    /// the furthest engine feeding it. Worked out by settling rather than by
    /// walking, because two of the ten topologies are loops and a walk through
    /// a loop does not end: an engine whose feeds are all still unplaced waits
    /// for the next pass, and the pass that places nothing is the one that
    /// stops.
    ///
    /// What is left unplaced after that is an engine reachable only through the
    /// loop, and it is put past everything placed, in the instrument's own
    /// order. That is what makes a loop draw as a loop: the edge from it back
    /// into the chain runs the wrong way through these columns, which is how
    /// [`edges`](Self::edges) tells the two apart without being told.
    fn columns(self) -> [usize; ENGINE_COUNT] {
        let mut at = [None; ENGINE_COUNT];
        for engine in Engine::ALL {
            if self.feeds(engine).contains(&Source::Input)
                && let Some(slot) = at.get_mut(usize::from(engine.index()))
            {
                *slot = Some(0);
            }
        }
        loop {
            let mut settled = false;
            for engine in Engine::ALL {
                let index = usize::from(engine.index());
                if at.get(index).copied().flatten().is_some() {
                    continue;
                }
                let furthest = self
                    .feeds(engine)
                    .iter()
                    .filter_map(|source| match source {
                        Source::Input => Some(0),
                        Source::Engine(from) => {
                            at.get(usize::from(from.index())).copied().flatten()
                        }
                        // A source a later library adds is a source this
                        // picture cannot place, and an engine placed at a guess
                        // is a wire drawn where none was published.
                        _ => None,
                    })
                    .max();
                if let Some(furthest) = furthest
                    && let Some(slot) = at.get_mut(index)
                {
                    *slot = Some(furthest + 1);
                    settled = true;
                }
            }
            if !settled {
                break;
            }
        }
        let past = at.iter().flatten().copied().max().map_or(0, |at| at + 1);
        let mut spare = past;
        let mut columns = [0; ENGINE_COUNT];
        for engine in Engine::ALL {
            let index = usize::from(engine.index());
            let at = at.get(index).copied().flatten().unwrap_or_else(|| {
                let column = spare;
                spare += 1;
                column
            });
            if let Some(slot) = columns.get_mut(index) {
                *slot = at;
            }
        }
        columns
    }

    /// Returns what reaches `engine`.
    fn feeds(self, engine: Engine) -> &'static [Source] {
        self.routing.feeds(engine)
    }

    /// Returns how many dots of glass this picture wants.
    ///
    /// As tall as the topology is, which is what makes a serial chain a strip
    /// and four engines in parallel a block: a display cut to the tallest of
    /// the ten would draw `Serial 1-2-3-4` with three rows of empty glass under
    /// it, and one cut to the shortest would draw `Parallel 1/2/3/4` with two
    /// of its engines missing.
    fn rows(self) -> i32 {
        let stacked = i32(self.stacked());
        HEADING + stacked * PLATE + (stacked - 1) * APART + self.under()
    }

    /// Returns how much room the lane under the graph is given.
    fn under(self) -> i32 {
        if self.routing.is_feedback() || self.mode.is_some_and(Mode::analog_path) {
            UNDER
        } else {
            SPARE
        }
    }

    /// Returns the most engines any one column of the picture holds.
    fn stacked(self) -> usize {
        let columns = self.columns();
        columns
            .iter()
            .map(|at| columns.iter().filter(|other| *other == at).count())
            .max()
            .unwrap_or(1)
            .max(1)
    }

    /// Returns whether the engines run at all, which is what `Bypass` answers
    /// no to.
    ///
    /// Asked of the mode rather than matched against its name: `Bypass` is a
    /// true bypass with the DSP out of circuit, and the library says so because
    /// a host matching on the value table's word is a host that would call a
    /// muted block a bypassed one.
    fn running(self) -> bool {
        self.mode.is_none_or(Mode::digital_path)
    }
}

/// Draws the chain over the settings it is a picture of.
///
/// A blank screen where nothing has been read, which is the answer every
/// drawing in this window gives to the same question: the glass is there, lit
/// and empty, rather than the layout moving when a sound arrives.
pub(crate) fn display<'a, Renderer>(patch: &Patch, firmware: Version) -> Element<'a, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    let claim = patch.claim_across([Routing::parameter(), Mode::parameter()]);
    let chain = (!matches!(claim, Confidence::Unknown))
        .then(|| Chain::read(patch, firmware))
        .flatten();
    let rows = chain.map_or(BLANK, Chain::rows);
    container(responsive(move |room| {
        let mut screen = Screen::new(lcd::fits(room.width), rows);
        if let Some(chain) = chain {
            draw(&mut screen, chain);
        }
        lcd::lcd(screen, claim)
    }))
    .height(Length::Fixed(lcd::room(rows)))
    .into()
}

/// Returns what the specification records about a topology beyond its graph.
///
/// Which is where the loop taps, on the two that have one. Printed under the
/// glass rather than on it: it is a sentence and a display is a picture, and a
/// sentence set in a 5 by 7 face across a screen is a line nobody reads.
pub(crate) fn note(patch: &Patch) -> Option<&'static str> {
    Routing::for_value(patch.value(Routing::parameter())?)?.note()
}

/// Draws the whole picture onto `screen`.
fn draw(screen: &mut Screen, chain: Chain) {
    let ink = if chain.running() {
        Ink::Solid
    } else {
        // The block is out of circuit rather than quiet, so the chain is drawn
        // as something the signal is not going through.
        Ink::Dotted
    };
    heading(screen, chain);
    // A dot of glass either side, so that `OUT` stands on the display rather
    // than in the bezel it is cut into.
    let glass = Band::new(1, HEADING, screen.columns() - 2, screen.rows() - HEADING);
    let graph = Band::new(glass.x, glass.y, glass.width, glass.height - chain.under());
    let boxes = plates(chain, graph);
    for (engine, band) in Engine::ALL.into_iter().zip(boxes) {
        plate(screen, engine, chain, band, ink);
    }
    rails(screen, chain, &boxes, graph, ink);
    edges(screen, chain, &boxes, graph, ink);
    analog(screen, chain, glass);
}

/// Writes which topology this is, and what the mode is doing with it.
fn heading(screen: &mut Screen, chain: Chain) {
    let pen = screen.write(0, 0, chain.routing.label(), Size::Small);
    let mode = chain.mode.map_or("", Mode::name);
    // A dot of glass at either end, so that nothing is written into the bezel.
    let room = screen.columns() - Screen::width_of(mode, Size::Small) - 1;
    let name = chain.routing.name();
    if pen + 6 + Screen::width_of(name, Size::Small) < room {
        screen.write(pen + 6, 0, name, Size::Small);
    }
    if !mode.is_empty() {
        screen.write(room.max(pen + 6), 0, mode, Size::Small);
    }
}

/// Returns where each engine's plate stands on the glass.
///
/// The columns are the library's graph and the rows are this file's: a column
/// holding one engine centres it on the input's own line, and a column holding
/// three stacks them about it, so a parallel pair reads as a fork rather than
/// as two chains that happen to be near each other.
fn plates(chain: Chain, graph: Band) -> [Band; ENGINE_COUNT] {
    let columns = chain.columns();
    let across = columns.iter().copied().max().unwrap_or(0) + 1;
    let deepest = (0..across)
        .map(|column| columns.iter().filter(|at| **at == column).count())
        .max()
        .unwrap_or(1)
        .max(1);
    let inner = Band::new(
        graph.x + IN,
        graph.y,
        (graph.width - IN - OUT).max(0),
        graph.height,
    );
    let width = across_room(inner.width, across);
    let height = ((graph.height - APART * (i32(deepest) - 1)) / i32(deepest)).min(PLATE);
    let mut bands = [Band::new(0, 0, 0, 0); ENGINE_COUNT];
    for engine in Engine::ALL {
        let index = usize::from(engine.index());
        let column = columns.get(index).copied().unwrap_or(0);
        let stacked = columns
            .iter()
            .take(index)
            .filter(|at| **at == column)
            .count();
        let held = columns.iter().filter(|at| **at == column).count();
        // Centred on the graph's own middle, whatever this column holds.
        let block = i32(held) * height + i32(held.saturating_sub(1)) * APART;
        let top = graph.y + (graph.height - block) / 2;
        if let Some(slot) = bands.get_mut(index) {
            *slot = Band::new(
                inner.x + i32(column) * (width + BETWEEN),
                top + i32(stacked) * (height + APART),
                width,
                height,
            );
        }
    }
    bands
}

/// Returns how wide one column of engines is drawn.
fn across_room(room: i32, columns: usize) -> i32 {
    let gutters = i32(columns.saturating_sub(1)) * BETWEEN;
    ((room - gutters) / i32(columns).max(1)).max(6)
}

/// Counts a handful of columns as a coordinate does.
///
/// Four engines, four columns and four rows at the outside, so the conversion
/// is exact and the fallback is unreachable.
fn i32(of: usize) -> i32 {
    i32::try_from(of).unwrap_or(i32::MAX)
}

/// Draws one engine: a box, and what it is running written in it.
fn plate(screen: &mut Screen, engine: Engine, chain: Chain, band: Band, ink: Ink) {
    if band.width < 6 || band.height < Screen::height_of(Size::Small) + 2 {
        return;
    }
    screen.frame(band, ink);
    let number = engine.number().to_string();
    let running = chain
        .running
        .get(usize::from(engine.index()))
        .copied()
        .flatten();
    let named = running.map_or_else(|| number.clone(), |name| format!("{number} {name}"));
    let room = band.width - 4;
    // The abbreviation where it fits and the number where it does not: a name
    // written past the edge of its own box is a box belonging to whichever
    // engine the reader guesses.
    let written = if Screen::width_of(&named, Size::Small) <= room {
        named
    } else {
        number
    };
    let x = band.x + (band.width - Screen::width_of(&written, Size::Small)) / 2;
    let y = band.y + (band.height - Screen::height_of(Size::Small)) / 2;
    screen.write(x.max(band.x + 1), y, &written, Size::Small);
}

/// Draws what the block's input reaches, and what leaves it.
fn rails(screen: &mut Screen, chain: Chain, boxes: &[Band; ENGINE_COUNT], graph: Band, ink: Ink) {
    let middle = graph.y + graph.height / 2;
    screen.write(
        0,
        middle - Screen::height_of(Size::Small) / 2,
        "IN",
        Size::Small,
    );
    let entry = graph.x + IN - 4;
    for engine in Engine::ALL {
        if !chain.feeds(engine).contains(&Source::Input) {
            continue;
        }
        let Some(band) = boxes.get(usize::from(engine.index())) else {
            continue;
        };
        let into = band.y + band.height / 2;
        screen.down(entry, middle.min(into), (middle - into).abs() + 1, ink);
        screen.across(entry, into, band.x - entry, ink);
        arrow(screen, band.x - 1, into);
    }
    screen.across(
        Screen::width_of("IN", Size::Small) + 2,
        middle,
        entry - Screen::width_of("IN", Size::Small) - 1,
        ink,
    );
    let leaving = graph.x + graph.width - OUT + 4;
    for engine in chain.routing.output() {
        let Some(band) = boxes.get(usize::from(engine.index())) else {
            continue;
        };
        let out = band.y + band.height / 2;
        screen.across(
            band.x + band.width,
            out,
            leaving - band.x - band.width + 1,
            ink,
        );
        screen.down(leaving, middle.min(out), (middle - out).abs() + 1, ink);
    }
    let word = graph.x + graph.width - Screen::width_of("OUT", Size::Small) - 1;
    screen.across(leaving, middle, word - leaving - 2, ink);
    arrow(screen, word - 3, middle);
    screen.write(
        word,
        middle - Screen::height_of(Size::Small) / 2,
        "OUT",
        Size::Small,
    );
}

/// Draws what feeds what.
///
/// An edge that runs forward through the columns is a wire along the chain. One
/// that runs back is the loop: it is drawn under the engines rather than
/// through them, dashed, because a line returning through the boxes it came
/// from is a picture nobody can follow.
fn edges(screen: &mut Screen, chain: Chain, boxes: &[Band; ENGINE_COUNT], graph: Band, ink: Ink) {
    let columns = chain.columns();
    let mut forward: Vec<Wire> = Vec::new();
    let mut back: Vec<Wire> = Vec::new();
    for engine in Engine::ALL {
        let Some(into) = boxes.get(usize::from(engine.index())) else {
            continue;
        };
        for source in chain.feeds(engine) {
            let Source::Engine(from) = source else {
                continue;
            };
            let (Some(out), Some(at), Some(to)) = (
                boxes.get(usize::from(from.index())),
                columns.get(usize::from(from.index())),
                columns.get(usize::from(engine.index())),
            ) else {
                continue;
            };
            let wire = Wire {
                from: *out,
                into: *into,
            };
            if at < to {
                forward.push(wire);
            } else {
                back.push(wire);
            }
        }
    }
    lanes(screen, &forward, ink);
    returns(screen, &back, graph);
}

/// One edge of the graph, as the two boxes it joins.
#[derive(Debug, Clone, Copy)]
struct Wire {
    from: Band,
    into: Band,
}

impl Wire {
    /// Where it leaves, which is the right hand side of the box it comes from.
    const fn leaves(self) -> (i32, i32) {
        (
            self.from.x + self.from.width,
            self.from.y + self.from.height / 2,
        )
    }

    /// Where it arrives, which is the left hand side of the box it feeds.
    const fn enters(self) -> (i32, i32) {
        (self.into.x - 1, self.into.y + self.into.height / 2)
    }
}

/// Draws the edges that run forwards, each in a lane of its own.
///
/// Every forward edge on this instrument joins one column to the next, so each
/// one turns in the gutter between them. The turn used to be at the midpoint of
/// the two boxes, which put every edge crossing a gutter on the same column:
/// three engines feeding a fourth drew three lines down one column and two
/// engines fed by one drew two, and what a reader saw was a single bar with
/// stubs rather than three wires and a junction.
///
/// So the edges crossing a gutter are counted first and shared out across it,
/// in the order they leave. Two wires never stand on the same column, which
/// means no two of them can overlap: a merge is several lines arriving at one
/// box and a fan-out is several leaving one, and both are countable.
fn lanes(screen: &mut Screen, wires: &[Wire], ink: Ink) {
    let mut gutters: Vec<(i32, i32, Vec<Wire>)> = Vec::new();
    for wire in wires {
        let (from, _) = wire.leaves();
        let (to, _) = wire.enters();
        match gutters
            .iter_mut()
            .find(|(start, end, _)| *start == from && *end == to)
        {
            Some((_, _, sharing)) => sharing.push(*wire),
            None => gutters.push((from, to, vec![*wire])),
        }
    }
    for (from, to, mut sharing) in gutters {
        // In the order they leave, so wires crossing one gutter keep the order
        // of the boxes they come from and cross each other as little as the
        // graph allows.
        sharing.sort_by_key(|wire| wire.leaves().1);
        let count = sharing.len();
        for (index, wire) in sharing.into_iter().enumerate() {
            let (start_x, start_y) = wire.leaves();
            let (end_x, end_y) = wire.enters();
            let turn = turn(from, to, index, count);
            screen.across(start_x, start_y, turn - start_x + 1, ink);
            screen.down(turn, start_y.min(end_y), (start_y - end_y).abs() + 1, ink);
            screen.across(turn, end_y, end_x - turn, ink);
            arrow(screen, end_x, end_y);
        }
    }
}

/// Which column the `index`th of `count` wires crossing a gutter turns in.
///
/// Spread across the gutter rather than bunched at one end of it, so a single
/// wire still turns in the middle of the gap the way it always did, and so that
/// no two wires crossing the same gutter ever stand on the same column.
fn turn(from: i32, to: i32, index: usize, count: usize) -> i32 {
    let room = to - from;
    from + room * (i32(index) + 1) / (i32(count) + 1)
}

/// Draws the edges that run backwards, which are the loops.
///
/// Under the whole graph and dashed, because a loop is the one edge that does
/// not read left to right and drawing it among the others would be drawing the
/// picture's one exception as though it were the rule. A row each, for the
/// reason the forward edges get a lane each.
fn returns(screen: &mut Screen, wires: &[Wire], graph: Band) {
    for (index, wire) in wires.iter().enumerate() {
        let below = graph.y + graph.height + UNDER / 2 + i32(index) * APART;
        let leave = wire.from.x + wire.from.width / 2;
        let enter = wire.into.x + wire.into.width / 2;
        screen.down(
            leave,
            wire.from.y + wire.from.height,
            below - wire.from.y - wire.from.height + 1,
            Ink::Dashed,
        );
        screen.across(
            enter.min(leave),
            below,
            (leave - enter).abs() + 1,
            Ink::Dashed,
        );
        screen.down(
            enter,
            wire.into.y + wire.into.height,
            below - wire.into.y - wire.into.height,
            Ink::Dashed,
        );
        up(screen, enter, wire.into.y + wire.into.height);
    }
}

/// Draws the path the voices take around the block, where they take one.
///
/// `Insert` is the block in the signal path and nothing beside it; `Send` runs
/// the voices to the output stage as well; `Bypass` is the analog path alone,
/// with the engines drawn as something the signal is not going through. Read
/// off [`Mode::analog_path`] rather than the mode's name, for the reason the
/// library publishes it: the word is a label and the two paths are the fact.
fn analog(screen: &mut Screen, chain: Chain, glass: Band) {
    let Some(mode) = chain.mode else {
        return;
    };
    if !mode.analog_path() {
        return;
    }
    let y = glass.y + glass.height - 1;
    let label = "analog";
    let pen = screen.write(0, y - Screen::height_of(Size::Small), label, Size::Small);
    screen.across(pen + 3, y, glass.width - pen - 3, Ink::Dashed);
    screen.down(pen + 3, y - 3, 4, Ink::Dashed);
    screen.down(glass.width - 1, y - 3, 4, Ink::Dashed);
}

/// Draws the head of an arrow pointing right, at the dot it lands on.
fn arrow(screen: &mut Screen, x: i32, y: i32) {
    screen.dot(x, y);
    screen.dot(x - 1, y - 1);
    screen.dot(x - 1, y + 1);
    screen.dot(x - 2, y - 2);
    screen.dot(x - 2, y + 2);
}

/// The same, pointing up.
fn up(screen: &mut Screen, x: i32, y: i32) {
    screen.dot(x, y);
    screen.dot(x - 1, y + 1);
    screen.dot(x + 1, y + 1);
    screen.dot(x - 2, y + 2);
    screen.dot(x + 2, y + 2);
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{Chain, draw, plates};
    use crate::lcd::{Band, Screen};
    use deepmind_midi::effect::{Engine, Mode, Routing, Source};

    /// The chain a topology makes, with nothing read about the engines.
    fn wired(value: u8) -> Chain {
        Chain {
            routing: Routing::for_value(value).expect("ten topologies"),
            mode: Mode::for_value(0),
            running: [None; 4],
        }
    }

    #[test]
    fn no_two_wires_crossing_a_gutter_stand_on_the_same_column() {
        // What the picture was getting wrong. Every forward edge on this
        // instrument joins one column to the next, so they all turn in the same
        // gap; turning at the midpoint put three engines feeding a fourth on one
        // column, and three wires drawn down one column are one wire as far as
        // anybody reading it is concerned.
        for count in 1..=super::ENGINE_COUNT {
            let turns: Vec<i32> = (0..count)
                .map(|at| super::turn(30, 40, at, count))
                .collect();
            let mut apart = turns.clone();
            apart.sort_unstable();
            apart.dedup();

            assert_eq!(
                apart.len(),
                count,
                "{count} wires share a column: {turns:?}"
            );
            for turn in turns {
                assert!(turn > 30 && turn < 40, "{turn} turns outside the gutter");
            }
        }
    }

    #[test]
    fn one_wire_still_turns_in_the_middle_of_the_gap() {
        assert_eq!(super::turn(30, 40, 0, 1), 35);
    }

    #[test]
    fn the_input_stands_at_the_head_of_every_topology() {
        // The one thing every one of the ten has in common, and the thing a
        // column worked out by settling could get wrong: something has to be at
        // nothing, or the picture has an input wired to a chain that starts
        // somewhere in the middle of itself.
        for routing in Routing::all() {
            let chain = wired(routing.value());
            let columns = chain.columns();

            assert!(
                columns.contains(&0),
                "{} puts nothing at the head of the chain",
                routing.label()
            );
            for engine in Engine::ALL {
                if chain.feeds(engine).contains(&Source::Input) {
                    let at = columns
                        .get(usize::from(engine.index()))
                        .copied()
                        .expect("four engines");
                    assert_eq!(
                        at, 0,
                        "{routing} draws {engine} past the input that feeds it"
                    );
                }
            }
        }
    }

    #[test]
    fn a_loop_is_the_one_edge_that_runs_backwards() {
        // What the drawing tells the two apart by. The library flags the two
        // topologies that have a loop; this asserts the columns agree with the
        // flag, because an edge running backwards is what makes one draw as a
        // loop rather than as a wire through the boxes it came from.
        for routing in Routing::all() {
            let chain = wired(routing.value());
            let columns = chain.columns();
            let backwards = Engine::ALL.into_iter().any(|engine| {
                chain.feeds(engine).iter().any(|source| match source {
                    Source::Engine(from) => {
                        columns.get(usize::from(from.index()))
                            >= columns.get(usize::from(engine.index()))
                    }
                    _ => false,
                })
            });

            assert_eq!(
                backwards,
                routing.is_feedback(),
                "{routing} is drawn with {} backwards edge",
                if backwards { "a" } else { "no" }
            );
        }
    }

    #[test]
    fn every_engine_has_a_box_of_its_own_on_the_glass() {
        // Four plates, none of them standing on another: two engines sharing a
        // box is a picture of a three-engine synthesizer.
        let graph = Band::new(0, 0, 400, 40);
        for routing in Routing::all() {
            let boxes = plates(wired(routing.value()), graph);

            for (at, band) in boxes.iter().enumerate() {
                assert!(band.width > 0 && band.height > 0, "{routing} draws no box");
                for (other, beside) in boxes.iter().enumerate() {
                    if at == other {
                        continue;
                    }
                    let apart = band.x + band.width <= beside.x
                        || beside.x + beside.width <= band.x
                        || band.y + band.height <= beside.y
                        || beside.y + beside.height <= band.y;
                    assert!(apart, "{routing} draws two engines in one place");
                }
            }
        }
    }

    #[test]
    fn every_topology_draws_something_on_a_screen_a_window_has_room_for() {
        // A picture that came out blank would be a display that said the
        // effects were doing nothing, on a panel where doing nothing is one of
        // the ten things it can say.
        for routing in Routing::all() {
            for columns in [160, 260, 420] {
                let mut screen = Screen::new(columns, wired(routing.value()).rows());
                draw(&mut screen, wired(routing.value()));

                assert!(
                    !screen.is_blank(),
                    "{routing} draws nothing on {columns} dots of glass"
                );
            }
        }
    }
}

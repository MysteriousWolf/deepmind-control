# Plan

What this repository builds, in what order, and what has already been decided.
Stages 0 to 5 are written: there is a window that edits every parameter, lays
every panel out by hand, and a librarian that reads and writes the files those
parameters live in. What is left before the plugin is polish. The
[order](#order) says which is which.

[`deepmind-midi`](https://github.com/MysteriousWolf/deepmind-midi) owns the
protocol and refuses to own anything else: it never opens a port, never spawns a
thread and never blocks. What it leaves is exactly the work here. Owning the
port. Driving the clock. Putting the result on a screen. Doing the last part
twice, once in a window of its own and once inside somebody else's.

## Goal

An editor a DeepMind owner would reach for instead of the front panel, and a
librarian that does not lose their patches. Every parameter the instrument
exposes, laid out the way the instrument lays it out. A plugin so that a patch
travels with a project.

## The split

| | |
| --- | --- |
| `deepmind-midi` | frames, the 242 parameters, programs, `.syx`, the state machine |
| this repository | the port, the clock, the threads, the files, the window, the pixels |

The line is not negotiable in either direction. Nothing here re-decodes a frame
or re-tabulates a parameter, and nothing there learns what a MIDI backend is.
Anything discovered here that is a fact about the protocol is a change to the
library, its `spec/`, or both, and comes back as a released version.

## Crates

```
crates/
  deepmind-host/    port ownership and the device loop. No interface.
  control-ui/       every view and widget. No runtime, no window.
  control/          the desktop application.
  control-plugin/   the plugin.
```

`deepmind-host` is the only one that would be useful to somebody else, so it is
the only one carrying the prefix. It implements `Port` over
[midir](https://github.com/Boddlnagg/midir) and `Clock` over the system clock,
runs the state machine on a thread of its own, and publishes what happened. It
has no opinion about interfaces and no dependency on one.

`control-ui` is the views. It depends on `iced_core` and `iced_widget` rather
than on `iced`, so it carries no runtime, opens no window, and compiles into
both builds.

`control` is the desktop application: the window, the file dialogs, the
librarian.

`control-plugin` is the plugin: the same views in a baseview window, built as a
CLAP and wrapped into an AU and a VST3.

## The device loop lives on its own thread

A bank is one request and 128 answers. Thirty-five kilobytes at MIDI speed is
about twelve seconds, and the interface has to stay alive for all of it, so the
loop that reads, feeds, ticks and sends runs on a thread that owns the `Device`
and the port and nothing else.

That thread uses `Transport::pump` and none of the blocking helpers.
`edit_buffer()` and `bank()` are the right shape for a command-line tool and the
wrong shape for a window: they wait, and a waiting thread cannot report progress
or be cancelled. Requests go out, answers come back as events, and the twelve
seconds are a progress bar rather than a freeze.

The blocking helpers stay useful in tests, where waiting is the point.

## Events cross the boundary, state does not

The device thread holds the truth. The interface holds a copy it builds out of
events, and there is no lock between them.

```rust
let ports = host::ports()?;                      // what midir sees, and the simulator
let link = host::open(&ports[0])?;               // one thread, two channels
link.send(Command::ReadEditBuffer)?;
for event in link.drain() { /* iced update */ }
```

Sharing an `Arc<Mutex<Device>>` instead would be shorter to write and would put
a lock in a view, which is a dropped frame waiting for a bank transfer to let
go. Two copies of the state, one direction of flow, and the copy the user sees
is the one their own events built.

`Command` is this repository's vocabulary, not the library's: read the edit
buffer, read a slot, read a bank, set a parameter, load a program, cancel. The
`Device` never leaves the thread.

The window drains that queue once a frame, from a thread and a `thread::sleep`,
and holds no async runtime to do it. What the drain finds was queued by a thread
that is still answering the port while the window draws, so the only thing the
frame rate bounds is how late an answer can appear on the screen. A window with
no port open subscribes to nothing at all.

## Assumed and confirmed are different, and are drawn differently

`Known<T>` is the library's most useful idea and the easiest one to throw away
at the boundary. A value the host sent and a value the synthesizer reported are
not the same claim, and the interface is where that distinction finally means
something to a person: a knob that shows where you put it, and a knob that shows
where the instrument says it is.

So an assumed value is drawn differently from a confirmed one. Reading the edit
buffer back is what turns one into the other, and the application asks for it
after a load, on reconnect, and when the user asks. Never on a timer: a poll that
costs 290 bytes of a 3 kB/s link is a poll that competes with the edits it is
checking.

The interface keeps the claim per parameter and the library keeps it per program,
which is the one place the two copies of the state are shaped differently and it
is deliberate. `Known<Program>` is a claim about the whole sound, so one dragged
knob makes all 242 values assumed; on a screen that greys out an entire panel
because one control moved, and throws away the fact that the other 241 are still
exactly as the synthesizer described them. `Patch` holds a `Confidence` per
parameter, downgrades the one that moved, and keeps the library's program-wide
claim beside it for the header. A dump rewrites both.

What the interface cannot work out for itself is whether a report that arrived
carried the whole value. An NRPN does and a control change carries seven bits of
it, and the library says which by what it does to the program it tracks rather
than on the event, so the host crate reads the tracked claim the moment the
report lands and publishes it alongside: `Event::Parameter` carries `confirmed`,
and that is the library's answer and not this repository's arithmetic. It is
honestly pessimistic in one case, which is the case where nobody could do better:
a program that was already assumed hides the difference, because a claim nothing
has confirmed cannot be made more confirmed by a message that might be coarse.

## Edits are coalesced, not queued

One NRPN is four control changes, twelve bytes, about four milliseconds. A knob
drag at 60 fps is 60 of those a second for as long as the mouse is down, and
every one of them but the last is obsolete when it is sent.

The host crate keeps one pending value per parameter and sends the newest at a
fixed rate. Dropping intermediate values of a drag is free; dropping the last
one is a desynchronised instrument, so the last one is always sent.

Coalescing lives in the host crate rather than in the views, because the plugin
and the desktop build have the same problem and the views should not know that a
wire has a speed.

It costs one ordering rule. A read asked for while edits are still going out
waits for them, because a dump that arrived first would confirm the values those
edits are replacing, and the interface would draw the old sound as the
instrument's own answer. "Put it there, then tell me where it is" has one honest
order. Edits themselves never queue behind each other: that is what coalescing
is, and a drag that queued would grow without end.

For the same reason, loading a patch sends the difference and not the program.
`Program::changes` is the whole implementation. Sending all 242 parameters is
2904 bytes, a little under a second of solid MIDI, and a patch change that takes
a second is a patch change the player notices.

## The instrument is the other editor

Turning a knob on the front panel raises `Event::Parameter`, and the view
follows it. Last writer wins, and the application never echoes a value back that
it just received, because the instrument already has it and the echo is a loop.

An NRPN carries the whole value and a control change carries seven bits of it.
The library says which arrived; the interface shows a coarse value as the
coarse value it is rather than quietly rounding, and a confirmed program stays
confirmed only where the library says it does.

## Nothing here writes a program into the synthesizer

The manual describes no message that stores a program into a unit. The library
refuses to invent one, and so does this.

The consequence is a shape, not a limitation to apologise for: the application
edits the edit buffer live, and the librarian's output is a file. Storing a
sound into a slot is done by the player, at the panel, with the instrument's own
WRITE. The application says so where a "save to synth" button would otherwise
go.

Whether the unit accepts its own dump back is the first question of the first
hardware session. If it does, the librarian grows a write path and the plan
gains a stage. If it does not, nothing that was built has to be unbuilt.

## A patch is a `.syx` file

The only file format is the protocol's. A patch is one program dump; a pack is
128 of them end to end; the library reads and writes both, and anything else
that speaks DeepMind SysEx can read what this writes.

No project format, no JSON, no database. A patch that leaves this application
loses nothing on the way out, which is the property a librarian is for.

Reading is lenient and writing is not byte-identical to every file in the wild,
both of which the library documents and neither of which the user needs to hear
about.

## The librarian is a second surface, not a second window

An editor and a librarian are the same application looking at two different
things: the sound somebody is playing, and the sounds they keep. Two windows
would make that a filing decision. One window with two surfaces and a switch
between them makes it what it is, and the sound survives the move, because
putting a pack down to look at a filter and finding the filter gone is the wrong
thing to teach anybody about an editor.

The shelf is what the librarian holds: the programs a `.syx` file turned out to
contain, or the ones a bank read produced, each with the slot its dump named. An
edit buffer dump names none, because the edit buffer is where a sound is played
rather than where one is kept, so a program that names no slot is drawn as one
that names no slot. It is also how a patch this application wrote reads back —
one sound, belonging nowhere in particular, which is exactly what it was when it
was saved.

Reading is lenient, and says so. A file is untrusted input: the library skips
the bytes between frames and hands back a frame that will not parse as the error
it failed with, so one bad dump in a pack is one missing program rather than a
refused file. What could not be read is counted and printed, because a librarian
that quietly holds 127 of 128 programs is worse than one that will not open the
file at all.

A program taken off the shelf is assumed and never confirmed. The host sends it
as the difference against what the synthesizer is believed to hold, and the
window marks all 242 values as its own claim, because nothing has heard the
instrument play them yet. Reading the edit buffer back is what settles it, which
is the same rule a dragged fader follows.

None of this is in `control-ui`. Editing is the same in both builds because it
is the same crate; bank and librarian operations are desktop only, for the
reasons the plugin section gives, so they are written in the crate that is only
ever a desktop application.

## The simulator is a port you can choose

The library's `sim` feature is the other end of the conversation. It compiles
into the desktop build, and it appears in the port list as an ordinary choice,
alongside the real ones.

That is worth the feature flag three times over. The interface can be built
without a synthesizer on the desk, the timeout and cancel paths can be exercised
by not draining it, and a `.syx` pack can stand in for the unit's memory, so
"read a bank" is testable against a preset pack. It also means the first bug
report that arrives without hardware still has something to reproduce against.

Opening one hands back its front panel as well as its port, because the
instrument is the other editor and a simulated unit nobody can stand in front of
cannot demonstrate that. Turning a knob on it sends the NRPN a hand would, which
is how "the view follows the instrument" is tested with no cable in the room.

What it cannot do is find out that the manual is wrong. Both ends are generated
from the same specification.

## Raw values stay raw here too

The library publishes each parameter's range and its value tables, and refuses
to guess the curve between a raw byte and the number the instrument displays,
because the manual almost never prints it.

The interface inherits that refusal. A parameter with a value table shows the
name of the value. A parameter without one shows its raw value, and shows a unit
only where `spec/measurements.toml` has established the curve. Inventing a
plausible "2.4 kHz" for a byte is worse than showing the byte: it is wrong in a
way the user cannot see.

Measured curves arrive in the library, parameter by parameter, and the interface
picks them up when it upgrades.

## Firmware is asked for before anything else

A device inquiry is the only thing that reports firmware, and firmware 1.1
renumbered three value tables, so 17 of 23 modulation sources mean something
else on 1.0. Every table lookup the interface makes has a `_for` form that takes
the reported version.

So the connection sequence is: open the port, broadcast a device inquiry, and
wait. A port that answers has a DeepMind on it. The reply carries the unit's
device ID, which the manual says is also its global MIDI channel, so one message
settles who to address, which channel to send NRPN on, and which value tables
are true. The `Device` is rebuilt with that identity, which costs nothing and is
more honest than mutating it.

A port that does not answer within the timeout is not a DeepMind, and that is
how the port list is filtered rather than by matching names. The port stays open
either way: a synthesizer that was switched on late is a likelier explanation
than a host that should give up, and asking again costs one message.

Rebuilding happens when the address actually changes, because a rebuilt device
is a device with nothing outstanding. A second inquiry into a unit that has
already answered is then free, which is what makes it the way to look for one
that was not there a moment ago.

Until an identity arrives, lookups use the library's default firmware and the
interface says which one it is assuming.

## The effect panels are the library's data

Four FX engines, twelve raw parameter bytes each, and what those bytes mean
depends on which of 35 algorithms is loaded. The library names them `FX 1 Param
3`, which is the truth and is unusable in an editor.

They are not transcribed here. Duplicating 35 panels into this repository means
maintaining a second copy of a table that is generated from a specification,
which is the thing the library exists to prevent.

`deepmind-midi` 26.2 published them, which is what the FX editor was waiting
for: `effect::Engine` is the four engines and which parameter each of their
slots is, `effect::Algorithm` is the 35 and what value selects one on a given
firmware, and `effect::FxSlot` is what a slot of the loaded algorithm is called.
So the panel is all four engines at once, in the two-by-two the four of them
make. Each carries its own strip: which algorithm it is running, chosen from the
parameter's own value table under the display's own abbreviations, what that is
called in full written out beside the list rather than substituted into it, and
how loud its output. Nothing above them and nothing below — a row of tabs and a
header both said which algorithm an engine was running, and neither of them was
where the engine was. The settings that are no engine's, the connection mode and
whether the effects are inserted, sent or bypassed, sit in the band under the
chain they shape, found by subtraction rather than by name, which is also where
a parameter a later library adds to this group will appear. Under rather than
beside is what the wider glass cost and what bought the ten topologies a layout:
they are two columns of lit legends now instead of a drop-down, and a drop-down
of ten shows whichever one is already chosen and hides the nine somebody is
choosing between.

**A named slot is the same control it was.** The library says two things about a
slot and they are not the same thing: `FxSlot::kind` is how the display reads
the byte, and the parameter table is what the byte is. `Freeze` is two states on
the panel and a parameter that accepts 256 values on the wire, and the curve
between them is not published. So the control is chosen by the same code from
the same table as every other slot in the editor, and what the algorithm
supplies is the naming: the title, the abbreviation the instrument prints, the
band the slot belongs to, and the two ends of the reading it shows. It is the
rule the rest of the hand layout follows, at the one panel where breaking it
would mean inventing a mapping the manual does not give.

That rule decides the two things this panel cannot do. A slot whose display
shows names — `Ambience`, `Church`, `Gate` — is not a list, because the manual
prints those names and never the bytes they sit at, and a list that sent one of
them would be sending a guess; the names are printed under the row that slot
stands on, as what the display will show, and the byte stays draggable. And a
slot's reading stays the
byte, with the two ends the manual prints written under the title, because
`0.1` to `6.0 s` is a range and not a curve.

**A case is as deep as what is in it.** The grid is two rows and most of the 35
fill one, so a plate drawn at the grid's own depth was a rack unit with more
blank panel under its knobs than knobs — and the band of display names at the
foot of it was sized for the longest of the 35 whatever was loaded. Both
reservations bought four cases of one height and were paid for by every
algorithm that is not the deepest one. So a plate draws the rows its algorithm
fills and a display name stands under the row its own slot is on. What is still
reserved is the shape of a row, which is what the columns line up against: a
column stands a column's height whether or not a slot is in it, and a control
stands in a band of one depth whether the figure calls for a knob or a fader.

**Twelve bytes, however many the algorithm uses.** An engine holds twelve
whatever it is running and `Algorithm::slots` is as short as five. The ones the
algorithm names stand on the grid; the rest go in a strip cut into the plate
below it, at half the size, under the library's own `Fx 1 Param 6`. Below rather
than among, because seven controls the size of the five that do something is a
plate whose loudest half is the half that does nothing; and with nothing written
over them, because below the grid, cut into the case, half size and named by the
library rather than by the algorithm are four ways of saying the same thing. They are still in the program, still reachable from
the modulation matrix, and still sent, because a panel that quietly stopped
sending them would lose part of a sound the moment somebody changed an effect.
An engine whose algorithm this firmware's table does not name, or whose type
nobody has read, is all twelve in that strip and no grid at all, which is stage
3's rack for exactly as long as there is nothing better to say.

**The matrix mark grows a second state here.** Every slot is addressable from
the modulation matrix as `Fx n Param m`, and the library says which ones the
engine actually acts on. A routing pointed somewhere the engine ignores gets the
mark as an outline rather than filled: the matrix really is pointed there and
really is doing nothing, and drawing that the same way as an effective routing
would hide the reason a sound is not moving.

**The grid is the instrument's own, and so is the shape.** 26.3 published what
was held back ([deepmind-midi#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)):
the grid the synthesizer's own FX page lays a slot out on, measured off the 35
screenshots in the manual, and what the figure printed beside each algorithm is
made of. Six columns and two rows, with the column and row published per slot.
So a plate draws six columns, each of them there whether or not a slot stands in
it and all of them spread across the page, and puts every slot in the cell
measured for it — which is the arrangement anybody who has edited an effect on
the hardware already knows, and the arrangement a second row can be read down
against. Reading the publication as the order of a row instead, and packing each
row against the left at the width of a rack's slot, is what left two rows that
did not line up in the left half of an empty plate.

What the grid also publishes is a proportion: a control 11.9 wide in a column
pitched 20, on the 128 point display those screenshots were measured on. That is
not taken. It measures a dot matrix drawing its own labels in dots, and a page
this wide would make it a knob a hundred points across with its title set in a
face a tenth of its size. The arrangement is exact and is taken whole; the size
is the window's, and twelve controls on a page have room the rack's forty do
not.

A slot on one of the 29 algorithms whose figure draws rotary knobs is drawn as a
knob. A knob is the fader turned — the same byte, the same relative grab, the
same claim in the fill of the thing that moves — which is why the shape belongs
beside the axis a fader runs along rather than anywhere near what a parameter
is.

**The measured colours are an identity, and how much of it fits depends on how
many units are on the page.** The library publishes four per algorithm, and they
are the colours of 35 imaginary rack units: a cream fader panel, a black one, a
blue-grey one. Painted as measured, four of them side by side would be a collage
of other people's instruments in a window whose whole argument is that it is one
instrument.

That objection is about how many, not about the colours, which is why the page
has spent a different number of them at each of its three shapes. With one
engine open at a time it wore the lot — case, the face its controls stand on, a
hairline of the accent, and the cap a finger moves. Four stand on it now, so the
face is back to the window's own plate and the cap back to the window's own
metal, and an engine wears its case and its accent: enough to tell the reverb
from the distortion at arm's length.

The cap is the one measurement this repository has spent and handed back, and
the ledger is worth keeping. It was refused first on the grounds that what
carries a claim is the fill of the thing that moves, so a control repainted to
match a figure would break the rule everywhere. The argument was wrong and the
refusal was right: the claim is *which* fill and not which colour — a reported
value is the cap filled and an assumed one is the cap as a stroke, whatever the
cap is made of — but there is only room for a repaint on a page with one unit on
it, and this is not one.

**Nothing is printed in a colour that cannot be read on what is under it.** Half
the 35 chassis are pale, so no word on this page names an ink. It names the
surface, and `style.rs` answers with whichever of the instrument's two that
surface is further from, lifted until it clears a written-down threshold. Two
things fell out of measuring rather than eyeballing, and neither was visible in
a screenshot: a case is carried as far into the panel as it can go and still be
printed on, because the cream ones half way in land in the exact middle of the
two inks where the best either can manage is 4.45; and a value is never printed
on a case at all, because on the palest of them an amber claim and a green one
both have to be lifted so far that they arrive as the same ink, which would
spend the one distinction this editor exists to draw on a livery. The output
gain an engine's strip carries sits in a recess cut into the case instead, which
is where a control belongs anyway.

**The chain is drawn, because the graph is published.** What the connection mode
does to each engine's output was in the library's specification and not in what
it published, so the routing was the list its value table names and nothing was
drawn from it. 26.3 publishes the ten topologies as edge lists with the two
loops declared ([deepmind-midi#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23)),
so the effects now open on a display of the chain itself: what the block's input
reaches, what feeds what, what is summed at the end, the loop dashed under the
engines it returns through, and the analog path along the foot where `FX Mode`
puts one. Nothing in it knows a topology by name — a picture assembled by
matching `Serial 1-2-3-4` against a string would be an eleventh copy of the
table — and where an engine stands is worked out from the edges rather than
written down, so a column is how far it is from the input and a backwards edge
is the loop.

The glass it is drawn on is cut to the longest abbreviation the library
publishes: four boxes in a line, each wide enough to name what is running in it,
with the gutters between them and the rails at either end. That is nearly three
times the instrument's own display, and what it buys is that the name of an
effect is in the box for that effect rather than in a list under the picture.
The number stacks over the name where a box has the height and not the width,
which is what four engines in a line have. Nothing about the width is chosen — a
firmware that adds a longer abbreviation widens the glass on the same day.

## One view layer, two runtimes

`control-ui` depends on `iced_core` and `iced_widget`. The desktop crate brings
`iced`; the plugin crate brings the baseview adapter. Neither the widgets nor
the views know which one they are in.

The iced version is pinned, and it is pinned to whatever the baseview adapter
supports rather than to the newest release. This is the load-bearing constraint
of the whole repository and the most likely thing to break it: the two builds
share a view layer only for as long as they share an iced. If the adapter falls
far enough behind that the pin is intolerable, the split is to give the plugin
its own thin view layer over the same `control-ui` model, not to fork the model.

## The plugin carries the program, not a pointer to it

Plugin state is the 242 bytes, plus the slot they came from if they came from
one. Saving a project saves the sound.

A plugin that stored "bank C, program 41" would restore a different sound on a
different instrument, or on the same instrument after a pack was loaded, and it
would do it silently, months later, to somebody's finished record.

## AU and VST3 are what ships, and CLAP is what is written

**The first thing that ships is not a plugin.** The desktop application is the
whole editor and the whole librarian, it owns its own port, and it can be handed
to somebody the day it works: no SDK agreement, no developer account, no host
that has to agree to load it. So the standalone desktop build is the first
release, and everything in this section is about what comes after it.

A DeepMind owner with a DAW open is in Logic or Ableton or Cubase, and what
those load on the machine in front of them is an Audio Unit or a VST3. CLAP is
the better format and almost nothing they already own reads it. So AU and VST3
are the formats this project supports, and CLAP is not a third build: it is the
one the other two are made from.

[clap-wrapper](https://github.com/free-audio/clap-wrapper) takes a CLAP and
produces an AUv2 and a VST3 around it. One plugin is written, once, against one
API, and three bundles come out of it. Writing three plugins against three APIs
to ship the same synthesizer editor is how a project ends up with three sets of
bugs.

**The licence is the part to settle before the first binary leaves the
machine.** nih-plug's own VST3 exporter carries GPL-3.0 terms from its
`vst3-sys` bindings, and an Apache-2.0 repository cannot ship that binary
without relicensing what it ships, so that exporter is not used. The wrapper
takes a different route: it links Steinberg's VST3 SDK, which is dual-licensed
under GPL-3.0 or Steinberg's own agreement, with no third-party GPL binding in
the path. Shipping an Apache-2.0 VST3 therefore means accepting Steinberg's
terms, which is paperwork and a decision, not a checkbox in a build script. It
is answered before a release, not after one.

AU is macOS and AUv2: what Logic and GarageBand load. `auval` has to pass before
Logic will look at it, so passing `auval` is what "AU is supported" means here,
and the bundle is signed and notarised like any other macOS binary. AUv3, iOS
and the App Store are not planned.

The build grows a step that is not Cargo. `cargo nih-plug bundle` builds the
CLAP; the wrapper is CMake and a C++ toolchain around that. Windows and Linux
get VST3 and CLAP, macOS gets all three, and the CLAP is shipped rather than
thrown away because it costs nothing to keep and is the best of the three where
a host reads it.

**What matters most and what arrives first are different questions.** AU and
VST3 are what a DeepMind owner needs; the CLAP is what the wrapper eats, so it
exists before either of them whether or not anybody loads it. It is also the one
with nothing to settle: no SDK agreement, no developer account, no notarisation.
So it is released as soon as it works, and the other two follow as their
paperwork clears, AU once the bundle signs and `auval` passes and VST3 once
Steinberg's terms are accepted. Releasing in that order costs nothing and keeps
a licence question from standing between a working plugin and the people who
want one.

What none of this changes: the plugin is one crate, editing is the same
`control-ui` in every build, and the window is the same baseview window. Whether
that window survives the wrapper on AU is the first thing checked in the stage
that builds it, because everything after it depends on the answer.

## The plugin does the editing and not the library work

Two modes. *Simple* selects programs and nothing else, which is what most
projects want from an instrument: the right sound recalled with the session.
*Advanced* is the desktop editing surface in a plugin window.

Editing is identical in both builds, because it is the same crate.

Bank and librarian operations are desktop only. A preset pack is thirty-five
kilobytes of SysEx, a plugin event buffer is sized for a few notes, and a
twelve-second transfer through a DAW's MIDI output is twelve seconds of a host
wondering why its instrument stopped responding. The plugin does not own a port
either: in advanced mode its MIDI output is the port, and the DAW decides where
that goes.

## The mark is the library's, one bench over

`docs/logo.svg` and `docs/banner.svg` are drawn on the same construction as
`deepmind-midi`'s: the same case, the same wooden end cheeks, the same panel and
seams, the same metal, and the same horizontal lines of the DeepMind wordmark
slicing through it. Only the object on the panel changes. The library's mark is
the socket the bytes arrive through; this one is the surface a player puts their
hands on. Three faders, and the cap heights are chosen so that each cap is
crossed by one of those lines through its middle rather than at its edge, with
the highest cap standing clear of them.

The name is outlined from Liberation Sans Bold rather than set in a font
reference, so that it renders identically wherever the file is shown. The recipe
is in the file: tracked -20 units of the 2048 em, fitted so the ink runs from x
244.5 to x 884.5 on a baseline of 130, which is the box the library's own name
occupies. A longer name in the same box is a smaller name.

The same mark becomes the desktop application's window icon and the plugin's,
generated from `logo.svg` in the stage that first needs one. Nothing here is
traced from Behringer's artwork; it is the same drawing the library already
publishes, with a different thing on the panel.

## The window is the instrument's colours, and its face

Dark, because a `DeepMind` is a dark panel between wooden end cheeks with metal
fader caps on it, and an editor for it that opens white is an editor for
something else. The palette is read off `docs/logo.svg`, which draws exactly
those materials: the panel is the ground, the metal of a cap is anything that can
be touched, and the copper edge of the wood is a claim rather than a fact. Green
and red are not on the instrument and are chosen to sit with it, because a
confirmed value has to be told apart from an assumed one at a glance and two
greys would not do it.

One file holds it. `control-ui/src/style.rs` is the only place in this repository
that writes down a colour: the theme is built there, both builds ask for it
there, and every widget that colours anything asks `tint` what the claim is
worth. A control that hard-coded a colour would be a control that stops matching
the instrument the day the palette moves, and there is nowhere in this
repository to hard-code one.

The same file holds the face and the chrome, for the same reason. The mark sets
the project's name in Liberation Sans Bold and outlines it so that it renders
identically wherever the file is shown; a window cannot outline anything, so it
asks for that family and for the metrically compatible face other platforms
ship under another name, in one place, with a fallback to the machine's own
sans. Carrying the file in the binary is what would make both builds identical
everywhere, and that is a decision about a licence and a megabyte of plugin
bundle rather than a line of code.

And the parts of a window that are not parameters are drawn on the instrument
too: the ground is the panel gradient the mark fills its case with, a button is
the panel with a metal rim rather than a toolkit's filled slab, and a port is
chosen in the same recess a fader's track is cut as. The panel is for
parameters and the toolkit is right for everything else; everything else is
still on the front of the thing being edited.

## Versioning is the year and the release

`Cargo.toml` holds the version and nothing else does, and the scheme is
`deepmind-midi`'s: `YY.RELEASE.PATCH`. `26.1.0` is the first release of 2026,
`26.1.1` its first patch, `26.2.0` the second release of the year. Cargo reads
that as semver, so the year is the major version and a release is a minor one.
A release within a year must not break the public API of the one crate here that
somebody else could depend on, and a breaking change is a new year.

That crate is `deepmind-host`. `control`, `control-ui` and `control-plugin` are
`publish = false`: two of them are binaries and the third is this repository's
own furniture. They carry the same number anyway, because one repository
releasing several numbers is a support question nobody needs.

A person edits that one line, not the release job. CI fails when the number in
the tree is not ahead of the newest release tag, so the first change merged
after a release has to move it and the tree always states what it will release
next. `cargo-semver-checks` holds the compatibility promise against the same
tag. Both checks pass trivially until a release tag exists, which is the state
this repository is in.

The workflow that tags and publishes arrives with the first desktop release,
not before it. A release job for an application is not the library's: it builds
binaries for three platforms and signs two of them, and writing it against a
release that has not been designed yet is writing it twice.

## The window opens on the instrument

A `DeepMind` shows you its front before it shows you anything else: two rows of
section plates with a screen between them, twenty-odd faders under four-letter
legends, and on every group the press it calls `EDIT`, which puts that group on
the display. The 242 parameters are all behind one of those.

The window does the same, and for the instrument's own reason: somebody who has
just plugged a synthesizer in wants to see the sound rather than a list of
fourteen sections, and the section they want is one press away either way. So
the front panel is the surface a window opens on, the fourteen racks are what an
`EDIT` opens, and the librarian is the third surface beside them. The bar and
the panel's `EDIT` are the same press — one asks for a section and the window
shows it — so there is one idea of "open that section" rather than two.

**This is the one thing this repository transcribes.** Which parameters the
hardware puts a fader under, and what is silkscreened over them, is a fact about
the instrument that the library does not publish: the parameter table says what
exists, the controller table says what has a CC, and neither says what a hand
can reach. Everything else here is derived — the matrix finds its eight routings
by what the library calls them, the sequencer finds its steps, the effects read
the algorithm each engine is running — and this one is a table, in one file,
written as parameter identifiers so that a rename fails the build rather than
mislabelling a fader. The file's first paragraph says it is there until
[deepmind-midi#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)
lands, and `docs/waiting.md` is where that is remembered.

It is worth being exact about what the table does and does not decide. It says
*which* control is on the panel and what is printed over it. What that control
*is* is still the library's answer — a sweep gets a fader, two states get a
lamp, a named set gets its names — which is the same line hand layout has not
crossed since stage 3. A parameter this table names that the library later makes
enumerated arrives on the panel as a list, with nothing here to change.

**Every plate has a display, and the instrument has one.** This is the one place
the panel is deliberately not the instrument, and the reason is the one thing a
window has that a front panel does not: room. A `DeepMind` has space on its face
for a screen and twenty-odd faders, so its screen shows whichever section was
pressed last. A window has space for a screen over every plate, and the thing a
player wants to know about a filter is the shape of it rather than three numbers
that imply one. So each plate carries the drawing of its own part — the shapes
the oscillators are making, where the corner is, the gates the arpeggiator is
opening — and the envelopes carry the drawing no `DeepMind` can show: all three
at once, on one screen, which an instrument with one display and three envelope
buttons cannot do.

They are the instrument's own screen and not a dark rectangle with words in it.
A `DeepMind`'s display is a dot matrix, so these are dots: one quad per printed
dot at a pitch every display in the window shares, a 5×7 cell for anything
written, and a curve drawn as the dots nearest one. A display given more room
gets more dots and never bigger ones, which is what makes the strip over a plate
and the panel's own screen two windows into one instrument rather than one of
them magnified.

And it is a *positive* display: a pale green-white backlit panel with its dots
printed dark on it. That is the one bright rectangle in a photograph of the
instrument, and it is worth getting right, because a window that drew pale dots
on a dark pane would be drawing the negative of the thing it is a picture of —
every other synthesizer of the decade, and not this one.

The three envelopes are laid out rather than overlaid. Three curves sharing one
band and told apart by a dash pattern is a drawing with all three envelopes in
it that shows you none of them: on a grid of dots two lines crossing are the
same dots. A pane each, named after the button the hardware selects it with, is
what the room is for — and having the room is the whole reason this panel has
ten screens where the instrument has one.

What they draw is under the refusals everything else here is under, and the
refusals are what decides the drawings. No axis is in anybody's units: a filter's
corner is at the fraction of its own range the byte sits at and never at a
frequency, and a rate is how many cycles fit across a screen and never a speed,
because the manual prints the ends of a range and not the curve between them.
Nothing is drawn from a value nobody has read. And three things are left out
rather than guessed — the bass boost is printed as a word instead of drawn as a
shelf nobody has measured, the LFO's `Delay / Fade` is not drawn because one
byte doing two things has no published crossover, and pulse width modulation is
the depth's own travel marked either side of the edge rather than a duty cycle.
[interface.md](interface.md#display) has the list and the anatomy.

The claim is how hard the dots are printed, which is the second control to carry
it without a fill and for the first one's reason: a display has no moving part,
exactly as the name field has none. The pale ground answers it better than a
dark one could — a fact is printed hard, a claim in the copper mixed most of the
way to the same black, and what nobody has read barely at all. Three depths of
one ink is an ordering before it is a set of hues, so it survives a photograph
and the readers who would not see the copper.

Three things the hardware has that the panel does not. The row of twelve lamps
over `POLY` says how many voices are sounding, and nothing on a MIDI port says
that, so it is not drawn: a lamp that cannot be lit honestly is worse than no
lamp. The `DATA ENTRY` fader edits whatever the display is showing, and a window
has the value under the pointer instead. And the row of twelve `VOICES` lamps is the
only thing on the hardware's front this panel leaves out.

The buttons are the instrument's, and that is a decision reversed. The plan
argued that the hardware's white, yellow and cyan should be drawn in this
window's own materials, because a colour here already meant something — copper a
claim, green the instrument's own account — and a fourth meaning would be a
thing to learn rather than a thing to read. Moving the claim onto the glass as a
depth of ink took it off the panel's lamps entirely, and with it the objection:
the panel now lights amber where the hardware lights `EDIT` and cyan where it
lights `MOD`, which is two meanings taken from the instrument rather than three
invented here. A way in is a legend silkscreened on the panel over a lit square,
because that is what it is on the hardware and not a word in a box.

The panel also fills the window it is in, through one scale measured from its
widest row: the lanes, the faders, the buttons, the type and the gaps between
and inside the plates. Scaling the gaps is the half that decides whether it
reads as an instrument or as a panel with its parts pushed apart.

Filling it is every row and not only the widest one. The signal path measures a
plate less than the top row and the envelopes measure two plates less, and rows
drawn at what they measure leave that difference as bare panel at the right hand
end — plates bunched into a corner, which is not a front panel. So a row shares
its spare width out among its own plates in proportion to what each already
holds: no proportion changes, and what the extra room buys is display, because a
plate given more glass gains dots. Every plate stands the same height as well,
so a row has one edge along the bottom of it and the `EDIT` presses are on one
line, which is what they are on the instrument.

## A panel is generated before it is drawn

Every one of the 242 parameters is a slot in the rack of its group, and what the
slot holds is whatever the library says the parameter is: a sweep gets a fader,
two states get a lamp, a named set gets its names, and a table that does not name
every value the parameter accepts is not used at all. Nothing in this repository
decides what a value means.

That gets the whole instrument editable in one stage and leaves it ugly in the
places where the instrument is not a list of forty faders. Hand layout is per
group and changes only the arrangement, never what a control is:

| | |
| --- | --- |
| Program | **Done.** The name is 17 parameters, one character each, and 17 faders is not a name, so the 17 slots are the one display the instrument shows them on |
| VCF, VCA and Mod envelopes | **Done.** Four faders and the shape they make, drawn above the rack, which is the one group whose meaning is a picture |
| Mod Matrix | **Done.** Eight rows of source, destination and depth, read across rather than down, because twenty-four slots in one wrapping line are eight sentences with their words in the wrong order. A destination is chosen from a list that can be typed into, or by mapping the routing onto the window and taking hold of the control it should move — which sets the depth too, from how far the drag went |
| Control Sequencer | **Done.** 32 steps as one strip in the order they are played, with the six settings that are not steps left in the rack |
| Effects | **Done.** Four plates, each an engine's own settings and its twelve bytes under the names the loaded algorithm gives them, read for the firmware that answered |

Until a group is laid out, it is complete and honest and looks like the
specification it came from, which is the trade this order is making.

The modulation matrix is the first one whose arrangement is a table. Eight
routings of source, destination and depth are eight sentences, and a rack puts
the three words of one in three places with the next sentence between them: the
row is what makes it readable, and the parameters' own names move into the
column headings rather than being repeated eight times over. The eight are
found by what the library calls them — a `Source` with a `Destination` and a
`Depth` sharing its prefix — so a ninth routing draws a ninth row and a lone
`Source` somewhere else is not a matrix.

The rows can say where they go, since `deepmind-midi` 26.2:
`ValueTable::parameters_of` joins `VCF Freq` to `ParamId::VcfFrequency` where
the table lives, rather than by matching those names here, which is the second
copy of a generated table this repository refuses to keep. A slot the eight
routings are pointed at carries the one saturated mark on the panel, read once
per panel from the destinations the patch holds. A destination nobody has read
moves nothing: a mark drawn from a value this window has not seen says the
instrument is doing something it may not be.

**A routing is pointed as well as chosen.** The two columns of that table are
hard for opposite reasons, and both of them are answered now. A destination is
one of 133 names the instrument's display prints abbreviated, so the row chooses
it from a list that can be typed into rather than scrolled — three letters and
`VCF Envelope Attack` is the only one left. And a depth is a number nobody can
pick without having already heard it.

The second answer covers both, and it is the one an editor has and a front panel
does not: `map` sends the routing out into the window. Every control the matrix
can reach lights up, on the front panel and in all fourteen racks and on the four
effect engines at once; everything else is passed over; the sections still open
and the panel still scrolls and nothing edits the sound. Taking hold of a lit
control is the answer — a click chooses it, and a drag sets the depth as well,
from how far the drag would have moved it. It is the answer to what actually
makes that column hard, which is that knowing which abbreviation stands over the
fader you have in mind is harder than knowing the fader.

Which controls light is the same join the mark is drawn from, read backwards, so
nothing is written down for it either. What *is* assumed is one number: the drag
is read as if a full depth moved the control over its whole range, because the
manual prints no law relating a depth byte to its destination's range. It is
marked in `control-ui/src/aim.rs`, printed on the page, asked for in
[deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38) and
listed in [waiting.md](waiting.md). The ranking that picks `VCF Attack` over `All
Attack` when both reach a control is the other judgement this page is making that
the library would make better, and it is
[#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39).

That is also where hand layout stops being only an arrangement of whole
controls. A row gives a list the width its names need and turns the depth fader
onto its side, because eight rows cannot each be 128 points tall and eight
depths that do not line up are eight numbers nobody can compare. The control is
still whatever the library says the parameter is, chosen by the same code from
the same table: a depth a later library gives a value table arrives as a list,
in the row, with nothing here to edit. How much room a control has and which way
it runs is what a hand layout may change; what the control is, is not.

An envelope is the first group whose hand layout adds a drawing rather than
rearranging controls, and it is drawn under the same two refusals as everything
else here. Its horizontal axis is proportion and not time, because the manual
gives the ends of each range and not the curve between them. And its four curve
parameters do not bend the segments: what a curve byte does to the shape is not
published, a drawing that guessed would be wrong in a way nobody could see, and
they stay faders in the rack until the library measures them.

Hand layout changes the arrangement and never what a control is, and the name is
the one place even that is bent. Seventeen parameters holding a character each
are one word to the person reading them, so they are drawn as one field and the
rack keeps them where the table put them. It costs the view layer one message
that is not about a single parameter, and nothing below the view layer at all: a
keystroke is still one NRPN to the one character it moved, worked out by
`Program::changes` against the program with the new name in it, so typing a
letter onto the end of a name is twelve bytes rather than two hundred.

The same rule covers the arrangement and not only the controls. Which panels
exist and what order they are in is read off the parameter table's own offsets
rather than written down here, so a group a later library adds appears in its
right place with nothing in this repository to edit. A number a person would
read — how many algorithms an effect engine has, how many panels there are — is
asked of the library at the moment it is drawn, for the firmware that answered
the inquiry, rather than typed into a sentence that goes quietly wrong on the
next release.

## Order

Each stage ends somewhere usable. Nothing is built two stages before it is
needed.

| | | |
| --- | --- | --- |
| 0 | Workspace | **Done.** Four crates, pinned dependencies, CI running the four commands in the README, licence and notice files. |
| 1 | `deepmind-host` | **Done.** Port enumeration, `Port` over midir, `Clock`, the device thread, commands in and events out, the simulator as a selectable port. Tested against `sim` with no hardware. |
| 2 | First light | **Done.** Desktop window, port picker, identity, read the edit buffer, VCF editable end to end with assumed and confirmed drawn differently. |
| 3 | Every parameter | **Done.** All fourteen groups from `Group::parameters`, one at a time behind a section bar, because a complete ugly editor beats a beautiful partial one. Every panel that is not a rack is laid out: the program's name, the three envelopes, the modulation matrix and the control sequencer. |
| 4 | The librarian | **Done.** Read and write `.syx`, read a bank with progress and cancel, browse a pack on a surface of its own beside the editor, and load a program into the edit buffer as a difference. |
| 5 | The effects | **Done.** Four engines and 35 algorithms, from the tables the library published in 26.2 ([deepmind-midi#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18)), laid out on the FX page's own grid with the shape each algorithm's figure draws, and opening on a picture of the chain the four are wired into — both of which 26.3 published ([#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22), [#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23)). 26.4 finished it: the mark of each algorithm's family at the head of its strip, the three engines that can be switched out of circuit saying so, and a screen on the two whose response is published ([#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30), [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31), [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33)). |
| 6 | The plugin | Simple mode, then state, then advanced mode in the same window. The CLAP comes out first, having nothing to settle; then the AU, once the bundle signs and `auval` passes; then the VST3, once Steinberg's terms are. |
| 7 | Hardware | The questions below, answered with a cable. Findings go to the library. |

**Everything up to stage 6, and then polish, before stage 6 itself.** The
decision is the desktop application's: stage 5 was the last thing it was
missing, and the work now is making the application good rather than making it
bigger. The wording, the edges, the panels whose arrangement is right and whose
proportions are not, and the thousand small things between "every parameter is
reachable" and "a DeepMind owner reaches for this instead of the front panel". A
plugin wrapping a half-finished editor is a half-finished editor in three
formats, and a format is the most expensive place to discover that a layout was
wrong. So stage 6 starts when the desktop application is finished rather than
when it is complete.

Polish has started, and it is where the front panel above came from: the window
opens on the instrument rather than on its first section, the name is set as the
mark sets it, the parts of the window that are not parameters are drawn on the
instrument too, and every plate of that panel has the instrument's own screen
over its faders with the drawing of its own section on it.

What is left of it, in no order yet: the fourteen racks have no displays over
them the way the panel's plates and the effects now do, which is an arrangement
rather than a decision; and keyboard focus is not wired anywhere, which is the
one gap in this editor that is not waiting on anybody else. The buttons are
moulded and their legends are printed under them, the way the hardware's are;
the effect slots have the knobs, the rows and the measured colours the manual's
figures use; the sequencer strip has its centre line, its skip mark and its
dimmed tail; and the front panel is read off the library rather than out of a
table in `home.rs`. Every one of those was a library release away, and the
release came.

26.4 is the release that finished the drawings. The shapes the plate displays
draw are the library's own functions now — an envelope's bends, an LFO's wave,
a filter's roll-off, the arpeggiator's gates — so the four curve faders under an
envelope move the envelope, a filter stands on the decibel vertical the slope of
a pole is published in, and the arithmetic about the instrument that was written
in `scene.rs` and `envelope.rs` is deleted. The effects page got the rest of it:
a mark at the head of every engine, a picture on the two engines whose panels
say what they do to a signal, and the answer to how an effect is switched off,
which is that on 32 of the 35 it is not.

Which of those are somebody else's to answer is written down rather than
remembered: [waiting.md](waiting.md) is the list of what this repository has
asked the library for, what each answer would change here, and what the window
does meanwhile. It is the file to read at the start of a batch of work, because
a closed issue on it is a panel that can stop apologising. There is nothing open
on it today, which is the first time that has been true.

Stage 7 waits with it. A cable answers questions about a protocol, and what it
finds goes to the library rather than here, so nothing in stages 4 and 5 is
blocked on one; what it would change about a window is worth knowing once that
window is worth showing somebody.

**The first release is the desktop application.** Stage 4 made it an editor and
a librarian rather than one of the two, which is the bar a release had to clear;
stage 5 and the polish after it are what make it one worth downloading. Stage 6
is the first build a DAW can load, and it is deliberately the later of the two:
a plugin nobody can run is worth less than an application somebody can.

## Not planned

The globals, the sequencer patterns, the chord memories and the calibration data
stay unedited. The manual gives their dumps a length and never says what is at
which offset, so there is nothing to draw. `Event::Unhandled` is reported and
dropped. If hardware ever names those bytes, they are a library change first.

No command-line tool. No patch database, no cloud, no sharing. No audio: this
application never sees a sample.

## Questions a cable answers

1. Does the unit accept its own program dump back, and does that store it or
   only load it? Everything about a write path hangs on this.
2. How fast can NRPN be sent before the unit drops edits, and is there a
   difference between one parameter changing quickly and many changing at once?
   The coalescing rate is a guess until this is measured.
3. Does the unit echo an edit it receives? If it does, the loop guard is
   necessary rather than precautionary.
4. What does the unit do with `ControlAppNotifyRequest`, and does announcing
   itself change what a host receives? The reply carries the transmit and
   receive channels and the currently selected program, which the application
   would otherwise have to infer. The state machine does not decode that reply
   today, so reading it is a library change and not a local workaround
   ([deepmind-midi#20](https://github.com/MysteriousWolf/deepmind-midi/issues/20)).
5. Is the device ID really the global MIDI channel, as the manual's global
   settings table says?
6. How long does a bank dump actually take, and does the unit pause between
   dumps? The twelve-second figure above is arithmetic, not a measurement.

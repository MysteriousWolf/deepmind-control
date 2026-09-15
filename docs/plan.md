# Plan

What this repository builds, in what order, and what has already been decided.
Stages 0 to 2 are written, stage 3 is generated and is being laid out by hand
one group at a time, and the rest is not written. The [order](#order) says which
is which.

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

The readable version exists: `spec/panels.toml` has the full name, the control
kind and the grouping for every slot of all 35 algorithms, and
`spec/layout.toml` has the grid, the control shape and four measured colours per
panel. Both are in the library's repository, and neither is in its API.

They are not transcribed here. Duplicating 35 panels into this repository means
maintaining a second copy of a table that is generated from a specification,
which is the thing the library exists to prevent. The FX editor waits on a
feature in `deepmind-midi` that publishes them
([deepmind-midi#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18)),
and until it lands the FX section shows the twelve slots under their protocol
names and the algorithm under its own, which is honest and ugly.

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

## The window is the instrument's colours

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
| Mod Matrix | **Done.** Eight rows of source, destination and depth, read across rather than down, because twenty-four slots in one wrapping line are eight sentences with their words in the wrong order |
| Control Sequencer | 32 steps, which is a sequencer and not a rack |
| Effects | Four engines, waiting on the library publishing `spec/panels.toml` |

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

What the rows cannot do yet is say where they point. A destination is a value
in a table of 133 names the display prints abbreviated, and nothing joins
`VCF Freq` to `ParamId::VcfFrequency`, so neither the mark on a modulated slot
nor a way from a row to the parameter it moves can be drawn without matching
those names here — which is the second copy of a generated table this
repository refuses to keep. It waits on the library
([deepmind-midi#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19)).

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
| 3 | Every parameter | **Generated, and being laid out.** All fourteen groups from `Group::parameters`, one at a time behind a section bar, because a complete ugly editor beats a beautiful partial one. The program's name, the three envelopes and the modulation matrix are laid out; the control sequencer is what remains. |
| 4 | The librarian | Read and write `.syx`, read a bank with progress and cancel, browse a pack, load a program into the edit buffer as a difference. |
| 5 | The effects | Waits on the library publishing the panel tables ([deepmind-midi#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18)). Four engines, 35 algorithms, the routing graph. |
| 6 | The plugin | Simple mode, then state, then advanced mode in the same window. The CLAP comes out first, having nothing to settle; then the AU, once the bundle signs and `auval` passes; then the VST3, once Steinberg's terms are. |
| 7 | Hardware | The questions below, answered with a cable. Findings go to the library. |

Stage 5 can move ahead of stage 4 if the library gets there first. Stage 7 can
start the day a synthesizer is available and will change something in every
stage before it.

**The first release is the desktop application**, once stage 4 has made it an
editor and a librarian rather than one of the two. Stage 6 is the first build a
DAW can load, and it is deliberately the later of the two: a plugin nobody can
run is worth less than an application somebody can.

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

# Waiting on the library

What this repository has asked [`deepmind-midi`](https://github.com/MysteriousWolf/deepmind-midi)
for, what each answer would change here, and what the window does meanwhile.

**Check this file at the start of a batch of work.** Every row is something the
editor is drawing less well than it could, and the fix is a released version and
a pin bump rather than anything in this tree. A closed issue here is work
unblocked.

The split is the reason the list exists rather than the problem: anything that
is a fact about the instrument lives on the other side of it, and a panel drawn
from a fact this repository transcribed is a second copy of a generated table,
going stale silently on the next release. So the answer to "the window cannot
say that" is an issue, not a table.

There were two, and both are gone. The front panel's own contents were a table in
`control-ui/src/home.rs`, because the home screen could not be built without
them; 26.3 published the same thing and the table is deleted. The shapes the
plate displays draw were arithmetic about the instrument written in
`control-ui/src/scene.rs` and `envelope.rs` — where a filter's corner sits for a
byte, what a `Sample & Hold` looks like — and 26.4 published those as functions,
so they are deleted too.

That is the measure of the arrangement working: both were named out loud, scoped
to the files they were in, and removed by the other side of the split rather than
becoming permanent. What is left in those two files is a sample loop and the one
number the library declines to draw, which is the high-pass slope, marked where
it is used.

## Open

None. The four rows that were here — the effects page asking for more than the
library published — were answered together in 26.4 and are in the table below.

That is the arrangement working rather than a milestone: every one of them was
something this editor was drawing less well than it could, the fix was a
released version and a pin bump, and the tree they were blocking is smaller for
each one landing. When the next thing this window cannot say turns up, it goes
here as an issue and not as a table in `control-ui`.

## Answered

| | | |
| --- | --- | --- |
| [#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18) | The FX panel tables | 26.2. Stage 5: the effects are four engines chosen a tab at a time, and every slot is named by the algorithm its engine is running |
| [#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19) | Which parameter a modulation destination moves | 26.2. The mark on a slot the matrix is pointed at, drawn from the destinations the patch holds rather than by matching names here |
| [#20](https://github.com/MysteriousWolf/deepmind-midi/issues/20) | The Control App Notify reply, which says what is selected | 26.2 decodes it. Nothing here reads it yet: what a unit actually does with the request is [question 4](plan.md#questions-a-cable-answers), and the librarian's "which slot is this" is worth building once a cable has answered |
| [#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22) | The FX panel layout: the grid, the control shape, the measured colours | 26.3, as data rather than as the drawings. Spent in full: a slot stands in the column and row the instrument's own FX page draws it in, a slot on a knob panel is drawn as a knob, and all four measured colours are on the open plate — chassis, face, accent and the cap a finger moves. See [the plan](plan.md#the-effect-panels-are-the-librarys-data) for why the last one took two goes |
| [#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23) | The FX routing graph, and what Insert, Send and Bypass do | 26.3, as edge lists with the two loops declared. Spent: the chain is drawn on a display over the two settings it is a picture of, with the loop dashed under the engines and the analog path along the foot |
| [#24](https://github.com/MysteriousWolf/deepmind-midi/issues/24) | A sequencer step is bipolar, `0` skips it, and the length bounds the run | 26.3, as `ParamId::shape`, `ParamId::inactive` and `ParamId::bounded_by`. Spent: the strip reads each step about its centre, says `skip` where a step is skipped, and rules the steps that are played |
| [#25](https://github.com/MysteriousWolf/deepmind-midi/issues/25) | A parameter's displayed range, as the manual prints it | 26.3, as `ParamId::display`. Spent: the footer prints `50.0 Hz to 20000.0 Hz` where the manual has it, and the raw ends where it does not |
| [#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26) | Which parameters the front panel puts a control under | 26.3, as `front::sections`. **The transcription is deleted.** `control-ui/src/home.rs` reads the panel off the library and spends what it saved on layout: two of the nine sections are drawn as more than one plate |
| [#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28) | What a parameter does, behind a default-off feature | 26.3, as `ParamId::description` under `descriptions`. Spent: the workspace turns the feature on and the footer prints the sentence |
| [#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30) | How an FX slot is turned off, or that it cannot be | 26.4, and the answer is that on 32 of the 35 it is not: `FX n Type` has no `Off`, and what takes effects out of circuit is the `Bypass` mode, which is the whole block of four. `FxSlot::is_enable` names the three that carry a switch of their own. Spent: the strip says `out of circuit` where one of those three is off and the chain draws that engine as something the signal goes past. Nothing collapses, because there is nothing to collapse |
| [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31) | A mark per algorithm, as unit geometry or as a `Family` | 26.4, as both: nine families across the 35 and a `Mark` per family, published as strokes in a unit box and as a seven by seven grid for a display with no room to stroke anything. Spent in full — `control-ui/src/mark.rs` lays the strokes out in the window's own ink at the head of every engine's strip, and the chain's boxes blit the grid |
| [#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32) | The generators the panels draw: envelopes, LFO shapes, filter responses, arpeggiator gates | 26.4, as `generator`. **The arithmetic is deleted.** The envelope corners, the seven waves and the scatter behind the sampled two, the roll-off a pole count gives and the gates a rate byte used to stretch are all gone from `scene.rs` and `envelope.rs`, which are a sample loop over a published function now. The four curve faders under an envelope bend it, the filter stands on the published decibel vertical, and the refusal to label an axis moved into the library's own `Scale` where it can be tested |
| [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33) | What an effect does to a signal: a `Quantity` per slot, and a response where it is known | 26.4. Spent: the line under every slot says what kind of quantity the byte is where the manual prints no range for it, and the two tap delays have the screen every other panel in this window has. The other 33 have none, which is the answer — a reverb's impulse response is its designer's |

## What an answer costs here

A minor version in `Cargo.toml`'s `[workspace.dependencies]`, which is a
deliberate edit rather than a `cargo update`: a value table that changes is a
change to what the window draws, and it arrives when somebody moves the pin.
Then the panel that was waiting stops apologising, and the apology it was
printing — the caveat under a section, the note in
[the plan](plan.md#the-effect-panels-are-the-librarys-data) — comes out with it.

## Asking for something new

An issue there, not a table here. What has worked, in every row above: what is
missing and where it already exists in `spec/`, why this editor cannot do its
job without it, the accessor that would answer it, and what the library should
*not* do — because half of what is useful about this library is what it refuses
to guess, and an issue that does not say where that line falls is an issue that
invites it to be crossed.

Ask for getters rather than for the file. The specification's shape is the
library's business and changing it should not reach a host; what a host needs is
a method it can call and a type it can hold, which is what every answered issue
above turned into.

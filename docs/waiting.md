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

There was one exception, and it is gone. The front panel's own contents were a
table in `control-ui/src/home.rs`, because the home screen could not be built
without them: written as `ParamId`s so a rename failed the build rather than
mislabelling a fader, in one file, with that file's first paragraph saying it
was there until the library published the same thing and the issue that would
delete it linked from the code. 26.3 published it and the table is deleted.
That is the measure of the arrangement working: the transcription was named out
loud, scoped to one file, and removed by the other side of the split rather than
becoming permanent.

## Open

Four, all of them the effects page asking for more than the library publishes.
The first is a gap; the other three are the same move made three times — the
library owns what a thing *is*, and a window that decides it instead is a second
copy of a generated table, going stale silently on the next release.

| | | |
| --- | --- | --- |
| [#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30) | How an FX slot is turned off, or that it cannot be | `fx_type` is 35 algorithms with no `Off`, the Effects group has no per-slot enable, and `FX Mode` is the whole block. Three algorithms carry their own enable in their twelve bytes (`EdisonEX1` `ON`, `NoiseGate` `PWR`, `RackAmp` `CAB`), which is what makes the absence look like a gap. **When it lands:** an engine running nothing collapses to a marker and gives its room back, instead of drawing twelve controls that reach nothing. Meanwhile every engine draws its full panel, because this window has no way to know which one is doing nothing |
| [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31) | A mark per algorithm, as unit geometry or as a `Family` | **When it lands:** the mark goes in each engine's header, where four long names currently have to be read one at a time. The `Family` half alone would do most of it. Meanwhile the header carries the full name and the category, which is the most the library can currently answer |
| [#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32) | The generators the panels draw: envelopes, LFO shapes, filter responses, arpeggiator gates | The big one, and the one that would take real decisions out of this tree. Ten screens in `control-ui/src/scene.rs` and `envelope.rs` are arithmetic about the instrument written in a window — where a filter's corner sits for a byte, what a `Sample & Hold` looks like. **When it lands:** every one of them becomes a sample loop over a published function, and the refusal to label an axis the manual does not print moves into the library's own `Scale`, where it can be tested. This is the nearest thing left to the `home.rs` transcription, and it should go the same way |
| [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33) | What an effect does to a signal: a `Quantity` per slot, and a response where it is known | #32 pointed at the effects. **When it lands:** the effects get the screen every other panel has — four small pictures, one per engine, and a parameter that shows what it is doing as it moves. The `Quantity` half is worth having first and on its own, because grouping an algorithm's slots into a picture currently means matching on titles across 35 algorithms. Meanwhile the effects are the one panel with no screen on it, which is honest: there is nothing published to draw there |

Nothing is banked against any of them. The window does the most the library can
currently answer for, and each row says what that is.

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

## What an answer costs here

A minor version in `Cargo.toml`'s `[workspace.dependencies]`, which is a
deliberate edit rather than a `cargo update`: a value table that changes is a
change to what the window draws, and it arrives when somebody moves the pin.
Then the panel that was waiting stops apologising, and the apology it was
printing — the caveat under a section, the note in
[the plan](plan.md#the-effect-panels-are-the-librarys-data) — comes out with it.

## Asking for something new

An issue there, not a table here. What has worked, in all five above: what is
missing and where it already exists in `spec/`, why this editor cannot do its
job without it, the accessor that would answer it, and what the library should
*not* do — because half of what is useful about this library is what it refuses
to guess, and an issue that does not say where that line falls is an issue that
invites it to be crossed.

Ask for getters rather than for the file. The specification's shape is the
library's business and changing it should not reach a host; what a host needs is
a method it can call and a type it can hold, which is what every answered issue
above turned into.

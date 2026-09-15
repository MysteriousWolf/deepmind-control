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

There is one exception, and it is [#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26):
the front panel's own contents, which the home screen could not be built without.
It is written as `ParamId`s so that a rename fails the build rather than
mislabelling a fader, it is in one file, and that file's first paragraph says it
is there until the library publishes the same thing. It is the measure of how
much this repository is prepared to transcribe: one table, said out loud, with
the issue that deletes it linked from the code.

## Open

| | | |
| --- | --- | --- |
| [#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22) | The FX panel layout: the grid, the control shape and the four measured colours per algorithm | The effects panel would be laid out and coloured as the instrument's own figures are, instead of in this editor's rack and materials. It is what the knob in [the interface](interface.md#knob) is waiting for |
| [#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23) | The FX routing graph as an edge list, and what Insert, Send and Bypass do to the two paths | The chain above the four plates could be drawn, and the window could say which engines currently reach the output. Today the connection mode is the list its value table names and nothing more |
| [#24](https://github.com/MysteriousWolf/deepmind-midi/issues/24) | A sequencer step is bipolar, `0` skips it, and `Sequence Length` bounds which steps play | The strip gains a centre line, a skip mark and a dimmed tail. Also where the `kind = "switch"` on two steps that accept 256 values gets settled |
| [#25](https://github.com/MysteriousWolf/deepmind-midi/issues/25) | A parameter's displayed range, as the manual prints it | A fader could say what its two ends mean, the way every effect slot already does. 26 parameters have one |
| [#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26) | Which parameters the front panel puts a control under, what it prints over them, and in which row | **The one table this repository transcribes.** `control-ui/src/home.rs` holds it until this lands, and says so in its own first paragraph. It is also what a DeepMind 6 would need to be drawn as a 6 |

## Answered

| | | |
| --- | --- | --- |
| [#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18) | The FX panel tables | 26.2. Stage 5: the effects are four engine plates, and every slot is named by the algorithm its engine is running |
| [#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19) | Which parameter a modulation destination moves | 26.2. The mark on a slot the matrix is pointed at, drawn from the destinations the patch holds rather than by matching names here |
| [#20](https://github.com/MysteriousWolf/deepmind-midi/issues/20) | The Control App Notify reply, which says what is selected | 26.2 decodes it. Nothing here reads it yet: what a unit actually does with the request is [question 4](plan.md#questions-a-cable-answers), and the librarian's "which slot is this" is worth building once a cable has answered |

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

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

**There is nothing about the instrument written down in this tree.** That
sentence could not be written before 26.5. The front panel's own contents were a
table in `control-ui/src/home.rs` and 26.3 published them. The shapes the plate
displays draw were arithmetic in `control-ui/src/scene.rs` and `envelope.rs` and
26.4 published those. What was left after that was one number — `HIGH_PASS_SLOPE`,
6 dB per octave, transcribed out of a doc comment in the library — and 26.5
published the curve it belonged to. It is deleted, and with it the last place
this repository kept a fact it did not own.

That is the measure of the arrangement working. Every one of those was named out
loud, scoped to the file it was in, and removed by the other side of the split
rather than becoming permanent.

## Open

Two. Six of the seven rows this file carried closed in 26.5, the seventh closed
as far as it can until somebody puts a meter on the instrument, and the second
row here is what reading the new answers turned up: a list of the ends that came
back without a picture.

| | | What the window does meanwhile |
| --- | --- | --- |
| [#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38) | What a modulation depth is worth: the law between `Mod n Depth` and the destination's own range | **Assumes full depth moves the destination over its whole range.** 26.5 published `ParamId::modulation_reach`, which is the accessor and not the answer: it returns `None` for every pair, because the manual prints the depth's own range and nothing relating a depth to what it does at the other end of a routing, and nobody has measured it. What changed here is that the assumption is now a fallback behind a question — `reach_of` in `control-ui/src/mapping.rs` is the one place it lives, and both the drag that sets a depth and the band that says how far the existing depths reach go through it. The day a measurement lands in `spec/measurements.toml` the fallback stops being reached and nothing in this tree changes. It blocks nothing: the drag puts a number in the same byte the depth fader already held |

| [#43](https://github.com/MysteriousWolf/deepmind-midi/issues/43) | The ends `cell_of` and `glyph` still have none for | **Draws an empty seven-by-seven box and the name beside it**, which is the same refusal the effect families' marks were drawn under before `Algorithm::mark` existed. Measured against 26.5's own tables rather than eyeballed: 3 of the 25 modulation sources (`BreathCtrl`, `Voice Num`, `Uni Voice` — `Off` is rightly none), 11 destinations that name a *set* with no one narrowest among them (`All Attack`, `Env Rates`, the three `Env n CurveS`), and 8 that name no program parameter at all (`OSC1 Pitch`, `VCA Pan` and the rest of the voice's own quantities), which are also the ones no control can light for. The 48 `FX n Param m` are a question rather than a gap: what one *is* depends on the algorithm loaded, so the issue asks whether the family's own mark should stand in or whether "no glyph by design" should be said out loud |

That row is the newest kind of ask and the one that shows what the arrangement
is for. It was answered *structurally* without being answered factually: the
library gave the window somewhere to ask, said plainly that it has no answer
yet, and said where a measurement would go. The window's guess did not get
better and it did get smaller — one function instead of two copies — and it will
disappear without a diff here.

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
| [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31) | A mark per algorithm, as unit geometry or as a `Family` | 26.4, as both: nine families across the 35 and a `Mark` per family, published as strokes in a unit box and as a seven by seven grid for a display with no room to stroke anything. Spent in full — `control-ui/src/mark.rs` lays the strokes out in the window's own ink at the head of every engine's strip, and the chain's boxes blit the grid. 26.5 went further than the issue asked: `Algorithm::own_mark` tells a plate reverb from a hall where a symbol can carry the difference, which 26.4 had declined to, and `Algorithm::mark` hands back whichever applies — so this window got the better marks with nothing to change |
| [#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32) | The generators the panels draw: envelopes, LFO shapes, filter responses, arpeggiator gates | 26.4, as `generator`. **The arithmetic is deleted.** The envelope corners, the seven waves and the scatter behind the sampled two, the roll-off a pole count gives and the gates a rate byte used to stretch are all gone from `scene.rs` and `envelope.rs`, which are a sample loop over a published function now. The four curve faders under an envelope bend it, the filter stands on the published decibel vertical, and the refusal to label an axis moved into the library's own `Scale` where it can be tested |
| [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33) | What an effect does to a signal: a `Quantity` per slot, and a response where it is known | 26.4. Spent: the line under every slot says what kind of quantity the byte is where the manual prints no range for it, and the two tap delays have the screen every other panel in this window has. The other 33 have none, which is the answer — a reverb's impulse response is its designer's |
| [#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35) | `generator::lfo` reads the shape byte and ignores the four other parameters that shape an LFO | 26.5. Spent in full: the slew rate is in the shape the library hands back, so corners round and a square becomes a ramp between its levels; `Generator::rest` is where an LFO sits rather than an assumed middle, and `lfo_unipolar` is the same wave read from the floor; and `lfo_fade` is the `Delay / Fade` byte, which was a fader that moved nothing on the glass and is now a dotted line the wave is brought in under |
| [#36](https://github.com/MysteriousWolf/deepmind-midi/issues/36) | The marks along a generator's horizontal | 26.5, as `Generator::marks` and `Generator::anchored`. Spent: the envelope plates write `A`, `D`, `S` and `R` where the library says each segment begins — the one thing four faders under a line could not say — a filter's corner and a pulse's falling edge are marks rather than arithmetic, and a free-running LFO's left edge is left unruled, because a phase the instrument does not define is not a phase a screen should draw |
| [#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37) | The high-pass response, and what the oscillators are making | 26.5. **The last transcribed number is deleted.** `high_pass_response` is the curve, on the low-pass's own vertical and the low-pass's own span, so the two plates of one section agree because the library says so rather than because this window arranged it. `oscillator` sums `OSC 1`'s two waves at a weight it states is its own reading, which retires the side-by-side drawing this window used instead of guessing, and `noise` retires the scatter table |
| [#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39) | The destination join, read backwards, and which of several destinations is the narrowest | 26.5, as `ValueTable::values_naming`. Spent: `Mapping::names` is one call and a `next`. The walk was over the library's own slices and was fine; the *ranking* was a judgement about the instrument being made in a window, and it is on the other side of the split now — down to what happens when two destinations move the same number of parameters, which the library answers in value order and says outright that nothing makes one of them narrower |
| [#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40) | A dot-matrix cell for every modulation source and destination | 26.5, as `ValueTable::cell_of`. Spent in full, and it took `ParamId::glyph` with it: a source's cell is the library's drawing of it and a destination's picture is the glyph of the parameter it moves, so the patch bay's two columns of abbreviations are two columns of pictures and the empty 7×7 boxes beside each matrix row are filled wherever the library has drawn the end. Which is most of them and not all: what is left is [#43](https://github.com/MysteriousWolf/deepmind-midi/issues/43) |
| [#41](https://github.com/MysteriousWolf/deepmind-midi/issues/41) | Which way a modulation source swings a control it reaches | 26.5, as `ValueTable::swing_of` and `Swing`. Spent: the band on a control runs either side of where it sits for a `Centred` source and from it for a `Rising` one. This window took the depth's own sign as the whole answer, which is right for a wheel and for an envelope and wrong for an LFO — a band drawn one way from a filter's corner said the filter could only ever open |

## What arrived unasked

26.5 published two things this repository had not filed an issue for, and both
are spent:

- **A glyph per parameter and per effect slot.** `ParamId::glyph`,
  `FxSlot::glyph` and `Controller::glyph`: the picture of what a control *does*,
  on the same seven by seven grid as the marks and the cells, and the same
  picture wherever the same job is met. The footer puts it before a parameter's
  name and every effect slot wears its own, so a decay reads as a tail and a mix
  as wet against dry before the word is read. 177 of the 242 parameters carry
  one; the rest are the effect slots, which have their own, and the seventeen
  characters of the program's name, which are letters and not a control.
- **What kind of thing an effect is.** `Algorithm::characters`: vintage,
  modelled, stereo, dual, multiband, two in one, lo-fi, modulated, dynamic. Read
  rather than derived — the library's own file carries a reason per membership,
  and the reasons are the manual's name for the effect, the slots it gives it, or
  the unit a name refers to. A window matching on `Vintage` in a `full_name`
  would be right until the day it was not. They are a caption beside an engine's
  name, and empty for the plain reverbs and the noise gate, whose family says
  everything a word can.

Neither was a row in this file, which is worth noticing: the arrangement works
in the other direction too. A library that knows what a host is drawing publishes
the thing the host has not thought to ask for yet.

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

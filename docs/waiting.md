# Waiting on the library

What this repository has asked
[`deepmind-midi`](https://github.com/MysteriousWolf/deepmind-midi) for, what
each answer would change here, and what the window does meanwhile.

**Check this file at the start of a batch of work.** Every row is something the
editor draws less well than it could, and the fix is a released version and a
pin bump rather than a change in this tree. A closed issue here is work
unblocked.

The split is why the list exists. Anything that is a fact about the instrument
lives in the library, so a panel drawn from a fact transcribed here would be a
second copy of a generated table, going stale silently on the next release. The
answer to "the window cannot say that" is an issue, not a table.

**Almost nothing about the instrument is written down in this tree.** The front
panel's contents were a table in `control-ui/src/home.rs` until 26.3 published
them. The shapes the plate displays draw were arithmetic in
`control-ui/src/scene.rs` and `envelope.rs` until 26.4 published them.
`HIGH_PASS_SLOPE`, 6 dB per octave, was copied out of a library doc comment;
26.5 published the curve it belonged to and it is deleted.

`SILKSCREEN`, `CYAN` and `BLUE` were the last of it: six rows saying where on
the front of the instrument a press the library already described is printed. A
smaller kind of transcription than the one 26.3 replaced — no parameter, no
range, no meaning — but a transcription, and a transcription nobody is watching
is the thing this file exists to stop. So it was watched by a test that failed
when the library caught up.

**26.5.2 is when it caught up**, and the test failed exactly as designed. All
six rows are gone. **There is no transcription of the instrument left in this
repository.**

## Open

Two, both from 26.5, and both answered structurally rather than factually.

| | | What the window does meanwhile |
| --- | --- | --- |
| [#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38) | What a modulation depth is worth: the law between `Mod n Depth` and the destination's own range | **Assumes full depth moves the destination over its whole range.** 26.5 published `ParamId::modulation_reach`, which is the accessor and not the answer: it returns `None` for every pair, because the manual prints the depth's own range and nothing relating a depth to what it does at the far end of a routing. The assumption is now a fallback behind that question, in `reach_of` in `control-ui/src/mapping.rs`, which both the depth-setting drag and the reach bands go through. When a measurement lands in `spec/measurements.toml` the fallback stops being reached, with no change here. It blocks nothing: the drag writes to the same byte the depth fader already held |
| [#43](https://github.com/MysteriousWolf/deepmind-midi/issues/43) | The ends `cell_of` and `glyph` have no picture for | **Mostly answered by 26.5.1, with no diff here.** The three undrawn sources are drawn (`BreathCtrl` by a new breath glyph, `Voice Num` and `Uni Voice` by cells that count), so every source but `Off` has a picture; the eleven destinations naming a set with no one narrowest member are drawn as the set; and the rest take the glyph of the one parameter they move. The patch bay picks the sources up through `ValueTable::cell_of`, which it was already calling; the destinations took a two-line change, because this window asked the destination table for the glyph of the one parameter a value moves and never for the value's own cell, which did not exist when it was written. What is left is the 48 `FX n Param m`, where the library states the position rather than filling it: what one *is* depends on the algorithm loaded, so there is no one picture, and the window draws the empty box and the name beside it |

Both are the kind of ask that gets answered structurally before it is answered
factually: the library gave the window somewhere to ask, said what it does not
know, and said where the answer would land. #43 has since been most of the way
answered that way — 26.5.1 drew 22 more cells and this window picked up every
one of them without a diff, because the call it makes had not changed. What is
left of it, and the whole of #38, is a measurement nobody has taken. The guess
did not get better; it got smaller, and it will disappear without a diff here
too.

## Answered

| | | |
| --- | --- | --- |
| [#18](https://github.com/MysteriousWolf/deepmind-midi/issues/18) | The FX panel tables | 26.2. Stage 5: four engines, every slot named by the algorithm its engine is running |
| [#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19) | Which parameter a modulation destination moves | 26.2. The mark on a slot the matrix points at, drawn from the destinations the patch holds rather than by matching names here |
| [#20](https://github.com/MysteriousWolf/deepmind-midi/issues/20) | The Control App Notify reply, which says what is selected | 26.2 decodes it. Nothing here reads it yet: what a unit actually does with the request is [question 4](plan.md#questions-a-cable-answers), and the librarian's "which slot is this" is worth building once a cable has answered |
| [#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22) | The FX panel layout: the grid, the control shape, the measured colours | 26.3, as data rather than as drawings. Spent in full: a slot stands in the column and row the instrument's own FX page draws it in, a slot on a knob panel is drawn as a knob, and all four measured colours are on the open plate |
| [#23](https://github.com/MysteriousWolf/deepmind-midi/issues/23) | The FX routing graph, and what Insert, Send and Bypass do | 26.3, as edge lists with the two loops declared. Spent: the chain is drawn over the two settings it is a picture of, with the loop dashed under the engines and the analog path along the foot |
| [#24](https://github.com/MysteriousWolf/deepmind-midi/issues/24) | A sequencer step is bipolar, `0` skips it, and the length bounds the run | 26.3, as `ParamId::shape`, `ParamId::inactive` and `ParamId::bounded_by`. Spent: the strip reads each step about its centre, says `skip` where a step is skipped, and rules the steps that are played |
| [#25](https://github.com/MysteriousWolf/deepmind-midi/issues/25) | A parameter's displayed range, as the manual prints it | 26.3, as `ParamId::display`. Spent: the footer prints `50.0 Hz to 20000.0 Hz` where the manual has it, and the raw ends where it does not |
| [#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26) | Which parameters the front panel puts a control under | 26.3, as `front::sections`. **The transcription is deleted.** `control-ui/src/home.rs` reads the panel off the library and spends what it saved on layout: two of the nine sections are drawn as more than one plate |
| [#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28) | What a parameter does, behind a default-off feature | 26.3, as `ParamId::description` under `descriptions`. Spent: the workspace turns the feature on and the footer prints the sentence |
| [#30](https://github.com/MysteriousWolf/deepmind-midi/issues/30) | How an FX slot is turned off, or that it cannot be | 26.4. On 32 of the 35 it cannot: `FX n Type` has no `Off`, and what takes effects out of circuit is the `Bypass` mode, which covers all four engines. `FxSlot::is_enable` names the three algorithms that carry a switch of their own. Spent: the strip says `out of circuit` where one of those three is off, and the chain draws that engine as something the signal goes past |
| [#31](https://github.com/MysteriousWolf/deepmind-midi/issues/31) | A mark per algorithm, as unit geometry or as a `Family` | 26.4, as both: nine families across the 35, and a `Mark` per family published as strokes in a unit box and as a 7x7 grid for a display with no room to stroke anything. Spent in full: `control-ui/src/mark.rs` lays the strokes out at the head of every engine's strip, and the chain's boxes blit the grid. 26.5 added `Algorithm::own_mark`, which tells a plate reverb from a hall, and `Algorithm::mark` hands back whichever applies, so this window got the better marks with nothing to change |
| [#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32) | The generators the panels draw: envelopes, LFO shapes, filter responses, arpeggiator gates | 26.4, as `generator`. **The arithmetic is deleted.** The envelope corners, the seven waves and the scatter behind the sampled two, the roll-off a pole count gives and the gates a rate byte used to stretch are gone from `scene.rs` and `envelope.rs`, which are a sample loop over a published function now. The four curve faders under an envelope bend it, and the filter stands on the published decibel vertical |
| [#33](https://github.com/MysteriousWolf/deepmind-midi/issues/33) | What an effect does to a signal: a `Quantity` per slot, and a response where it is known | 26.4. Spent: the line under every slot says what kind of quantity the byte is where the manual prints no range, and the two tap delays have a screen. The other 33 have none, which is the answer: a reverb's impulse response is its designer's |
| [#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35) | `generator::lfo` read the shape byte and ignored the four other parameters that shape an LFO | 26.5. Spent in full: the slew rate is in the shape the library hands back, so corners round and a square becomes a ramp between its levels; `Generator::rest` is where an LFO sits rather than an assumed middle; and `lfo_fade` is the `Delay / Fade` byte, which was a fader that moved nothing on the glass and is now a dotted line the wave comes in under |
| [#36](https://github.com/MysteriousWolf/deepmind-midi/issues/36) | The marks along a generator's horizontal | 26.5, as `Generator::marks` and `Generator::anchored`. Spent: the envelope plates write `A`, `D`, `S` and `R` where the library says each segment begins, a filter's corner and a pulse's falling edge are marks rather than arithmetic, and a free-running LFO's left edge is left unruled |
| [#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37) | The high-pass response, and what the oscillators are making | 26.5. **The last transcribed number is deleted.** `high_pass_response` is the curve, on the low-pass's own vertical and span, so the two plates of one section agree because the library says so. `oscillator` sums `OSC 1`'s two waves at a weight it states is its own reading, and `noise` retires the scatter table |
| [#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39) | The destination join read backwards, and which of several destinations is narrowest | 26.5, as `ValueTable::values_naming`. Spent: `Mapping::names` is one call and a `next`. The walk was fine here; the ranking was a judgement about the instrument being made in a window, and it is on the library's side now, down to what happens when two destinations move the same number of parameters (value order, and the library says outright that nothing makes one narrower) |
| [#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40) | A dot-matrix cell for every modulation source and destination | 26.5, as `ValueTable::cell_of`, and it took `ParamId::glyph` with it. A source's cell is the library's drawing of it and a destination's picture is the glyph of the parameter it moves, so the patch bay's two columns of abbreviations are two columns of pictures. What is left undrawn is [#43](https://github.com/MysteriousWolf/deepmind-midi/issues/43) |
| [#41](https://github.com/MysteriousWolf/deepmind-midi/issues/41) | Which way a modulation source swings a control it reaches | 26.5, as `ValueTable::swing_of` and `Swing`. Spent: the band on a control runs either side of where it sits for a `Centred` source and from it for a `Rising` one. Taking the depth's sign as the whole answer, as this window did, is right for a wheel and an envelope and wrong for an LFO |
| [#45](https://github.com/MysteriousWolf/deepmind-midi/issues/45) | What the front panel looks like, beyond which controls are on it | 26.5.2, as four accessors. **The last transcription in this repository is deleted.** `PanelControl::silkscreen` returns `Word` or `Drawing(Glyph)`, which is the half a `&'static str` could not say — the two presses that choose `OSC 1`'s mix are printed as the waves themselves; `PanelControl::cluster` numbers the clusters a printed rule divides a plate into, so `ruled` in `home.rs` is a walk watching the number change; `PanelControl::lamp` is the instrument's own three-colour rule, and `Arp Hold` being the one cyan program parameter is now the library's assertion rather than this window's; `Section::banner` is the colour a plate's name is printed on and the ink it is knocked out of, measured off the product photographs. The three missing presses arrived in the table itself, so `PANEL_CONTROL_COUNT` is 35 |
| [#46](https://github.com/MysteriousWolf/deepmind-midi/issues/46) | The front-panel presses that are not program parameters | 26.5.2, as `Section::presses` and `PanelPress`, filed from here and answered in the same release as #45. `CHORD` and `POLY CHORD` are drawn on the arpeggiator's plate now, as caps that do not go down: `Sends::Nothing` is the library saying that no controller number and no message in the manual presses one, which is what makes an inert cap honest rather than broken. Each carries the note the library records — the dump request that reads the memory it plays from — in the footer while the pointer is on it |

## What arrived unasked

26.5 published two things this repository had not filed an issue for, and both
are spent:

- **A glyph per parameter and per effect slot.** `ParamId::glyph`,
  `FxSlot::glyph` and `Controller::glyph`: a picture of what a control does, on
  the same 7x7 grid as the marks and the cells, and the same picture wherever
  the same job is met. The footer puts it before a parameter's name and every
  effect slot wears its own, so a decay reads as a tail and a mix as wet against
  dry before the word is read. 177 of the 242 parameters carry one; the rest are
  the effect slots, which have their own, and the seventeen characters of the
  program's name, which are letters and not a control.
- **What kind of thing an effect is.** `Algorithm::characters`: vintage,
  modelled, stereo, dual, multiband, two in one, lo-fi, modulated, dynamic. Read
  rather than derived, with a reason per membership in the library's own file:
  the manual's name for the effect, the slots it gives it, or the unit a name
  refers to. A window matching on `Vintage` in a `full_name` would be right
  until the day it was not. They are a caption beside an engine's name, and
  empty for the plain reverbs and the noise gate.

## What an answer costs here

A minor version bump in `Cargo.toml`'s `[workspace.dependencies]`, which is a
deliberate edit rather than a `cargo update`: a value table that changes is a
change to what the window draws, and it arrives when somebody moves the pin.
The panel that was waiting then stops apologising, and the caveat it was
printing comes out with it.

## Asking for something new

File an issue there rather than a table here. What has worked in every row
above: what is missing and where it already exists in `spec/`, why this editor
cannot do its job without it, the accessor that would answer it, and what the
library should *not* do. Half of what is useful about this library is what it
refuses to guess, and an issue that does not say where that line falls invites
it to be crossed.

Ask for getters rather than for the file. The specification's shape is the
library's business and changing it should not reach a host. What a host needs is
a method it can call and a type it can hold, which is what every answered issue
above turned into.

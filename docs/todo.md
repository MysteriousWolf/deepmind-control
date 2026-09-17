# To do

Decisions this repository owes itself. Not a backlog of everything unfinished —
[the plan](plan.md) has the order of the work and [waiting](waiting.md) has what
has been asked of the library. This is the shorter and more uncomfortable list:
things a change here has left open, where the change shipped anyway and
somebody has to say what happens next.

**Check this file when a surface changes.** Every row is a hole somebody can
fall into today.

## Open

### The bar said something no one sheet can

Each of the fourteen tabs carried its own section's claim dot, so a bar of green
dots with one copper one among them read as "the sound is the synthesizer's,
except the part I moved" without anything being opened. One sheet at a time
cannot say that. A sheet carries the claim of what is on it, beside its name,
and the front panel carries a claim per control, but nothing now says it about
the sections that are not on the screen.

Whether that is worth replacing, and where — the plate headings, the window's
own display, the footer — is open.

## Answered

### Four sections have no way in

**Answered by a band of ways in over the rack.** The panel is drawn from the
library's table of what the instrument puts a *fader* under, so the four
sections with no fader anywhere had no plate to carry an `EDIT`: the modulation
matrix, the effects, the control sequencer and the program's own settings. They
were reachable only by the code that asked for them directly.

The panel now opens with a band of four caps, one per section, each carrying the
section's mark and its name in the display's own dots. `control_ui::unplated` is
that list and it is *subtracted* rather than written down: it is every section
the library has, less every section a plate opens, so a fifteenth arriving in a
later firmware gets a way in without anybody noticing it had to. Which is also
what `the_last_band_carries_exactly_what_no_plate_does` holds, beside
`every_section_the_instrument_has_has_a_way_in`.

**Over the rack and not under it**, which is the one thing about the layout that
was not a free choice. The panel is three racks deep and a window opens on
roughly one of them, so a band under the panel is a band below the fold — and a
press that has to be scrolled to is the problem the band was added to solve.

The other options in this row's first draft, kept because they are what somebody
would reach for next if this one ever stops working: putting them on the
instrument's own buttons, which would be an ask of `deepmind-midi` because
`front::Section` carries controls and not presses; putting them in the window's
chrome beside the port; and fixing only the matrix and the effects, which are
what this editor is for, and leaving the other two.


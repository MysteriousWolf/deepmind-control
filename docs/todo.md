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
cannot say that. A sheet is now titled the way the band titles a tab — the
section's mark and its word in the display's own dots — and the claim it used to
carry beside that name is carried by the display underneath it instead, which is
drawn in the claim the way every display here is. The front panel carries a
claim per control. Nothing says it about the sections that are not on the
screen.

Whether that is worth replacing, and where — the plate headings, the window's
own display, the footer — is open. The band of ways in is a row of caps again
and could carry a dot per cap, which would say it about four of the fourteen:
the four it opens. That is a quarter of the answer and it is the quarter about
the sections a player cannot see the state of anywhere else, so it is the first
thing to try and it is still not the thing the bar did.

### A catalogue carries a digest nothing checks

`Entry::sha256` is read out of an index and never compared against anything,
because nothing in this repository fetches a file and nothing in it hashes one.
That is the right order to build it in — a digest is worth having the day a pack
arrives over a wire, not the day it is read off a disk somebody already trusts —
and it leaves a field in a format that says more than the reader does.

What is open is only *when*: the first reader that downloads a pack has to check
it, refuse what does not match, and say which pack lied, in the same change that
first opens a socket. Until then, anybody generating an index should fill the
field, because an index written today is one a verifying reader can check
tomorrow, and a field that was left blank for a year is a field that never gets
filled in.

Nothing in the window says any of this, because nothing in the window draws a
catalogue yet. The day one does, it has to: a pack drawn as verified when it was
merely read is the same lie as a value drawn as the synthesizer's when it was
this window's claim. See [presets.md](presets.md).

## Answered

### The front panel had presses no program parameter stands behind

**Answered by deepmind-midi 26.5.2, filed from here as
[#46](https://github.com/MysteriousWolf/deepmind-midi/issues/46).** The
arpeggiator on a `DeepMind` has `CHORD` and `POLY CHORD` beside its `ON/OFF` and
`HOLD`, and this window drew neither: a sweep over all 242 `ParamId` finds one
arpeggiator switch, `Arp Hold`, so there was no byte for a cap to move and
`front::PanelControl`, keyed by parameter, could not describe the press either.

`Section::presses` is the other kind of entry, and `PanelPress::sends` is the
fact that made it worth drawing: `Sends::Nothing` says outright that no
controller number and no message in the manual presses one. Both are on the
plate now, as caps that do not go down, with their mark at half ink and the
library's own note in the footer. A panel that left them out was a drawing of
the front with two buttons missing; one that drew them live would have been a
lie about what a cable can do.

The rest of that class stays out for the reason it always did: `DATA ENTRY`, the
encoder, `WRITE`, `COMPARE` and the `EDIT` row are presses whose whole function
is the instrument's own display, and a program drawing this panel *is* the
display.

### Four sections have no way in

**Answered by a band of ways in above the surface.** The panel is drawn from the
library's table of what the instrument puts a *fader* under, so the four
sections with no fader anywhere had no plate to carry an `EDIT`: the modulation
matrix, the effects, the control sequencer and the program's own settings. They
were reachable only by the code that asked for them directly.

The window now carries a band of caps above the surface, one per section, each
carrying the section's mark and its name in the display's own dots, with a fifth
in front of them for the front panel itself. `control_ui::unplated` is that list
and it is *subtracted* rather than written down: it is every section the library
has, less every section a plate opens, so a fifteenth arriving in a later
firmware gets a way in without anybody noticing it had to. Which is also what
`the_last_band_carries_exactly_what_no_plate_does` holds, beside
`every_section_the_instrument_has_has_a_way_in`.

**Above the surface and not on the panel**, which is the one thing about the
layout that was not a free choice. It was drawn at the foot of the panel first,
which put it about 500pt below the fold of a window opened on a panel three
racks deep; then at the head of the panel, which was reachable but inside the
scroll and, worse, underneath the shade a sheet casts. A tab that cannot be
pressed while a sheet is open is not a tab: pressing it landed on the shade and
shut what was up. So the band is chrome now, drawn between the surface switch
and the surface, and the shade covers the surface alone.

The other options in this row's first draft, kept because they are what somebody
would reach for next if this one ever stops working: putting them on the
instrument's own buttons, which would be an ask of `deepmind-midi` because
`front::Section` carries controls and not presses; putting them in the window's
chrome beside the port; and fixing only the matrix and the effects, which are
what this editor is for, and leaving the other two.


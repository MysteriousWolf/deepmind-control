# To do

Decisions this repository owes itself. Not a backlog of everything unfinished —
[the plan](plan.md) has the order of the work and [waiting](waiting.md) has what
has been asked of the library. This is the shorter and more uncomfortable list:
things a change here has left open, where the change shipped anyway and
somebody has to say what happens next.

**Check this file when a surface changes.** Every row is a hole somebody can
fall into today.

## Open

### The front panel has presses no program parameter stands behind

The arpeggiator on a `DeepMind` has a `CHORD` press and a `POLY CHORD` press
beside its `ON/OFF` and `HOLD`, and this window draws neither. It is not an
omission that can be fixed here: a sweep of all 242 `ParamId`s turns up exactly
one arpeggiator switch, `Arp Hold`. The other two are performance functions —
they latch what the keyboard is doing rather than set a byte in the program — so
there is nothing in the edit buffer for a cap to move, and `front::PanelControl`
is keyed by parameter, so the library has no way to describe the press either.

The same is true of the data entry group and the `EDIT`/`COMPARE`/`WRITE` row,
which this window has its own answers for.

Asked for as
[deepmind-midi#46](https://github.com/MysteriousWolf/deepmind-midi/issues/46): a
description of the front-panel presses that are *not* parameters, with what each
one sends, and "nothing over the wire" named outright where that is the answer.
Until then the window draws what the program holds, which is the honest subset,
and a player reaching for `CHORD` finds nothing.

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

## Answered

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


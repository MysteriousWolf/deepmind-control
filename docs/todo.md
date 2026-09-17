# To do

Decisions this repository owes itself. Not a backlog of everything unfinished —
[the plan](plan.md) has the order of the work and [waiting](waiting.md) has what
has been asked of the library. This is the shorter and more uncomfortable list:
things a change here has left open, where the change shipped anyway and
somebody has to say what happens next.

**Check this file when a surface changes.** Every row is a hole somebody can
fall into today.

## Open

### Four sections have no way in

**The front panel's `EDIT` presses reach ten of the fourteen sections. These
four are not among them:**

| | What is behind it |
| --- | --- |
| **Mod Matrix** | The eight routings, their sources, destinations and depths, and the patch bay drawn from them. The one surface this editor has that the instrument's own display cannot show at all |
| **Effects** | All four engines, the algorithm each is running, its twelve parameters, and the chain they are wired into |
| **Control Sequencer** | The 32 steps. The panel's `ARP/SEQ` plate carries the arpeggiator's own `EDIT` and nothing opens the sequencer beside it |
| **Program** | The name, the category, and the settings that are about the program rather than the sound |

**Why.** A section opens as a modal now, and the press that opens it is the
`EDIT` on a plate of the front panel. The panel is drawn from `front::sections`,
which is the library's table of what the instrument puts a *fader* under, so a
section with no physical control has no plate to carry an `EDIT`. It did not
matter while there was also a bar of fourteen tabs above the rack; the bar is
gone with the surface it sat on, and these four went with it.

Nothing is lost from the *sound*: every one of those parameters is still in the
patch, still sent, still read back, and still drawn the moment its section is
opened. What is missing is the opening. `control_ui::ways_in` is the list the
panel can reach and
`the_front_panel::the_sections_with_no_way_in_are_the_four_that_are_written_down`
is where these four are pinned, so the gap cannot widen or quietly close without
a test saying so.

**What to decide.** Some ways this could go, none of them chosen:

- **A row of plates for the sections the hardware has no fader for.** The panel
  already draws plates the instrument does not — three envelopes where it has
  one set of faders, two oscillators where it prints one — so a plate with a
  display, no faders and an `EDIT` is the same hand layout again. It is the only
  option that keeps the front panel as the one way in.
- **Put them on the instrument's own presses.** A `DeepMind` reaches its
  effects, its matrix and its sequencer from buttons on the panel, not from
  faders. Which buttons, and whether the library publishes them, is the
  question; `front::Section` currently carries controls and not presses, so this
  may be an ask of `deepmind-midi` rather than a layout here.
- **A way in that is not a plate.** The window's own chrome, along the header or
  the foot, the way the port and the inquiry are reached.
- **Leave three of them and fix one.** The matrix and the effects are what this
  editor is *for*; the program's own settings and the sequencer are less often
  reached for.

Until one of those is decided, the four are reachable only by the code that asks
for them directly, which is `docs/previews/` and the tests.

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

Nothing yet.

# Interface

What the editor looks like, why, and the numbers to build it from. The views
themselves are stages 2, 3 and 5 of [the plan](plan.md); this is what they are
written against.

## It is a panel, not a window

The thing being edited has a front. It is a dark panel between wooden cheeks
with a row of faders on it, and every drawing this project already owns says so:
the mark in `docs/logo.svg`, and the 35 effect panels the library generates from
its specification. The editor is a continuation of those, not an application
that happens to control a synthesizer.

That decides more than it looks like it does. **No general-purpose widget
appears anywhere a parameter is edited.** A toolkit slider is a different object
from a fader on an instrument: different proportions, different hit target,
different reading distance, and no scale. Editors that mix the two end up
looking like spreadsheets with a picture of a synthesizer at the top. Every
control here is drawn by this crate.

Toolkit widgets are correct everywhere else: the port picker, the file dialogs,
the librarian's list. The panel is for parameters.

What the rule is about is the anatomy, not the crate a widget came from. A
selector too long for legends is a list, and a name is a field of text, because
that is what those controls are on an instrument too; what neither of them is is
a slider standing in for a fader.

## The vocabulary is the library's

`deepmind-midi` draws all 35 effect panels from `spec/`, and those drawings
already settle what a control on this instrument looks like. The editor inherits
the anatomy rather than inventing a second one:

| | |
| --- | --- |
| Abbreviation above | Mono, as the synthesizer's own display writes it: `DCY`, `PDY`, `HiSvFreq` |
| Control in the middle | Fader, knob, switch, selector or readout |
| Title below | The same parameter written out, for a panel with room to be readable |
| Modulation dot | A small mark at the top right of a slot the modulation matrix reaches, read off the eight destinations the patch holds. `ValueTable::parameters_of`, from `deepmind-midi` 26.2, is what joins `VCF Freq` to the parameter it moves ([deepmind-midi#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19)); a destination this window has not read moves nothing, because a mark drawn from an unread value says the instrument is doing something it may not be |
| Column pitch | One slot per column, filled left to right, wrapping onto a second row |

Two of those are worth keeping even though a bigger screen does not need them.
The abbreviation is what is printed on the instrument, so a player looking
between the two reads the same word twice. The dot is the only thing on the
panel that says a parameter can be moved by something other than a hand.

## The front panel is the home screen

**The window opens on the instrument, not on a section of it.** A `DeepMind` is
two rows of section plates with a screen between them: twenty-odd faders under
four-letter legends, a few lit buttons under each group, and on every group a
yellow `EDIT` that opens that group on the display. Everything else — 242
parameters' worth — is behind one of those presses.

So is this. The panel is where a window opens, because it is where a player
looks first, and the rack of a section is one press behind it exactly as it is
on the hardware.

```
┌ ARP / SEQ ┐ ┌── LFO 1 ──┐ ┌── LFO 2 ──┐ ┌─────────────┐ ┌ POLY ┐
│ ┌───────┐ │ │ ┌───────┐ │ │ ┌───────┐ │ │▛PROGRAM Pad▜│ │┌────┐│
│ │▔╷ ▔╷ ▔│ │ │ │╭─╮ ╭─╮│ │ │ │╶╴ ┌─┐ │ │ │             │ ││ ╷╷╷││
│ └───────┘ │ │ │╯ ╰─╯ ╰│ │ │ │  ─┘ └─│ │ │ Modular Fun │ │└────┘│
│  ▮     ▮  │ │ └───────┘ │ │ └───────┘ │ │ ▁▁▁▁▁▁▁▁▁▁▁ │ │  ▮   │
│ RATE GATE │ │ ▮  ▮ ○Sine│ │ ▮  ▮ ○Sine│ │ this claim  │ │DETUNE│
│ [on][off] │ │      ●Tri │ │      ●Ramp│ │ ▓▓▓▓▓░░░ 84 │ │      │
│      EDIT │ │      EDIT │ │      EDIT │ │ Read it.    │ │MOD FX│
└───────────┘ └───────────┘ └───────────┘ └─────────────┘ └──────┘
┌─── DCO 1 & 2 ───┐ ┌─── VCF ───┐ ┌VCA┐ ┌HPF┐ ┌ ENVELOPES ┐
│ ▮ ▮ ▮ ▮ ▮ ▮ ▮   │ │ ▮ ▮ ▮ ▮ ▮ │ │ ▮ │ │ ▮ │ │ ▮ ▮ ▮ ▮   │
│      [on]  EDIT │ │ [2 Pole]  │ │   │ │   │ │ A D S R   │
└─────────────────┘ └───────────┘ └───┘ └───┘ └───────────┘
```

- **The screen is the application's, and the panel leaves a hole for it.** What a
  display says is which sound is on it, what backs that, and what last happened
  — none of which the view layer knows, and all of which a plugin answers
  differently from a desktop window.
- **Every plate has a display too, and the instrument has one.** This is the one
  place the panel deliberately stops being the instrument, and it is the one
  place where a window has something the hardware does not: room. A `DeepMind`
  shows whichever section was pressed last, because there is space on its front
  for a screen and twenty faders; here each plate carries the drawing of its own
  part, and the envelopes carry the drawing no `DeepMind` can show — all three of
  them at once. See [Display](#display).
- **A legend is printed over a fader and under a button**, because that is where
  the instrument prints each: the hardware has a screen for readings and no room
  under a fader, and it silkscreens `POLY` and `EDIT` *below* the caps they name,
  where a finger on the button cannot cover them.
- **Every plate carries the press the hardware calls `EDIT`**, and the
  envelopes' `VCA`, `VCF` and `MOD` are the three ways into the three envelope
  panels, where the hardware uses them to choose which envelope its four faders
  address.
- **A way in is a legend and a lamp, not a word in a box.** `EDIT` is not
  written on the button on the instrument: it is silkscreened on the panel under
  a blank rubber cap that is lit amber the whole time the synthesizer is
  powered, and a row of those along the foot of every plate is the thing you see
  first in a photograph of one. So the legend is printed where the panel prints
  it and what is pressed is the lamp, lit at rest and brighter under the
  pointer.
- **A button is moulded, and the same cap wherever it is.** Square-ish, as wide
  as a fader and a little over half that tall, cut to a round corner, its face a
  gradient lit across the crown and shadowed at the foot — which is what a soft
  thing standing proud of a dark panel looks like. Pressing turns that gradient
  over rather than reaching for a second colour. An unlit button is moulded too:
  it is still a rubber cap when nothing is behind it. The band along the foot of
  a plate is one band, so the `EDIT` press and the switch beside it are the same
  cap at the same size, and that band is the one part of the panel the window
  does not stretch.
- **The plate's name is knocked out of a light bar**, which is how the
  instrument prints `ARP / SEQ`, `VCF` and `ENVELOPES`: a pale strip across the
  top of each group with the name dark on it. It is what the eye follows across
  the panel before it reads a single legend.
- **The panel fills the window it is in.** Every dimension here is written at
  the instrument's own proportions and then drawn through one scale, measured
  from the widest row against the room there actually is — the lanes, the travel
  of a fader, the buttons, the type and *the gaps between and inside the plates*.
  Scaling the gaps is the half that is easy to forget and the half that decides
  whether it reads as an instrument or as a panel with its parts pushed apart.
  It stops at 1.75, because a fader as long as an arm is not an improvement, and
  it never goes below 1: a narrow window wraps its rows, which is an arrangement
  somebody can still read, where shrunken type is not. The displays are the one
  thing that does not simply grow — a wider window is a filter curve drawn more
  finely, not a magnified one. Past 1.75 the panel stands in the middle of the
  window rather than against its left edge, because that is where an instrument
  left on a desk that wide would be.
- **Every row fills it, and not just the widest one.** The top row is the one
  the panel is measured from; the signal path is a plate narrower and the
  envelopes are two plates narrower, and drawn at what they measure they leave
  that difference as bare panel at the right hand end — a photograph of a
  synthesizer with the end sawn off. A row of a front panel runs the whole width
  of the instrument, so the difference is shared out among that row's plates in
  proportion to what each already holds. Every plate of a row grows by the same
  fraction of itself, so `VCF`, which is five faders, stays twice the width of
  `OSC 1`, which is two; and nothing inside any of them moves, because what the
  extra room buys is display — a plate given more glass gains dots rather than
  magnifying the ones it has. The screen is the exception it has always been: a
  written width, not what is left over. A row a narrow window has had to break
  is the one arrangement that is not drawn out, because a line of a broken row
  is a fraction of a row, and a fraction of a row filling the panel is `POLY`,
  which is one fader, drawn as wide as the window.
- **Every plate stands the same height, and so does the screen between them.**
  A panel whose plates were each as tall as their contents happened to be has a
  ragged edge under every row and its `EDIT` presses at five different heights,
  which is the one thing a front panel never is. The height is added up from the
  parts rather than written down beside them, because two of the parts do not
  grow with the rest: a display gains dots instead of getting bigger, and the
  buttons along the foot are drawn in the room this editor gives a control that
  is not a fader, which is the rack's room and not the panel's. The `EDIT` press
  stands in the middle of that band, where the switches beside it are.
- **A lit set is given room for all of it.** A column of legends is laid out
  into the room it is given and the ones past the end of that room are drawn no
  lines tall, which is how the panel came to name five of the instrument's seven
  LFO shapes with nothing saying that `Sample & Hold` and `Sample & Glide` were
  missing. The strip is a whole lane tall now — the fader's travel, the gap
  under it and the reading it would have had — and as wide as the longest name
  it lights, and a test fails if a later table names something that does not
  fit.
- **The arrangement is the instrument's, and now so is the livery.** The
  hardware's buttons are white, amber and cyan, and this window takes the amber
  and the cyan for the two jobs it has that need a colour: amber opens a
  section, which is what the hardware's `EDIT` is, and cyan marks a control
  something other than a hand can move, which is what the hardware's `MOD` is.
  Earlier this file argued the opposite — that a yellow `EDIT` would be a fourth
  meaning to learn beside the copper and the green. Taking the instrument's own
  pairing turned out to be the way through it: the claim moved onto the glass as
  a depth of ink and off the panel's lamps entirely, so there is no fourth
  meaning, only the instrument's own two.
- **The row of twelve lamps over `POLY` is not drawn.** It says how many voices
  are sounding, and nothing on a MIDI port says that. A lamp that cannot be lit
  honestly is not drawn at all.
- **Nothing on the panel is transcribed any more.** Which parameters have a
  fader, what is silkscreened over them and which row they are in was the one
  table this repository kept; `deepmind-midi` 26.3 publishes it
  ([#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)) and the
  table is deleted. What is left in `home.rs` is layout, which is this window's
  to decide.
- **Two sections are drawn as more than one plate**, and both are the same
  trade as the screens: a window has room the front of a synthesizer does not.
  `OSC 1` and `OSC 2` are the brackets the instrument prints inside its own
  `DCO 1 & 2` plate, promoted to a plate each. The envelopes become one plate
  each with their own four faders and their own screen, because the hardware
  has four envelope faders and three envelopes and a button pointing one set at
  the other — and they get a row of their own, which is what the instrument had
  no room to give them. Both splits are derived through the library: an
  oscillator's bracket is a slice of its parameters' own names, and an
  envelope's four faders are the parameters whose short names match the four
  the section carries.

## Fourteen panels, one press behind it

Two hundred and forty-two parameters do not fit on a screen, and the instrument
does not put them on one surface either: a player presses a section and the
display becomes that section. So does this. A bar of fourteen tabs sits above
the rack, it does not scroll with it, and the panel below it is the one section.

The bar and the front panel's `EDIT` are the same press: one asks for a section
and the window shows it, whichever surface asked. The way back is named after
the section it holds rather than "Editor", so that somebody who pressed `VCF`
can see where they would be returning to.

**The order of the tabs is not written down anywhere.** A parameter's offset is
its NRPN number and its place in a dump, the library's parameter table is in
offset order, and so the order the groups first appear in that table is the
order the instrument itself keeps them in: LFOs, oscillators, filter, the
envelopes and the VCA, voicing, modulation, sequencing, effects, and the
program's own settings last. Reading it off the table is both more honest than
an order invented here and one less thing to edit when the library grows a
group; what it is not is alphabetical, which is how the library hands the groups
over and which puts the effects third and the oscillators eighth.

**Each tab carries its own section's claim**, as the same dot the legend
explains, and a section is as confirmed as its least confirmed parameter. That
is what keeps one panel at a time from hiding the thing this editor is for: a
bar of green dots with one copper one among them reads as "the sound is the
synthesizer's, except the part I moved" without opening anything.

Which section somebody is looking at is this window's business and never the
synthesizer's. It outlives a port being put down, because the sound went away
and the person did not.

## Three surfaces, and the sound survives the switch

The panel is the instrument. The editor is one section of it. The library is the
sounds somebody keeps. They are one application looking at three things, so they
are three surfaces of one window and not three windows: a switch above them, and
everything below it changes. The patch does not. Putting a pack down to look at a
filter and finding the filter gone is the wrong thing to teach anybody about an
editor.

**The shelf is a grid and not a list.** A pack is 128 programs, and the one
thing this surface can offer that the instrument's own two-line display cannot
is all of them at once: four across at the window's opening width, in slot
order, with the slot written the way the front panel writes it and the name
beside it. A column of 128 rows would be a list of the same length that showed a
quarter as much of it.

**A program is drawn like a section tab**, because it is the same idea: the one
that has been pressed is the face plate a rack sits on, lit along its edge, and
the rest are the panel they are cut into. One window, one drawing of "this one".

**A slot that names nothing is drawn as naming nothing.** A stored dump carries
its bank and program; an edit buffer dump carries neither, because the edit
buffer is where a sound is played rather than where one is kept. That program
gets an em dash where the others get `A19`, which is both the truth and how a
patch this application saved reads back in.

**Loading one is a claim.** Every value on the screen goes copper at once: the
host sends the difference, and nothing has heard the instrument play any of it.
Reading the edit buffer back is what turns the panel green, exactly as it is
after a fader is dragged.

## Confidence is a fill, not a colour

A claim per parameter, which is more than the library tracks: the library's
claim is about the whole sound, and one dragged fader leaves the other 241
values exactly as confirmed as they were. `Patch` keeps the 242, and the three
states are drawn as three different objects:

| | | |
| --- | --- | --- |
| **Unknown** | The track, its scale, and no cap at all | Nothing has been read back. The control is inert: an edit before anything is known is refused by the host crate, so a control that cannot be moved must not look like one that can |
| **Assumed** | The cap in outline: the metal as a stroke around the panel colour | The host put the value there. Nothing has contradicted it, and nothing has confirmed it |
| **Confirmed** | The cap filled with metal | The synthesizer reported it |

**The difference is never carried by hue.** Outline against fill survives
greyscale, a projector, a screenshot in a bug report, and the readers who would
not see the pair. Colour reinforces it: copper for a claim, green for a fact,
the panel's own grey for a sound nobody has read. It never does the work alone.

The same three drawings apply to every control kind. A switch is unlit,
outlined-lit, or lit; a readout is blank, outlined, or solid.

Reading the edit buffer back is what turns a panel of outlines into a panel of
fills, which is the most useful thing a glance at this application can tell
somebody: whether they are looking at the sound or at their intentions.

**So the window reads it without being asked.** The moment a synthesizer answers
the inquiry, the editor asks for the sound it is making: one message and one
dump on a port that has just proved it works. Until that lands every control on
the panel is the editor's arithmetic about an instrument sitting right there
with the answer, and that is not a state worth keeping a score of.

Which is why there is no longer a score. The window used to carry a key reading
*reported · claimed · unread* and a bar counting how many of the 242 values were
the synthesizer's own account. Both were bookkeeping about a gap the window
should be closing rather than measuring: a reader who has to consult a key to
know what a control means is being asked to do the drawing's job. The three
drawings stay, because a dragged fader is still a claim until a dump agrees with
it, and they now have to be legible without a legend beside them — which is the
bar they should have been held to from the start.

## Controls

Colours live in `control-ui/src/style.rs`, which is the only file in this
repository that writes down one. A control's geometry lives with the widget that
draws it, because it is that widget's anatomy and nothing else reads it.

### Fader

The default for anything continuous, and the control the instrument itself is
covered in.

```
         39          the parameter's address, mono, dim
   |     |     |
   -  [=====]  -     cap 38 x 15, radius 2.5, indicator 2 tall
   |     |     |     scale ticks in pairs, 8 long, at 0 25 50 75 100%
   |     |     |     track 7 wide, 128 tall, rounded to its width
   |     |     |
        198          the raw value, mono
     Frequency       the name, less the group's own
```

- The slot is 88 wide; the fader in it is 44 by 128.
- The track is a recess: dark, with the lit lower wall the mark uses on its
  slots, so a fader at the bottom of its travel still reads as a fader.
- The cap is the only metal in the slot, which is what makes a panel of them
  scannable: the caps are the data and everything else is the instrument.
- Travel is the track minus the cap, so both ends of the range put the cap flush
  with the ends of the track. A control whose extremes are unreachable is a bug
  people file.
- **A press does not jump.** A toolkit slider moves its value to wherever the
  pointer landed; nothing on an instrument teleports, and a stray click that
  throws `VCF Frequency` from 20 to 200 is audible, sent, and not undone by
  letting go. Pressing takes hold of the cap where it is, and the drag is
  relative from there.
- **It runs down the panel, and across it where a row needs it to.** Down
  everywhere the rack draws one, which is the way every fader on the instrument
  runs. A panel laid out by hand as rows turns the same fader onto its side and
  changes nothing else about it: the same recessed track, the same scale in
  pairs, the same cap, the same relative grab, and right is more where up was.
  It is the arrangement that turned, not the control.
- The address above each slot is the parameter's NRPN number, which is also its
  byte offset in a dump: one number is both its name on the wire and where it
  lives in memory.

### Knob

Where the source uses one, which means the effect panels, so that half of the
editor keeps looking like the figures it was drawn from. A body seated in a
recess, a pointer from the middle out to the rim, and a five-mark scale around
the outside. The turn is three quarters of a circle, open at the bottom: nothing
is at the bottom left, the top of the range is at the bottom right, and the
middle points straight up, which is where a hardware knob's own ends are.

- **It is the fader, turned.** The same byte, the same range, the same relative
  grab so a press never jumps, the same shift for a fine drag and the same
  wheel, and the same claim in the fill of the thing that moves. A whole drag is
  a rack fader's travel, so a hand moving between the two controls of one panel
  does not have to learn a second rate.
- **A drag is up and down whatever the shape is.** Turning a knob by dragging
  round it is a gesture nobody performs twice.
- **The sweep is a shape and never a reading.** The instrument publishes no
  angle for a parameter. What the byte is worth is the reading under it, the
  same as under every fader in this window.
- **Which slots get one is the library's.** 26.3 publishes what the figure
  printed beside each algorithm is made of: 29 of the 35 are rotary knobs, five
  are vertical faders and one is a numeric display. The display is drawn as a
  fader — one shape for one algorithm, invented here, would be a worse lie than
  the fader that is already honest about the byte underneath.

Knobs are not an alternative to faders for the main editor. Two ways to draw the
same kind of parameter is how a panel stops being readable; the effects are the
one place the source draws something else, and they follow it.

### Switch

Two states. A lamp and a legend, not a checkbox and not a toggle that slides.
Off is a dark cap; on is the lamp colour. Both are moulded, because an unlit
button on the instrument is still a rubber cap and drawing that one as a hole
and the lit one as a light would be two controls wearing one name. The lamp is
the one saturated thing this design allows, and it only ever means *on*.

### Selector

One of a named set, and the value's name is the control: the library hands over
the label, so the control shows `Ramp Up` rather than `3`. Six values or fewer
it is a column of legends with one lit; beyond that a list, because the
modulation matrix has 130 destinations and a column of 130 legends is a joke at
the reader's expense.

A table that does not name every value the parameter accepts is not used at all:
the fader stays, and the number with it. A control that silently drops the
values it has no name for is a control that moves the sound when somebody opens
it.

### Readout

For a value with no useful continuum, and for the raw number beside any control
while it is being dragged. Mono, tabular figures, so the digits do not jiggle
while a value runs through them.

### Name

The one control that covers more than one parameter, because the one value the
instrument stores in more than one. A name is seventeen parameters holding a
character each — sixteen characters and a terminator — and seventeen faders
sweeping 0 to 127 is a panel nobody can name a sound on: `Program Name Char 4`
reads `115`, and what that means is that the fourth letter is an `s`.

So the seventeen slots are drawn as the one display the instrument shows them
on. Mono, sixteen characters wide, cut into the plate as the same recess a
fader's track is. It keeps the slot anatomy the rest of the rack has, with the
address above it reading `223–239` because that is the run it occupies, and the
reading below it counting the characters used out of the sixteen.

- **It sits where its first character sits.** The rack is in the table's own
  offset order, and the field takes the place of the seventeen slots rather
  than being lifted to the front of the panel.
- **The claim is the weakest of the seventeen.** A display that called itself
  the synthesizer's because sixteen of its characters were is the one lie this
  editor exists to avoid. Confidence has no moving part to fill here, so it is
  carried by the colour of the letters, which is the one control where it is.
- **A character the name cannot hold never appears.** Seventeen letters, or a
  `è` the display has no glyph for, leave the field exactly as it was rather
  than appearing and then being taken back.
- **A keystroke costs the parameter it moved.** The word becomes edits by
  writing it into a copy of the program and taking the difference, so typing a
  letter onto the end of a name is one NRPN and not seventeen. Inserting one in
  the middle shifts what follows it, and costs what it shifts.
- The field is inert until a sound has been read, like every other control.

### Envelope

Four faders and the shape they make, drawn above them. An envelope is the one
group whose meaning is a picture, and four numbers that do not draw it are four
numbers.

**All three at once, as three plates.** A `DeepMind` has three envelopes, one
display and three buttons choosing which of them the four faders address, so
comparing the filter's decay with the amplifier's means comparing one with a
memory of the other. All three are on screen here, which is the drawing no
hardware can make — and each is a plate with its own four faders and its own
full screen rather than a third of one shared drawing. That went through two
answers: three curves sharing a band told apart by a dash pattern (on a grid of
dots, two lines crossing are the same dots), then three named panes of one
strip, and now three plates. Each was the best available given how much room
the layout had, and unfolding the section is what finally gave them room.

### Cap

A moulded rubber button, and **nothing is written on it**. A `DeepMind`'s panel
is blank caps pushed through holes in the metal and lit from behind, with what
each one is called silkscreened beside it — so the legend is drawn where the
panel draws it and the cap carries only what a lamp can say by being lit.

That leaves the cap to carry the claim, which is where it belonged: an unread
switch is the hole with no cap in it, flat and recessed, exactly as a fader
nobody has read is a track with nothing to take hold of. The word `off` printed
inside an unlit cap used to do that job, which is the one thing those words were
really for.

### Footer

A strip along the foot of the window saying what the pointer is over. A panel is
twenty-odd faders under four-letter legends and a rack is forty slots under
abbreviated ones, and both are readable only because a hand can ask what one of
them is: `KYBD` is `VCF Keyboard Tracking`, it takes `0` to `255`, and
controller 74 drives it — none of which fits over a lane and all of which fits
along the bottom of a window.

- **Every word of it is the library's answer.** The name, the section, the value's
  own name, the range and the controller are five questions put to
  `deepmind-midi`. The footer writes down nothing about a parameter, which is
  the rule the controls themselves are drawn under.
- **The claim is in words here, not in a colour.** A footer is a sentence, and a
  sentence that said what backs a value by being a different colour would be
  saying it only to the readers who see the colour.
- **The range is in raw bytes**, never the number the synthesizer's display
  shows, because the manual prints the two ends of a range and almost never the
  curve between them. See [#25](https://github.com/MysteriousWolf/deepmind-midi/issues/25).
- **It does not vanish when the pointer is over nothing.** A footer that
  disappears is a footer nobody learns is there, and the row would jump every
  time the pointer crossed the gap between two faders.
- **What it cannot say yet is what the parameter *does*** — the manual's
  sentence. The library deliberately keeps the prose out of the binary for the
  targets it is built for, so the ask is a default-off feature rather than a
  reversal: [#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28).

### Display

A grid of dots, and the one surface in this editor that gives off light instead
of catching it. One quad per printed dot, at a pitch every display in the window
shares: `PITCH` is 2.5 points, of which 1.9 is the dot, and a display given more
room does not get bigger dots — it gets more of them. That is the difference
between a second screen and a magnified one, and it is what makes the strip over
a plate's faders and the panel's own display read as two windows into one
instrument.

**It is a positive display, and it can be turned over.** A `DeepMind`'s screen
is a pale green-white backlit panel with its dots printed dark on it, which is
why a photograph of the instrument has one bright rectangle in the middle of a
dark panel. Pale dots on a dark pane would be the negative of the instrument
this is a picture of — every other synthesizer of the decade, and not this one.
So that is where the window starts, and a press at the end of the row that
chooses a surface turns every display in it over at once, the way a screen has
one backlight.

The negative livery is the same two greens driven the other way: the lit one
becomes the dots and the dark one becomes the ground. Nothing else in the window
changes — the panel, the metal, the recesses and both claims stay exactly what
they are — and the claim on the glass needs no second rule to follow it. Each of
the three is already mixed towards the material it is printed *on* or the
material it is printed *in*, so swapping those two swaps the ordering with them:
the strongest reading is the one furthest from the ground either way, which is
the whole of what that ordering has to mean.

```
┌────────────────────────┐   glass: #d6e7cd falling to #bfd3b6, the one
│ ▔▔▔▔▔▔▔▔▔╲╱▔▔▔▔        │   lit surface on the panel, in a dark bezel
│         ╲    ╱         │   dots 1.9 of a 2.5 pitch, radius 0.5, dark
│ ▔▔▔▔▔▔▔╲      ╱▔▔▔     │   2 points of moulding, 2 of dead border
└────────────────────────┘
```

**It is set into a surround rather than printed on the panel.** Two points of
moulding, dark with a hairline of the panel's own metal along it, and two points
of dead glass inside that. The two used to be one number and are different
things: the moulding is what catches the light in the room, and the dead border
is what a drawing stops short of.

What makes it read as *set into* something is what the moulding does to the
glass under it — a line of its own shadow across the top and its own lit lower
edge coming back off the foot, both landing in the dead border rather than over
any dot, which is what the dead border is for.

- **Only the printed dots are drawn.** A dot the display has not printed is the
  backlight coming through, and at arm's length there is no grid to see until
  something is written. Drawing all of them is both a picture nobody can see and
  eight thousand quads a display. What carries the matrix is the glass between
  the printed ones.
- **Text is dots, not a font.** `Screen::write` draws a 5×7 cell out of the
  table in `glyphs.rs`, which is the fourth face this window sets anything in
  and the only one that is not asked of the machine. A program name set in the
  system sans on the instrument's own screen would be the one thing in this
  window pretending to be something it is not. `Size::Large` is every dot drawn
  as four, which is what a display does when it has one thing to say and room to
  say it twice as loudly.
- **The claim is how hard the dots are printed.** Every other control puts it in
  the fill of the part that moves; a display has no part that moves. It is
  exactly the position the [name](#name) field is in, and the pale ground answers
  it better than a dark one could: `written()` prints a fact hard, a claim in the
  copper mixed most of the way to the same black, and what nobody has read barely
  at all. Three depths of one ink — an ordering before it is a set of hues, so it
  survives a photograph, a projector and the readers who would not see the
  copper. A screen with nothing read behind it is left blank, which on this
  display means lit and empty.
- **A dash pattern is never a claim.** `Ink` is solid, dashed or dotted, and it
  only ever tells one curve from another — three envelopes on one screen, a
  drawing against a marker. A screen has one colour of light and this is what it
  has instead of a second one.
- **Reverse video is a heading**, for the same reason: `invert` over a band is
  how a display with one colour says a line is a heading rather than a reading.

The drawings themselves are in `scene.rs`, one per plate, and they are found
rather than assigned: a scene names the controls it needs, a plate offers the
ones it holds, and the first that is satisfied is drawn. The two filters are the
only place a parameter is named instead of a section, because `VCF` and `HPF`
are two plates of one group.

| | |
| --- | --- |
| VCF | `generator::filter_response`, with the corner put where the byte sits in its own travel. The vertical is decibels from the library's floor to its ceiling with unity ruled across it, so the resonant peak has headroom to rise into rather than a passband chosen to leave room for it. A dotted rule along the top says how far the envelope's depth would move the corner, and which way the polarity points it |
| HPF | The high-pass, on the same decibel vertical and across as many octaves as its own slope needs to reach the floor, and `BOOST` printed under the curve where a high-pass has nothing |
| DCO 1 & 2 | Two lanes. Whichever of `DCO 1`'s shapes are switched on, side by side; `DCO 2`'s square as tall as its own level, with the noise scattered over it at its |
| ENVELOPES | One envelope a plate, from `generator::envelope`: the four times and levels, bent by the four curve bytes |
| VCA | The amplifier's envelope under the level it is played at, with the level as a dotted ceiling |
| LFO 1, LFO 2 | `generator::lfo` — the seven the value table names, the sampled two included — over as many of the library's own horizontals as the rate's travel, never fewer than two, about a ruled centre and with a tick a cycle along the foot |
| ARP / SEQ | `generator::arpeggiator_gates`: four steps, each as open as the gate time says. An arpeggiator that is switched off is a flat line |
| POLY | The polyphony mode in words, and the unison detune as five marks spreading from a centre |

**The shapes are the library's.** `deepmind-midi` 26.4 publishes them as
functions a host samples ([#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32)),
and what is left in `scene.rs` is a sample loop: walk the columns of a band, ask
`Generator::at` what the shape is doing there, print the dot nearest the answer.
Where a filter's corner sits for a byte, what a `Sample & Hold` looks like, how
an attack bends at a curve of 200 — all of it was arithmetic about the
instrument written in a window, and all of it is deleted.

**What none of them claim.** The two refusals the envelope drawing is already
under, because they are the library's:

- **No axis is in anybody's units unless the library publishes one.** Two are
  and are drawn as published: a filter's vertical is decibels, because the slope
  of a pole is, and an LFO's horizontal is turns, because a cycle is a cycle
  whatever the rate byte does. Those two are also the only two that are *ruled* —
  a tick an octave along the foot of the filter plates, a tick a cycle along the
  foot of the LFOs, both taken off the library's own `Scale` rather than
  measured here. Everything else is `Scale::Normalised`, which is the library
  saying outright that the axis is an ordering — so a corner is at the fraction
  of its own range the byte sits at, not at a frequency, and a plate drawn on
  one gets no ticks, because a scale nobody measured is a screen that looks like
  information. What each of those says is *where in its travel* a value is,
  which is exactly what the fader beside it says.

  Ruling the rest is [#36](https://github.com/MysteriousWolf/deepmind-midi/issues/36):
  the marks worth drawing on a `Normalised` shape are where its segments join,
  and the library is the one that knows.
- **Nothing is drawn from a value nobody has read.** A scene's claim is the
  weakest of everything it read, and a scene with anything unread is not drawn
  at all: a filter assembled from four values the synthesizer described and one
  this window invented is a picture of no filter.

**A wave is drawn about the line it swings around.** Every shape in the
library's table starts at the bottom of its range, so that the seven can be
drawn side by side without one looking shifted — which means one turn of a sine
is a hill. It leaves the floor, reaches the top and comes back, and a picture of
that reads as a bump rather than as something going round. So an LFO gets at
least two turns however slow its rate is, and a dotted rule across the middle of
the band for the level it swings about. The two turns are this window's; the
middle is an assumption, marked where it is made and asked for in
[#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35), because
`LFO n Unipolar` is a byte and the generator is not reading it.

Three things are left out by that rule rather than by oversight. The bass boost
is printed as a word instead of drawn as a shelf, because what it lifts is not
published and a shelf would be this window choosing a height and then drawing it
as confidently as the corner beside it. The LFO's `Delay / Fade` is not drawn,
because it is one parameter doing two things and the manual does not say where
the byte stops doing one and starts the other. And pulse width modulation is two
dotted marks either side of the pulse's edge — the depth's own travel, drawn
where the edge is — because what a byte of it does to a duty cycle is nowhere in
the manual.

The arpeggiator's rate is out of the picture for the same reason. The gate time
is published against a step — 0 is no note, 255 a full one and 128 half of one,
which the manual states outright — and what a step is worth in seconds is
exactly what it does not print, so a rate byte stretched across the glass was
this window drawing an axis nobody published. The fader says what the rate is.

One number about a filter is still here, marked where it is used: the high-pass
slope. The library draws the low-pass alone, because putting both corners on one
axis needs the spacing between two bytes whose curves are both unpublished, and
a plate with one filter on it is not asking for that. How far past its own fall
the axis reaches is read back out of the span the library publishes rather than
chosen, and a test holds the two together.

### Matrix

Eight modulation routings, one to a row, read across: the routing's name, where
the modulation comes from, an arrow, where it goes, and how much.

```
        Source              Destination                    Depth

Mod 1   [ LFO 1        v]  ->  [ VCF Freq      ] [map]    [======|========]
93-95   2                   44                            138
```


- **A rack is the wrong drawing for it.** Three parameters are one sentence, and
  twenty-four slots in one wrapping line put the words of a sentence in three
  places with the next sentence between them.
- **Each cell is a slot with what the row already says taken out of it**: the
  control and the reading under it. The title is gone because the column
  heading says `Source`, `Destination` and `Depth` once rather than eight times,
  which is the rule that takes a group's own name off the front of a slot's
  title.
- **The addresses are the row's**, printed under its name as the run they are —
  `93-95` — the way the name field prints the seventeen it occupies. Three
  addresses above three controls in a row is three numbers where the row needs
  one; a library that scattered the three would print all three instead.
- **The controls are the rack's own.** A list where the library names every
  value the parameter accepts, a fader where it does not, and the same claim
  drawn the same way under both. A depth a later library gives a value table
  arrives here as a list with nothing in the layout to change.
- **The depth fader runs across.** Eight rows cannot each be 128 points tall,
  and eight depths in a column of bars can be compared at a glance where eight
  numbers cannot.
- **The rows are the library's.** A parameter whose name ends in `Source`, with
  a `Destination` and a `Depth` sharing its prefix, is a routing: a ninth
  routing draws a ninth row, and a lone source somewhere else — the oscillators
  have one — is not a matrix and is not drawn as one.
- **A parameter no row claimed stays in the rack**, under the table, so a group
  that grows one keeps it rather than losing it to a layout.
- **The destination is typed into rather than scrolled.** 133 names in a
  drop-down is a drop-down somebody scrolls past what they wanted; the same
  names in a list that filters as you type are three letters and one answer. It
  is still the parameter's own value table, asked of the library for the
  firmware that answered, and a destination the library has no complete table
  for keeps whatever control the library says the parameter is.
- **Or the routing is pointed at the window, and you take hold of what it should
  move.** `map` is the press. While it is down, every control the matrix can
  reach is outlined in the modulation colour — on the front panel, in all
  fourteen racks, on all four effect engines — and everything it cannot reach is
  covered by the panel it stands on until it is barely there. Lit *and* dimmed,
  because forty faders with six outlined is a page somebody searches and the
  same page with thirty-four faded is a page with six faders on it.

  Nothing edits the sound while it is up. The sections still open, the panel
  still scrolls, the lists still say what they are showing, and the one thing a
  control no longer does is send anything. Taking hold of a lit one answers the
  question: a click chooses it, and a drag sets the depth as well.

  It is the answer to what actually makes that column hard. A destination is an
  abbreviation the instrument's display prints, and knowing which abbreviation
  stands over the fader you have in mind is harder than knowing the fader.
- **The drag is the same drag.** The same relative grab over the same range at
  the same rate, and what comes out of it is the fraction of the control's own
  travel rather than the value it would have reached — laid onto the depth's own
  range about its own centre, so dragging a control a third of the way up asks
  for a third of the depth and dragging it down asks for the same the other way.

  **What full depth is worth is assumed.** The manual prints no law relating a
  depth byte to its destination's range, so the window assumes full depth moves
  the control over all of it. That is the one assumption on the page, it is
  marked in the code and printed under the table while a routing is pointed, and
  it is asked for in
  [deepmind-midi#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38).
  Until it is answered the gesture is a way of *saying* an amount into the byte
  the depth fader already held, and the fader is unchanged.
- **Which controls light is the mark's own join, read backwards.** The
  destinations whose `parameters` include the control, so a firmware that moves
  a destination lights a different set with nothing here to edit. Where several
  reach one control the narrowest wins — `VCF Attack` over `All Attack`, because
  somebody who took hold of the filter envelope's attack meant the filter's —
  and *that* ranking is a judgement about the instrument being made in a window,
  which is [deepmind-midi#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39).
- **A mode says so where somebody can see it.** The footer carries the routing's
  name while it is pointed, on all three surfaces, and pressing it stops. A mode
  that could only be left from the page it was started on is a mode somebody
  gets stuck in, and somebody in this one is by definition somewhere else.

### Effects

All four engines at once, in the two-by-two the four of them make, each cut
into the group's face plate the way a section tab is cut into the panel. The
chain runs across the top of the block, at the width it needs to name what each
engine is running, and the two settings that shape it — the connection mode, and
whether the effects are inserted, sent or bypassed — stand in the band under it.

Every engine carries its own strip, and it is read left to right in two halves.
Four things on one line: **the slot's own number**, the list that is also the
title, the picture of what it is doing where there is one, and how loud it comes
out.

**The number is the number, and it is behind the line rather than on it.** It
read `FX 1`, which is two characters saying what the page it is on already says
and a third competing with the name of an algorithm. What is left is the one
thing on the strip that has to be read without reading — which of the four this
is — so it is set the way the family's mark is set on the case: large and faint,
standing behind what the strip carries. The gutter it stands in is a fixed
width, so four cases stacked two by two start their names in the same place down
the page.

**It is printed in the display's own dots**, on the case rather than on any
glass. The screen this window is a picture of writes its characters as a five by
seven cell of square dots, and the same cell drawn straight onto a surface with
no pane under it is what a number stencilled on a piece of equipment looks like —
so the one digit on the strip is the one piece of writing on the page that is
not set in a typeface. There is nothing behind it: no glass, no moulding, no
light on it, which is the whole difference between this and a display.

It is at the pitch every display in this window shares, so it is seventeen and a
half points tall — as large as the strip is deep and no larger. A dot covers
about half the cell it stands in and a glyph about half the cells of its box, so
a quarter of the ink a solid numeral laid down lands on the case: it is carried
further towards the case's own ink than the solid one was, and still reads
quieter than the name beside it.

Its layer is given the strip's depth rather than allowed to shrink to the
numeral. A stack lays an under-layer out at its own size and puts it at its own
origin, so a layer that shrinks to its contents is a layer aligned against
nothing: the numeral came out four points below the name it stands beside, which
is the sort of offset that reads as a mistake rather than as a mark on a case.
Both are measured now — the digit's dots and the name's glyphs share a centre to
the pixel.

That is how a rack unit puts a channel number on a case, and it is the second
thing on this page drawn as a ground rather than as an item in a row.

**The list is the title.** It was a list of the display's own abbreviations with
what they stand for written out beside it, which is two controls' worth of room
saying one thing — the name is what a reader wants and the list is what a hand
wants, and a list whose entries are the names is both. `Algorithm::full_name` is
the library's own join between its two names for an algorithm, so this is the
panel choosing which published name has the room rather than translating
anything. The abbreviation has not gone anywhere: it is what the chain's glass
prints in every box, and what the footer says about the byte.

**The level is a knob.** It was a fader lying on its side and it was the widest
thing on the strip by some way — a hundred and fifty points for one byte, on a
line that also had to hold the name of the algorithm. A knob is a quarter of
that, and what the difference bought is the room the response picture stands in,
which is why that picture no longer needs a line of its own or a page-wide
reservation to keep four cases level.

**The category and the badge are gone from it.** The category was a second line
of text on every strip, which is four extra lines down a page of four engines
for one word about a family whose mark is already on the case; the footer says
it of whatever is under the pointer. The mark left the strip for the case
itself — see below. Six things strung along
one line left the name squeezed between a drop-down and a fader, and the name is
the thing on the strip that is read rather than operated. The slot number is
held at one width so that four cases stacked two by two start their names in the
same place down the page.

There is no header below that and no row of tabs above it, because both of those
were a second place saying which algorithm an engine was running and neither of
them was the engine.

```
 ╱╲  FX 2  Midas Equaliser  Processing   [ MidasEQ  v]  ┌ gain [==|=] 150 ┐
╱  ╲
  ╶── low ─────╴ ╶── low-mid ───────────╴ ╶ high-mid ╴
   180 LSG  181 LSF  182 LMG  183 LMF  184 LMQ  185 HMG
    ( | )    ( | )    ( | )    ( | )    ( | )    ( | )
      0        19       38       57       76       95
  Low Shelf Low Shelf  Low-Mid  Low-Mid  Low-Mid Q High-Mid
    Gain    Frequency   Gain   Frequency            Gain
 -12.0-12.0 30-20000Hz -12.0-12 30-20000  0.3-5.0  -12.0-12
```

Six columns, every one of them there whether or not a slot stands in it, spread
across the whole of this engine's half of the page. That is what makes the
second row readable down against the first.

- **A slot is named by the algorithm, and drawn by the parameter table.** Those
  are two different claims and only one of them is published: `Freeze` is two
  states on the display and a parameter that accepts 256 values on the wire, and
  the curve between them is nowhere in the manual. So the control is the same
  code from the same table as every other slot in the editor, and the algorithm
  supplies the title, the abbreviation, the band and the two ends of the
  reading.
- **The ends are printed under the title and never interpolated.** `0.1-6.0 s`
  says what the two ends of the display read; the readout above it stays the
  byte, because a plausible `2.4 s` for a byte is wrong in a way nobody can see.
- **A slot the display names is not a list.** The manual prints `Ambience`,
  `Church`, `Gate` and never the bytes they sit at. The names are printed as
  what the display will show, the fader stays, and nothing offers to send one of
  them. Under the row that slot stands on, rather than at the foot of the case:
  nine preset names are two lines long on a case a quarter of this page wide,
  and printing them all together put the longest thing on the plate as far as it
  could get from the control it is about.
- **The grid is the instrument's own FX page.** Six columns and two rows,
  measured off the 35 screenshots in the manual, and every slot drawn in the
  column and row published for it — so a plate is the arrangement anybody who
  has edited an effect on the hardware already knows, and the third column of
  the second row stands under the third column of the first. Reading the
  publication as the *order* of a row instead, and packing each row against the
  left at the width of a rack's slot, is what left two rows that did not line up
  in the left half of an empty plate. The library's own `align` is what it
  offers a host laying a partial row out some other way; a slot here is in its
  measured column, so there is no room left over to place.
- **The size is the window's, and only the size.** The grid also publishes a
  proportion — a control 11.9 wide in a column pitched 20, on a 128 point
  display — and a page this wide would make that a knob a hundred points across
  with its title set in a face a tenth of its size. That proportion measures a
  dot matrix drawing its own labels in dots. So the arrangement is taken, which
  is exact, and the size is not: twelve controls on a page have room the rack's
  forty do not, and it is spent on the thing a hand touches.
- **The shape is the algorithm's own figure.** 29 of the 35 draw rotary knobs,
  five draw faders, one draws numeric displays. A knob is [the fader turned](#knob)
  and nothing about the control changes with it.
- **A band is a surface its own columns stand on.** One side of a stereo
  engine, one band of an equaliser: the library's runs, and each run is a block
  on the plate — a tint over the columns it covers, with its name knocked out of
  a pale strip along the top, which is how a `DeepMind` prints `ARP / SEQ` and
  `VCF` across the top of a group. The strip and the columns stand in the same
  container, so the strip cannot divide the row differently from the row.

  It was a bar with controls somewhere under it, and nothing said where a band
  stopped; two runs side by side read as one strip with two words on it. The
  tint is what says where it stops, and it is only a tint — the strip is
  silkscreen and is as pale as silkscreen, and a group of controls drawn on a
  surface that pale is a group whose readings have to be re-inked to be seen.

  Two columns the library labelled nothing are two columns and not a pair, and a
  row it grouped nothing on has no strip on it — but it keeps the room one
  takes, so every row of the grid is one depth.
- **The mark is the library's strokes and this window's layout.** Nine of them
  across the 35, one per family, published as a polyline, an arc, a sine and a
  filled disc in a unit box — not as a picture, for the same reason the panels
  are data: this window and the plugin want the same mark at two sizes and in
  two inks, and neither can theme an image it did not draw. `mark.rs` lays a
  stroke down as a run of round quads, because what every renderer behind this
  crate can do is fill one. On the chain's own glass, where a box is narrow,
  the library's seven by seven grid is blitted instead: at forty-nine pixels
  which of them are lit is the whole of the design.

  **The window places what the strokes reach, not the box they arrived in.**
  None of the nine fills that box and no two of them leave it the same way: the
  reverb's wavefronts leave a third of the width empty on one side, the imaging
  mark uses less than half the height, the delay's bars use nearly all of it.
  Drawn straight onto the room they are given, a row of engine strips has the
  reverb a third of the way off centre and the rest hanging at their own
  heights. So `mark.rs` measures what each one reaches — walking the curves with
  the same function that draws them, so the two cannot disagree — and centres
  that in the room.

  **It is a hero on the case rather than a badge on the strip.** Sixteen points
  of line drawing beside a name that says the same thing in words is a hard
  place for a mark: at that size a shallow drawing and a round one cannot be
  made to sit against each other, and three attempts at placing them all looked
  like a drawing that had slipped. The same nine strokes at ten times the area
  have no such problem. It is drawn across the face the controls stand on, in
  that surface's own ink carried a thirteenth of the way towards it, anchored
  into the bottom right and running a third of itself off the corner, clipped to
  the case.

  Two numbers make it a watermark rather than a picture the controls are
  standing on. It is taken from the case's *depth* and not its width, because a
  case is half again as wide as it is deep and a mark sized off the width sweeps
  the whole plate. And its strokes are laid down at a fraction of their usual
  weight: weight is a share of the side, so the rule that keeps a badge visible
  at sixteen points gives a hero lines a quarter of an inch thick, which is not
  faint at any colour.

  It says which family without being read, which is what a mark is for, and it
  never competes with a word because it is barely there. It also lands exactly
  where a case that is deeper than its algorithm needs has nothing on it.
- **An effect that is out of circuit says so, and 32 of the 35 cannot.**
  `FX n Type` is 35 effects with no `Off` in the table, and what takes effects
  out is the `Bypass` mode, which is the whole block of four. Three algorithms
  spend one of their twelve bytes on a switch of their own — Stereo Imaging and
  Chorus D on an `ON`, the Noise Gate on a `PWR` — and the library names which,
  so this window does not match on those two words across 35 panels. Where one
  of those three is off the strip says `out of circuit` and the chain draws that
  engine as something the signal goes past. Which way round the switch reads is
  the slot's own two ends, because the Noise Gate is the one that reads `ON` at
  the bottom of its range.
- **Two engines get a screen and 33 do not.** The tap delays' panels are
  literally a time and a gain per tap and their times are ratios of the master
  delay that the manual prints as fractions, so the impulse train follows from
  the parameters; `effect::response` draws it. It has its own line under the
  engine's strip, which is the first place it has had any room: at the head of
  the grid it was a pale rectangle two of six columns wide with nothing beside
  it, and squeezed into the strip itself it took the width the engine's name was
  standing in — the name being the thing it is a picture of. On its own line it
  is small: three or four taps along a time is what it draws, and a screen big
  enough to be a panel of its own would claim to say more about the engine than
  four gains and four times can. The grid starts at the top of the plate where
  it belongs.
  Every other engine gets nothing, which is the answer rather than a gap —
  a reverb's impulse response is its designer's, and a plausible one drawn here
  would look like information and not be any.
- **A slot with no printed range says what kind of quantity it is.** A `Mix`, a
  `Feedback` and a `Pre-Delay` are all a byte `0..=255` and they do three
  unrelated things, and `FxSlot::quantity` is the library's answer to which —
  derived from the parameter rather than read off the title, which is what makes
  it worth having. So the line under a slot says the two ends where the manual
  prints them and what the byte does where it does not, and every slot of the 35
  says something.
- **The chain is drawn, on a display over the settings it is a picture of.**
  Ten topologies as edge lists: what the block's input reaches, what feeds what,
  what is summed at the end, the loop dashed under the engines it returns
  through on the two that have one, and the analog path along the foot where
  `FX Mode` puts one. Nothing in it knows a topology by name, and where an
  engine stands is worked out from the edges — a column is how far it is from
  the input, and a backwards edge is the loop. `Bypass` draws the engines as
  something the signal is not going through, because the library says the DSP is
  out of circuit rather than muted.
- **The chain's glass takes the band, and never less than naming them costs.**
  64 dots down, which is the instrument's own display. Across, it is whatever
  the band divides into at the pitch every display in this window shares — a
  display given more room does not get bigger dots, it gets more of them, which
  is what lets this fill its band without becoming a picture stretched across
  one.

  What is derived is the *floor*: the longest abbreviation in the library's own
  table of 35, written at the size this glass writes, inside a frame, four of
  those across with the gutters and the rails at either end. Below that a box
  cannot name what is running in it. It stood exactly at that floor and was
  centred, with the band's own dark either side — which reads as a picture that
  did not know how much room it had.

  Nothing about the floor is written down. A firmware that adds a longer
  abbreviation raises it on the same day and without anybody editing a number,
  which is the rule the front panel already opens its window by. A box says the
  engine's number and what it is running on one line where it has the width, and
  stacks the number over the name where it has the height instead. The list
  under the graph is still there and is now never used; it is what would happen
  on the day a name outgrew the derivation.

  The graph is centred in what is left: the glass is cut for the deepest of the
  ten topologies and four in a line is the shallowest, so the difference used to
  be a third of the screen blank under a row of boxes.
- **The settings are under the display, not beside it.** That is what the wider
  glass cost, and it bought the better half of the trade: the ten topologies are
  laid out as three columns of lit legends rather than hidden in a drop-down. A
  list of ten shows whichever one is already chosen and hides the nine somebody
  is choosing between, on the one control on this page where the choice *is* the
  picture above it. Three columns rather than two because the band is wide and
  the page is better spent across than down: ten of them four deep is a block
  the eye takes in, and five deep was a column as tall as the plates beside it.

  **They take the whole band.** `Room::spread` says the room left over rather
  than a width, because the routing is the widest thing in the group and there
  is nothing else on that line to give the room to: a number written down for it
  was a band half full with the rest of the page blank beside it. The sentence
  the specification records about the two topologies that have a loop sits under
  the settings rather than beside them, for the same reason — a sentence sharing
  that row is a sentence taking the room the ten are laid out in.
- **The bytes an algorithm does not use are not drawn.** An algorithm can leave
  seven of its twelve unnamed. Moving one of those does nothing anybody can
  hear — the loaded algorithm does not read it — so it is not a control, and a
  strip of seven of them under the five that do something was the loudest half
  of a plate spent on the half that does nothing. They are still in the program,
  still sent, and still reachable from the modulation matrix, which is where a
  byte with no panel belongs.

  An engine whose algorithm nobody has read is the one case left: all twelve
  under the library's own `Param 9`, which is stage 3's rack for exactly as long
  as there is nothing better to say.
- **A case is as deep as what is in it, and a row is still the same shape.** The
  reservation went the other way first: a plate drew the grid's own two rows
  whether or not its algorithm filled them, and kept the deepest band of display
  names any of the 35 needs, so that four rack units bolted onto one page were
  four rack units of a height. What that bought was paid for by most of the 35 —
  an algorithm using five of its twelve bytes fills one row of a two-row grid,
  so more than half of that case was blank panel, and a rack unit with nothing
  on its lower half is a rack unit somebody looks for the missing knobs on.

  So the grid is drawn as deep as the algorithm on it, and the lines saying what
  a display shows stand under the row their own slot is on rather than at the
  foot of the plate. What is still reserved is the shape of a row, which is what
  makes the columns line up: a column stands a column's height whether or not a
  slot is in it, a run keeps the room a strip takes whether or not it has a
  name, and a control stands in a band of one depth whether the figure calls for
  a knob or a fader. Two engines running algorithms of one shape still come out
  level; two running a reverb and a delay do not, which is what they are.

  Whether a line is kept for the response picture is still the *page's* question
  rather than an engine's: two of the 35 publish one, so a page holding one of
  them keeps that line on all four of its strips and a page holding none keeps
  it on none. That one is not about the depth of a case — it is the strip along
  the top of every case, where an engine reserving the line for itself alone
  would be the page moving under the hand the moment a type byte changed.
- **Units land on one line.** A title is set in a box two lines tall whatever it
  needs, so a name that wraps pushes nothing down but itself and the readings
  across a row are read along one line rather than up and down a ragged one.
- **The modulation dot has a second state here.** Every slot is addressable from
  the matrix as `Fx n Param m`, and the library says which ones the engine acts
  on. A routing pointed somewhere the engine ignores draws the mark as an
  outline: it really is pointed there, and really is doing nothing.

- **The case has a grain.** A rack unit's face is brushed rather than painted
  flat, and what a window can do about that at this size is put a little light
  across it: a gradient from a fraction above the case's own colour at the top,
  through the colour itself, to a fraction below at the foot. Faint enough to
  read as a surface rather than as stripes. It is there to stop four large
  blocks of flat colour looking like four large blocks of flat colour, which is
  the one way a case measured off a photograph still gives itself away.
- **Nothing on the page is an essay.** Two paragraphs used to stand under every
  rack — what the effects page cannot know about a byte, and what this window
  does not write into the synthesizer — printed under whichever panel somebody
  was trying to read, whether or not it was the one they were about. What they
  were defending is defended by the parts of the window that are already about
  one control at a time: the line under a slot gives the two ends the manual
  prints and says what kind of quantity the byte is where it prints none, the
  footer describes whatever is under the pointer in the library's own words, and
  a value nobody has read is drawn as a value nobody has read. The rest of it —
  that this window writes no program into the instrument, and why — is in
  `README.md` and in this file, which is where somebody reading about the
  editor is.

## Four at once, and what that costs the livery

The effects are the one section with four of everything, and the page has been
three shapes. Four plates stacked was four algorithms' worth of controls on one
surface and a scroll to reach the fourth. One at a time behind a row of tabs was
an engine you could see and three you had to remember — on a page whose whole
subject is what four effects are doing *together*, with the chain drawn across
the top of it as a picture of exactly that.

So all four are on it, in the two-by-two the four of them make, with the chain
and the block's own settings above them: the things that are about the block
rather than about an engine.

**That is what let the measured colours be spent properly.** The library
publishes four per algorithm ([#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)):
the chassis around the controls, the face they sit on, the cap a finger moves
and the accent, and all four are spent. This file used to argue that they had to
stay a tint, because
four measured liveries side by side would be a collage of other people's
instruments in a window whose whole argument is that it is one instrument. With
**how many of the four can be spent depends on how many units are on the
surface.** With one open at a time the page wore the lot — case, face and cap,
the colour of the knob body printed on the algorithm's own figure — because a
single unit can be that unit without the window becoming a shelf of other
people's boxes. Four stand on it now, and four liveries side by side are exactly
the objection. So the face went back to the window's own plate and the cap back
to the window's own metal, and an engine wears its case and a hairline of its
accent: enough to tell the reverb from the distortion at arm's length, which is
all four units at once can afford to say.

The cap is therefore the one measurement this window has spent and handed back,
and the ledger is worth keeping. It was refused first on the grounds that what
carries a claim is the fill of the thing that moves, so a control repainted to
match a figure would break the one rule that holds everywhere. That argument was
wrong and the refusal was right: the claim is *which* fill and not which colour,
so a cap can be repainted — but there is only room for it on a page with one
unit on it, and this is not one.

### Nothing is printed in a colour that cannot be read on what is under it

Half the 35 measured chassis are pale and half are dark, so no word on this page
names an ink of its own. It names the surface it lands on — the face, a case, a
band's strip, the recess the chain is cut into — and the ink is whichever of the
instrument's two that surface is further from, measured rather than judged. A
legend is that ink half way back towards the surface and then lifted until it
clears the [contrast threshold](#contrast-is-measured-not-judged); a reading
keeps the colour of its claim and is lifted the same way, because how light it
is is what contrast is made of and which colour it is is what it means.

Two things fell out of measuring rather than eyeballing, and neither was visible
in a screenshot:

- **A case is carried as far into the panel as it can go and still be printed
  on.** The measured chassis half way in puts the cream ones in the exact middle
  of the instrument's two inks, where the best either can manage is 4.45 against
  it. So the carry is not a number somebody picked: it is the most of the livery
  that leaves the case readable, found by backing off towards the panel until it
  is.
- **A value is never printed on a case.** On the palest of the 35, an amber
  claim and a green one both have to be lifted so far to be read that they
  arrive as the same ink — which would spend the one distinction this editor
  exists to draw on a livery. The one value an engine's strip carries, its
  output gain, sits in a recess cut into the case, which is where a control
  belongs anyway.

## Colour is meaning, not decoration

The chassis is monochrome. Panel, recess, metal and ink carry the whole
interface, and every one of those comes from the mark:

| | |
| --- | --- |
| Panel | `#282c36` to `#15181e`, the gradient the mark uses |
| Seam and lip | `#000` at half, and `#454b58` |
| Recess | `#05070a` through `#11141a` to `#242932`, lit along its lower wall with `#7d838f` |
| Glass | `#d6e7cd` where the backlight enters falling to `#bfd3b6`, with the dots printed dark on it — three depths mixed towards `#101a12` |
| Metal | `#f4f5f8`, `#c9cdd6`, `#8e939f`; `#8e939f` is also the bar a plate's name is knocked out of |
| Ink | `#f2f2f4` for a title, `#a5a9b5` for a label, `#7d838f` for anything dim |
| Lamps | `#ffbe3d` on a way in and `#40d0e6` on a modulated control, and nothing else is saturated |

The two lamps are the instrument's own and so is the pairing. A `DeepMind`'s
panel is dark and the only colour on it is the light through its buttons: amber
on every `EDIT`, cyan on `MOD`, `CHORD` and `CURVES`. This window uses the same
two for the same two jobs — amber opens a section, cyan says something other
than a hand can move this control — rather than inventing a third.

The effect panels are the exception, and a deliberate one: their colours are
measured from the manual's own figures, four per algorithm, and a plate that
carries the chorus panel's own colours is telling the truth about what it is
editing. 26.3 publishes them, and they are spent as an identity rather than as a
finish — enough of the algorithm's chassis in the plate to tell the reverb from
the distortion at arm's length, and a hairline of its accent around the edge.
Painted as measured, four imaginary rack units side by side would be a collage
in a window whose whole argument is that it is one instrument — and four is what
is on the page, so what an engine wears is its case and a hairline of its
accent. The face and the cap went back to the window's own materials when the
fourth unit arrived. The rule that holds through all of it is the one that holds
everywhere: what carries a claim is the fill of the thing that moves. Nothing
else in the editor gets a colour for being itself: fourteen groups in fourteen
hues is decoration pretending to be information.

### Contrast is measured, not judged

A colour that is right in a palette and unreadable on a panel is not right, and
half the measured chassis are pale. So `style.rs` carries the measurement as
well as the colours:

| | |
| --- | --- |
| `contrast` | the ratio the web has used since anybody measured it: both colours' relative luminance, larger over smaller, lifted by a twentieth so black on black is 1 |
| `READABLE` | 4.5, the threshold for text at the sizes this window sets it in, written down once rather than judged per panel |
| `ink_on` | whichever of the instrument's two inks a surface is further from, by distance rather than by a threshold — a chassis half way between them is exactly where a threshold flips on a rounding error |
| `legible` | an ink moved the least it has to be to clear `READABLE` on what it is printed on, towards an ink rather than replaced by one, so an amber claim stays amber |

Relative luminance and not a mean of the channels: a saturated green is bright
and a saturated blue of the same numbers is not, and an average puts dark ink on
the second one.

The effects page is drawn under all four, and a test walks every word it prints
against every surface it can land on — the face, the recess, and the case in all
35 of its liveries. Both of the two findings above came out of that test rather
than out of a screenshot.

## Type

Two faces, because the instrument has two voices.

- **Mono** for abbreviations, raw values and anything the synthesizer's own
  display would show. Tabular figures required.
- **Sans** for titles, group headings and the rest of the application.

Sizes are 3u for a label, 3.5u for a readout, 4.5u for a group heading. Nothing
is bold except a group heading and the project's own name, and nothing is
italic.

**The sans is the mark's.** `docs/banner.svg` outlines the name from Liberation
Sans Bold so that it renders identically wherever the file is shown; a window
cannot outline anything, so it asks for that family by name, and for the
metrically compatible face other platforms ship under a different one — Arial,
which Liberation Sans is a clone of. A machine with neither falls back to its
own sans: the letters stay readable and the proportions are somebody else's.

Carrying the file in the binary is what would make both builds identical on
every machine, and it is a decision about a licence and about a megabyte in a
plugin bundle rather than a line of code. Until it is taken, the family is named
in one place — `control-ui/src/style.rs`, beside the palette — and asked for
there by both builds.

**The name is set as the mark sets it**: bold, in the metal of a fader cap, over
the panel. Without the wordmark's slices through it, which at 22 points are a
smudge rather than a slice — the mark is not improved by being approximated at a
tenth of its size.

## The window is a panel, and so is its chrome

The port picker, the buttons and the status bay are toolkit widgets, and they
are still on the instrument:

- **The ground is the panel's gradient**, `#282c36` falling to `#15181e`, which
  is what `logo.svg` fills its case with. A flat dark window is the bottom of
  the panel stretched over all of it, which is the one part that is not lit.
- **A button is the panel with a metal rim**, not a filled slab. A row of filled
  slabs is the brightest thing on a dark window, and the brightest thing here
  has to be a fader cap. Pressing lights the rim, the way a section button
  lights.
- **Anything chosen from a list is a recess**, closed and open alike: the track
  of a fader, the field a name is typed in, and the picker a port is chosen
  from are the same cut into the same panel.
- **A panel of words is the face plate** a rack of slots sits on, so that what
  the window says about the instrument sits on the instrument.

All of it comes from `materials()`, which means restyling the chrome is the same
one file as restyling a fader.

## Movement

- **A control never animates toward a value that arrived from the instrument.**
  An eased transition is a lie about when the synthesizer answered, and this
  application's whole claim is that it tells the truth about that.
- Nothing grows, lifts or glows on hover. A hover shows the raw value and a
  one-pixel brighter rim, and that is the whole hover treatment.
- The only motion is the cap under a finger.

## Input

- Vertical drag moves a fader, relative to where the cap already is.
- Shift is fine: an eighth of the rate. A drag carries its value as a float and
  moves it by each pointer step rather than measuring from where the press
  landed, so reaching for shift half way through bends the rate without moving
  the value.
- The wheel is one step, and shift-wheel is eight.
- Keyboard and a focus ring are not wired yet. They need the focus machinery the
  application does not have, and claiming them before they work would be worse
  than the gap.

Every one of those sends its value on every change. Coalescing lives in
`deepmind-host`, which keeps one pending value per parameter and sends the
newest at a fixed rate, so a view that throttles its own output is a view
fighting the layer that already solved this.

## What this costs in iced

The renderer draws quads cheaply: a rounded rectangle with a border and a fill
is one primitive, so a fader is six of them — track, lit wall, two scale arms
per tick, cap, indicator — and the panel it sits on is one more. Arcs are not
quads, so a knob is a `canvas`, which is another reason they stay in the half of
the editor that needs them.

Text is the expensive part on a panel of 242 parameters, and most of it never
changes. Addresses and names are static; only the readings move.

A display is one quad per lit dot, which is why only the lit ones are drawn: the
panel's own screen is 132 by 100 and the ten of them together would be twenty
thousand quads a frame if the glass were drawn dot by dot. A drawing is a few
hundred, a line of writing is a few dozen, and the one expensive thing on any of
them is a heading in reverse video.

## Not this

- No brushed-metal photographs, no bitmap knobs, no screws in the corners. The
  mark is drawn with gradients and so is the editor.
- No drop shadow except the one the mark already uses under a cap.
- No hue as the only difference between two states, anywhere.
- No control that is inert but looks live.

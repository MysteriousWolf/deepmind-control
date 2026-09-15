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
- **A legend is printed over its control**, not under it, because that is where
  the instrument prints it: the hardware has a screen for readings and no room
  under a fader.
- **Every plate carries the press the hardware calls `EDIT`**, and the
  envelopes' `VCA`, `VCF` and `MOD` are the three ways into the three envelope
  panels, where the hardware uses them to choose which envelope its four faders
  address.
- **The arrangement is the instrument's; the livery is this window's.** The
  hardware's buttons are white, yellow and cyan. Here a colour already means
  something — copper is a claim, green is the synthesizer's own account — and a
  yellow `EDIT` beside them would be a fourth meaning to learn.
- **The row of twelve lamps over `POLY` is not drawn.** It says how many voices
  are sounding, and nothing on a MIDI port says that. A lamp that cannot be lit
  honestly is not drawn at all.
- **What is on the panel is the one thing this repository transcribes.** Which
  parameters have a fader, and what is silkscreened over them, is a fact about
  the hardware that the library does not publish
  ([deepmind-midi#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)).
  The table is written as parameter identifiers so a rename fails the build, it
  is in one file, and that file says it is there until the library answers.

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
editor keeps looking like the figures it was drawn from. A 270 degree arc open
at the bottom, a body, a pointer, and a tick ring when the parameter selects
rather than sweeps.

**Not drawn yet**, and the effect slots are the rack's own faders until it is.
What the library publishes about a slot is its name, its band and the two ends
of its reading; the grid, the control shapes and the measured panel colours are
in its `spec/layout.toml` and are not published, so an editor drawing knobs
there today would be choosing the shapes itself. It is an arrangement and never
a control, which is the one kind of change this design lets a later pass make.

Knobs are not an alternative to faders for the main editor. Two ways to draw the
same kind of parameter is how a panel stops being readable.

### Switch

Two states. A lamp and a legend, not a checkbox and not a toggle that slides.
Off is the panel colour behind a hairline; on is the lamp colour. The lamp is
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

### Display

A grid of dots, and the only thing in this editor that is neither panel, metal
nor ink. One quad per lit dot, at a pitch every display in the window shares:
`PITCH` is 2.5 points, of which 1.9 is lit, and a display given more room does
not get bigger dots — it gets more of them. That is the difference between a
second screen and a magnified one, and it is what makes the strip over a plate's
faders and the panel's own display read as two windows into one instrument.

```
┌────────────────────────┐   glass: the deepest recess on the panel,
│ ▁▁▁▁▁▁▁▁▁╱╲▁▁▁▁        │   lit at the top like the panel itself
│         ╱    ╲         │   dots 1.9 of a 2.5 pitch, radius 0.5
│ ▁▁▁▁▁▁▁╱      ╲▁▁▁     │   4 points of dead border inside the bezel
└────────────────────────┘
```

- **Only the lit dots are drawn.** An unlit dot on the instrument is the glass,
  and at arm's length there is no grid to see until something lights. Drawing
  all of them is both a picture nobody can see and eight thousand quads a
  display. What carries the matrix is the gap between the lit ones.
- **Text is dots, not a font.** `Screen::write` draws a 5×7 cell out of the
  table in `glyphs.rs`, which is the fourth face this window sets anything in
  and the only one that is not asked of the machine. A program name set in the
  system sans on the instrument's own screen would be the one thing in this
  window pretending to be something it is not. `Size::Large` is every dot drawn
  as four, which is what a display does when it has one thing to say and room to
  say it twice as loudly.
- **The claim is the colour of the whole screen**, and it is the one control
  where colour carries it alone. Every other control puts it in the fill of the
  part that moves; a display has no part that moves. It is exactly the position
  the [name](#name) field is in, and it is answered the same way: the lit dots
  take the claim's colour, the control beside the screen keeps the fill, and a
  screen drawn from anything unread is drawn dark rather than in a colour
  nobody should trust.
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
| VCF | The passband, the corner where the byte sits in its own travel, the resonant peak out of it, and the fall past it — twice as steep on four poles as on two. A dotted rule along the top says how far the envelope's depth would move the corner, and which way the polarity points it |
| HPF | The high-pass corner, and `BOOST` printed where the passband is empty |
| DCO 1 & 2 | Two lanes. Whichever of `DCO 1`'s shapes are switched on, side by side; `DCO 2`'s square as tall as its own level, with the noise scattered over it at its |
| ENVELOPES | All three at once. The amplifier's solid and shaded under, the filter's dashed, the modulation envelope's dotted |
| VCA | The amplifier's envelope under the level it is played at, with the level as a dotted ceiling |
| LFO 1, LFO 2 | The shape the value table names, over as many cycles as the rate's own travel |
| ARP / SEQ | A gate train: as many gates as the rate's travel, each as open as the gate time's. An arpeggiator that is switched off is a flat line |
| POLY | The polyphony mode in words, and the unison detune as five marks spreading from a centre |

**What none of them claim.** The two refusals the envelope drawing is already
under, because they are the library's:

- **No axis is in anybody's units.** A corner is at the fraction of its own
  range the byte sits at, not at a frequency; a rate is how many cycles fit
  across a screen, not a speed. What each of these says is *where in its travel*
  a value is, which is exactly what the fader beside it says.
- **Nothing is drawn from a value nobody has read.** A scene's claim is the
  weakest of everything it read, and a scene with anything unread is not drawn
  at all: a filter assembled from four values the synthesizer described and one
  this window invented is a picture of no filter.

Three things are left out by that rule rather than by oversight. The bass boost
is printed as a word instead of drawn as a shelf, because what it lifts is not
published and a shelf would be this window choosing a height and then drawing it
as confidently as the corner beside it. The LFO's `Delay / Fade` is not drawn,
because it is one parameter doing two things and the manual does not say where
the byte stops doing one and starts the other. And pulse width modulation is two
dotted marks either side of the pulse's edge — the depth's own travel, drawn
where the edge is — because what a byte of it does to a duty cycle is nowhere in
the manual.

The pole count is the one number read out of a name rather than a byte. `0` is
`4 Pole` and `1` is `2 Pole` on this instrument, so a drawing that counted the
value would draw every filter the wrong way round; it reads the digit off what
the value table calls the value, for the firmware that answered.

### Matrix

Eight modulation routings, one to a row, read across: the routing's name, where
the modulation comes from, an arrow, where it goes, and how much.

```
        Source              Destination          Depth

Mod 1   [ LFO 1        v]  ->  [ VCF Freq    v]  [======|========]
93-95   2                   44                   138
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

### Effects

Four engine plates, cut into the group's face plate the way a section tab is cut
into the panel. Each is the engine's own settings and then its twelve bytes, and
the settings that are no engine's — the connection mode, and whether the effects
are inserted, sent or bypassed — are the first thing on the panel rather than a
rack of two underneath four plates.

```
FX 1   179 Type              Midas Equaliser          219 Output Gain
       [ MidasEQ        v]   Processing                   [=====|======]
       13                                                 0

  low                          low-mid
  180 LSG     181 LSF          182 LMG     183 LMF     184 LMQ
  [ | ]       [ | ]            [ | ]       [ | ]       [ | ]
  0           19               38          57          76
  Low Shelf   Low Shelf        Low-Mid     Low-Mid     Low-Mid Q
  Gain        Frequency        Gain        Frequency
  -12.0-12.0  30.0-20000.0 Hz  -12.0-12.0  30.0-20000  0.3-5.0
```

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
  `Church`, `Gate` and never the bytes they sit at. The names are printed under
  the plate as what the display will show, the fader stays, and nothing offers
  to send one of them.
- **Bands are the library's runs.** One side of a stereo engine, one band of an
  equaliser: a heading over the cluster, and a slot the library labelled nothing
  stands on its own.
- **Twelve bytes, however many the algorithm uses.** The rest are at the end of
  the plate under `Param 9`, in a cluster that says the algorithm does not use
  them. They are still in the program, still reachable from the modulation
  matrix, and still sent. An engine whose algorithm nobody has read draws all
  twelve that way.
- **The modulation dot has a second state here.** Every slot is addressable from
  the matrix as `Fx n Param m`, and the library says which ones the engine acts
  on. A routing pointed somewhere the engine ignores draws the mark as an
  outline: it really is pointed there, and really is doing nothing.

## Colour is meaning, not decoration

The chassis is monochrome. Panel, recess, metal and ink carry the whole
interface, and every one of those comes from the mark:

| | |
| --- | --- |
| Panel | `#282c36` to `#15181e`, the gradient the mark uses |
| Seam and lip | `#000` at half, and `#454b58` |
| Recess | `#05070a` through `#11141a` to `#242932`, lit along its lower wall with `#7d838f` |
| Glass | `#101620` at the top of a display falling to the recess's own `#080a0e`, with the lit dots in the claim's own colour |
| Metal | `#f4f5f8`, `#c9cdd6`, `#8e939f` |
| Ink | `#f2f2f4` for a title, `#a5a9b5` for a label, `#7d838f` for anything dim |
| Lamp | `#ffb95c`, and nothing else is saturated |

The effect panels are the exception, and a deliberate one: their colours are
measured from the manual's own figures, four per algorithm, and a host that
draws a chorus in the chorus panel's colours is telling the truth about what it
is editing. Those come from the library when it publishes them, which it has not
— they are in its `spec/layout.toml` and stay there — so the four engine plates
are the editor's own materials today. Nothing else in the editor gets a colour
for being itself: fourteen groups in fourteen hues is decoration pretending to
be information.

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

# Interface

What the editor looks like, why, and the numbers to build it from. The views
themselves are stages 2, 3 and 5 of [the plan](plan.md); this is what they are
written against.

## It is a panel, not a window

The thing being edited has a front: a dark panel between wooden cheeks with a
row of faders on it. Every drawing this project already owns says so, from the
mark in `docs/logo.svg` to the 35 effect panels the library generates from its
specification. The editor is a continuation of those, not an application that
happens to control a synthesizer.

That decides more than it looks like it does. **No general-purpose widget
appears anywhere a parameter is edited.** A toolkit slider is a different object
from a fader on an instrument: different proportions, different hit target,
different reading distance, and no scale. Editors that mix the two look like
spreadsheets with a picture of a synthesizer at the top. Every control here is
drawn by this crate.

Toolkit widgets are correct everywhere else: the port picker, the file dialogs,
the librarian's list. The panel is for parameters.

The rule is about anatomy, not about which crate a widget came from. A selector
too long for legends is a list, and a name is a field of text, because that is
what those controls are on an instrument too. What neither of them is is a
slider standing in for a fader.

## The vocabulary is the library's

`deepmind-midi` draws all 35 effect panels from `spec/`, and those drawings
already settle what a control on this instrument looks like. The editor inherits
the anatomy rather than inventing a second one:

| | |
| --- | --- |
| Abbreviation above | Mono, as the synthesizer's own display writes it: `DCY`, `PDY`, `HiSvFreq` |
| Control in the middle | Fader, knob, switch, selector or readout |
| Title below | The same parameter written out, for a panel with room to be readable |
| Modulation dot | A small mark at the top right of a slot the modulation matrix reaches, read off the eight destinations the patch holds. `ValueTable::parameters_of`, from `deepmind-midi` 26.2, joins `VCF Freq` to the parameter it moves ([deepmind-midi#19](https://github.com/MysteriousWolf/deepmind-midi/issues/19)). A destination this window has not read moves nothing, because a mark drawn from an unread value says the instrument is doing something it may not be |
| Column pitch | One slot per column, filled left to right, wrapping onto a second row |

Two of those are worth keeping even though a bigger screen does not need them.
The abbreviation is what is printed on the instrument, so a player looking
between the two reads the same word twice. The dot is the only thing on the
panel that says a parameter can be moved by something other than a hand.

## The front panel is the home screen

**The window opens on the instrument, not on a section of it.** A DeepMind is
two rows of section plates with a screen between them: twenty-odd faders under
four-letter legends, a few lit buttons under each group, and on every group a
yellow `EDIT` that opens that group on the display. The other 242 parameters'
worth is behind one of those presses.

So is this. The panel is where a window opens, because it is where a player
looks first, and the rack of a section is one press behind it exactly as it is
on the hardware.

```
┌ ARP / SEQ ┐ ┌── LFO 1 ──┐ ┌── LFO 2 ──┐ ┌───────────────┐
│ ┌───────┐ │ │ ┌───────┐ │ │ ┌───────┐ │ │▛PROGRAM   Pad▜│
│ │▔╷ ▔╷ ▔│ │ │ │╭─╮ ╭─╮│ │ │ │╶╴ ┌─┐ │ │ │               │
│ └───────┘ │ │ │╯ ╰─╯ ╰│ │ │ │  ─┘ └─│ │ │  Modular Fun  │
│  ▮     ▮  │ │ └───────┘ │ │ └───────┘ │ │ ▁▁▁▁▁▁▁▁▁▁▁▁▁ │
│ RATE GATE │ │ ▮  ▮ ○Sine│ │ ▮  ▮ ○Sine│ │  this claim   │
│ [on][off] │ │      ●Tri │ │      ●Ramp│ │  Read it.     │
│      EDIT │ │      EDIT │ │      EDIT │ │               │
└───────────┘ └───────────┘ └───────────┘ └───────────────┘
┌ OSC 1 ┐ ┌── OSC 2 ──┐ ┌─── VCF ───┐ ┌HPF┐ ┌ POLY ┐
│ ▮  ▮  │ │ ▮ ▮ ▮ ▮ ▮ │ │ ▮ ▮ ▮ ▮ ▮ │ │ ▮ │ │  ▮   │
│  EDIT │ │ [on] EDIT │ │ [2 Pole]  │ │   │ │DETUNE│
└───────┘ └───────────┘ └───────────┘ └───┘ └──────┘
┌VCA┐ ┌VCA ENVELOPE┐ ┌VCF ENVELOPE┐ ┌MOD ENVELOPE┐
│ ▮ │ │ ▮ ▮ ▮ ▮    │ │ ▮ ▮ ▮ ▮    │ │ ▮ ▮ ▮ ▮    │
│   │ │ A D S R    │ │ A D S R    │ │ A D S R    │
└───┘ └────────────┘ └────────────┘ └────────────┘
```

**Two plates stand somewhere other than where the instrument prints them**, and
both are hand layout: the arrangement changes and what a control is does not.

- **The amplifier heads the envelope row.** `VCA` is one fader, how loud the
  voice is, and the plate after it is the envelope that moves that fader while a
  note is held. The instrument has them two rows apart because its envelopes are
  multiplexed onto four faders in the middle of the panel. Unfolded, the level
  and the three shapes that drive levels are one row.
- **The voicing drops to the second row**, at the end of the signal path it is
  about. How many voices a note takes and how far they are detuned is a fact
  about the voice the row builds, not about the two modulators and the
  arpeggiator it was printed beside. It is also what lets the display stand at
  the end of the top row rather than in the middle of it, which is where the
  hardware has it relative to the voicing.

The rest of the panel's rules:

- **What the library records about a plate is said in the footer.**
  `front::Section::note` is the specification's own sentence about a section
  beyond its controls: which fader of the instrument's is missing from this
  plate and why, or which of three envelopes the shared faders address at power
  on. Three of the nine sections carry one. It is a sentence about a plate, so
  it goes where this window already says what is under the pointer.
- **The screen is the application's, and the panel leaves a hole for it.** What
  a display says is which sound is on it, what backs that, and what last
  happened, none of which the view layer knows and all of which a plugin answers
  differently from a desktop window.
- **Every plate has a display too, and the instrument has one.** This is the one
  place the panel deliberately stops being the instrument, and the reason is the
  one thing a window has that the hardware does not: room. A DeepMind shows
  whichever section was pressed last; here each plate carries the drawing of its
  own part, and the envelopes carry the drawing no DeepMind can show, all three
  at once. See [Display](#display).
- **A legend is printed over a fader**, because that is where the instrument
  prints it: `RATE` over its fader, all the way along the panel.
- **A press carries its mark instead**, on the cap, in the display's own dots.
  The band of ways in has done that since it was drawn, and the panel's presses
  had words above them, which is two languages for the same row of buttons: a
  nine-dot mark is what every other small drawing in this window is made of, and
  a cap is large enough to hold one. A power symbol on the arpeggiator's
  `ON/OFF`, a snowflake on its `HOLD`, a low shelf on the high-pass `BOOST`, a
  sawtooth and a pulse on the pair that choose the first oscillator's mix —
  those two the instrument's own silkscreen, and its pulse is drawn as a narrow
  one with a dotted line standing out in the low part of the wave: a second,
  later falling edge, which is where the `PWM` fader beside the press would move
  the one that is drawn — two linked rings on `SYNC`, a driver seen face on for the high-pass `BOOST`,
  and three dots on every `EDIT` — two by two each, because three specks on a
  cap the size of a thumb is a cap that looks blank — because what is behind that press is the rest
  of the section: the plate carries the four or five controls a hand reaches for
  and the press opens the twenty it had no room for, which is what an ellipsis
  has meant in every toolbar since menus had them. It was a pencil, and a pencil
  says *write here* about a press that writes nothing.
  Where a grid that size has no honest answer a press may keep its word, and no
  press on this panel does: `SYNC` did, because nine dots cannot draw *the
  second oscillator restarts with the first*, until it turned out they can draw
  *sync*, which is what the press is called. A blank cap in a row of marked ones
  is the one thing worse than a word.
- **Every plate carries the press the hardware calls `EDIT`**, and the
  envelopes' `VCA`, `VCF` and `MOD` are the three ways into the three envelope
  panels, which is what the hardware uses them for.
- **And above the panel there is a band of caps for the sections no plate
  carries.** The plates are the library's table of what the instrument puts a
  *fader* under, so four sections have no plate and had no way in at all: the
  modulation matrix, the effects, the sequencer and the program's own settings. A `DeepMind` reaches all four from buttons rather than from faders,
  so the band is that arrangement continued rather than an invention. It is
  described with the panel because it is the way into the sections, and it is
  drawn by the window: see [the band of ways in](#the-band-of-ways-in).
- **A way in is a legend and a lamp, not a word in a box.** `EDIT` is not
  written on the button on the instrument: it is silkscreened on the panel under
  a blank rubber cap lit amber the whole time the synthesizer is powered, and a
  row of those along the foot of every plate is the first thing you see in a
  photograph. So the legend is printed where the panel prints it, and what is
  pressed is the lamp, lit at rest and brighter under the pointer.
- **A button is a rubber cap, and the same cap wherever it is.** Five wide by
  three tall, which is what a DeepMind's caps measure in a photograph of the
  front, and large enough to hold a nine-dot mark — which is what set the size,
  since a cap that cannot hold its own printing is a cap with its picture under
  the bezel. Built out of the four
  things that make one read as rubber rather than as a coloured rectangle: a
  dark bezel all the way round, because the cap is moulded into a rim and there
  is no metal edge anywhere on it; a dome down its own height, so the light
  lands on the crown and the foot sits in shadow; a diffuser, which is the
  falloff from a single point of light under the middle out to the rim, drawn as
  a stack of rounded quads standing inside one another because a renderer here
  has no radial gradient; and relief, which pressing spends. Pressing turns the
  dome over rather than reaching for a second colour.
- **An unlit cap is pale, not dark.** The buttons on a DeepMind are moulded from
  a translucent off-white, and one with no lamp behind it still catches the room
  and reads as pale against a panel this dark. A cap drawn at the panel's own
  colour is a hole, and the instrument has none — which is also why what is
  printed on a cap is printed in dark ink whether the cap is lit or not. The
  slot with genuinely nothing in it is what a switch nobody has read is drawn
  as, and it is a difference in relief rather than in colour.
  The band along the foot of a plate is one band, so the `EDIT` press and the
  switch beside it are the same cap at the same size, and that band is the one
  part of the panel the window does not stretch.
- **A thin rule divides the clusters inside a plate**, where the instrument
  prints one: `VCF` is ruled between `RES` and `ENV`, so the filter's own two
  faders are separated from the three that modulate it. The oscillators and the
  LFOs are ruled on the hardware too, and this window already draws each of
  those as two plates, so the rule is what is left of that idea inside a plate
  it kept whole.
- **The plate's name is printed the way the instrument prints it**, and there
  are two instruments. A `DeepMind 12` and the desktop `12D` put every section
  name in white caps on the bare panel, ruled off from its neighbours by a
  hairline. A `12X` knocks the same names out of filled banners — red down the
  signal path, blue on the arpeggiator and the high-pass, white on the envelopes
  — and a photograph of one is a dark panel with a dozen red stripes across it.
  Both are the instrument. The window opens as the `12`, because it is the
  plainer of the two and the one most DeepMinds in the world are, and a press in
  the footer wears the other. Either way that band is what the eye follows
  across the panel before it reads a legend.
- **The window opens as wide as the widest surface it has to draw**, which is
  the panel or the effects page, not whichever of them the window opens on. The
  panel's width is its widest row at the instrument's own proportions; the
  effects page's is its chain, which is a fixed count of dots because every box
  on it has to name what is running in it. A window sized for one draws the
  other past its own edge.
- **The panel fills the window it is in.** Every dimension is written at the
  instrument's own proportions and then drawn through one scale, measured from
  the widest row against the room there actually is: the lanes, the travel of a
  fader, the buttons, the type, and the gaps between and inside the plates.
  Scaling the gaps is the half that decides whether it reads as an instrument or
  as a panel with its parts pushed apart. It stops at 1.75, because a fader as
  long as an arm is not an improvement, and it never goes below 1: a narrow
  window wraps its rows, which is still readable, where shrunken type is not.
  Displays do not simply grow, since a wider window is a filter curve drawn more
  finely rather than a magnified one. Past 1.75 the panel stands in the middle
  of the window rather than against its left edge.
- **Every row fills it, not only the widest one.** The top row is what the panel
  is measured from; the signal path is a plate narrower and the envelopes are
  two plates narrower, and drawn at what they measure they leave that difference
  as bare panel at the right-hand end. A row of a front panel runs the whole
  width of the instrument, so the difference is shared among that row's plates
  in proportion to what each already holds. Every plate of a row grows by the
  same fraction of itself, so `VCF`, which is five faders, stays twice the width
  of `OSC 1`, which is two. Nothing inside any of them moves, because what the
  extra room buys is display: a plate given more glass gains dots. The screen is
  the exception, at a written width rather than what is left over. A row a
  narrow window has had to break is not drawn out, because a fraction of a row
  filling the panel is `POLY`, one fader, drawn as wide as the window.
- **Every plate stands the same height, and so does the screen between them.** A
  panel whose plates were each as tall as their contents has a ragged edge under
  every row and its `EDIT` presses at five different heights, which is the one
  thing a front panel never is. The height is added up from the parts rather
  than written down beside them, because two of the parts do not grow with the
  rest: a display gains dots instead of getting bigger, and the buttons along
  the foot are drawn in the room this editor gives a control that is not a
  fader. The `EDIT` press stands in the middle of that band.
- **A lit set is given room for all of it.** A column of legends is laid out
  into the room it is given, and legends past the end of that room are drawn no
  lines tall, which is how the panel came to name five of the instrument's seven
  LFO shapes with nothing saying `Sample & Hold` and `Sample & Glide` were
  missing. The strip is a whole lane tall now, which is the fader's travel, the
  gap under it and the reading it would have had, and as wide as the longest
  name it lights. A test fails if a later table names something that does not
  fit.
- **The livery is the instrument's.** The hardware's buttons are white, amber
  and cyan, and this window takes the amber and the cyan for the two jobs that
  need a colour: amber opens a section, which is what the hardware's `EDIT`
  does, and cyan marks a control something other than a hand can move, which is
  what the hardware's `MOD` does. This file used to argue the opposite, that a
  yellow `EDIT` would be a fourth meaning to learn beside the copper and the
  green. Moving the claim onto the glass as a depth of ink took it off the
  panel's lamps entirely and took the objection with it.
- **The row of twelve lamps over `POLY` is not drawn.** It says how many voices
  are sounding, and nothing on a MIDI port says that. A lamp that cannot be lit
  honestly is not drawn at all. It is the only thing on the hardware's front
  this panel leaves out.
- **Nothing on the panel is transcribed.** Which parameters have a fader, what
  is silkscreened over them and which row they are in was the one table this
  repository kept; `deepmind-midi` 26.3 publishes it
  ([#26](https://github.com/MysteriousWolf/deepmind-midi/issues/26)) and the
  table is deleted. What is left in `home.rs` is layout, which is this window's
  to decide.
- **Two sections are drawn as more than one plate**, and both are the same trade
  as the screens: a window has room the front of a synthesizer does not. `OSC 1`
  and `OSC 2` are the brackets the instrument prints inside its own `DCO 1 & 2`
  plate, promoted to a plate each. The envelopes become one plate each with
  their own four faders and their own screen, because the hardware has four
  envelope faders, three envelopes and a button pointing one set at the other.
  Both splits are derived through the library: an oscillator's bracket is a
  slice of its parameters' own names, and an envelope's four faders are the
  parameters whose short names match the four the section carries.

## Fourteen panels, each one a sheet over the panel it was opened from

Two hundred and forty-two parameters do not fit on a screen, and the instrument
does not put them on one surface either: a player presses `EDIT` on a section
and the display becomes that section. Nothing about the front of the instrument
moves while that happens. The faders are where they were, the plate is where it
was, and what changed is what is *over* it.

So a section here is a modal. The rack comes up on a sheet across the window,
the front panel stays visible underneath in the shadow the sheet casts, and
putting the sheet away puts somebody back exactly where they were. A second
`EDIT` swaps the sheet rather than stacking one, the way a second press on the
instrument makes its one display show the other section.

There was a bar of fourteen tabs instead, on a surface of its own. Tabs say the
sections are peers of one another and of the panel; what they actually are is
the detail behind one press on a panel that does not move, and a modal is what
that shape of thing is.

**A sheet takes almost the window and never all of it.** The margin is one of
the three ways out, and a hand that misses the sheet has to land on something: a
border two points wide is a target nobody hits on purpose.

**There are three ways out, and one of them is drawn.**

| | |
| --- | --- |
| A press on the window around the sheet | Anywhere outside it. Why the margin exists |
| The mark on the sheet's own bar | The one that is drawn, because the other two are gestures and a gesture nobody was told about is not a way out |
| The escape key | Heard by the application rather than by the view layer: a key is an event before it is a press, and `control-ui` has no runtime to listen in |

All three send one message. What it says is that whatever is open should close,
and never which thing, because one thing is open: a second sheet over the first
would be a window nobody can find the bottom of.

**A sheet is made of what the window is already made of.** The face plate every
rack already sat on, its title in the display's own dots, a mark stencilled the
way the chrome's other marks are, and behind it the panel's own darkest
material at three-quarters alpha. Not a dialog borrowed from a toolkit and not
a grey invented for one.

**And it is one frame and not three.** The title was in a recessed bordered
band and the rack was on a bordered face plate of its own, both inside the
bordered sheet — three rectangles deep before anything is a control. A plate
needs a border on the *front panel*, where it stands in a row of ten on a dark
panel; a rack is never on the front panel, it is on a sheet, which is already a
plate lifted off the window, or on a page, which is the window. So the sheet
keeps its own edge and the two inside it are gone. What is left framed is every
individual control, which is drawn as a recess with a lit lower wall and needs
no help reading as cut.

**The shade covers the surface and not the whole window.** What a sheet is the
detail *of* is the panel it was opened from, and that is what goes dark under it.
What stays lit is the chrome that is true whatever is open: the port, the switch
between the surfaces, the band of ways in, and the footer saying what is under
the pointer. The band is the reason — a shade across everything is a shade across
the tabs — and the rest follows from it, because those are the parts of the
window a sheet is not standing in front of. All three ways out are untouched: the
margin round the sheet is still shade, and a press on it still closes.

**A sheet is titled the way a tab is**: the section's mark and its word, in the
display's own dots, on the bar a plate on the front panel prints its name in.
Every one of the fourteen has a mark now — four had one while only the band drew
a plaque, and a title that is a picture and a word next to a title that is a
word is an exception with ten cases rather than a style. Each new one is the
picture that section already is somewhere else in this window: a corner and a
slope for the filter, a contour for the three envelopes, a cycle for an `LFO`,
the amplifier triangle for the `VCA`.

What that replaced was the section's name set in the machine's sans with a
coloured claim dot in front of it — the one thing in this window that looked
like it had been lifted out of a dialog box. The claim is still said: it is said
by the display under the title, which is drawn in it the way every display here
is. Each of the fourteen tabs used to carry a dot, and a bar of green dots with
one copper one among them read as "the sound is the synthesizer's, except the
part I moved" without opening anything. One sheet at a time cannot say that, and
what replaces it is [open](todo.md).

**A section reached from the band has no title at all.** The cap that opened it
is lit directly above it and already carries the same mark and the same word: a
title bar under a lit tab is a name printed twice, one line apart.

**A sheet carries the section's display, across the top of its rack.** The plate
on the front panel has one and what its `EDIT` opened did not, which is the one
thing a section used to lose by being opened: the picture that says what the
twenty numbers below it add up to. The instrument answers the same way — press
`EDIT` and the screen becomes the section — so the sheet carries the screen the
plate was carrying, drawn over the whole width it now has. A section this window
has no picture for gets nothing rather than a blank strip: the matrix is the one
that asks, and its own table already draws the eight routings on glass twice
that size.

**The order of the sections is not written down anywhere.** A parameter's offset
is its NRPN number and its place in a dump, the library's parameter table is in
offset order, and so the order the groups first appear in that table is the
order the instrument keeps them in: LFOs, oscillators, filter, the envelopes and
the VCA, voicing, modulation, sequencing, effects, and the program's own
settings last. Reading it off the table is more honest than an order invented
here and one less thing to edit when the library grows a group. What it is not
is alphabetical, which is how the library hands the groups over and which puts
the effects third and the oscillators eighth.

Which section somebody is looking at is this window's business and never the
synthesizer's. It outlives a port being put down, because the sound went away
and the person did not.

**Four sections have no `EDIT` to open them.** The panel is the library's table
of what the instrument puts a fader under, so a section with no fader anywhere
has no plate and no way in: the modulation matrix, the effects, the control
sequencer and the program's own settings. The tab bar reached them and nothing
did for a while. The band of ways in is what reaches them now.

## The band of ways in

A row of caps above the surface, one for the front panel itself and one for each
section no plate carries. `control_ui::ways` draws it and the window places it,
between the switch that chooses a surface and the surface itself.

**The list is subtracted rather than written down.** `control_ui::unplated` is
every section the library has, less every section a plate opens, so a fifteenth
arriving in a later firmware gets a cap without anybody noticing it had to, and
the band is empty on the day every section has a plate. `control_ui::band` is
that list with the way home in front of it, as `Option<Group>`: `None` is the
panel with nothing over it and every other cap is the section it opens, which is
the same thing the window records about what is open. So which cap is lit is one
comparison rather than a flag kept beside the list.

**The caps behave like tabs, because that is what they are.** Exactly one is lit
at a time, and it is the one that would do nothing if it were pressed: the
section on the screen, or the front panel while nothing is over it. A lit cap is
the lamp behind amber rubber that every `EDIT` on the panel is. An unlit one is
the *same cap* with no lamp under it — the pale translucent rubber a `DeepMind`'s
buttons are moulded from, which is exactly what a switch that is not engaged is
drawn as elsewhere in this window. Nothing about the shape changes, because
nothing about the button changes; what changes is whether there is a light in it.

**The way home is a cap and not a fifth section.** It carries a rack of faders as
its mark and the words `FRONT PANEL`, and it sends the same message the mark on a
sheet sends: whatever is over the window should close. It is the one cap in the
band that means something while a *plated* section's sheet is up — a `VCF` sheet
leaves every cap unlit, because none of them is where somebody is, and the way
home is still the way back.

**The band stands outside the shade, and that is the whole of why it moved.** It
was the first band of the panel, inside the scroll. A tab under a sheet's shade
is a tab that cannot be pressed while a sheet is open, and a row of tabs you have
to close a sheet to use is a row of buttons: pressing one would land on the shade
and shut what was up. Above the surface, a press on it swaps the sheet, the way a
second `EDIT` on the instrument makes its one display show the other section.

**Each cap carries the section's mark and its name**, both in the display's own
dots, stencilled the way a legend is stencilled on the panel. The name is on the
cap rather than in the footer because a cap here is a fifth of the window wide: a
mark alone in that much room is a mark somebody has to hover to read. The ink is
dark on a lit cap, which is the one place in this window something is printed on
a surface brighter than itself, and the panel's own pale metal on an unlit one,
because an unlit cap is panel-dark and dark ink on it is ink nobody can read.

**The caps are as wide as their own printing needs**, with the slack shared out
evenly, because equal shares cut `CONTROL SEQUENCER` down to `CONTROL
SEQUENCE`. Where even that does not fit, the *words* go and the marks stay,
which is what a button on the instrument carries anyway — half a word is the one
thing a press must never say. The window opens wide enough that it does not come
to that.

## Two liveries, one instrument

The livery press sits at the foot of the window beside the one that turns the
displays over, and it follows the same rule: it shows the livery it is about to
*give* you, not the one you have. A swatch of a red banner while the panel is
plain, the panel's own outline while it is not. A picture of what you are asking
for says what a press does without being read.

Which of the two a window is wearing is a fact about the window rather than
about the sound, so it survives a port being put down: a `12` and a `12X` have
the same 242 parameters and two different silkscreens.

## Two surfaces, one row of ways in

The panel is the instrument. The library is the sounds somebody keeps. The patch
survives moving between them: putting a pack down to look at a filter and
finding the filter gone is the wrong thing to teach anybody about an editor.

They used to be chosen by a switch of their own, above the band. That said the
shelf is a different *kind* of thing from the sections, when what it is, to a
hand, is another place this window can be — so it is the last cap of the band
now, and one row of presses does what two did. `Way` is what that row is a list
of: the panel, the sections no plate carries, and the shelf.

**A cap of the band opens a tab and a plate's `EDIT` opens a sheet**, and the
difference is not a style. An `EDIT` is on the front of the instrument, so what
is behind it is the detail behind one press on that front: a sheet, with the
panel still there underneath. The four sections no plate carries are behind no
press at all, so they are not the detail behind anything — they are places, and
the band is a row of tabs. A tab that opened a sheet would be a tab that covers
the surface it is part of, which is what the effects looked like when they did.

**A cap of the band is a place and the escape key is not.** Pressing `FRONT`
puts you on the front panel from wherever you were, shelf included.
Escape, the mark on a sheet's bar and a press on the shade all mean *put away
what is over the window*, which says nothing about the surface underneath: escape
on the shelf leaves you on the shelf.

There were three surfaces once, and the middle one held whichever section was
open. It is gone with the tab bar: a section is not a third thing this
application is.

**The shelf is a grid and not a list.** A pack is 128 programs, and the one
thing this surface can offer that the instrument's two-line display cannot is
all of them at once: four across at the window's opening width, in slot order,
with the slot written the way the front panel writes it and the name beside it.
A column of 128 rows would show a quarter as much.

**A program is drawn as a chosen thing among unchosen ones**: the one that has
been pressed is the face plate a rack sits on, lit along its edge, and the rest
are the panel they are cut into.

**A slot that names nothing is drawn as naming nothing.** A stored dump carries
its bank and program; an edit buffer dump carries neither, because the edit
buffer is where a sound is played rather than where one is kept. That program
gets a dash where the others get `A19`, which is both the truth and how a patch
this application saved reads back in.

**Loading one is a claim.** Every value on the screen goes copper at once: the
host sends the difference, and nothing has heard the instrument play any of it.
Reading the edit buffer back is what turns the panel green, exactly as after a
fader is dragged.

## Confidence is a fill, not a colour

A claim per parameter, which is more than the library tracks: the library's
claim is about the whole sound, and one dragged fader leaves the other 241
values as confirmed as they were. `Patch` keeps the 242, and the three states
are drawn as three different objects:

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

Reading the edit buffer back turns a panel of outlines into a panel of fills,
which is the most useful thing a glance at this application can tell somebody:
whether they are looking at the sound or at their intentions.

**So the window reads it without being asked.** The moment a synthesizer answers
the inquiry, the editor asks for the sound it is making: one message and one
dump on a port that has just proved it works. Until that lands, every control on
the panel is the editor's arithmetic about an instrument sitting right there
with the answer.

There is no score of how many values are confirmed. The window used to carry a
key reading *reported, claimed, unread* and a bar counting the 242. Both were
bookkeeping about a gap the window should be closing rather than measuring, and
a reader who has to consult a key to know what a control means is being asked to
do the drawing's job. The three drawings have to be legible without a legend
beside them.

## Controls

Colours live in `control-ui/src/style.rs`, the only file in this repository that
writes down one. A control's geometry lives with the widget that draws it,
because it is that widget's anatomy and nothing else reads it.

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
  everywhere the rack draws one, which is how every fader on the instrument
  runs. A panel laid out by hand as rows turns the same fader onto its side and
  changes nothing else: the same recessed track, the same scale in pairs, the
  same cap, the same relative grab, and right is more where up was.
- The address above each slot is the parameter's NRPN number, which is also its
  byte offset in a dump.

### Knob

Where the source uses one, which means the effect panels, so that half of the
editor keeps looking like the figures it was drawn from. A body seated in a
recess, a pointer from the middle out to the rim, and a five-mark scale around
the outside. The turn is three quarters of a circle, open at the bottom: the
bottom of the range is at the bottom left, the top at the bottom right, and the
middle points straight up, which is where a hardware knob's own ends are.

- **It is the fader, turned.** The same byte, the same range, the same relative
  grab so a press never jumps, the same shift for a fine drag, the same wheel,
  and the same claim in the fill of the thing that moves. A whole drag is a rack
  fader's travel, so a hand moving between the two controls of one panel does
  not have to learn a second rate.
- **A drag is up and down whatever the shape is.** Turning a knob by dragging
  round it is a gesture nobody performs twice.
- **The sweep is a shape and never a reading.** The instrument publishes no
  angle for a parameter. What the byte is worth is the reading under it, the
  same as under every fader in this window.
- **Which slots get one is the library's.** 26.3 publishes what the figure
  printed beside each algorithm is made of: 29 of the 35 are rotary knobs, five
  are vertical faders and one is a numeric display. The display is drawn as a
  fader, because one shape invented here for one algorithm would be a worse lie
  than the fader that is already honest about the byte underneath.

Knobs are not an alternative to faders for the main editor. Two ways to draw the
same kind of parameter is how a panel stops being readable; the effects are the
one place the source draws something else, and they follow it.

### Switch

Two states. A lamp and a legend, not a checkbox and not a toggle that slides.
Off is a dark cap; on is the lamp colour. Both are moulded, because an unlit
button on the instrument is still a rubber cap, and drawing that one as a hole
and the lit one as a light would be two controls wearing one name. The lamp is
the one saturated thing this design allows, and it only ever means *on*.

### Selector

One of a named set, and the value's name is the control: the library hands over
the label, so the control shows `Ramp Up` rather than `3`. Six values or fewer
it is a column of legends with one lit; beyond that a list, because the
modulation matrix has 130 destinations.

A table that does not name every value the parameter accepts is not used at all:
the fader stays, and the number with it. A control that silently drops the
values it has no name for is a control that moves the sound when somebody opens
it.

### Readout

For a value with no useful continuum, and for the raw number beside any control
while it is being dragged. Mono, tabular figures, so the digits do not jiggle
while a value runs through them.

### Name

The one control that covers more than one parameter, because it is the one value
the instrument stores in more than one. A name is seventeen parameters holding a
character each, sixteen characters and a terminator, and seventeen faders
sweeping 0 to 127 is a panel nobody can name a sound on: `Program Name Char 4`
reads `115`, and what that means is that the fourth letter is an `s`.

So the seventeen slots are drawn as the one display the instrument shows them
on. Mono, sixteen characters wide, cut into the plate as the same recess a
fader's track is. It keeps the slot anatomy the rest of the rack has, with the
address above it reading `223-239` because that is the run it occupies, and the
reading below it counting the characters used out of sixteen.

- **It sits where its first character sits.** The rack is in the table's own
  offset order, and the field takes the place of the seventeen slots rather than
  being lifted to the front of the panel.
- **The claim is the weakest of the seventeen.** A display that called itself
  the synthesizer's because sixteen of its characters were is the one lie this
  editor exists to avoid. Confidence has no moving part to fill here, so it is
  carried by the colour of the letters.
- **A character the name cannot hold never appears.** A seventeenth letter, or a
  `è` the display has no glyph for, leaves the field exactly as it was rather
  than appearing and then being taken back.
- **A keystroke costs the parameter it moved.** The word becomes edits by being
  written into a copy of the program and differenced, so typing a letter onto
  the end of a name is one NRPN and not seventeen. Inserting one in the middle
  shifts what follows it, and costs what it shifts.
- The field is inert until a sound has been read, like every other control.

### Envelope

Four faders and the shape they make, drawn above them. An envelope is the one
group whose meaning is a picture, and four numbers that do not draw it are four
numbers.

**All three at once, as three plates.** A DeepMind has three envelopes, one
display and three buttons choosing which of them the four faders address, so
comparing the filter's decay with the amplifier's means comparing one with a
memory of the other. All three are on screen here, and each is a plate with its
own four faders and its own screen rather than a third of one shared drawing.
Three curves sharing a band told apart by a dash pattern was the first attempt,
and on a grid of dots two lines crossing are the same dots.

### Cap

A moulded rubber button, and **nothing is written on it**. A DeepMind's panel is
blank caps pushed through holes in the metal and lit from behind, with what each
one is called silkscreened beside it. So the legend is drawn where the panel
draws it, and the cap carries only what a lamp can say by being lit.

That leaves the cap to carry the claim, which is where it belongs: an unread
switch is the hole with no cap in it, flat and recessed, exactly as a fader
nobody has read is a track with nothing to take hold of.

### Footer

A strip along the foot of the window saying what the pointer is over. A panel is
twenty-odd faders under four-letter legends and a rack is forty slots under
abbreviated ones, and both are readable only because a hand can ask what one of
them is: `KYBD` is `VCF Keyboard Tracking`, it takes `0` to `255`, and
controller 74 drives it. None of that fits over a lane and all of it fits along
the bottom of a window.

- **Every word of it is the library's answer.** The name, the section, the
  value's own name, the range and the controller are five questions put to
  `deepmind-midi`. The footer writes down nothing about a parameter.
- **It opens with a picture.** `ParamId::glyph`, published in 26.5: seven dots
  by seven before the name, on the same grid as the effects' marks and the
  modulation sources' cells. A decay as a tail, a mix as wet against dry, a
  pedal as a treadle. It is first because a picture is read before a word is,
  and because somebody who points at the same control twice should stop needing
  the word. 177 of the 242 parameters carry one; the rest are the effect slots,
  whose picture depends on the algorithm and is on the slot itself, and the
  seventeen characters of the program's name, which are letters rather than a
  control.
- **The claim is in words here, not in a colour.** A footer is a sentence, and a
  sentence that said what backs a value by being a different colour would be
  saying it only to the readers who see the colour.
- **The range is in raw bytes**, never the number the synthesizer's display
  shows, because the manual prints the two ends of a range and almost never the
  curve between them. See
  [#25](https://github.com/MysteriousWolf/deepmind-midi/issues/25).
- **It does not vanish when the pointer is over nothing.** A footer that
  disappears is a footer nobody learns is there, and the row would jump every
  time the pointer crossed the gap between two faders.
- **It prints what a parameter does** where the library has a sentence for it,
  behind the default-off `descriptions` feature this workspace turns on
  ([#28](https://github.com/MysteriousWolf/deepmind-midi/issues/28)).

### Display

A grid of dots, and the one surface in this editor that gives off light instead
of catching it. One quad per printed dot, at a pitch every display in the window
shares: `PITCH` is 2.5 points, of which 1.9 is the dot. A display given more
room does not get bigger dots, it gets more of them. That is the difference
between a second screen and a magnified one, and it is what makes the strip over
a plate's faders and the panel's own display read as two windows into one
instrument.

**It is a positive display, and it can be turned over.** A DeepMind's screen is
a pale green-white backlit panel with its dots printed dark on it, which is why
a photograph of the instrument has one bright rectangle in the middle of a dark
panel. So that is where the window starts, and a press at the end of the row
that chooses a surface turns every display over at once, the way a screen has
one backlight.

The negative livery is the same two greens driven the other way: the lit one
becomes the dots and the dark one becomes the ground. Nothing else in the window
changes, and the claim on the glass needs no second rule: each of the three
depths is mixed towards the material it is printed on or printed in, so swapping
those two swaps the ordering with them. The strongest reading is the one
furthest from the ground either way.

```
┌────────────────────────┐   glass: #d6e7cd falling to #bfd3b6, the one
│ ▔▔▔▔▔▔▔▔▔╲╱▔▔▔▔        │   lit surface on the panel, in a dark bezel
│         ╲    ╱         │   dots 1.9 of a 2.5 pitch, radius 0.5, dark
│ ▔▔▔▔▔▔▔╲      ╱▔▔▔     │   2 points of moulding, 2 of dead border
└────────────────────────┘
```

**It is set into a surround rather than printed on the panel.** Two points of
moulding, dark with a hairline of the panel's own metal along it, and two points
of dead glass inside that. They are different things: the moulding is what
catches the light in the room, and the dead border is what a drawing stops short
of. What makes it read as set into something is what the moulding does to the
glass under it: a line of its own shadow across the top and its own lit lower
edge coming back off the foot, both landing in the dead border rather than over
any dot.

- **Only the printed dots are drawn.** A dot the display has not printed is the
  backlight coming through, and at arm's length there is no grid to see until
  something is written. Drawing all of them is both a picture nobody can see and
  eight thousand quads a display.
- **Text is dots, not a font.** `Screen::write` draws a 5x7 cell out of the
  table in `glyphs.rs`, the only face in this window that is not asked of the
  machine. A program name set in the system sans on the instrument's own screen
  would be the one thing here pretending to be something it is not.
  `Size::Large` is every dot drawn as four.
- **A field too small for its name scrolls, and one that fits never moves.**
  `Screen::marquee` holds at the beginning of the name, travels, holds at the
  end and starts again, which is what an instrument with a two-line screen has
  always done. It does not run round with its tail chasing its head: a name that
  wraps is two names on the glass at once, and the first thing anybody wants
  from a label is its beginning. It is one clock for every display in the
  window, and it advances with the window's own redraws. Cutting the tail off
  was the alternative, and `Pitch Ben` is a name somebody has to already know to
  read.
- **The claim is how hard the dots are printed.** Every other control puts it in
  the fill of the part that moves, and a display has no part that moves. It is
  the position the [name](#name) field is in, and the pale ground answers it
  better than a dark one could: `written()` prints a fact hard, a claim in the
  copper mixed most of the way to the same black, and what nobody has read
  barely at all. Three depths of one ink is an ordering before it is a set of
  hues, so it survives a photograph, a projector and the readers who would not
  see the copper. A screen with nothing read behind it is left blank, which on
  this display means lit and empty.
- **A dash pattern is never a claim.** `Ink` is solid, dashed or dotted, and it
  only ever tells one curve from another. A screen has one colour of light and
  this is what it has instead of a second one.
- **Reverse video is a heading.** `invert` over a band is how a display with one
  colour says a line is a heading rather than a reading.

The drawings are in `scene.rs`, one per plate, and they are found rather than
assigned: a scene names the controls it needs, a plate offers the ones it holds,
and the first that is satisfied is drawn. The two filters are the only place a
parameter is named instead of a section, because `VCF` and `HPF` are two plates
of one group.

| | |
| --- | --- |
| VCF | `generator::filter_response`, with the corner put where the byte sits in its own travel. The vertical is decibels from the library's floor to its ceiling with unity ruled across it, so the resonant peak has headroom to rise into. A dotted rule along the top says how far the envelope's depth would move the corner, and which way the polarity points it |
| HPF | `generator::high_pass_response`, on the same decibel vertical and octave span as the low-pass beside it, with `BOOST` printed under the curve where a high-pass has nothing |
| DCO 1 & 2 | Two lanes. Whichever of `DCO 1`'s shapes are switched on, summed; `DCO 2`'s square as tall as its own level, with the noise scattered over it at its own |
| ENVELOPES | One envelope a plate, from `generator::envelope`: the four times and levels, bent by the four curve bytes |
| VCA | The amplifier's envelope under the level it is played at, with the level as a dotted ceiling |
| LFO 1, LFO 2 | `generator::lfo`, the seven the value table names including the sampled two, over as many of the library's own horizontals as the rate's travel, never fewer than two, about a ruled centre and with a tick a cycle along the foot |
| ARP / SEQ | `generator::arpeggiator_gates`: four steps, each as open as the gate time says. An arpeggiator that is switched off is a flat line |
| POLY | The polyphony mode in words, and the unison detune as five marks spreading from a centre |

**The shapes are the library's.** `deepmind-midi` 26.4 publishes them as
functions a host samples
([#32](https://github.com/MysteriousWolf/deepmind-midi/issues/32)), and what is
left in `scene.rs` is a sample loop: walk the columns of a band, ask
`Generator::at` what the shape is doing there, print the dot nearest the answer.
Where a filter's corner sits for a byte, what a `Sample & Hold` looks like, how
an attack bends at a curve of 200: all of it was arithmetic about the instrument
written in a window, and all of it is deleted.

**What none of them claim:**

- **No axis is in anybody's units unless the library publishes one.** Two are: a
  filter's vertical is decibels, because the slope of a pole is, and an LFO's
  horizontal is turns, because a cycle is a cycle whatever the rate byte does.
  Those two are also the only ones that are ruled, with a tick an octave along
  the foot of the filter plates and a tick a cycle along the foot of the LFOs,
  both taken off the library's own `Scale`. Everything else is
  `Scale::Normalised`, the library saying outright that the axis is an ordering,
  so a corner is at the fraction of its own range the byte sits at and a plate
  drawn on one gets no ticks. What each of those says is where in its travel a
  value is, which is what the fader beside it says.

  The rest is ruled from `Generator::marks`, published in 26.5
  ([#36](https://github.com/MysteriousWolf/deepmind-midi/issues/36)): where an
  envelope's segments join, where a filter's corner sits, where each cycle ends,
  where a pulse falls, and how far modulation swings that edge. Every one is a
  number the library computed to build the shape rather than a second derivation
  from the same bytes. What is drawn at one is the screen's decision: a division
  of the axis gets a tick along the foot, and the two marks that are moments in
  a wave stand up the band. The envelope plates take it furthest, with `A`, `D`,
  `S` and `R` at the four boundaries, which is the one thing four faders under a
  line could not say.
- **Nothing is drawn from a value nobody has read.** A scene's claim is the
  weakest of everything it read, and a scene with anything unread is not drawn
  at all: a filter assembled from four values the synthesizer described and one
  this window invented is a picture of no filter.

**A letter on a drawing is knocked out of a block.** The four segment initials
stand along the foot of an envelope's glass, which is the line its sustain runs
along, and written in the same dots as the curve `A` and `D` are four dots of a
dither. So each is drawn in reverse: a solid block with the letter left unlit
inside it, and a ring of dark glass around the block so that it has an edge
where the curve is densest. It is what this instrument's display does to say a
line is a heading.

**A wave is drawn about the line it swings around**, and one turn of it is
enough. The level is `Generator::rest`, published in 26.5
([#35](https://github.com/MysteriousWolf/deepmind-midi/issues/35)): the middle
for an LFO read as it swings, the floor for one read unipolar, and the library
is what knows which. The same release put `Slew Rate` into the shape itself, so
corners round and a square becomes a ramp between its levels.

One turn is enough because the wave is drawn across its rest line: a crest and a
trough either side of it is the picture of a cycle, where a wave drawn from the
floor of its range is a hill and needs a second turn to say it repeats. The rate
still reads as more of them.

Where the left edge of a picture is a moment the instrument has is published as
`Generator::anchored`. An LFO whose `Key Sync` is on restarts with each note, so
phase 0 is where that note finds it, and the glass rules it. A free-running one
is caught wherever it had got to, so the glass leaves it alone: a picture has to
start somewhere and the instrument does not.

One thing is left out by that rule rather than by oversight. The bass boost is
printed as a word instead of drawn as a shelf, because what it lifts is not
published and a shelf would be this window choosing a height and then drawing it
as confidently as the corner beside it. The library declines it for the same
reason.

Two things that used to be on that list are not any more. The LFO's
`Delay / Fade` is `generator::lfo_fade`, drawn as its own dotted line under the
wave, as two readings rather than multiplied together, because the library
publishes them as two shapes and their product is not a third thing it
published. And pulse width modulation is a `Width` mark with two `Sweep` marks
either side of it, which is the library saying where the edge falls and how far
the modulation moves it.

The arpeggiator's rate is out of the picture for the same reason. The gate time
is published against a step, where 0 is no note, 255 a full one and 128 half of
one, and what a step is worth in seconds is exactly what the manual does not
print. The fader says what the rate is.

**No number about the instrument is written down in this repository.** There was
one, the high-pass's 6 dB per octave, transcribed out of a doc comment in the
library, and 26.5 published the curve it belonged to
([#37](https://github.com/MysteriousWolf/deepmind-midi/issues/37)). The two
filter plates now agree because they are two calls to one library on one span
and one vertical.

The refusal under it stands, and it is the library's: the two corners are not
put on one axis, because doing that needs the spacing between two bytes whose
curves are both unpublished.

The same release published what the oscillators are making. `OSC 1`'s saw and
pulse used to be drawn side by side, because how they sum is not something the
manual gives. `generator::oscillator` sums them at equal weight and states in
its own documentation that the equal weight is the library's reading, which is
the difference between a guess and a published one.

### Matrix

Eight modulation routings, one to a row, read across: the routing's number,
where the modulation comes from, an arrow, where it goes, and how much.

```
         Source                Destination                 Depth

   ^
   1   [~][ Pitch Bend  ]  ->  [/][ VCF Freq        ]  (+)     (o)
   v
```

- **A rack is the wrong drawing for it.** Three parameters are one sentence, and
  twenty-four slots in one wrapping line put the words of a sentence in three
  places with the next sentence between them.
- **The table takes the room the window has.** The two lists are shares rather
  than widths, and a source gets the smaller of them because `LFO 1` is five
  characters and `VCF Envelope Attack` is nineteen. The glass beside them is a
  fixed count of dots and stays one: a display stretched is a magnified screen
  rather than a bigger one.
- **Each cell is a slot with what the row already says taken out of it**, which
  leaves the control alone. The title is gone because the column heading says
  `Source`, `Destination` and `Depth` once rather than eight times, and the
  addresses are on the glass beside the table.
- **Both ends are the same control.** A source is one of 24 names and a
  destination one of 133, which is a difference in how far somebody scrolls and
  in nothing else. Both are the searchable list, because the end that is hard to
  scroll decides: three letters and `VCF Envelope Attack` is the only one left,
  and the same three letters cost a source nothing. Where the library has no
  complete table for an end, that end keeps whatever control the library says
  the parameter is.
- **Each of them has its picture beside it.** Seven dots square, stencilled on
  the card: an LFO's wave, a wheel, a filter's corner. A source's is the
  library's own cell for it and a destination's is the glyph of the parameter it
  moves, both published in 26.5
  ([#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40)), and both
  the same picture the patch bay draws against the same name. An end that is
  `Off` or that nobody has drawn keeps the empty box.
- **The depth is a dial.** A depth is read about its centre, `-128` at one end,
  `+127` at the other and no modulation in the middle, and a dial is the control
  that shows a middle by pointing at it. Eight faders at four places along four
  tracks are eight positions to compare; eight dials are eight hands on eight
  clocks, and the one pointing straight up is the one doing nothing. A drag on
  it runs the rack fader's travel, because every control in this window moves at
  one rate under one hand.
- **A control nobody has read stands where its range is read from**: the floor
  for a value that counts up from one, the centre for a value read about one.
  Both are drawn with nothing to take hold of, and only one of them would
  otherwise be a picture of full negative modulation.
- **The rows are the library's.** A parameter whose name ends in `Source`, with
  a `Destination` and a `Depth` sharing its prefix, is a routing. A ninth
  routing draws a ninth row, and a lone source somewhere else, as the
  oscillators have, is not a matrix and is not drawn as one.
- **A parameter no row claimed stays in the rack**, under the table, so a group
  that grows one keeps it rather than losing it to a layout.
- **An end is typed into rather than scrolled.** 133 names in a drop-down is a
  drop-down somebody scrolls past what they wanted; the same names in a list
  that filters as you type are three letters and one answer. It is still the
  parameter's own value table, asked of the library for the firmware that
  answered.
- **No row prints a value.** A list's reading would be the name of a value over
  the byte that name stands for, `Pitch Bend` over `1`, which says nothing the
  control above it does not. A depth's reading is a number nobody needs to the
  byte: the dial is the picture of it, the glass beside the table prints the
  exact amount on the wire, and the footer prints it with the name and the range
  whenever somebody points at one.
- **What backs a routing is the colour of its number.** The numeral is about the
  routing rather than about any one of its three parameters, and it is already
  being read, so it carries the claim: green for what the synthesizer reported,
  copper for what this window claims, washed grey for what nobody has read.
- **Which routing is which is a numeral, and the presses that move it are either
  side of it.** `Mod` is what the heading over the table already says, and three
  offsets is where bytes live in a program nobody edits by offset. What is left
  is the number, printed in the display's own dots, which is the numeral the
  glass beside these rows prints on every wire the routing draws.

  Above and below it are the two presses that **move a routing up or down the
  table**, which is the one thing a matrix of eight identical slots gives nobody
  a way to do. The number is between them because the number is what moves:
  sending a routing up is `3` becoming `2`.

  Each is a solid triangle seven dots across and four deep, drawn in the same
  dots as the numeral and stencilled on the card the way a number is stencilled
  on a rack unit's case. A triangle and no shaft, because at this size a
  three-dot head on a one-dot stem reads as a cross rather than as a direction.
  A press with nothing to trade keeps its mark and loses the metal in it.

  The eight are read as a set and the instrument does not care which of them
  says what, so where a routing sits is for whoever has to read the table next.
  A matrix filled in over a week is eight rows in the order they were thought
  of; the same eight grouped by what they move is the same sound and a page
  somebody can read.

  Nothing about the sound changes. What moves is six bytes trading places: a
  source for a source, a destination for a destination, a depth for a depth. A
  press is dead where there is nothing to trade, and a routing whose bytes
  nobody has read cannot be moved anywhere, because writing an unread value into
  a slot is the one thing this window does not do.
- **A row is a card, and everything on it is on one line.** Eight rows of
  controls at four heights, floating on the plate the rack stands on, are eight
  rows nothing lines up against. A routing is one sentence and the card is the
  paper it is written on: the same face plate a rack's slots already stand on,
  with the seam and the lit lip every cut surface here presents.
- **Or the routing is mapped onto the window, and you take hold of what it
  should move.** The press is a **reticle** stencilled on the card in the
  display's own dots, a ring with a crosshair through it and a dot in the
  middle, with no rim, like every other press in this window whose face is a
  drawing rather than a label. What the press does is put the routing over the
  panel and wait for somebody to aim it at a control, and the word for that is
  in the footer.

  What changes while the mode is up is the ink: it goes to the one saturated
  colour on the panel, the same cyan every control the routing can reach is lit
  in, because the press and the lit controls are one thing happening. The mark
  does not change, because the colour has already said which of the two states
  it is in. While it is down, every control the matrix can reach is lit, on the
  front panel, in all fourteen racks and on all four effect engines, and
  everything it cannot reach is covered by the panel it stands on until it is
  barely there. Lit and dimmed both, because forty faders with six outlined is a
  page somebody searches and the same page with thirty-four faded is a page with
  six faders on it.

  **Lit means lit, not outlined.** A hairline rectangle around the control is a
  focus ring on a web page and nothing at all on a piece of equipment. A
  DeepMind says a press is on by lighting it from behind, so that is what this
  does: the instrument's own cyan coming up through the panel, brightest at the
  foot of the control where the light enters and falling away across it, with
  the wall it comes past catching it hardest. That is the rule the display's
  glass and a fader's track are drawn under too.

  **What is already there is drawn on the thing it is already on.** A control
  another routing lands on carries a band up its own travel for each one, from
  where the control sits to as far as that routing's depth can push it, and the
  footer names them: `Mod 3 and Mod 5 already → here`. Somebody choosing where a
  routing goes is choosing against the other seven, and a second routing onto
  the same filter corner is a thing people do on purpose and a thing people do
  by accident; the difference is whether they could see the first one. Which
  routing a band belongs to is the footer's to say, because a bar on a fader
  cannot carry a name.

  Which way a band swings is the source's, published in 26.5 as `Swing`
  ([#41](https://github.com/MysteriousWolf/deepmind-midi/issues/41)). A
  `Centred` source, an LFO or the pitch bender, gets a band either side of where
  the control sits; a `Rising` one, an envelope, a wheel or a pressure, gets a
  band from it. Taking the depth's own sign as the whole answer is right for the
  second and wrong for the first: a band drawn one way from a filter's corner
  said the filter could only ever open.

  Nothing edits the sound while the mode is up. The sections still open, the
  panel still scrolls, the lists still say what they are showing, and the one
  thing a control no longer does is send anything.
- **The drag is the same drag.** The same relative grab over the same range at
  the same rate, and what comes out of it is the fraction of the control's own
  travel rather than the value it would have reached, laid onto the depth's own
  range about its own centre. Dragging a control a third of the way up asks for
  a third of the depth, and dragging it down asks for the same the other way.

  **What full depth is worth is assumed**, and it is the last assumption on this
  page. The manual prints no law relating a depth byte to its destination's
  range, so the window assumes full depth moves the control over all of it. 26.5
  published `ParamId::modulation_reach`, which is where to ask
  ([#38](https://github.com/MysteriousWolf/deepmind-midi/issues/38)) and not the
  answer: it returns nothing for every pair, because nobody has measured it. So
  the guess is a fallback behind a question, written once and reached by both
  the drag and the bands, and the day a measurement lands in the library it
  stops being reached with no diff here.
- **Which controls light is the library's own join, read backwards.**
  `ValueTable::values_naming`, published in 26.5
  ([#39](https://github.com/MysteriousWolf/deepmind-midi/issues/39)): the
  destinations that reach a control, narrowest first, and `next` is the one to
  take. `VCF Attack` beats `All Attack`, because somebody who took hold of the
  filter envelope's attack meant the filter's. That ranking used to be made
  here, which is a judgement about the instrument being made in a window. It is
  on the library's side now, down to what happens when two destinations move the
  same number of parameters: value order, and the library says outright that
  nothing makes one of them narrower.
- **A mode says so where somebody can see it.** The footer carries the routing's
  name while it is pointed, wherever it is drawn, and pressing it stops. A mode
  that could only be left from the page it was started on is a mode somebody
  gets stuck in, and somebody in this one is by definition somewhere else.

### The patch bay

Beside the eight rows, on the instrument's own glass: the sources down one side,
the destinations down the other, and a wire for every routing between them.

```
  4 OF 8                                            93-116

  +-------------+
  |[#]Pitch Bend|
  |       1  +72|---------+   +----------------+
  |       2  -68|-------+ +---|[#]VCF Freq     |
  +-------------+       |     +----------------+
  +-------------+       |
  |[#]BreathCtrl|       +-----+----------------+
  |       4 +122|-------------|[#]VCF Res      |
  +-------------+             +----------------+
```

It is the one thing the table cannot show. Eight rows read one sentence each,
and what somebody wants to know about a modulation matrix is the shape of it:
that one LFO is driving three things, that two routings are fighting over the
filter corner, that the aftertouch goes nowhere. Reading that off eight rows
means holding eight sentences in your head at once.

**The glass is as deep as the rows it stands beside**, and the two are locked
together by one number rather than by two somebody has to keep in step: a
routing's row is a fixed height whatever is in it, and the glass is that height
eight times over with the heading on top.

**The heading is where the addresses went.** Every row used to print the three
offsets its parameters occupy, twenty-four numbers down the left-hand edge of a
page where nobody edits a program by offset. The one thing worth saying about
where these bytes are is where they start and stop, which is said once, on the
glass, beside how many of the eight are wired: `4 OF 8` and `93-116`.

**A name is drawn once, and what leaves it is a list.** One LFO driving three
things is one cell with three wires out of it. The same thing drawn as three
cells reading `LFO 1` is a table with lines on it. So both columns are the
names, once each.

**What each wire carries is written at the end it leaves from**: the routing's
number and its depth, one line each, down the source's own cell, with a knot on
the edge beside every one of them. `1 +72` and `2 -68` under `Pitch Bend` are
the two routings that leave it.

**A source or a destination is a cell, and every cell is the same cell.** A thin
frame, a seven-by-seven box for its picture, its name in the field beside it,
and the routings that leave it underneath, so that the two columns read as two
columns of the same thing rather than as words at different lengths in roughly
the right places. A destination has no readings under it: what arrives there is
written where it left from.

**A name too long for its field scrolls rather than losing its tail.** `Pitch
Bend` and `BreathCtrl` are ten characters in a field cut for nine. The field
holds still at the beginning of the name, travels, holds at the end and starts
again. Nothing that fits ever moves, so a page of short names is a still page.
It is one clock for every display in the window.

**A wire leaves flat, turns down a track of its own, and arrives flat.** The
glass is as deep as eight rows of controls and the gap the wires cross is a
fifth of that across, so anything drawn as a single sweep between two distant
nodes comes out as a near-vertical scratch that could have started anywhere. A
track each, because two wires down the same part of the glass have to be two
wires and not one heavier one. The corners are taken off by three dots, which at
this pitch is the most a dot matrix can say about a radius.

The cells are spread down the glass as far as it allows, up to a limit of three
lines of glass between them, and the group is centred: three of them are a group
in the middle of the glass rather than three cells in its corners.

Only the routings the patch has actually wired are drawn. The instrument ships
with all eight sitting on `Off`, and eight wires from `Off` to `Off` is a
picture of nothing drawn eight times. The names are cut to what a cell holds,
which is what the instrument's own screen does for all but a handful.

Every wire is solid. The ink a line is laid down in distinguishes one line from
the next and never says how much of anything there is. That rule is written down
in `lcd.rs`, and how much is the depth, which is the dial beside the glass.

**The picture in each cell is a picture.** Seven dots by seven, which is the
cell this display writes a character in: an LFO's wave, a wheel, an envelope's
corner on one side, and what the destination's parameter does on the other. 26.5
publishes the sources' as `ValueTable::cell_of` and the parameters' as
`ParamId::glyph`
([#40](https://github.com/MysteriousWolf/deepmind-midi/issues/40)), so two
columns of abbreviations are two columns of pictures, which is what a patch bay
is for: the shape of a matrix is something you read at a glance or not at all.

They are the library's for the same reason the effect families' marks are: a
picture of `LFO 1` is a fact about the instrument, and one drawn here would be
this window inventing it. The names stay beside them, because a picture and a
name say different amounts to somebody who has not met either, and an end nobody
has drawn keeps the empty box.

### Effects

All four engines at once, in the two-by-two the four of them make, each cut into
the group's face plate the way a recess is cut into the panel. The chain
runs across the top of the block, at the width it needs to name what each engine
is running, and the two settings that shape it, the connection mode and whether
the effects are inserted, sent or bypassed, stand in the band under it.

Every engine carries its own strip, read left to right: the slot's own number,
the list that is also the title, the picture of what it is doing where there is
one, and how loud it comes out.

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

**The number is behind the line rather than on it.** It is the one thing on the
strip that has to be read without reading, which of the four this is, so it is
set the way the family's mark is set on the case: large and faint, standing
behind what the strip carries. The gutter it stands in is a fixed width, so four
cases stacked two by two start their names in the same place down the page.

It is printed in the display's own dots, on the case rather than on any glass: a
five by seven cell of square dots drawn straight onto a surface with no pane
under it is what a number stencilled on a piece of equipment looks like. At the
pitch every display in this window shares it is seventeen and a half points
tall, as large as the strip is deep and no larger, and it lays down about a
quarter of the ink a solid numeral would.

Its layer is given the strip's depth rather than allowed to shrink to the
numeral, because a layer that shrinks to its contents is aligned against
nothing. The digit's dots and the name's glyphs share a centre to the pixel.

**The list is the title.** `Algorithm::full_name` is the library's own join
between its two names for an algorithm, so a list whose entries are the full
names is both what a reader wants and what a hand wants. The abbreviation is
what the chain's glass prints in every box, and what the footer says about the
byte.

**The level is a knob**, which is a quarter of the width a fader on its side
took, and the difference is the room the response picture stands in.

There is no header below the strip and no row of tabs above it, because both
were a second place saying which algorithm an engine was running and neither of
them was the engine.

- **A slot wears the picture of what it does.** `FxSlot::glyph`, published in
  26.5: seven dots by seven beside the title, and every slot has one, because
  what a slot does is the one thing the algorithm always knows about it. It is
  finer than the `Quantity` the line under it is drawn from and it is for a
  different job: a pre-delay and a decay are both a time, and a plate with
  twelve of these on it wants the gap drawn on one and the tail on the other.
  The same glyph serves every slot doing the same job, so a `Low Cut` on a
  reverb and one on a delay are one picture, and it is the picture the footer
  puts beside a program parameter doing that job.
- **An engine says what kind of thing it is.** `Algorithm::characters`, also
  26.5: one or two quiet words after the name (*vintage*, *modelled*, *stereo*,
  *dual*, *multiband*, *two in one*, *lo-fi*, *modulated*, *dynamic*), and
  nothing at all for the plain reverbs and the noise gate, whose family says
  everything a word can. Read rather than derived: the library carries a reason
  per membership, and a window matching on `Vintage` in a name would be right
  until the day it was not.
- **A slot is named by the algorithm, and drawn by the parameter table.** Those
  are two different claims and only one is published: `Freeze` is two states on
  the display and a parameter that accepts 256 values on the wire, and the curve
  between them is nowhere in the manual. So the control is the same code from
  the same table as every other slot in the editor, and the algorithm supplies
  the title, the abbreviation, the band and the two ends of the reading.
- **The ends are printed under the title and never interpolated.** `0.1-6.0 s`
  says what the two ends of the display read; the readout above it stays the
  byte, because a plausible `2.4 s` is wrong in a way nobody can see.
- **A slot the display names is not a list.** The manual prints `Ambience`,
  `Church` and `Gate` and never the bytes they sit at. The names are printed as
  what the display will show, the fader stays, and nothing offers to send one of
  them. They go under the row that slot stands on rather than at the foot of the
  case, because printing them all together puts the longest thing on the plate
  as far as it can get from the control it is about.
- **A plate does not print the whole value table.** What somebody playing needs
  from a knob is what it is on, which the reading under it gives, and how many
  places it stops at, which the line under that says: `10 settings`. Where the
  name of a setting matters, the control is a list and the list has the names in
  it.
- **The grid is the instrument's own FX page.** Six columns and two rows,
  measured off the 35 screenshots in the manual, and every slot drawn in the
  column and row published for it, so a plate is the arrangement anybody who has
  edited an effect on the hardware already knows and the third column of the
  second row stands under the third column of the first. The library's own
  `align` is what it offers a host laying a partial row out some other way; a
  slot here is in its measured column, so there is no room left over to place.
- **The size is the window's, and only the size.** The grid also publishes a
  proportion, a control 11.9 wide in a column pitched 20 on a 128 point display,
  and a page this wide would make that a knob a hundred points across with its
  title set a tenth of that size. That proportion measures a dot matrix drawing
  its own labels in dots. The arrangement is taken, which is exact; the size is
  not.
- **The shape is the algorithm's own figure.** 29 of the 35 draw rotary knobs,
  five draw faders, one draws numeric displays. A knob is
  [the fader turned](#knob) and nothing about the control changes with it.
- **A band is a surface its own columns stand on.** One side of a stereo engine,
  one band of an equaliser: the library's runs, and each run is a block on the
  plate, a tint over the columns it covers with its name knocked out of a pale
  strip along the top, which is how a DeepMind prints `ARP / SEQ` and `VCF`
  across the top of a group. The strip and the columns stand in the same
  container, so the strip cannot divide the row differently from the row. The
  tint is what says where a band stops, and it is only a tint: the strip is
  silkscreen and is as pale as silkscreen. Two columns the library labelled
  nothing are two columns and not a pair, and a row it grouped nothing on has no
  strip, but it keeps the room one takes, so every row of the grid is one depth.
- **The mark is the library's strokes and this window's layout.** Nine families
  across the 35, and since 26.5 a finer mark where an effect's kind is something
  a symbol can carry: a plate reverb as a plate with wavefronts leaving it, a
  hall as wavefronts far from their source, an ambient reverb as the family's
  own. The window asks `Algorithm::mark` and gets whichever applies, so the
  better marks arrived with nothing here to change.

  They are published as a polyline, an arc, a sine and a filled disc in a unit
  box rather than as a picture, for the same reason the panels are data: this
  window and the plugin want the same mark at two sizes and in two inks, and
  neither can theme an image it did not draw. `mark.rs` lays a stroke down as a
  run of round quads, because filling one is what every renderer behind this
  crate can do. On the chain's own glass, where a box is narrow, the library's
  seven by seven grid is blitted instead: at forty-nine pixels, which of them
  are lit is the whole of the design.

  **The window places what the strokes reach, not the box they arrived in.**
  None of them fills that box and no two leave it the same way: the reverb's
  wavefronts leave a third of the width empty on one side, the imaging mark uses
  less than half the height, the delay's bars use nearly all of it. So `mark.rs`
  measures what each one reaches, walking the curves with the same function that
  draws them so the two cannot disagree, and centres that in the room.

  **It is a hero on the case rather than a badge on the strip.** Sixteen points
  of line drawing beside a name that says the same thing in words is a hard
  place for a mark: at that size a shallow drawing and a round one cannot be
  made to sit against each other. The same nine strokes at ten times the area
  have no such problem. It is drawn across the face the controls stand on, in
  that surface's own ink carried a twenty-sixth of the way towards it, anchored
  into the bottom right and running a third of itself off the corner, clipped to
  the case.

  Two numbers make it a watermark rather than a picture the controls are
  standing on. It is taken from the case's depth and not its width, because a
  case is half again as wide as it is deep and a mark sized off the width sweeps
  the whole plate. And its strokes are laid down at a fraction of their usual
  weight: weight is a share of the side, so the rule that keeps a badge visible
  at sixteen points would give a hero lines a quarter of an inch thick.

  It says which family without being read, and it never competes with a word
  because it is barely there. It also lands exactly where a case deeper than its
  algorithm needs has nothing on it.

  **It meets the case the way the case's own material would let it.** A mark on
  a worn panel is stamped into it, one on a modern face is raised off it, and
  one on a case that is neither is printed flat, which is three passes of the
  same nine strokes a few dots apart: the light edge and the shadow either side
  of the ink, in the order the relief calls for. A boss is lit along its
  top-left edge and casts below-right, because that is the light every cap,
  plate and display in this window is drawn under; an indent is the same two
  edges the other way round.
- **A case is finished the way the unit it is a picture of would be.** Two of
  `Algorithm::characters` are facts about the box rather than about the signal,
  and they are the two a surface can carry:

  - **vintage** is *worn*: broad soft blotches where the light has not fallen
    evenly for thirty years, short scratches scattered over them, a few long
    rubs where a hand goes, and corners darker than the middle;
  - **lo-fi** is *gritty*: a fine broken diagonal grain with blocks missing out
    of it, which is a converter running out of bits;
  - everything else is *brushed*, which is still not flat: a few dozen hairlines
    the long way, because four large blocks of one colour on a page is the one
    way a case measured off a photograph gives itself away.

  **None of the three is on a lattice.** A grid of jittered cells is the obvious
  way to draw a texture out of quads and it is the wrong one: however hard the
  cells are shaken, the eye finds the row and the column, and then a plate whose
  controls should be the loudest thing on it has a screen door over it. So the
  worn face is scattered, with blotches and scratches at free positions counted
  by density so that a wide case and a narrow one are the same material rather
  than the same number of marks stretched differently, and the gritty one runs
  diagonally, which is the one direction nothing standing on the page runs in. A
  blotch is laid down as three rectangles inside each other at a third of the
  weight each, because a quad has an edge and a patch of uneven light does not.

  A Tel-Ray delay is both vintage and lo-fi, and the box is the older fact, so
  vintage decides. The rest of the characters leave the face alone: a stereo
  chorus is a rack unit like any other.

  Every mark on a face is laid down from a counter run through a hash seeded by
  the algorithm's name, so the same algorithm is the same unit on every frame,
  two engines running it are two of the same unit, and a firmware that renumbers
  the 35 does not re-scuff anything. And it is light and shadow rather than ink:
  a scuff is what a room does to a surface, and it does the same thing to a
  cream panel as to a black one.

  What this does **not** do is wear the unit's own paint. The library publishes
  `Panel::face` and `Panel::cap` as well as the chassis and the accent, and
  those two are the ones this page declines: four measured liveries side by side
  are a collage, and a surface can say what kind of unit it is without them.
- **An effect that is out of circuit says so, and 32 of the 35 cannot.**
  `FX n Type` is 35 effects with no `Off` in the table, and what takes effects
  out is the `Bypass` mode, which covers the whole block of four. Three
  algorithms spend one of their twelve bytes on a switch of their own, Stereo
  Imaging and Chorus D on an `ON` and the Noise Gate on a `PWR`, and the library
  names which, so this window does not match on those two words across 35
  panels. Where one of those three is off, the strip says `out of circuit` and
  the chain draws that engine as something the signal goes past. Which way round
  the switch reads is the slot's own two ends, because the Noise Gate reads `ON`
  at the bottom of its range.
- **Two engines get a screen and 33 do not.** The tap delays' panels are a time
  and a gain per tap, and their times are ratios of the master delay that the
  manual prints as fractions, so the impulse train follows from the parameters
  and `effect::response` draws it. It has its own line under the engine's strip.
  It is small: three or four taps along a time is what it draws, and a screen
  big enough to be a panel of its own would claim to say more about the engine
  than four gains and four times can. Every other engine gets nothing, which is
  the answer rather than a gap: a reverb's impulse response is its designer's,
  and a plausible one drawn here would look like information without being any.
- **A slot with no printed range says what kind of quantity it is.** A `Mix`, a
  `Feedback` and a `Pre-Delay` are all a byte `0..=255` and they do three
  unrelated things. `FxSlot::quantity` is the library's answer to which, derived
  from the parameter rather than read off the title, so the line under a slot
  says the two ends where the manual prints them and what the byte does where it
  does not.
- **The bytes an algorithm does not use are not drawn.** An algorithm can leave
  seven of its twelve unnamed, and moving one of those does nothing anybody can
  hear, because the loaded algorithm does not read it. They are still in the
  program, still sent, and still reachable from the modulation matrix, which is
  where a byte with no panel belongs. An engine whose algorithm nobody has read
  is the one case left: all twelve under the library's own `Param 9`.
- **A case is as deep as what is in it, and a row is still the same shape.** The
  grid is drawn as deep as the algorithm on it, rather than always drawing two
  rows and the deepest band of display names any of the 35 needs. What is still
  reserved is the shape of a row, which is what makes the columns line up: a
  column stands a column's height whether or not a slot is in it, a run keeps
  the room a strip takes whether or not it has a name, and a control stands in a
  band of one depth whether the figure calls for a knob or a fader. Two engines
  running algorithms of one shape come out level; two running a reverb and a
  delay do not, which is what they are.

  Whether a line is kept for the response picture is the page's question rather
  than an engine's: two of the 35 publish one, so a page holding one of them
  keeps that line on all four strips and a page holding none keeps it on none.
  An engine reserving the line for itself alone would be the page moving under
  the hand the moment a type byte changed.
- **Units land on one line.** A title is set in a box two lines tall whatever it
  needs, so a name that wraps pushes nothing down but itself, and the readings
  across a row are read along one line.
- **The modulation dot has a second state here.** Every slot is addressable from
  the matrix as `Fx n Param m`, and the library says which ones the engine acts
  on. A routing pointed somewhere the engine ignores draws the mark as an
  outline: it really is pointed there, and really is doing nothing.
- **The case has a grain.** A rack unit's face is brushed rather than painted
  flat, and what a window can do about that at this size is put a little light
  across it: a gradient from a fraction above the case's own colour at the top,
  through the colour itself, to a fraction below at the foot. Faint enough to
  read as a surface rather than as stripes.

#### The chain

**The chain is drawn, on a display over the settings it is a picture of.** Ten
topologies as edge lists: what the block's input reaches, what feeds what, what
is summed at the end, the loop dashed under the engines it returns through on
the two that have one, and the analog path along the foot where `FX Mode` puts
one. Nothing in it knows a topology by name, and where an engine stands is
worked out from the edges: a column is how far it is from the input, and a
backwards edge is the loop. `Bypass` draws the engines as something the signal
is not going through, because the library says the DSP is out of circuit rather
than muted.

**The glass takes the band, and never less than naming the engines costs.**
Across, it is whatever the band divides into at the pitch every display in this
window shares. What is derived is the floor: the longest abbreviation in the
library's own table of 35, written at the size this glass writes, inside a
frame, four of those across with the gutters and the rails at either end. Below
that a box cannot name what is running in it. Nothing about the floor is written
down, so a firmware that adds a longer abbreviation raises it on the same day. A
box says the engine's number and what it is running on one line where it has the
width, and stacks the number over the name where it has the height instead. The
list under the graph is still there and is now never used; it is what would
happen on the day a name outgrew the derivation.

The graph is centred in what is left, because the glass is cut for the deepest
of the ten topologies and four in a line is the shallowest.

**Every box carries the mark of what is in it, and the ones that stack carry it
beside the name.** The mark goes over the name where there is a line to spare
over it, which is what four engines in a line have. The topologies that stack
three engines in a column have the opposite shape, each box as wide as the graph
and a third of it deep, so there the mark and the name are centred together as
one thing.

**A merge is one junction and not one wire per engine.** Three engines feeding a
fourth drew three lanes two dots apart across the gutter, each ending in its own
arrowhead on the same dot of the same frame, which on a dot matrix is a blot.
The gutter is shared out by where the wires arrive rather than by how many there
are: wires into different boxes never share a lane, because that is what tells
two paths apart, and wires into one box always do, because they are one
junction.

**An arrowhead is a solid triangle.** An outline is not a shape at this pitch,
it is a handful of specks arranged near one. The run stops where the head
starts, so the line is not drawn underneath it.

**Nothing is drawn through a box.** Two of the ten take the block's output from
an engine that has others after it. That wire goes under them, and it leaves
through the bottom rather than the side: beside the box it ran the depth of the
frame a dot away from it, which is a box with one edge drawn twice rather than a
wire leaving a box. It leaves three quarters of the way across, which is the
side it is headed for, rather than the middle, where a loop returning into that
same box puts its own head.

The loops go under for the same reason, and they keep a dot clear of the frames
at both ends: a line that starts on the bottom rule of a box has no visible
beginning, and a head whose tip lands on one is a thickening of the rule rather
than an arrow. Their lane is deeper than half the band, which left one dash
between the head and the rule it runs along.

**Every box has a dot of glass inside its own frame**, and the glass is as deep
as that makes it. It was 64 dots, the instrument's own display, and 64 is not a
measurement of anything this picture has to fit: four stacked boxes that each
keep a dot inside their rules, with a heading over them and the analog path's
foot under them, come to more.

**The two long wires under the graph say which way they run.** A loop goes back
and an output taken from the middle of the chain goes on; both leave a box
downwards, cross the width of the graph and rise at the far end, and dashes
against solid does not say which is which. So each carries a head half way along
it, with the rule running into its back and clear glass in front of its point.
The clear glass is the whole of it: a head with the line drawn on both sides is
a thickening of the line. That head is five dots along against the three a wire
arrives with, because an arrival's head has a frame standing behind it to say
what has been arrived at and this one is alone in open glass.

**The lane under the graph and the foot the analog path stands on are two
measurements**, and they were one constant. Every dot the loops were given was
therefore taken off the graph a second time at the bottom, and on the topology
that stacks four engines with the voices routed round the block that left each
box eight dots deep where nine is the least a box can be read at, which is also
the least `plate` will draw, so the picture came out empty. A test now measures
every box on every topology in every mode against that floor, and compares every
box's interior against the same box drawn on a screen of its own: what is inside
a frame has to be something the box itself put there.

**The settings are under the display, not beside it.** That is what the wider
glass cost, and it bought the better half of the trade: the ten topologies are
three columns of lit legends rather than a drop-down. A list of ten shows
whichever one is already chosen and hides the nine somebody is choosing between,
on the one control on this page where the choice is the picture above it. Three
columns rather than two because the band is wide and the page is better spent
across than down.

They take the whole band. `Room::spread` says the room left over rather than a
width, because the routing is the widest thing in the group and there is nothing
else on that line to give the room to. The sentence the specification records
about the two topologies that have a loop sits under the settings rather than
beside them, for the same reason.

#### Four at once, and what that costs the livery

All four engines are on the page, with the chain and the block's own settings
above them. The alternatives were four plates stacked, which is four algorithms'
worth of controls and a scroll to reach the fourth, and one at a time behind a
row of tabs, which is an engine you can see and three you have to remember on a
page whose whole subject is what the four are doing together.

The library publishes four colours per algorithm
([#22](https://github.com/MysteriousWolf/deepmind-midi/issues/22)): the chassis
around the controls, the face they sit on, the cap a finger moves, and the
accent. **How many of the four can be spent depends on how many units are on the
surface.** With one open at a time the page wore the lot. With four on it, four
liveries side by side would be a collage of other people's instruments in a
window whose whole argument is that it is one instrument. So the face is the
window's own plate and the cap the window's own metal, and an engine wears its
case and a hairline of its accent, which is enough to tell the reverb from the
distortion at arm's length.

The cap is the one measurement this window has spent and handed back. It was
refused first on the grounds that what carries a claim is the fill of the thing
that moves, so a control repainted to match a figure would break the one rule
that holds everywhere. The claim is *which* fill and not which colour, so a cap
can be repainted, but there is only room for it on a page with one unit on it.

#### Nothing is printed in a colour that cannot be read on what is under it

Half the 35 measured chassis are pale and half are dark, so no word on this page
names an ink of its own. It names the surface it lands on, the face, a case, a
band's strip or the recess the chain is cut into, and the ink is whichever of
the instrument's two that surface is further from, measured rather than judged.
A legend is that ink half way back towards the surface and then lifted until it
clears the [contrast threshold](#contrast-is-measured-not-judged); a reading
keeps the colour of its claim and is lifted the same way, because how light it
is is what contrast is made of and which colour it is is what it means.

Two things fell out of measuring rather than eyeballing, and neither was visible
in a screenshot:

- **A case is carried as far into the panel as it can go and still be printed
  on.** The measured chassis half way in puts the cream ones in the exact middle
  of the instrument's two inks, where the best either can manage is 4.45. So the
  carry is not a number somebody picked: it is the most of the livery that
  leaves the case readable, found by backing off towards the panel until it is.
- **A value is never printed on a case.** On the palest of the 35, an amber
  claim and a green one both have to be lifted so far to be read that they
  arrive as the same ink, which would spend the one distinction this editor
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
| Glass | `#d6e7cd` where the backlight enters falling to `#bfd3b6`, with the dots printed dark on it, three depths mixed towards `#101a12` |
| Metal | `#f4f5f8`, `#c9cdd6`, `#8e939f`; `#8e939f` is also the bar a plate's name is knocked out of |
| Ink | `#f2f2f4` for a title, `#a5a9b5` for a label, `#7d838f` for anything dim |
| Lamps | `#ffbe3d` on a way in and `#40d0e6` on a modulated control, and nothing else is saturated |

The two lamps are the instrument's own and so is the pairing. A DeepMind's panel
is dark and the only colour on it is the light through its buttons: amber on
every `EDIT`, cyan on `MOD`, `CHORD` and `CURVES`. This window uses the same two
for the same two jobs rather than inventing a third.

**A lamp is not the colour it is drawn in.** A cap is a translucent dome over an
LED, so what a hand sees is a rim below the lamp's own colour and a middle above
it, and the two together come out paler and softer than either. That matters the
moment the window says the same thing twice: the cyan on the `MOD` cap and the
cyan outlining a control something else moves are one statement, and drawn as
the raw `#40d0e6` the second is a harder, darker teal sitting next to the first.
So `style::modulated` is the lamp as the cap shows it — `cap::glow`, which is
the rim and the middle halved — and everything that means *what the cyan cap
means* is drawn in that. The instrument's own colour is still the one thing
written down; this is what the rubber does to it.

**The claim ring goes through the same call**, for the same reason and one more:
the ring stands in the band a backlit cap already has between its bezel and its
hot middle, so it is a band of *lamp*, and drawn at the raw colour it was the
one saturated thing on a panel of soft ones — a green printed on the cap rather
than lit from under it.

**And the press that opens a section is ringed rather than lit.** A `DeepMind`
leaves every `EDIT` lit the whole time it is powered, which is true and which
made ten amber rectangles the loudest thing in a window whose every other press
is off. The ring says which row the press is in at a tenth of the ink, and the
lamp stays what a press carrying a *value* spends.

The effect panels are the exception, and a deliberate one: their colours are
measured from the manual's own figures, four per algorithm, and a plate carrying
the chorus panel's own colours is telling the truth about what it is editing.
26.3 publishes them, and they are spent as an identity rather than as a finish.
Nothing else in the editor gets a colour for being itself: fourteen groups in
fourteen hues is decoration pretending to be information.

### Contrast is measured, not judged

A colour that is right in a palette and unreadable on a panel is not right, and
half the measured chassis are pale. So `style.rs` carries the measurement as
well as the colours:

| | |
| --- | --- |
| `contrast` | the ratio the web has used since anybody measured it: both colours' relative luminance, larger over smaller, lifted by a twentieth so black on black is 1 |
| `READABLE` | 4.5, the threshold for text at the sizes this window sets it in, written down once rather than judged per panel |
| `ink_on` | whichever of the instrument's two inks a surface is further from, by distance rather than by a threshold, because a chassis half way between them is exactly where a threshold flips on a rounding error |
| `legible` | an ink moved the least it has to be to clear `READABLE` on what it is printed on, towards an ink rather than replaced by one, so an amber claim stays amber |

Relative luminance and not a mean of the channels: a saturated green is bright
and a saturated blue of the same numbers is not, and an average puts dark ink on
the second one.

The effects page is drawn under all four, and a test walks every word it prints
against every surface it can land on: the face, the recess, and the case in all
35 of its liveries. Both findings above came out of that test rather than out of
a screenshot.

## Type

Two faces, because the instrument has two voices.

- **Mono** for abbreviations, raw values and anything the synthesizer's own
  display would show. Tabular figures required.
- **Sans** for titles, group headings and the rest of the application.

Sizes are 3u for a label, 3.5u for a readout, 4.5u for a group heading. Nothing
is bold except a group heading and the project's own name, and nothing is
italic.

**The sans is the mark's.** `docs/banner.svg` outlines the name from Liberation
Sans Bold so that it renders identically wherever the file is shown. A window
cannot outline anything, so it asks for that family by name, and for the
metrically compatible face other platforms ship under a different one, Arial,
which Liberation Sans is a clone of. A machine with neither falls back to its
own sans: the letters stay readable and the proportions are somebody else's.

Carrying the file in the binary is what would make both builds identical on
every machine, and it is a decision about a licence and about a megabyte in a
plugin bundle rather than a line of code. Until it is taken, the family is named
in one place, `control-ui/src/style.rs`, beside the palette.

**The name is not set at all**: it is `docs/wordmark.svg`, the banner's own
outlined letters in the metal of a fader cap, with the banner's own five slices
through them. Nothing about it depends on which bold sans a machine has, and the
slices are in the path's coordinates rather than in points, so they are cuts at
whatever height the window gives the file.

## The window is a panel, and so is its chrome

The port picker, the buttons and the status bay are toolkit widgets, and they
are still on the instrument:

- **The ground is the panel's gradient**, `#282c36` falling to `#15181e`, which
  is what `logo.svg` fills its case with. A flat dark window is the bottom of
  the panel stretched over all of it, which is the one part that is not lit.
- **A button with a word in it is the panel with a metal rim**, not a filled
  slab. A row of filled slabs is the brightest thing on a dark window, and the
  brightest thing here has to be a fader cap. Pressing lights the rim, the way a
  section button lights. **A button whose whole face is a mark has no rim at
  all**: the mark is already a shape on the panel, and what a hand gets back is
  the panel lifting under the pointer.
- **Anything chosen from a list is a recess**, closed and open alike: the track
  of a fader, the field a name is typed in, and the picker a port is chosen from
  are the same cut into the same panel.
- **A panel of words is the face plate** a rack of slots sits on, so that what
  the window says about the instrument sits on the instrument.

All of it comes from `materials()`, so restyling the chrome is the same one file
as restyling a fader.

### The foot of the window is the status bar

Three things are true of the window rather than of the page in it: what is under
the pointer, what can be asked of the instrument, and which way up the displays
are. They are along the foot, with the presses at the right-hand end. `Who` asks
the instrument what it is, which settles the firmware every value table in the
window is read from. `Read` asks it for its edit buffer, which turns this
window's claims into the synthesizer's facts. And the third is the display.

**The display press is a display.** A screen the size of a character showing
itself the way it is about to be says which way up they will be without being
read, and says it in the one material the press is about. It is the only display
in the window that does not take its glass from the theme, because the whole
point of it is that it is the other way round.

**Every press in the chrome is a mark, and nothing else.** Ask who is there is a
question mark; read the edit buffer is an arrow coming down into a tray; look
for ports again is a magnifier; open one is an empty `DIN` socket, and put it
down is the same socket with a plug in it, which is the connector this whole
application arrives through. They are dots, because everything small in this
window is dots, and they are stencilled on the panel rather than lit on glass.
The one press that is a display is the one whose subject is the display.

Nine dots square and not seven: seven is the cell a character stands in, and
these are not characters. Nine is the smallest odd grid with a middle dot, a dot
either side of it and a dot either side of those, which is what a circle, an
arrow and a plug all need before they stop being suggestions.

**Two presses that do the same thing to the same port differ by their mark.** An
open port is a socket with a plug seated in it and a closed one is a socket with
nothing in it, which is a fact about the cable rather than an instruction, and
reads at a glance the way a lit lamp does. Rescan is a magnifier rather than a
circular arrow, because at nine dots three quarters of a ring with a stub on the
end of it is a broken circle with specks round it, and what the press does is go
and look.

**A mark wears no rim, and what it does is said in the footer.** A rim drawn to
the height of a word puts a nine-dot mark in the middle of it with four points
of panel above and below, and a mark scaled to a box built for type is a mark
that is never the size it was drawn at. So the presses are the size of what is
drawn on them, and what a hand gets back is light rather than a frame: the panel
lifts to the plate under the pointer and dips to the recess while it is held.
The word goes to the footer, where it is a whole sentence rather than the one
word a rim had room for: `Ask the synthesizer what it is: device, firmware,
voices and channel.`

### What is known about the instrument is behind one press

Who answered, what firmware, what voice version, what channel, what the last
thing to happen was, and whatever went wrong. Four facts about a cable that
change perhaps twice a session do not need two rows of sentences under the
header on every page.

It is one `i` beside the port picker, and what it says opens under the pointer:
a name, a number, a number and a number lined up down a column, which is what
somebody comparing them against the back of an instrument is doing. Where nobody
has answered it says so, and says what this window is assuming meanwhile,
because the firmware is never *not* an answer: it is either the instrument's or
this window's, and which of those it is is the whole distinction this editor
turns on.

### The header is the name and the port

One line: the project's name, and what a port can be done with. The name is the
only thing on it that is not a control.

It used to be the whole of `docs/banner.svg`, the mark, a subtitle and a dark
panel between two wooden end cheeks. The banner is the picture that introduces
this project to somebody who has never seen it, and that is not the job of the
top of a window somebody has open all afternoon. The mark is the application's
icon, where an icon belongs.

**What is left is the banner's own name, as a file.** `docs/wordmark.svg` is cut
out of `docs/banner.svg`: the same outlined path, the same metal gradient down
it, and the same five horizontal slices, carried across as the mask they already
were and cropped to the box the path measures. The window renders it.

Drawing it instead meant a bold sans with five thin quads laid over it, which
came out as five hairlines landing where the renderer rounded them to, across
letters whose shapes were whichever bold sans the machine had. A vector file has
neither problem: the letterforms are the banner's, and the slices are in the
path's own coordinates, so a cut stays a cut at any height.

This is the one drawing in the window that is loaded rather than drawn. The
application's icon stays drawn, because it is a case with faders in it and it
has to turn over with the theme; a wordmark is a shape and has no surfaces in it
to light.

The press and the picker beside it sit on the name's baseline rather than in the
middle of the line it stands in. A word this size beside a twenty-four point
press, centred, is a press floating in the middle of a word.

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
newest at a fixed rate, so a view that throttled its own output would be
fighting the layer that already solved this.

## What this costs in iced

The renderer draws quads cheaply: a rounded rectangle with a border and a fill
is one primitive, so a fader is six of them (track, lit wall, two scale arms per
tick, cap, indicator) and the panel it sits on is one more. Arcs are not quads,
so a knob is a `canvas`, which is another reason they stay in the half of the
editor that needs them.

Text is the expensive part on a panel of 242 parameters, and most of it never
changes. Addresses and names are static; only the readings move.

A display is one quad per lit dot, which is why only the lit ones are drawn: the
panel's own screen is 132 by 100, and the ten of them together would be twenty
thousand quads a frame if the glass were drawn dot by dot. A drawing is a few
hundred, a line of writing is a few dozen, and the one expensive thing on any of
them is a heading in reverse video.

## Not this

- No brushed-metal photographs, no bitmap knobs, no screws in the corners. The
  mark is drawn with gradients and so is the editor.
- No drop shadow except the one the mark already uses under a cap.
- No hue as the only difference between two states, anywhere.
- No control that is inert but looks live.

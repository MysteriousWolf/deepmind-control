# Interface

What the editor looks like, why, and the numbers to build it from. The views
themselves are stages 2 and 3 of [the plan](plan.md); this is what they are
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

## The vocabulary is the library's

`deepmind-midi` draws all 35 effect panels from `spec/`, and those drawings
already settle what a control on this instrument looks like. The editor inherits
the anatomy rather than inventing a second one:

| | |
| --- | --- |
| Abbreviation above | Mono, as the synthesizer's own display writes it: `DCY`, `PDY`, `HiSvFreq` |
| Control in the middle | Fader, knob, switch, selector or readout |
| Title below | The same parameter written out, for a panel with room to be readable |
| Modulation dot | A small mark at the top right of a slot the modulation matrix reaches |
| Column pitch | One slot per column, filled left to right, wrapping onto a second row |

Two of those are worth keeping even though a bigger screen does not need them.
The abbreviation is what is printed on the instrument, so a player looking
between the two reads the same word twice. The dot is the only thing on the
panel that says a parameter can be moved by something other than a hand.

## Fourteen panels, one at a time

Two hundred and forty-two parameters do not fit on a screen, and the instrument
does not put them on one surface either: a player presses a section and the
display becomes that section. So does this. A bar of fourteen tabs sits above
the rack, it does not scroll with it, and the panel below it is the one section.

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
- The address above each slot is the parameter's NRPN number, which is also its
  byte offset in a dump: one number is both its name on the wire and where it
  lives in memory.

### Knob

Used where the source uses one, which today means the effect panels, so that
half of the editor keeps looking like the figures it was drawn from. A 270
degree arc open at the bottom, a body, a pointer, and a tick ring when the
parameter selects rather than sweeps.

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

### Envelope

Four faders and the shape they make, drawn above them. An envelope is the one
group whose meaning is a picture, and four numbers that do not draw it are four
numbers.

## Colour is meaning, not decoration

The chassis is monochrome. Panel, recess, metal and ink carry the whole
interface, and every one of those comes from the mark:

| | |
| --- | --- |
| Panel | `#282c36` to `#15181e`, the gradient the mark uses |
| Seam and lip | `#000` at half, and `#454b58` |
| Recess | `#05070a` through `#11141a` to `#242932`, lit along its lower wall with `#7d838f` |
| Metal | `#f4f5f8`, `#c9cdd6`, `#8e939f` |
| Ink | `#f2f2f4` for a title, `#a5a9b5` for a label, `#7d838f` for anything dim |
| Lamp | `#ffb95c`, and nothing else is saturated |

The effect panels are the exception, and a deliberate one: their colours are
measured from the manual's own figures, four per algorithm, and a host that
draws a chorus in the chorus panel's colours is telling the truth about what it
is editing. Those come from the library when it publishes them. Nothing else in
the editor gets a colour for being itself: fourteen groups in fourteen hues is
decoration pretending to be information.

## Type

Two faces, because the instrument has two voices.

- **Mono** for abbreviations, raw values and anything the synthesizer's own
  display would show. Tabular figures required.
- **Sans** for titles, group headings and the rest of the application.

Sizes are 3u for a label, 3.5u for a readout, 4.5u for a group heading. Nothing
is bold except a group heading, and nothing is italic.

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

## Not this

- No brushed-metal photographs, no bitmap knobs, no screws in the corners. The
  mark is drawn with gradients and so is the editor.
- No drop shadow except the one the mark already uses under a cap.
- No hue as the only difference between two states, anywhere.
- No control that is inert but looks live.

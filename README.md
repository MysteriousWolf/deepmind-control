<p align="center">
  <img src="https://raw.githubusercontent.com/MysteriousWolf/deepmind-control/main/docs/banner.svg" alt="deepmind-control" width="800">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-polish-orange" alt="Status: polish">
  <a href="https://github.com/MysteriousWolf/deepmind-control/actions/workflows/ci.yml"><img src="https://github.com/MysteriousWolf/deepmind-control/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/MysteriousWolf/deepmind-midi"><img src="https://img.shields.io/badge/built%20on-deepmind--midi-blue" alt="Built on deepmind-midi"></a>
  <a href="https://github.com/MysteriousWolf/deepmind-control/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-green" alt="License: Apache-2.0"></a>
</p>

An editor and librarian for the Behringer DeepMind, on the desktop and in a DAW.

Built on [`deepmind-midi`](https://github.com/MysteriousWolf/deepmind-midi), which
owns the protocol and refuses to own the port. This repository owns the port, and
the pixels.

```
DeepMind <--MIDI--> port <--bytes--> deepmind-midi <--events--> the interface
                     ^
              midir, or the DAW
```

**Status: an editor and a librarian, complete and being polished.** There is a
window with three surfaces, and it opens on the instrument's own front panel:
two rows of section plates with a screen between them, the controls the hardware
puts a fader under, and on every plate the press it calls `EDIT`. Every plate
has a screen of its own as well, which is the one place this window is not the
instrument — a dot matrix at the same pitch as the panel's own display, drawing
the shapes the oscillators are making, where the filter's corner is, the gates
the arpeggiator is opening, and the one picture no `DeepMind` can show: all
three envelopes at once. Behind those
presses is the editor, which lists the ports, opens one, finds out who is on it
and which firmware's value tables are true, reads the edit buffer, and edits all
242 parameters end to end — with the values the synthesizer reported drawn
differently from the ones this window put there. The library reads and writes `.syx` files, reads a bank
off the instrument with a progress bar and a stop button, browses what it found
in slot order, and loads any of it into the edit buffer as the difference rather
than as 242 parameters.

The parameters are fourteen panels, chosen from a section bar that carries each
section's own claim, and every panel is drawn from the library's table rather
than laid out by hand: complete first, beautiful group by group after. Every
panel that is not a rack now has its layout — the program's name, 17 parameters
holding a character each drawn as the one display the instrument shows them on;
the three envelopes, four faders and the shape they make; the modulation matrix,
eight routings read across as rows rather than down as twenty-four slots; the
control sequencer as one strip of thirty-two steps; and the effects as one
engine at a time, chosen from four tabs and drawn as the rack unit the manual
prints beside its algorithm — its case, its face, its knobs and their measured
colour — with its twelve bytes under the names that algorithm gives them, each
in the column and row the instrument's own FX page draws it in, over a display
of the chain the four engines are wired into, drawn from the library's edge
lists rather than out of the topology's name. What is left is polish, and then the plugin. See
[the plan](docs/plan.md) for what is being built and in what order.

It is dark, in the instrument's own colours: the palette is read off the mark in
`docs/logo.svg`, which draws the same panel, metal and wood, and one file holds
it for both builds.

Everything above runs against the library's simulated synthesizer, which is in
the port list as an ordinary choice, so none of it needs hardware.

## What it is

**A desktop application.** Owns its MIDI port, finds the synthesizer, edits it
live, and reads and writes `.syx` files. Every parameter the instrument exposes,
laid out as the instrument lays it out.

**A plugin**, after the desktop application, so that patches travel with a
project. AU and VST3, wrapped around a CLAP that ships too and arrives first.
Two modes:

- *Simple*, which selects programs and nothing else.
- *Advanced*, which is the desktop editing surface in a plugin window.

Editing is the same in both builds. Bank and librarian operations are desktop
only, because a preset pack is 35 kilobytes of SysEx and a plugin event buffer is
not a good place to put it.

## Hardware

DeepMind 6, 6X, 12, 12X, 12D and 12XD. Firmware 1.0 and 1.1, comms protocol
versions 6 and 7, all handled by the library.

Nothing here has been run against a synthesizer yet.

## Documentation

|  |  |
| --- | --- |
| [Plan](docs/plan.md) | What is being built, the decisions behind it, and the order |
| [Interface](docs/interface.md) | What a control looks like, what its three states are, and why |
| [Waiting](docs/waiting.md) | What has been asked of the library, and what each answer would change here |
| [Protocol](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/midi-spec.md) | Lives in the library, along with the specification it is generated from |
| [NOTICE](NOTICE) | Trademarks, and where the marks come from |

## Layout

```
crates/
  deepmind-host/    port ownership and the device loop. No interface.   written
  control-ui/       every view and widget. No runtime, no window.       generated
  control/          the desktop application, and the librarian.         editor and files
  control-plugin/   the plugin.                                         stage 6
```

`deepmind-host` is the only crate here that would be useful to somebody else, so
it is the only one carrying the prefix. The interface crate depends on
`iced_core` and `iced_widget` rather than on `iced`, so that one view layer
compiles under both the desktop runtime and baseview.

## Development

```
cargo run -p control
cargo run -p control -- pack.syx       # opens with that pack on the shelf
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

`cargo test --workspace` needs no hardware: the library ships a simulated
synthesizer, and `deepmind-host` puts it in the port list as an ordinary choice
alongside the real ones.

The plugin comes after the desktop application and is built and bundled
separately. The CLAP is what the other two formats are made from:

```
cargo nih-plug bundle control-plugin --release
```

The AU and the VST3 are wrapped around that CLAP with
[clap-wrapper](https://github.com/free-audio/clap-wrapper), which needs CMake
and a C++ toolchain. AU is macOS only.

Linux needs ALSA's development headers for midir (`libasound2-dev`, or
`alsa-lib` on Arch), and the usual X11 and GL development headers for baseview.
Running the window also wants `libxkbcommon-x11` at runtime, which a desktop
already has and a bare container does not.
macOS and Windows need nothing beyond the toolchain and CMake: midir talks to
CoreMIDI and WinMM, which are already there.

The iced version is pinned, and it is pinned to whatever the baseview adapter
supports rather than to the newest release. The two builds share a view layer
only for as long as they share an iced.

## License

Apache-2.0. See [LICENSE](LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.

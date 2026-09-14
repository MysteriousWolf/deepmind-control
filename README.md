<p align="center">
  <img src="https://raw.githubusercontent.com/MysteriousWolf/deepmind-control/main/docs/banner.svg" alt="deepmind-control" width="800">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-planning-orange" alt="Status: planning">
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

**Status: nothing here yet.** The plan is written down; the code is not. See
[the plan](docs/plan.md) for what is being built and in what order. Not usable.

## What it is

**A desktop application.** Owns its MIDI port, finds the synthesizer, edits it
live, and reads and writes `.syx` files. Every parameter the instrument exposes,
laid out as the instrument lays it out.

**A plugin**, so that patches travel with a project. AU and VST3 first, then
CLAP. Two modes:

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
| [Protocol](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/midi-spec.md) | Lives in the library, along with the specification it is generated from |

## Layout

```
crates/
  deepmind-host/    port ownership and the device loop. No interface.
  control-ui/       every view and widget. No runtime, no window.
  control/          the desktop application.
  control-plugin/   the plugin.
```

`deepmind-host` is the only crate here that would be useful to somebody else, so
it is the only one carrying the prefix. The interface crate depends on
`iced_core` and `iced_widget` rather than on `iced`, so that one view layer
compiles under both the desktop runtime and baseview.

## Development

```
cargo run -p control
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

The plugin is built and bundled separately. The CLAP is what the other two
formats are made from:

```
cargo nih-plug bundle control-plugin --release
```

The AU and the VST3 are wrapped around that CLAP with
[clap-wrapper](https://github.com/free-audio/clap-wrapper), which needs CMake
and a C++ toolchain. AU is macOS only.

Linux needs the usual X11 and GL development headers for baseview. macOS and
Windows need nothing beyond the toolchain and CMake.

The iced version is pinned, and it is pinned to whatever the baseview adapter
supports rather than to the newest release. The two builds share a view layer
only for as long as they share an iced.

## License

Apache-2.0. See [LICENSE](LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.

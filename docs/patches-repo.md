# Building `deepmind-patches`

Instructions for creating the repository of shared `DeepMind` sounds that
[`crates/control/src/catalogue.rs`](../crates/control/src/catalogue.rs) is
written to read. It is a separate repository from this one; nothing here is a
dependency of the editor, and the editor is not a dependency of it.

**The governing requirement is that it works without this program.** Somebody
who has never heard of `deepmind-control` should be able to open the repository
in a browser, understand what is in it, and download a `.syx` file that their
own librarian will load. Every decision below is downstream of that.

---

## 1. The repository

| | |
| --- | --- |
| Name | `deepmind-patches` |
| Visibility | Public |
| Licence | `CC0-1.0` for the repository's own files; each patch carries its own |
| README | Yes |
| `.gitignore` | None needed |

**Description** (the 350-character field):

> Shared patches for Behringer DeepMind synthesizers. One `.syx` and one `.toml` per sound, sorted by category, browsable on GitHub and usable with any librarian — no special software required. Presets, sound banks and audio demos, indexed by CI and published as releases.

**Topics**: `deepmind`, `behringer`, `synthesizer`, `presets`, `patches`,
`sysex`, `midi`, `sound-design`.

---

## 2. Layout

```
deepmind-patches/
  README.md
  CONTRIBUTING.md
  LICENSE
  taxonomy.toml                       the controlled vocabularies
  presets/
    Bass/
      Acid Growl - nyx.syx
      Acid Growl - nyx.toml
    Pad/
      Aurora Pads/                    a collection, one folder deep
        Aurora 1 - nyx.syx
        Aurora 1 - nyx.toml
        Aurora 2 - nyx.syx
        Aurora 2 - nyx.toml
      Slow Glass - someone.syx
      Slow Glass - someone.toml
  demos/
    Bass/
      Acid Growl - nyx.mp3
    Pad/
      Aurora Pads/
        Aurora 1 - nyx.mp3
  tools/
    validate/                         the CI validator, Rust
  .github/
    workflows/
      pr.yml
      release.yml
```

### One preset per file

A single program dump is about **291 bytes**, so a thousand presets is under
300 KB. One-per-file is what makes the directory listing *be* the catalogue: a
person lands on `presets/Bass/` in a browser and reads the sounds without a
tool, an index or a parser. A multi-program pack is an opaque 37 KB binary that
tells a browsing human nothing.

A pack is still how people move sounds around, so it stays — as an **export**,
built by the editor from a selection, rather than as how the repository stores
anything.

### Categories are folders, and they are checkable

The twelve the instrument itself has, which is the whole list and not a
preference:

```
Bass  Pad  Lead  Mono  Poly  Stab  SFX  Arp  Seq  Perc  Ambient  Modular
```

`None` and `User-1`…`User-4` are not folders. A patch whose category byte is one
of those is rejected with "set a category on the instrument first".

The category is **a byte inside the `.syx`** (parameter 240), so the folder can
be checked against the file. `presets/Bass/x.syx` whose program says `Pad` is a
build failure. A layout that verifies against its own content is worth more than
a tidy one.

### Collections are one subfolder deep

`presets/<Category>/<Collection>/<stem>.syx`. Never deeper. A preset sits either
directly in its category folder or in exactly one collection folder inside it.

A collection that spans categories uses **the same folder name under each
category it touches** — `presets/Pad/Aurora Pads/` and `presets/Lead/Aurora
Pads/` are one collection, and the index groups them by name. That keeps
browsing by category intact, which is the thing people actually do, without
making a collection un-nameable.

### Filenames

The stem is `{name} - {author}`, where `{name}` is the program's own name as
stored in the file:

```
Acid Growl - nyx.syx
Acid Growl - nyx.toml
```

Two people may both write an `Acid Growl`; the author makes them distinct
without anybody inventing a numbering scheme.

CI **computes** the expected stem from the TOML's `name` and `author` and
compares it to the actual one. It never parses the stem back into fields, so an
author called `Bits - Pieces` is not a problem. The slug rule: replace each of
`/ \ : * ? " < > |` and any control character with `_`, collapse runs of
whitespace to one space, trim. Nothing else is transformed — the names stay
readable, because a browser is the primary interface.

`{name}` must match the program's stored name exactly (16 characters, which is
all the instrument holds).

---

## 3. Every patch is normalised to A1

A program dump carries a bank and a program number — a contributed file
otherwise says "I am B07" for no reason but where its author happened to have
it, and another librarian will scatter a collection across eight banks.

So **every `.syx` in this repository is written as bank A, program 1**, and CI
rejects anything else. The slot is meaningless and the convention says so out
loud. A loader asks where to put it; `deepmind-control` does, and any librarian
worth using does too.

CI should rewrite rather than merely reject, if that is easy: a bot commit that
normalises the slot on a pull request is friendlier than a red build telling a
contributor to go and re-save.

---

## 4. `taxonomy.toml`

One file at the root defining every term that may be used. CI validates against
it and the editor reads it to build filter chips and to decide colour, so both
agree by construction and adding a term is one reviewable pull request.

```toml
# The vocabularies a patch may draw on. A term not in here fails validation,
# which is the point: free text cannot be filtered on, styled by, or counted.
#
# The instrument's own twelve categories are NOT here. They come out of the
# .syx file itself and are mirrored by the folder.

[genre]
ambient      = "Drifting, atmospheric, no fixed pulse."
techno       = "Four-to-the-floor, machine music."
house        = "Warmer and more swung than techno."
trance       = "Long builds, wide supersaws."
dnb          = "Drum and bass, jungle, breakbeat."
synthwave    = "Eighties pastiche, analogue nostalgia."
industrial   = "Harsh, metallic, distorted."
film         = "Score and trailer work."
game         = "Chiptune, game audio."
jazz         = "Electric piano, upright, brushed."
funk         = "Clavinet, slap, wah."
pop          = "Radio-facing, clean."
rock         = "Organ, distorted lead, power."
classical    = "Orchestral emulation."
experimental = "Anything with no other home."

[mood]
dark        = "Minor, low, closed."
bright      = "Open, high, major."
warm        = "Soft top end, gentle saturation."
cold        = "Clinical, glassy, digital."
aggressive  = "Loud, distorted, forward."
calm        = "Slow, quiet, still."
dreamy      = "Blurred, heavily modulated, reverberant."
tense       = "Unresolved, dissonant."
playful     = "Bouncy, rhythmic, light."
melancholic = "Sad without being dark."
epic        = "Large, wide, cinematic."

[timbre]
analogue = "Drifting, imperfect, warm."
digital  = "Exact, clean, bell-like."
gritty   = "Distorted, saturated, noisy."
clean    = "No distortion, no noise."
metallic = "Inharmonic, ringing."
glassy   = "Bright, brittle, transparent."
wooden   = "Hollow, damped, resonant."
vocal    = "Formant, throaty, speech-like."
noisy    = "Noise is a component, not a defect."
hollow   = "Square-ish, missing a fundamental."
rich     = "Many partials, detuned, thick."
thin     = "Few partials, narrow."

[role]
lead       = "Plays the tune."
bass       = "Plays the bottom."
pad        = "Held chords."
pluck      = "Short, pitched, decaying."
stab       = "One hit, chordal."
drone      = "One note, forever."
texture    = "Not pitched, not rhythmic."
fx         = "Sound effect rather than instrument."
key        = "Piano, electric piano, clav."
brass      = "Horn emulation."
string     = "Bowed emulation."
bell       = "Struck, inharmonic, ringing."
arp        = "Arpeggiator is the point."
sequence   = "Sequencer is the point."
percussion = "Drums, hits, clicks."
```

Four axes rather than one bag of tags, because they answer different questions:
`genre` is *what music is this for*, `mood` is *how does it feel*, `timbre` is
*what does it sound like*, `role` is *what does it do in a track*. A person
looking for a sound is usually holding one of those four in their head, and a
single flat tag list makes all four unsearchable at once.

---

## 5. The patch TOML

Next to each `.syx`, same stem. This carries **only what nothing else can
know** — everything derivable from the 242 parameter bytes is derived by CI and
never typed, because a typed fact goes stale the moment somebody tweaks the
patch.

```toml
# presets/Bass/Acid Growl - nyx.toml

name   = "Acid Growl"          # must equal the program's stored name
author = "nyx"                 # a person or a handle, not an email
about  = """
A resonant 303-ish bass that opens up under velocity. Play it low and short.
"""

licence = "CC0-1.0"            # SPDX identifier, or "All rights reserved"

genre  = ["techno", "industrial"]
mood   = ["dark", "aggressive"]
timbre = ["gritty", "analogue"]
role   = ["bass"]

# Optional from here down.

created  = 2026-09-21           # a TOML date, not a string
model    = "DeepMind 12"        # "DeepMind 6" | "DeepMind 12" | "DeepMind 12D"
firmware = "1.1.5"              # what it was made and checked on
demo     = "Acid Growl - nyx.mp3"   # a name, never a URL; see §7
source   = "Reworked from the factory A24, with permission."
tags     = ["303", "velocity"]  # free text; searched, never styled
```

### Required

`name`, `author`, `about`, `licence`, and at least one term in **one** of the
four vocabularies. Everything else is optional, because a repository that
refused a contribution for having no `firmware` is a repository with fewer
patches in it.

`model` is worth pressing for in review even though it is optional: a
unison-eight patch does something different on a `DeepMind 6`, and a person
downloading it deserves to know which instrument it was voiced on.

### Not in the TOML, ever

Anything CI can read out of the `.syx`. Do not accept a field for it, because a
field that can disagree with the file eventually will:

- the category (byte 240, and the folder)
- which of the four effect slots are loaded and with what algorithm
- whether the arpeggiator is on, its mode and rate
- unison voice count and detune
- whether the sequencer runs
- envelope times and LFO rates and shapes — *slow attack*, *long release*
- how many of the eight modulation routings are wired, and to what
- filter poles, resonance, bass boost
- the SHA-256 of the file, and how many programs it holds

---

## 6. The generated index

**Contributors never edit a central file.** One hand-maintained index means
every pull request touches the same file, so every pull request conflicts with
every other one — invisible at three patches, the whole maintenance burden at
three hundred. This is the lesson to take from HACS, whose hand-edited central
lists are tiny pointer files while everything rich is generated by CI and
published as data.

So CI walks `presets/**`, validates, derives, and writes `index.toml`:

```toml
[catalogue]
commit  = "599439d2c54e738464123ee96edbd6d94960f333"
built   = 2026-09-21T07:40:00Z
patches = 412

[[patch]]
id       = "bass/acid-growl-nyx"
name     = "Acid Growl"
author   = "nyx"
about    = "A resonant 303-ish bass that opens up under velocity."
category = "Bass"
file     = "presets/Bass/Acid Growl - nyx.syx"
sha256   = "9f2b1c…"
licence  = "CC0-1.0"
genre    = ["techno", "industrial"]
mood     = ["dark", "aggressive"]
timbre   = ["gritty", "analogue"]
role     = ["bass"]
demo     = "demos/Bass/Acid Growl - nyx.mp3"

# Derived, never typed:
effects  = ["Chorus", "Delay", "Reverb"]
arp      = false
unison   = 1
routings = 3
```

`commit` is what makes the whole thing coherent: an editor holding this index
fetches the files at **that exact revision** —
`raw.githubusercontent.com/<owner>/deepmind-patches/<commit>/presets/Bass/Acid%20Growl%20-%20nyx.syx`
— so somebody on last week's index gets last week's files rather than a mixture
of two states.

---

## 7. Audio demos

Worth having; the editor plays them in-app. One constraint governs the format:
**binaries in git are permanent**, so the cost is not today's size but every
version of every clip, forever.

| | |
| --- | --- |
| Format | **MP3, 128 kbps, stereo** |
| Length | about 10 seconds, 20 at the outside |
| Size | 250 KB hard cap, enforced by CI |
| Name | identical to the preset's stem, under `demos/` mirroring the tree |
| Content | the patch alone, no drums, no bed, no mastering |

Opus at 96 kbps sounds better in half the bytes, and it is the wrong choice
here: `.opus` does not reliably preview in Windows Explorer or play in every
DAW, and the whole premise is that a stranger can use this repository with
nothing installed. Stereo is not negotiable either — the chorus is a large part
of what somebody is auditioning, and mono misrepresents the instrument.

WAV and FLAC are out at roughly 10 MB and 5 MB per minute.

**A demo is referenced by name, never by URL.** At 250 KB × 1000 patches the
repository reaches about 250 MB, which GitHub is fine with but which makes a
clone slow for somebody who only wants the `.syx` files. If that becomes the
complaint, demos move to a second repository and the editor's base URL changes —
a configuration change rather than a rewrite of a thousand files. Referencing by
URL now would forfeit that.

---

## 8. CI

### `tools/validate` — a small Rust binary

Use `deepmind-midi` from crates.io rather than reimplementing the format. The
validator then cannot drift from the editor, because both read the same parser.
Contributors do not need Rust — CI validates their pull request for them.

```toml
# tools/validate/Cargo.toml
[package]
name = "validate"
version = "0.1.0"
edition = "2024"

[dependencies]
deepmind-midi = "26.5"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
sha2 = "0.10"
```

Every check it runs, each of which should name the file and say what to do:

1. Every `.syx` has a `.toml` beside it, and the reverse.
2. The `.toml` parses, and has `name`, `author`, `about`, `licence`, and at
   least one vocabulary term.
3. Every vocabulary term appears in `taxonomy.toml`.
4. The `.syx` holds **exactly one** program dump.
5. It is bank A, program 1.
6. Its category byte is one of the twelve, and equals the folder it is in.
7. Its stored name equals the TOML's `name`.
8. The filename stem equals `slug("{name} - {author}")`.
9. The stem is unique within its folder.
10. Depth under a category is 0 or 1 folders, no more.
11. `licence` is an SPDX identifier or the exact string `All rights reserved`.
12. If `demo` is set, the file exists under `demos/` at the mirrored path, is an
    MP3, and is at most 250 KB.
13. No file outside `presets/`, `demos/`, `tools/`, `.github/` and the root
    documents.

### `.github/workflows/pr.yml`

```yaml
name: Check
on:
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Validate every patch
        run: cargo run --manifest-path tools/validate/Cargo.toml -- --strict
```

### `.github/workflows/release.yml`

```yaml
name: Release
on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: write

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Validate, then build the index
        run: |
          cargo run --manifest-path tools/validate/Cargo.toml -- --strict
          cargo run --manifest-path tools/validate/Cargo.toml -- \
            --index index.toml --commit "$GITHUB_SHA"

      - name: Bundle
        run: |
          tar czf patches.tar.gz presets
          tar czf demos.tar.gz demos || true

      - name: Publish
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          TAG="$(date -u +%Y.%m.%d)-${GITHUB_RUN_NUMBER}"
          gh release create "$TAG" \
            --title "$TAG" \
            --notes "$(cargo run --manifest-path tools/validate/Cargo.toml -- --summary)" \
            index.toml patches.tar.gz demos.tar.gz
```

Dated tags rather than a rolling release, so that history is history, and
GitHub's own `latest` still points at the newest one.

---

## 9. What the editor fetches

The contract, so that both sides can be written independently:

| | |
| --- | --- |
| Check for updates | `GET https://github.com/<owner>/deepmind-patches/releases/latest/download/index.toml` |
| Everything, once | `…/releases/latest/download/patches.tar.gz` |
| Demos, on demand | `…/releases/latest/download/demos.tar.gz` |
| One file, pinned | `https://raw.githubusercontent.com/<owner>/deepmind-patches/<commit>/<file>` |

Release asset URLs are **not** the GitHub API, which matters: the
unauthenticated API allows 60 requests an hour per address, and an editor
crawling a repository tree would spend that on startup. A release download is a
plain redirect to a CDN, needs no token, and is not rate-limited in that way.

The bundle exists precisely *because* the repository is one file per patch: a
thousand patches is a thousand requests without it and about 300 KB with it.
One-per-file for people, one archive for programs.

---

## 10. Contributing, as the guide should describe it

1. Save the patch from your instrument or from `deepmind-control`.
2. Name it on the instrument — those sixteen characters become the filename and
   the index entry.
3. Set its category on the instrument. That is the folder it goes in.
4. Drop the `.syx` into `presets/<Category>/`, write the `.toml` beside it.
5. Optionally record ten seconds of it and put the MP3 under `demos/`.
6. Open a pull request. CI tells you what is wrong, by file and by line.

Three things the guide has to say plainly, because each is somebody else's
problem arriving in the issue tracker otherwise:

- **A patch is not a trademark.** Sounds named after a record, a film or another
  manufacturer's instrument get renamed at review. Say so before the pull
  request, not after.
- **Only contribute what is yours to give.** A patch lifted from a commercial
  soundbank is not.
- **The licence field is yours and is carried, not interpreted.** This repository
  prints what you wrote and does not decide what anybody may do with your
  sounds. `CC0-1.0` is the suggested default and is not imposed.

---

## 11. Seeding it

An empty repository attracts nothing. Before announcing it, put in thirty to
fifty patches across at least eight of the twelve categories, with demos, from
one or two authors. That is enough for the search, the filters and the styling
to be worth looking at, and enough that the first stranger to arrive can tell
what a good contribution looks like by reading one.

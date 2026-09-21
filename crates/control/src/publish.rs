//! Sharing a sound: the pair of files a pull request is made of, and how to
//! write them.
//!
//! A patch in
//! [`deepmind-patches`](https://github.com/MysteriousWolf/deepmind-patches) is
//! two files with the same stem in the folder of its category — the `.syx`
//! saved off the instrument, and a `.toml` saying what a person cannot work out
//! from the bytes. This writes both, laid out the way the repository wants
//! them, so that the folder it produces can be copied straight over a checkout.
//!
//! # Nothing derivable is asked for
//!
//! The name, the category, the effects, the arpeggiator, the unison count and
//! everything else about the sound come off the 242 program bytes, and the
//! repository's CI derives them again when it builds the index. So this asks
//! for exactly what nothing can work out: **a maker, a sentence, a licence and
//! at least one vocabulary term**, plus an optional collection and a drawing.
//! The same five the repository's own validator insists on.
//!
//! Asking for a category would be worse than useless: the folder has to match
//! the byte in the file or CI rejects it, so the byte decides and a person who
//! wants a different one sets it on the instrument.
//!
//! # The stem is computed, never typed
//!
//! `slug::stem(name, author)` is the repository's own rule, so the filename
//! this writes is the filename the validator expects to compute. A person who
//! renames the file by hand breaks it; one who never sees the rule cannot.
//!
//! # And there is a note beside them
//!
//! Four steps, in a file called `HOW TO SHARE THIS.md` in the folder somebody
//! chose. A person who has just drawn a seven-dot icon should not have to go
//! and find a contributing guide to learn what happens next, and four lines is
//! what it actually takes.

use std::fs;
use std::path::{Path, PathBuf};

use deepmind_midi::program::Program;
use deepmind_patches::{Category, Icon, PatchMeta, slug};

use crate::app::Publishing;

/// The repository a shared sound is bound for.
const REPOSITORY: &str = "https://github.com/MysteriousWolf/deepmind-patches";

/// Writes the pair, and the note beside them.
///
/// Returns where they went, for saying out loud.
///
/// # Errors
///
/// Whatever the disk refused, or whatever the program could not be packed with.
pub fn write(
    root: &Path,
    program: &Program,
    category: Category,
    said: &Publishing,
) -> Result<String, String> {
    let name = program.name().as_str().trim().to_owned();
    if name.is_empty() {
        return Err("the sound has no name: give it one on the instrument".to_owned());
    }
    let author = said.author.trim();
    let stem = slug::stem(&name, author);
    let folder = folder_for(root, category, said);
    fs::create_dir_all(&folder).map_err(|error| format!("{}: {error}", folder.display()))?;

    let bytes = crate::shelf::patch_to_syx(program).map_err(|error| error.to_string())?;
    let syx = folder.join(format!("{stem}.syx"));
    fs::write(&syx, &bytes).map_err(|error| format!("{}: {error}", syx.display()))?;

    let toml = folder.join(format!("{stem}.toml"));
    fs::write(&toml, meta_toml(&name, said)?)
        .map_err(|error| format!("{}: {error}", toml.display()))?;

    let note = root.join("HOW TO SHARE THIS.md");
    fs::write(&note, instructions(&stem, category, said))
        .map_err(|error| format!("{}: {error}", note.display()))?;

    Ok(folder.display().to_string())
}

/// Where in the laid-out tree this patch belongs.
///
/// `presets/<Category>/`, and one folder deeper when it names a collection,
/// which is the repository's whole layout: never deeper than that, and a
/// collection that spans categories uses the same folder name under each one
/// it touches.
fn folder_for(root: &Path, category: Category, said: &Publishing) -> PathBuf {
    // `name()` and not `label()`: the first is the folder the repository
    // keeps a category in (`Pads`, `Sound Effects`) and the second is the
    // short word the instrument's own display prints (`Pad`, `SFX`). A patch
    // written into the wrong one is a patch CI rejects.
    let mut folder = root.join("presets").join(category.name());
    let collection = slug::slug(said.collection.trim());
    if !collection.is_empty() {
        folder = folder.join(collection);
    }
    folder
}

/// The `.toml`, as the repository reads it.
///
/// Built as the library's own [`PatchMeta`] and serialised by the library's own
/// serialiser, rather than printed by hand here. A hand-written TOML is a
/// second opinion about the format, and this file exists precisely so there is
/// only one.
fn meta_toml(name: &str, said: &Publishing) -> Result<String, String> {
    let icon = Icon::from_rows(said.icon);
    let mut meta = PatchMeta {
        name: name.to_owned(),
        author: said.author.trim().to_owned(),
        version: 1,
        about: said.about.trim().to_owned(),
        licence: said.licence.trim().to_owned(),
        genre: Vec::new(),
        mood: Vec::new(),
        timbre: Vec::new(),
        role: Vec::new(),
        // A blank grid is not a drawing. Left out, the patch falls back to its
        // category's icon, which is what the repository draws twelve of.
        icon: (!icon.is_blank()).then_some(icon),
        created: None,
        model: None,
        firmware: None,
        source: None,
        tags: Vec::new(),
        demos: Vec::new(),
        unknown: toml::Table::default(),
    };
    for (axis, term) in &said.terms {
        let into = match axis.as_str() {
            "genre" => &mut meta.genre,
            "mood" => &mut meta.mood,
            "timbre" => &mut meta.timbre,
            "role" => &mut meta.role,
            _ => continue,
        };
        into.push(term.clone());
    }
    toml::to_string_pretty(&meta).map_err(|error| error.to_string())
}

/// The four steps, written beside the files.
fn instructions(stem: &str, category: Category, said: &Publishing) -> String {
    let where_to = match slug::slug(said.collection.trim()) {
        collection if collection.is_empty() => format!("presets/{}", category.name()),
        collection => format!("presets/{}/{collection}", category.name()),
    };
    format!(
        "# Sharing `{stem}`

Two files are beside this note, under `{where_to}/`:

- `{stem}.syx` — the sound, saved as bank A program 1, which is what the
  repository stores every patch as. The slot means nothing; a librarian asks
  where to put it.
- `{stem}.toml` — what you said about it. Everything else the library needs is
  worked out from the program bytes when the index is built, so there is
  nothing else to fill in.

## What to do with them

1. Fork and clone <{REPOSITORY}>.
2. Copy the `presets/` folder beside this note over the clone's own. It is laid
   out the way the repository is, so nothing has to be moved.
3. Commit, push, and open a pull request.
4. CI checks the pair and tells you what to fix. The usual ones: the folder has
   to match the category stored in the file, the filename has to be the one
   computed from the name and the maker, and every vocabulary term has to be in
   `taxonomy.toml`.

## If you want a demo to go with it

Record up to twenty seconds of the sound alone — no drums, no bed, no
mastering — as an MP3 at 128 kbps in stereo, and put it at
`demos/{where_to_demo}/{stem}.mp3` in the clone. It has to be under 250 KB.

## Sharing a family of them

Give every sound in the family the same collection name in this window before
writing it out, and they land in one folder together. A family that spans
categories uses that same folder name under each category it touches; the
library groups them by name.
",
        where_to_demo = where_to.strip_prefix("presets/").unwrap_or(&where_to),
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_patches::{Category, slug};

    use crate::app::Publishing;

    use super::{folder_for, meta_toml};

    #[test]
    fn a_patch_lands_where_the_repository_keeps_it() {
        let root = std::path::Path::new("/tmp/out");
        let plain = Publishing::default();
        assert!(
            folder_for(root, Category::Bass, &plain).ends_with("presets/Bass"),
            "a patch with no collection goes straight in its category"
        );
        let family = Publishing {
            collection: "Aurora Pads".to_owned(),
            ..Publishing::default()
        };
        assert!(
            folder_for(root, Category::Pad, &family).ends_with("presets/Pads/Aurora Pads"),
            "a collection is one folder deeper and never more, under the folder \
             name rather than the instrument's short label"
        );
    }

    #[test]
    fn the_stem_is_the_repositorys_own_rule() {
        // Computed by the library rather than printed here, so that the file
        // this writes is the file the validator expects to find.
        assert_eq!(slug::stem("Acid Growl", "nyx"), "Acid Growl - nyx");
    }

    #[test]
    fn a_blank_grid_is_not_a_drawing() {
        // An icon nobody drew is left out, so the patch falls back to the one
        // its category already has. Writing seven empty rows would be claiming
        // a picture that says nothing.
        let said = Publishing {
            author: "nyx".to_owned(),
            about: "A sound.".to_owned(),
            licence: "CC0-1.0".to_owned(),
            ..Publishing::default()
        };
        let written = meta_toml("Acid Growl", &said).expect("serialises");
        assert!(
            !written.contains("icon"),
            "a blank icon should not be written at all"
        );

        let drawn = Publishing {
            icon: [0, 0b010_0010, 0, 0b100_0001, 0b011_1110, 0, 0],
            ..said
        };
        let written = meta_toml("Acid Growl", &drawn).expect("serialises");
        assert!(written.contains("icon"), "a drawn icon is written");
    }

    #[test]
    fn nothing_derivable_is_ever_asked_for() {
        // The five the repository's validator insists on, minus the two that
        // come off the program bytes. A field added here that the index already
        // derives would be a field that can disagree with the file.
        let empty = Publishing::default();
        let missing = empty.missing();
        assert_eq!(missing.len(), 4, "{missing:?}");
        let filled = Publishing {
            author: "nyx".to_owned(),
            about: "A sound.".to_owned(),
            licence: "CC0-1.0".to_owned(),
            terms: vec![("role".to_owned(), "bass".to_owned())],
            ..Publishing::default()
        };
        assert!(filled.missing().is_empty());
    }
}

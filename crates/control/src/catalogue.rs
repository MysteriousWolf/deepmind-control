//! A shelf somebody else filled: the index a repository of shared presets
//! carries, and how it is read.
//!
//! The librarian holds what is on this machine. This is the other half of the
//! same question — the packs other `DeepMind` owners have made — and it is the
//! part that has to be designed before any of it is written, because everything
//! difficult about it is a decision rather than a line of code: what a pack says
//! about itself, who says it, what a reader is entitled to believe, and what
//! happens to a file that lies.
//!
//! Nothing here opens a file, talks to a port or reaches a network. It is the
//! format and the questions that can be asked of it, the way [`Shelf`] is the
//! programs and not the disk. See [the plan](../../../docs/presets.md) for what
//! is meant to stand on it.
//!
//! [`Shelf`]: crate::Shelf
//!
//! # It is a git repository, and the index is a text file in it
//!
//! Not a service, not an account, not a database. A repository of `.syx` files
//! with one text file listing them, which means the whole of it is `git clone`,
//! the history of who changed what is the history, a contribution is a pull
//! request that a person reads, and somebody who wants the packs and not this
//! application can have them.
//!
//! The index is that text file. It is records separated by blank lines and
//! `key: value` inside a record, because it is written and reviewed by people:
//! a pull request adding a pack should be eight readable lines and a `.syx`
//! file, and a diff in it should be a diff anybody can review. A serialised
//! object graph would need a code generator on one side and a schema on the
//! other to add a sentence to a description.
//!
//! ```text
//! catalogue: DeepMind presets
//! updated: 2026-09-21
//!
//! pack: aurora-pads
//! name: Aurora Pads
//! author: somebody
//! file: packs/aurora-pads/aurora-pads.syx
//! programs: 32
//! tags: pad, ambient, slow
//! licence: CC0-1.0
//! sha256: 9f2b1c...
//! about: Eight slow pads, and twenty-four variations on them.
//! ```
//!
//! The pack itself is a `.syx` file and nothing else. The only file format here
//! is the protocol's, which is the rule the librarian is already under: a pack
//! downloaded from a repository opens on the same shelf as a pack read off a
//! synthesizer, in the same reader, and a pack this application saves can be
//! contributed to one without being converted into anything.
//!
//! # Reading it is lenient, and that is a decision rather than a shortcut
//!
//! An index is untrusted input in the same sense a `.syx` file is: it was
//! written by somebody else, possibly by a newer version of whatever writes it.
//! So a key this reader does not know is skipped rather than refused, because
//! the alternative is a repository that cannot add a field without breaking
//! every editor already installed; and a record that is missing what a pack
//! needs is dropped and counted, because one malformed entry in a list of three
//! hundred is one pack nobody can download rather than a catalogue nobody can
//! open.
//!
//! What is counted is said out loud, the way the shelf says how many frames of
//! a file it could not read. A catalogue quietly holding 297 of 300 is worse
//! than one that refuses to open.
//!
//! # Nothing in an index is trusted with a path
//!
//! The one place lenient reading is not the rule. An entry names the file its
//! pack lives in, that name is going to be joined onto a directory on somebody
//! else's machine, and `../../../.ssh/id_ed25519` is a perfectly ordinary
//! string. So [`Entry::file_within`] is the only way to turn one into a path:
//! relative, no parent segments, no root, no prefix, or nothing at all.
//!
//! Which is also why an [`Entry::id`] is checked against a charset rather than
//! taken as read. It is what a cache directory and a link to the repository are
//! named after, and a pack calling itself `../latest` is a pack that would be
//! naming somebody else's.
//!
//! # What this cannot do yet, and what it is shaped for
//!
//! It cannot fetch. There is no HTTP client in this workspace and putting one
//! in is a decision about a dependency, a TLS stack and an update policy, none
//! of which should be made in passing. Meanwhile a catalogue is a directory:
//! clone the repository, point the application at it, and everything below
//! works, which is also how the fetching version is going to be tested.
//!
//! It cannot verify. An entry carries [`sha256`](Entry::sha256) and nothing
//! here hashes anything, because a hash is worth having the day a file arrives
//! over a wire rather than the day it is read off a disk somebody already
//! trusts. The field is carried now so that an index written today is one a
//! verifying reader can check tomorrow; until then it is a fact about the pack
//! and not a promise this application has kept, which is
//! [written down](../../../docs/todo.md) rather than remembered.

use std::path::{Component, Path, PathBuf};

/// What a catalogue calls itself, and what is in it.
///
/// Built by [`parse`](Catalogue::parse) and never edited afterwards: this is a
/// reading of somebody else's list, and an editor that could change it would be
/// an editor that had opinions about the repository it was reading.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalogue {
    /// What the repository calls itself, where it says.
    name: Option<String>,
    /// When it last changed, in whatever the repository writes there.
    ///
    /// A string rather than a date. It is printed beside the name and never
    /// compared against anything, and a reader that parsed it would be a reader
    /// that refused a catalogue over a timezone.
    updated: Option<String>,
    /// What it is a catalogue of, in a sentence.
    about: Option<String>,
    /// The packs, in the order the index lists them.
    ///
    /// The index's own order rather than sorted, for the reason a shelf keeps a
    /// pack's order: it is the repository's list, and whoever maintains it put
    /// the interesting things at the top.
    entries: Vec<Entry>,
    /// The records that were dropped, and why.
    complaints: Vec<Complaint>,
}

impl Catalogue {
    /// Reads an index.
    ///
    /// Never fails. A file that is not an index at all reads as a catalogue
    /// with nothing in it and one complaint per record it could not make sense
    /// of, which is what the caller is going to say out loud either way.
    #[must_use]
    pub fn parse(index: &str) -> Self {
        let mut catalogue = Self::default();
        for record in records(index) {
            catalogue.take(&record);
        }
        catalogue
    }

    /// Takes one record, as either the header or a pack.
    fn take(&mut self, record: &Record) {
        if record.has(HEADER) {
            self.name = record.value(HEADER);
            self.updated = record.value("updated");
            self.about = record.value("about");
            return;
        }
        match Entry::of(record) {
            Ok(entry) => self.entries.push(entry),
            Err(missing) => self.complaints.push(Complaint {
                line: record.line,
                missing,
            }),
        }
    }

    /// Returns what the repository calls itself.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns when the repository says it last changed.
    #[must_use]
    pub fn updated(&self) -> Option<&str> {
        self.updated.as_deref()
    }

    /// Returns what it says it is a catalogue of.
    #[must_use]
    pub fn about(&self) -> Option<&str> {
        self.about.as_deref()
    }

    /// Returns every pack in it, in the index's own order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Returns the records that were dropped, and what each was missing.
    #[must_use]
    pub fn complaints(&self) -> &[Complaint] {
        &self.complaints
    }

    /// Returns whether there is nothing in it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the pack that calls itself `id`.
    #[must_use]
    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    /// Returns every pack `words` are anywhere in.
    ///
    /// The same search the shelf does, asked of the other list: one field, one
    /// case, and everything a pack says about itself in it. Empty words are
    /// everything, because a search box nobody has typed in is not a filter.
    #[must_use]
    pub fn search(&self, words: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches(words))
            .collect()
    }

    /// Returns every tag anything in the catalogue carries, and how many carry
    /// it.
    ///
    /// Sorted by how many, and then alphabetically, which is the order a list
    /// of them is worth drawing in: the tags a repository actually uses first,
    /// and the one-offs somebody invented for a single pack at the bottom.
    ///
    /// Read off the packs rather than declared by the repository. A tag list
    /// somebody maintains by hand is a tag list that disagrees with the packs
    /// the first time one is renamed.
    #[must_use]
    pub fn tags(&self) -> Vec<(&str, usize)> {
        let mut counted: Vec<(&str, usize)> = Vec::new();
        for tag in self.entries.iter().flat_map(|entry| &entry.tags) {
            match counted.iter_mut().find(|(known, _)| *known == tag) {
                Some((_, carrying)) => *carrying = carrying.saturating_add(1),
                None => counted.push((tag, 1)),
            }
        }
        counted.sort_by(|(one, carrying), (other, carried)| {
            carried.cmp(carrying).then_with(|| one.cmp(other))
        });
        counted
    }
}

/// One pack in a catalogue.
///
/// Everything here is what the index said, not what a file turned out to hold:
/// a pack that says it has 32 programs and holds 31 is a pack whose index is
/// wrong, and the count that matters is the one the shelf reports once it has
/// actually read the `.syx`. Nothing in this struct is checked against a file,
/// because no file has been opened.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    /// What the pack calls itself, once, forever, in a name a path can hold.
    ///
    /// The one identifier: what a cache is filed under and what a link to a
    /// pack names. Lowercase letters, digits and dashes, which is
    /// [checked](Entry::of) rather than assumed.
    id: String,
    /// What it is called, for a person.
    name: String,
    /// The file it lives in, relative to the root of the repository.
    ///
    /// Never joined onto anything except through [`file_within`](Self::file_within).
    file: String,
    /// Who made it, where the index says.
    author: Option<String>,
    /// What it is, in a sentence.
    about: Option<String>,
    /// What the index says it is worth being found by.
    tags: Vec<String>,
    /// How many programs it says it holds.
    programs: Option<usize>,
    /// What it is licensed under, in whatever the repository writes there.
    ///
    /// Carried and never interpreted. This application is not entitled to
    /// decide what somebody may do with somebody else's sounds, and a field it
    /// silently dropped would be a field a contributor thought they had filled
    /// in.
    licence: Option<String>,
    /// The digest the file is expected to have.
    ///
    /// Nothing here checks it. See the module's own note about what that is for
    /// and when it starts being worth anything.
    sha256: Option<String>,
}

impl Entry {
    /// Reads one record as a pack.
    ///
    /// # Errors
    ///
    /// Returns what the record was missing, for the caller to count and say
    /// out loud. A record is a pack when it names itself, says what it is
    /// called and says where it lives; everything else about a pack is optional
    /// because a repository that refused a contribution for having no tags is a
    /// repository with fewer packs in it.
    fn of(record: &Record) -> Result<Self, Missing> {
        let id = record.value(PACK).ok_or(Missing::Id)?;
        if !is_a_name(&id) {
            return Err(Missing::Id);
        }
        let name = record.value("name").ok_or(Missing::Name)?;
        let file = record.value("file").ok_or(Missing::File)?;
        Ok(Self {
            id,
            name,
            file,
            author: record.value("author"),
            about: record.value("about"),
            tags: record
                .value("tags")
                .map(|tags| listed(&tags))
                .unwrap_or_default(),
            programs: record
                .value("programs")
                .and_then(|count| count.parse().ok()),
            licence: record.value("licence").or_else(|| record.value("license")),
            sha256: record.value("sha256"),
        })
    }

    /// Returns what the pack calls itself.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns what it is called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns who made it, where the index says.
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Returns what it says it is.
    #[must_use]
    pub fn about(&self) -> Option<&str> {
        self.about.as_deref()
    }

    /// Returns what it is worth being found by.
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Returns how many programs the index says it holds.
    #[must_use]
    pub const fn programs(&self) -> Option<usize> {
        self.programs
    }

    /// Returns what the index says it is licensed under.
    #[must_use]
    pub fn licence(&self) -> Option<&str> {
        self.licence.as_deref()
    }

    /// Returns the digest the file is expected to have.
    #[must_use]
    pub fn sha256(&self) -> Option<&str> {
        self.sha256.as_deref()
    }

    /// Returns where the pack is, under `root`, or nothing at all.
    ///
    /// The only way a name out of an index becomes a path. Nothing at all for
    /// anything that is not a plain relative walk downwards: an absolute path,
    /// a path with a root or a prefix on it, or any `..` anywhere in it. A
    /// catalogue is somebody else's file, and the first thing a hostile one
    /// would try is to name a file outside the directory it was unpacked into.
    #[must_use]
    pub fn file_within(&self, root: &Path) -> Option<PathBuf> {
        let named = Path::new(&self.file);
        let walks_down = named.components().all(|part| {
            matches!(part, Component::Normal(_))
                && part.as_os_str() != Component::CurDir.as_os_str()
        });
        (walks_down && named.components().next().is_some()).then(|| root.join(named))
    }

    /// Returns whether `words` are anywhere in what the pack says about itself.
    ///
    /// Its name, who made it, what it says it is, its tags and what it calls
    /// itself. One field and one case, because somebody looking for a pad does
    /// not know which of those the word `pad` is in.
    #[must_use]
    pub fn matches(&self, words: &str) -> bool {
        if words.is_empty() {
            return true;
        }
        let words = words.to_lowercase();
        let said = [
            Some(self.name.as_str()),
            Some(self.id.as_str()),
            self.author.as_deref(),
            self.about.as_deref(),
        ];
        said.into_iter()
            .flatten()
            .any(|field| field.to_lowercase().contains(&words))
            || self
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&words))
    }
}

/// A record the reader dropped, and what it was missing.
///
/// Kept with the line it started on, because the person who is going to fix it
/// is looking at the file in an editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Complaint {
    /// The line the record started on, counting from one.
    pub line: usize,
    /// What it did not have.
    pub missing: Missing,
}

impl core::fmt::Display for Complaint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "line {}: {}", self.line, self.missing)
    }
}

/// What a record had to have and did not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Missing {
    /// No `pack`, or one that is not a name a path can hold.
    Id,
    /// No `name`.
    Name,
    /// No `file`.
    File,
}

impl core::fmt::Display for Missing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Id => "no `pack`, or one that is not a name a path can hold",
            Self::Name => "no `name`",
            Self::File => "no `file`",
        })
    }
}

/// The key that makes a record the catalogue's own rather than a pack.
const HEADER: &str = "catalogue";

/// The key that makes a record a pack.
const PACK: &str = "pack";

/// One record of an index: its keys, and where it started.
#[derive(Debug, Default)]
struct Record {
    /// The line the first key of it is on, counting from one.
    line: usize,
    /// Every key that was read, in the order they were written.
    ///
    /// A list rather than a map, because a record is eight lines and a map of
    /// eight things is a dependency and a hash for no gain. A key written twice
    /// keeps the first, which is the answer that makes an index editable by a
    /// tool without the tool having to know what is already in it.
    said: Vec<(String, String)>,
}

impl Record {
    /// Returns whether the record carries `key` at all.
    fn has(&self, key: &str) -> bool {
        self.said.iter().any(|(said, _)| said == key)
    }

    /// Returns what the record says `key` is.
    fn value(&self, key: &str) -> Option<String> {
        self.said
            .iter()
            .find(|(said, _)| said == key)
            .map(|(_, value)| value.clone())
    }
}

/// Walks an index, a record at a time.
///
/// A blank line ends a record and a line whose first character is a hash is a
/// comment, which are the two conventions every file of this shape has used.
/// A line with no colon in it is neither, and is dropped: an index is written by
/// people and a stray word in one is a typo rather than a reason to refuse the
/// other two hundred packs.
fn records(index: &str) -> Vec<Record> {
    let mut all = Vec::new();
    let mut record = Record::default();
    for (at, line) in index.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            if !record.said.is_empty() {
                all.push(core::mem::take(&mut record));
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if record.said.is_empty() {
            record.line = at.saturating_add(1);
        }
        record
            .said
            .push((key.trim().to_lowercase(), value.trim().to_owned()));
    }
    if !record.said.is_empty() {
        all.push(record);
    }
    all
}

/// Splits a comma-separated value into what it lists.
///
/// Trimmed, empties dropped, and nothing else: a tag is whatever somebody
/// wrote, in the case they wrote it in, because a reader that lowercased them
/// would be a reader that printed `12x` on a screen full of `12X`.
fn listed(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Returns whether `id` is a name this application will put in a path or a
/// link.
///
/// Lowercase ASCII letters, digits and dashes, starting with a letter or a
/// digit. It is deliberately narrower than what a filesystem would accept: this
/// name travels between three operating systems and into a URL, and the set
/// that survives all of that unchanged is a small one.
fn is_a_name(id: &str) -> bool {
    let plain = |character: char| character.is_ascii_lowercase() || character.is_ascii_digit();
    id.starts_with(plain)
        && id
            .chars()
            .all(|character| plain(character) || character == '-')
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "a failed expectation is the test failure"
    )]

    use super::{Catalogue, Missing};
    use std::path::Path;

    /// An index with one of everything in it.
    const INDEX: &str = "\
# The list itself.
catalogue: DeepMind presets
updated: 2026-09-21
about: Packs people have shared.

pack: aurora-pads
name: Aurora Pads
author: somebody
file: packs/aurora-pads/aurora-pads.syx
programs: 32
tags: pad, ambient, slow
licence: CC0-1.0
sha256: 9f2b1c
about: Eight slow pads.

pack: acid-lines
name: Acid Lines
file: packs/acid-lines/acid-lines.syx
tags: bass, pad
";

    #[test]
    fn an_index_is_its_header_and_its_packs() {
        let catalogue = Catalogue::parse(INDEX);

        assert_eq!(catalogue.name(), Some("DeepMind presets"));
        assert_eq!(catalogue.updated(), Some("2026-09-21"));
        assert_eq!(catalogue.entries().len(), 2, "and the two packs under it");
        assert!(catalogue.complaints().is_empty(), "with nothing dropped");

        let pads = catalogue.entry("aurora-pads").expect("the pack it names");
        assert_eq!(pads.name(), "Aurora Pads");
        assert_eq!(pads.author(), Some("somebody"));
        assert_eq!(pads.programs(), Some(32));
        assert_eq!(pads.licence(), Some("CC0-1.0"));
        assert_eq!(pads.tags(), ["pad", "ambient", "slow"]);
        assert_eq!(pads.about(), Some("Eight slow pads."));
    }

    #[test]
    fn a_key_this_reader_does_not_know_is_not_a_refused_catalogue() {
        // The whole point of reading leniently: a repository that adds a field
        // must not break the editors already installed.
        let catalogue = Catalogue::parse(
            "pack: later\nname: From A Later Format\nfile: packs/later.syx\nmood: hopeful\n",
        );

        assert_eq!(catalogue.entries().len(), 1);
        assert!(catalogue.complaints().is_empty());
    }

    #[test]
    fn a_record_that_is_not_a_pack_is_dropped_and_counted() {
        let catalogue = Catalogue::parse(
            "pack: nameless\nfile: packs/nameless.syx\n\npack: homeless\nname: Homeless\n",
        );

        assert!(catalogue.is_empty(), "neither of them is a pack");
        let said: Vec<Missing> = catalogue
            .complaints()
            .iter()
            .map(|complaint| complaint.missing)
            .collect();
        assert_eq!(said, [Missing::Name, Missing::File]);
        assert_eq!(
            catalogue.complaints().first().map(|first| first.line),
            Some(1),
            "and says which line to go and look at"
        );
    }

    #[test]
    fn nothing_in_an_index_gets_to_name_a_file_outside_the_catalogue() {
        // The one place this reader is not lenient. Everything else in an index
        // is words on a screen; this one is joined onto a path.
        let root = Path::new("/tmp/catalogue");
        let escaping = Catalogue::parse(
            "pack: hostile\nname: Hostile\nfile: ../../../.ssh/id_ed25519\n\npack: rooted\nname: \
             Rooted\nfile: /etc/passwd\n\npack: honest\nname: Honest\nfile: packs/honest.syx\n",
        );

        for id in ["hostile", "rooted"] {
            let entry = escaping.entry(id).expect("a pack the index lists");
            assert_eq!(
                entry.file_within(root),
                None,
                "{id} named a file outside the catalogue and was refused"
            );
        }
        let honest = escaping.entry("honest").expect("the one that walks down");
        assert_eq!(
            honest.file_within(root),
            Some(root.join("packs/honest.syx"))
        );
    }

    #[test]
    fn a_pack_that_calls_itself_something_a_path_cannot_hold_is_not_a_pack() {
        let catalogue = Catalogue::parse(
            "pack: ../latest\nname: Sneaky\nfile: packs/sneaky.syx\n\npack: Shouty Name\nname: \
             Shouty\nfile: packs/shouty.syx\n",
        );

        assert!(catalogue.is_empty(), "neither name survives a path");
        assert_eq!(catalogue.complaints().len(), 2);
    }

    #[test]
    fn a_search_reads_everything_a_pack_says_about_itself() {
        let catalogue = Catalogue::parse(INDEX);

        let by_name: Vec<&str> = catalogue
            .search("acid")
            .into_iter()
            .map(super::Entry::id)
            .collect();
        assert_eq!(by_name, ["acid-lines"], "what it is called");

        let by_tag: Vec<&str> = catalogue
            .search("PAD")
            .into_iter()
            .map(super::Entry::id)
            .collect();
        assert_eq!(
            by_tag,
            ["aurora-pads", "acid-lines"],
            "and a tag, in either case, wherever it is"
        );

        assert_eq!(
            catalogue.search("").len(),
            2,
            "and a field nobody has typed in is not a filter"
        );
    }

    #[test]
    fn the_tags_are_read_off_the_packs_that_carry_them() {
        let catalogue = Catalogue::parse(INDEX);

        assert_eq!(
            catalogue.tags(),
            [("pad", 2), ("ambient", 1), ("bass", 1), ("slow", 1)],
            "the ones the repository actually uses first"
        );
    }
}

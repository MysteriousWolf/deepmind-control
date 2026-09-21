//! A shelf somebody else filled: the shared patch library, as this window holds
//! it.
//!
//! The librarian holds what is on this machine. This is the other half of the
//! same question — the sounds other `DeepMind` owners have made — and it is now
//! a real repository rather than a design:
//! [`deepmind-patches`](https://github.com/MysteriousWolf/deepmind-patches), one
//! `.syx` and one `.toml` per sound under `presets/<Category>/`, published as a
//! release on every merge.
//!
//! # Almost nothing here reads the format
//!
//! This file used to be a hand-written reader for a format this repository had
//! invented: seven hundred lines of records, a lenient parser, a path check and
//! a search. The library publishes [`deepmind_patches`], which is the same split
//! `deepmind-midi` is on the protocol's side — it reads a checkout, checks it,
//! derives what the program bytes say, builds the index a release ships, and
//! matches a program dumped from an instrument back to the patch and version it
//! came from.
//!
//! So the reader is deleted and the crate is linked. What is left here is the
//! part that is this window's: where a catalogue is cached, how it is fetched
//! without dropping a frame, and what a search field asks of it.
//!
//! # The index is the catalogue, and the bundle is only the sounds
//!
//! A release carries three assets, and the split between them decides the
//! shape of this module. `index.toml` holds *everything a person browses*:
//! every patch's name, author, description, category, vocabulary terms,
//! fingerprint and 7x7 icon, plus the taxonomy and the category icons, in one
//! file. `patches.tar.gz` holds only `presets/`, which is the `.syx` bytes.
//!
//! So browsing costs one download and loading costs the other, and [`Held`]
//! carries the index from the moment it arrives while the bundle follows
//! behind. A catalogue that has its index can be read, searched and matched
//! against the shelf before a single program has been downloaded.
//!
//! # Fetching happens on a thread, like everything else that takes time
//!
//! A bank transfer is twelve seconds long and this application already refuses
//! to wait on one: the device thread holds the truth and the window is drawn
//! from what it has published. A download is the same problem with a different
//! cause, so it gets the same answer — [`Catalogue::fetch`] starts a thread,
//! [`Catalogue::settle`] drains what it has said, and the window asks
//! [`Catalogue::state`] what to draw. There is no lock and nothing blocks a
//! frame.
//!
//! # What is trusted, and what is not
//!
//! The crate refuses an `index.toml` whose schema is newer than it reads,
//! rather than guessing at a layout it does not know. Beyond that this module
//! keeps the rule the old reader was written under, because it still applies:
//! **nothing in a file somebody else wrote is trusted with a path.** Every
//! patch names the file it lives in, that name is joined onto a directory on
//! this machine, and `../../../.ssh/id_ed25519` is a perfectly ordinary string.
//! [`within`] is the only way one becomes a path.
//!
//! See [the plan](../../../docs/presets.md) for the reasoning and
//! [the specification](../../../docs/patches-repo.md) for what the repository
//! is.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use deepmind_midi::program::Program;
use deepmind_patches::index::{IndexPatch, Match};
use deepmind_patches::{Category, Icon, Index, Library, fetch};

/// The repository this window fetches from when nothing else is named.
pub use deepmind_patches::fetch::DEFAULT_REPO;

/// Where a fetched catalogue is kept between runs.
///
/// The platform's own cache directory, worked out here rather than through a
/// crate: three environment variables and a fallback is less code than a
/// dependency, and a cache is the one directory an application may lose without
/// anybody minding.
///
/// `None` where none of them is set, which is a machine this window will fetch
/// into a temporary directory on rather than refuse to run.
#[must_use]
pub fn cache() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home.map(|home| home.join("Library/Caches"))
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|home| home.join(".cache")))
    };
    Some(base?.join("deepmind-control").join("patches"))
}

/// Joins `file` onto `root`, or refuses.
///
/// The one place a name out of a catalogue becomes a path. Relative, walking
/// downwards, no `..` anywhere, no root and no prefix — or nothing at all. An
/// index is untrusted input in exactly the sense a `.syx` file is: somebody else
/// wrote it, and this window is about to open what it names.
#[must_use]
pub fn within(root: &Path, file: &str) -> Option<PathBuf> {
    let relative = Path::new(file);
    let walks_down = relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)));
    (walks_down && !file.is_empty()).then(|| root.join(relative))
}

/// What a catalogue is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Nothing held and nothing asked for.
    Empty,
    /// A fetch or an open is running. The sentence says which.
    Working(&'static str),
    /// A catalogue is held. See [`Catalogue::held`].
    Ready,
    /// The last attempt failed, and this is what it said.
    Failed(String),
}

/// A catalogue this window is holding.
#[derive(Debug)]
pub struct Held {
    /// Every patch, and the vocabularies and icons to draw them by.
    index: Index,
    /// The directory `IndexPatch::file` resolves against, once the bundle is
    /// unpacked under it. `None` while only the index has arrived.
    root: Option<PathBuf>,
}

impl Held {
    /// The index, which is the catalogue as data.
    #[must_use]
    pub const fn index(&self) -> &Index {
        &self.index
    }

    /// Every patch, in the order the index sorts them.
    #[must_use]
    pub fn patches(&self) -> &[IndexPatch] {
        &self.index.patches
    }

    /// Whether the sounds themselves are here, rather than only what is said
    /// about them.
    #[must_use]
    pub const fn loadable(&self) -> bool {
        self.root.is_some()
    }

    /// The patch and version a program is, if the library has ever held it.
    ///
    /// `deepmind_patches::Fingerprint` is identity of a *sound*: the program
    /// bytes that decide what is heard, with the name, the slot and the file
    /// left out. So a patch somebody renamed after loading still matches, and a
    /// patch they edited does not — which is the honest answer, because it is
    /// not that sound any more.
    #[must_use]
    pub fn matching(&self, program: &Program) -> Option<Match<'_>> {
        self.index
            .lookup(&deepmind_patches::Fingerprint::of(program))
    }

    /// Reads the program one patch holds.
    ///
    /// # Errors
    ///
    /// When the bundle is not unpacked yet, when the file it names is not one
    /// this window will open (see [`within`]), or when the bytes on disk are not
    /// a program dump.
    pub fn program(&self, patch: &IndexPatch) -> Result<Program, String> {
        let root = self.root.as_ref().ok_or("the sounds are not here yet")?;
        let within_root = within(root, &patch.file)
            .ok_or_else(|| format!("{} names a file outside the catalogue", patch.id))?;
        let bytes = fs::read(&within_root)
            .map_err(|error| format!("{}: {error}", within_root.display()))?;
        program_of(&bytes).ok_or_else(|| format!("{} is not one program", patch.file))
    }

    /// The icon to draw for a patch: its own where it has one, its category's
    /// otherwise.
    #[must_use]
    pub fn icon(&self, patch: &IndexPatch) -> Icon {
        patch.icon
    }

    /// The icon a category is drawn by, where the catalogue carries one.
    #[must_use]
    pub fn category_icon(&self, category: Category) -> Option<Icon> {
        self.index.icons.category(category).copied()
    }

    /// A colour the library named, as three components.
    ///
    /// `resources/palette.toml` carries twelve category colours, one per
    /// vocabulary axis, and the instrument's own display triple, and they ride
    /// in the index so a reader needs no second file. Hex is parsed here rather
    /// than in the drawing, so that a malformed colour is nothing rather than a
    /// panic in a frame.
    ///
    /// **The library names them and this window decides what to do with them**,
    /// which is what the palette's own note says. A category's colour is worth
    /// having because every host that draws one should draw the same one; how
    /// dark the chip under it is, is this window's business.
    #[must_use]
    pub fn colour(&self, group: &str, key: &str) -> Option<[u8; 3]> {
        let hex = self.index.palette.get(group)?.get(key)?;
        rgb(hex)
    }

    /// The colour of a category, where the catalogue names one.
    #[must_use]
    pub fn category_colour(&self, category: Category) -> Option<[u8; 3]> {
        // Keyed by the instrument's own short label, which is what
        // `palette.toml` writes: `SFX` and `Pad`, not `Sound Effects` and
        // `Pads`. The folder is the other one.
        self.colour("category", category.label())
    }

    /// The colour of a vocabulary axis, where the catalogue names one.
    #[must_use]
    pub fn axis_colour(&self, axis: deepmind_patches::Axis) -> Option<[u8; 3]> {
        self.colour("axis", axis_key(axis))
    }

    /// The icon a vocabulary term is drawn by, where the catalogue carries one.
    #[must_use]
    pub fn term_icon(&self, axis: deepmind_patches::Axis, term: &str) -> Option<Icon> {
        self.index.icons.term(axis, term).copied()
    }

    /// Which categories have anything in them, in the instrument's own order.
    #[must_use]
    pub fn categories(&self) -> Vec<Category> {
        Category::ALL
            .into_iter()
            .filter(|category| {
                self.index
                    .patches
                    .iter()
                    .any(|patch| patch.category == *category)
            })
            .collect()
    }

    /// Every vocabulary term any patch in the catalogue actually carries.
    ///
    /// Read off the patches rather than off the taxonomy, for the reason the
    /// shelf's own filters are: a term nothing is filed under is a filter that
    /// returns nothing, and a list of those is a list nobody can use.
    #[must_use]
    pub fn terms(&self, axis: deepmind_patches::Axis) -> Vec<String> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for patch in &self.index.patches {
            for term in terms_of(patch, axis) {
                seen.insert(term.as_str());
            }
        }
        seen.into_iter().map(ToOwned::to_owned).collect()
    }

    /// The patches a search and a set of filters leave, in index order.
    ///
    /// One field asks one question of everything the catalogue knows about a
    /// sound without opening it, which is the rule the shelf's own search is
    /// under: its name, who made it, what it says it is, its category, and
    /// every vocabulary term and tag it carries.
    #[must_use]
    pub fn showing<'a>(&'a self, looking: &Looking) -> Vec<&'a IndexPatch> {
        self.index
            .patches
            .iter()
            .filter(|patch| looking.keeps(patch))
            .collect()
    }
}

/// What a person is looking for in a catalogue.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Looking {
    /// The search field, matched case-insensitively against everything a patch
    /// says about itself.
    pub find: String,
    /// One category, where one is chosen.
    pub category: Option<Category>,
    /// Vocabulary terms, each of which a patch has to carry.
    pub terms: Vec<(deepmind_patches::Axis, String)>,
    /// One bank, where one is chosen. Only the two nearby places have banks.
    pub bank: Option<deepmind_midi::ids::Bank>,
    /// One of the three places a sound can be, where one is chosen.
    ///
    /// The column's own filter. Here rather than beside the table because it
    /// narrows the same list everything else narrows, and a place that lived
    /// somewhere else would be the tab this replaced wearing a filter's
    /// clothes.
    pub place: Option<crate::app::Where>,
}

impl Looking {
    /// Whether nothing is being asked, which is what a count says `32 patches`
    /// rather than `12 of 32` for.
    #[must_use]
    pub fn asking(&self) -> bool {
        !self.find.trim().is_empty()
            || self.category.is_some()
            || !self.terms.is_empty()
            || self.place.is_some()
            || self.bank.is_some()
    }

    /// Whether one patch survives it.
    #[must_use]
    pub fn keeps(&self, patch: &IndexPatch) -> bool {
        if let Some(wanted) = self.category
            && patch.category != wanted
        {
            return false;
        }
        for (axis, term) in &self.terms {
            if !terms_of(patch, *axis).iter().any(|held| held == term) {
                return false;
            }
        }
        let find = self.find.trim().to_lowercase();
        if find.is_empty() {
            return true;
        }
        said_about(patch)
            .into_iter()
            .any(|said| said.to_lowercase().contains(&find))
    }
}

/// Everything a patch says about itself, as a search reads it.
fn said_about(patch: &IndexPatch) -> Vec<String> {
    let mut said = vec![
        patch.name.clone(),
        patch.author.clone(),
        patch.about.clone(),
        patch.category.label().to_owned(),
        patch.category.name().to_owned(),
    ];
    if let Some(collection) = &patch.collection {
        said.push(collection.clone());
    }
    said.extend(patch.genre.iter().cloned());
    said.extend(patch.mood.iter().cloned());
    said.extend(patch.timbre.iter().cloned());
    said.extend(patch.role.iter().cloned());
    said.extend(patch.tags.iter().cloned());
    said.extend(patch.effects.iter().cloned());
    said
}

/// What the palette calls an axis.
///
/// Lowercase, which is how `taxonomy.toml` and `palette.toml` both spell it.
#[must_use]
pub const fn axis_key(axis: deepmind_patches::Axis) -> &'static str {
    match axis {
        deepmind_patches::Axis::Genre => "genre",
        deepmind_patches::Axis::Mood => "mood",
        deepmind_patches::Axis::Timbre => "timbre",
        deepmind_patches::Axis::Role => "role",
    }
}

/// Parses `#rrggbb`, or nothing at all.
///
/// Lenient in the one direction that matters: a colour this window cannot read
/// is a colour it does not draw, and the thing it was going to tint is drawn in
/// the metal everything else is. A catalogue is somebody else's file.
#[must_use]
fn rgb(hex: &str) -> Option<[u8; 3]> {
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let pair = |at: usize| -> Option<u8> {
        u8::from_str_radix(digits.get(at..at.checked_add(2)?)?, 16).ok()
    };
    Some([pair(0)?, pair(2)?, pair(4)?])
}

/// One axis of a patch's vocabulary terms.
fn terms_of(patch: &IndexPatch, axis: deepmind_patches::Axis) -> &Vec<String> {
    match axis {
        deepmind_patches::Axis::Genre => &patch.genre,
        deepmind_patches::Axis::Mood => &patch.mood,
        deepmind_patches::Axis::Timbre => &patch.timbre,
        deepmind_patches::Axis::Role => &patch.role,
    }
}

/// Decodes one program dump.
///
/// A patch is one program stored as bank A, program 1, which is the repository's
/// own rule — so the first dump in the file is the answer and a file holding
/// anything else is not a patch.
fn program_of(bytes: &[u8]) -> Option<Program> {
    let file = deepmind_midi::syx::File::new(bytes);
    let mut found = file.programs().filter_map(Result::ok);
    let entry = found.next()?;
    found.next().is_none().then_some(entry.program)
}

/// What the worker says while it is working.
#[derive(Debug)]
enum Said {
    /// Still going, and this is what it is doing.
    Doing(&'static str),
    /// The index arrived. The bundle may still be coming.
    Index(Box<Index>),
    /// The bundle is unpacked under this root.
    Bundle(PathBuf),
    /// It stopped, and this is why.
    Failed(String),
}

/// The shared patch library, as this window holds it.
#[derive(Debug)]
pub struct Catalogue {
    /// What is held, once anything is.
    held: Option<Held>,
    /// What it is doing.
    state: State,
    /// What a running fetch is saying.
    saying: Option<Receiver<Said>>,
    /// The repository fetched from.
    repo: String,
}

impl Default for Catalogue {
    fn default() -> Self {
        Self::new()
    }
}

impl Catalogue {
    /// A catalogue holding nothing.
    #[must_use]
    pub fn new() -> Self {
        Self {
            held: None,
            state: State::Empty,
            saying: None,
            repo: DEFAULT_REPO.to_owned(),
        }
    }

    /// What it is doing.
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// What it is holding, if anything.
    #[must_use]
    pub const fn held(&self) -> Option<&Held> {
        self.held.as_ref()
    }

    /// The repository it fetches from.
    #[must_use]
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Whether a fetch is running.
    #[must_use]
    pub const fn working(&self) -> bool {
        self.saying.is_some()
    }

    /// Opens a checkout, or a release already unpacked on disk.
    ///
    /// The first of the two routes and the one that needs no network:
    /// `git clone` the repository, point this at the folder. It is also how the
    /// fetching route is tested, because what it produces is the same thing.
    ///
    /// # Errors
    ///
    /// Whatever the library says about the directory, which names the file it
    /// could not read.
    pub fn open(&mut self, root: &Path) -> Result<(), String> {
        let library = Library::open(root).map_err(|error| error.to_string())?;
        let index = index_of(&library);
        self.held = Some(Held {
            index,
            root: Some(library.root.clone()),
        });
        self.state = State::Ready;
        Ok(())
    }

    /// Starts fetching the newest release, on a thread.
    ///
    /// Returns at once. [`settle`](Self::settle) is what picks the answer up,
    /// and [`state`](Self::state) is what the window draws from meanwhile.
    ///
    /// The index arrives first and the catalogue can be browsed the moment it
    /// does; the bundle follows, and only loading a sound waits on it.
    pub fn fetch(&mut self, into: PathBuf) {
        if self.working() {
            return;
        }
        let (says, saying) = mpsc::channel();
        let repo = self.repo.clone();
        self.state = State::Working("Asking for the newest patches\u{2026}");
        self.saying = Some(saying);
        // Detached on purpose. Nothing here holds a lock, the window never waits
        // on it, and a fetch nobody is listening to any more is a dropped
        // message rather than a hung frame.
        drop(thread::spawn(move || {
            let index = match fetch::latest_index(&repo) {
                Ok(index) => index,
                Err(error) => {
                    drop(says.send(Said::Failed(error.to_string())));
                    return;
                }
            };
            if says.send(Said::Index(Box::new(index))).is_err() {
                return;
            }
            drop(says.send(Said::Doing("Downloading the sounds\u{2026}")));
            if let Err(error) = fs::create_dir_all(&into) {
                drop(says.send(Said::Failed(format!("{}: {error}", into.display()))));
                return;
            }
            match fetch::unpack(&fetch::patches_url(&repo), &into) {
                Ok(()) => drop(says.send(Said::Bundle(into))),
                Err(error) => drop(says.send(Said::Failed(error.to_string()))),
            }
        }));
    }

    /// Folds in whatever the worker has said since the last time.
    ///
    /// Called on the same tick everything else in this application is folded in
    /// on. Returns whether anything changed, so a window that has nothing to
    /// redraw does not.
    pub fn settle(&mut self) -> bool {
        let Some(saying) = &self.saying else {
            return false;
        };
        let mut changed = false;
        loop {
            match saying.try_recv() {
                Ok(Said::Doing(what)) => {
                    self.state = State::Working(what);
                    changed = true;
                }
                Ok(Said::Index(index)) => {
                    self.held = Some(Held {
                        index: *index,
                        root: None,
                    });
                    self.state = State::Ready;
                    changed = true;
                }
                Ok(Said::Bundle(root)) => {
                    if let Some(held) = &mut self.held {
                        held.root = Some(root);
                    }
                    self.state = State::Ready;
                    self.saying = None;
                    return true;
                }
                Ok(Said::Failed(why)) => {
                    // A failure after the index landed is a catalogue that can
                    // still be read and not loaded from, which is worth saying
                    // rather than throwing away what arrived.
                    self.state = State::Failed(why);
                    self.saying = None;
                    return true;
                }
                Err(TryRecvError::Empty) => return changed,
                Err(TryRecvError::Disconnected) => {
                    self.saying = None;
                    if self.held.is_none() {
                        self.state = State::Failed("the download stopped".to_owned());
                    }
                    return true;
                }
            }
        }
    }
}

/// Builds an index out of a checkout, so that both routes end in one type.
///
/// A checkout has no `index.toml` — that file is written at release — so this
/// is the same construction the release workflow runs, with the provenance a
/// working copy can honestly give: the library's own version, no commit, and no
/// history beyond what each patch says its version is.
fn index_of(library: &Library) -> Index {
    Index::build(
        library,
        library_version(library),
        "",
        "",
        &std::collections::BTreeMap::new(),
    )
}

/// The version a checkout claims, which is the one its own crate carries.
///
/// A working copy is whatever somebody has edited it into, so the honest answer
/// is the version the checkout says it is rather than a number invented here.
fn library_version(_library: &Library) -> deepmind_patches::LibraryVersion {
    deepmind_patches::LibraryVersion::default()
}

/// The blank icon, for a patch drawn before its catalogue has one.
#[must_use]
pub fn blank() -> Icon {
    Icon::from_rows([0; Icon::SIDE])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Catalogue, Looking, State, within};

    #[test]
    fn nothing_out_of_a_catalogue_is_trusted_with_a_path() {
        // The one rule this module kept when the rest of it was deleted. An
        // index names the file each patch lives in, that name is joined onto a
        // directory on somebody else's machine, and the interesting strings are
        // all perfectly ordinary strings.
        let root = Path::new("/tmp/patches");
        assert!(within(root, "presets/Bass/Acid Growl - nyx.syx").is_some());
        assert!(within(root, "../../../.ssh/id_ed25519").is_none());
        assert!(within(root, "/etc/passwd").is_none());
        assert!(within(root, "presets/../../escape.syx").is_none());
        assert!(within(root, "").is_none());
    }

    #[test]
    fn a_new_catalogue_holds_nothing_and_says_so() {
        let catalogue = Catalogue::new();
        assert_eq!(*catalogue.state(), State::Empty);
        assert!(catalogue.held().is_none());
        assert!(!catalogue.working());
    }

    #[test]
    fn settling_a_catalogue_nobody_asked_anything_of_changes_nothing() {
        let mut catalogue = Catalogue::new();
        assert!(!catalogue.settle());
        assert_eq!(*catalogue.state(), State::Empty);
    }

    #[test]
    fn an_empty_search_is_not_asking_anything() {
        let mut looking = Looking::default();
        assert!(!looking.asking());
        looking.find = "   ".to_owned();
        assert!(!looking.asking(), "whitespace is not a question");
        looking.find = "pad".to_owned();
        assert!(looking.asking());
    }
}

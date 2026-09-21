//! Pictures of every surface this application has, taken by the application.
//!
//! A release ships screenshots, and screenshots go stale silently: a plate
//! moves, a legend changes, a case is finished differently, and the picture in
//! `README.md` is of a window nobody can open any more. Nothing catches that,
//! because nothing *reads* a picture. So the pictures are generated, and
//! generating them is a step of the release rather than an afternoon with a
//! screen grabber.
//!
//! # What it draws
//!
//! The same window the application opens, in the same crate, through the same
//! `window::view` the application is drawn by. Not a second drawing of it: a
//! preview assembled by its own code would be a picture of the preview.
//!
//! What differs is where the sound comes from. The window is pointed at the
//! library's simulated synthesizer, and the unit powers up holding a program
//! built from a seed, with every parameter somewhere inside its own range, so
//! the pictures are of an instrument holding a sound rather than of a window that
//! has just opened and knows nothing. The window then reads the edit buffer, the
//! way it does when any port opens, which is why every claim dot in a preview is
//! the green one: this is what the synthesizer says it is, which is what a
//! picture of the editor should be showing.
//!
//! # Why it is random
//!
//! A fixed sound is one arrangement of the window, and the things worth catching
//! are the ones a single arrangement hides: a case whose algorithm is worn
//! against one that is brushed, an envelope whose segments are long against one
//! that is short, a matrix with eight routings in it against a matrix with two.
//! A seed is printed and recorded with the pictures, so a run that turned
//! something up can be run again exactly.
//!
//! # How it is run
//!
//! `tools/previews.fish`, which is a step of the release. This module is behind
//! the `previews` feature and nothing in the shipped binary reaches it.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use deepmind_host::{Options, Pack, PortRef, open_simulator_holding};
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
use deepmind_midi::param::{Group, ParamId};
use deepmind_midi::program::{Program, ProgramName};
use iced::window::Screenshot;
use iced::{Element, Task};

use crate::app::{App, Message, View};
use crate::window;

/// One picture: what the window is showing, and what the file is called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// The surface the window is put on.
    view: View,
    /// The section open *over* it, where the picture is of a sheet.
    ///
    /// The ten sections a plate carries an `EDIT` for are sheets laid over the
    /// front panel, so a picture of one is a picture of the panel with that
    /// sheet on it, which is what pressing `EDIT` actually shows. The other
    /// four are surfaces of their own and arrive in [`Page::view`] instead.
    section: Option<Group>,
}

impl Page {
    /// Returns the file this page is written to, without its suffix.
    ///
    /// The section's own name, lowered and hyphenated. `LFO 1` is `lfo-1` and
    /// `Mod Matrix` is `mod-matrix`, so the file is the thing the sheet's own
    /// bar says and a group renamed in the library renames its picture.
    #[must_use]
    pub fn slug(self) -> String {
        let name = match self.section {
            Some(group) => control_ui::section_name(group),
            None => match self.view {
                View::Library => "library",
                View::Panel => "front-panel",
                View::Section(group) => control_ui::section_name(group),
            },
        };
        let mut slug = String::with_capacity(name.len());
        let mut hyphen = false;
        for letter in name.chars() {
            if letter.is_ascii_alphanumeric() {
                slug.extend(letter.to_lowercase());
                hyphen = false;
            } else if !hyphen && !slug.is_empty() {
                slug.push('-');
                hyphen = true;
            }
        }
        slug.trim_end_matches('-').to_owned()
    }
}

/// Every surface the window has, in the order somebody meets them.
///
/// The front panel, then the fourteen sections in the instrument's own order,
/// then the shelf. Read off [`control_ui::sections`] rather than written out,
/// so a group a later library adds gets a picture with nothing here to edit.
///
/// Each section is photographed as the window actually shows it, which is two
/// different things: the ten a plate carries an `EDIT` for are sheets over the
/// front panel, and the four no plate carries are surfaces the band's own tabs
/// open. Which is which comes from [`control_ui::unplated`], the same list the
/// band is built from, so a section that gains a plate in a later library stops
/// being photographed as a tab without anybody editing this.
#[must_use]
pub fn pages() -> Vec<Page> {
    // Which of the two a section is drawn as is the window's own answer, asked
    // rather than written down here: the band carries exactly the sections no
    // plate does, so a section that gains a plate in a later library stops
    // being photographed as a tab without anybody editing this.
    let tabbed = control_ui::unplated();
    let mut pages = vec![Page {
        view: View::Panel,
        section: None,
    }];
    pages.extend(control_ui::sections().iter().copied().map(|group| {
        if tabbed.contains(&group) {
            Page {
                view: View::Section(group),
                section: None,
            }
        } else {
            Page {
                view: View::Panel,
                section: Some(group),
            }
        }
    }));
    pages.push(Page {
        view: View::Library,
        section: None,
    });
    pages
}

/// Takes every picture in `pages` into `out`, with the sound `seed` builds.
///
/// Opens one window, poses it once per page and writes a file each time, then
/// closes. One window rather than one per picture: a window takes a second to
/// open and there are sixteen of them.
///
/// # Errors
///
/// Returns whatever iced could not do. A file that will not be written is
/// reported on the error stream and does not stop the rest, because a run that
/// gave up on the tenth picture would leave nine new ones beside seven old.
pub fn run(out: PathBuf, seed: u64) -> iced::Result {
    let pages = pages();
    iced::application(
        move || Session::new(out.clone(), seed, pages.clone()),
        Session::update,
        Session::view,
    )
    .title(|session: &Session| window::title(&session.app))
    .default_font(control_ui::printed())
    .theme(|session: &Session| window::theme(&session.app))
    .subscription(|session: &Session| window::subscription(&session.app).map(Shot::App))
    .window_size(window::window_size())
    .run()
}

/// Everything that happens while the pictures are being taken.
#[derive(Debug)]
enum Shot {
    /// Something the window itself asked for, which is every ordinary message.
    App(Message),
    /// A picture, come back from the renderer.
    Took(Box<Screenshot>),
}

/// Where a run has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// Waiting for the device thread to be listening before a knob is turned.
    Opening,
    /// The buffer has been asked for; waiting for the sound to come back.
    Reading,
    /// On a page, waiting for it to stop moving.
    Posing,
    /// Asked for a picture, waiting for it.
    Taking,
}

/// A run: the window, the unit's panel, and where it has got to.
struct Session {
    /// The window, which is the application's own.
    app: App,
    /// The pictures still to take, and which one is next.
    pages: Vec<Page>,
    at: usize,
    /// Where the run has got to, and since when.
    stage: Stage,
    since: Instant,
    /// Where the pictures go.
    out: PathBuf,
}

/// How long a page is given to stop moving before it is photographed.
///
/// A display with a name too long for its field scrolls, an envelope is drawn
/// from its own times and an LFO turns, so there is no moment at which this
/// window is still. What this waits for is the layout: a page that has just been
/// switched to has a frame or two in which its plates have not been measured.
///
/// Counted in time rather than in frames. A window drawn by a graphics card
/// manages sixty a second and one drawn in software on a machine with no screen
/// manages two, so a count of frames is a wait of a quarter of a second on the
/// desk and of ten seconds on the thing that actually runs this.
const SETTLE: Duration = Duration::from_millis(600);

/// How long the unit is given to answer before the run gives up.
///
/// The simulator answers in well under a second. This is long enough that a
/// machine labouring over a software-drawn window is not mistaken for a
/// synthesizer that has gone quiet, and short enough that a release is not held
/// up by a run that is never going to finish.
const PATIENCE: Duration = Duration::from_secs(30);

impl Session {
    /// Opens the window on the simulated unit and turns its knobs.
    fn new(out: PathBuf, seed: u64, pages: Vec<Page>) -> Self {
        let mut app = App::new();
        let (link, _panel) = open_simulator_holding(Pack::empty(), sound(seed), Options::default());
        app.attach(PortRef::simulator(), link);
        app.hold(SHELF, &pack(seed));
        // And the shared library, where this machine has a checkout of it. The
        // table's whole subject is every sound *wherever it is*, so a picture
        // taken with nothing but a shelf in it is a picture of a third of the
        // columns: no maker, no vocabulary, no version. It is read from the
        // environment rather than downloaded, because a picture that needs the
        // network is a picture that cannot be taken twice the same.
        if let Ok(root) = std::env::var("PATCHES_CHECKOUT")
            && let Err(trouble) = app.read_patches(std::path::Path::new(&root))
        {
            eprintln!("previews: {root} is not a patch library: {trouble}");
        }
        Self {
            app,
            pages,
            at: 0,
            stage: Stage::Opening,
            since: Instant::now(),
            out,
        }
    }

    /// Draws the window, which is the application's own view and nothing else.
    fn view(&self) -> Element<'_, Shot> {
        window::view(&self.app).map(Shot::App)
    }

    /// Folds in a message, and moves the run along on every frame.
    fn update(&mut self, message: Shot) -> Task<Shot> {
        match message {
            Shot::App(message) => {
                let tick = matches!(message, Message::Tick);
                self.app.update(message);
                if tick { self.tick() } else { Task::none() }
            }
            Shot::Took(shot) => self.took(&shot),
        }
    }

    /// One frame: waits for the unit, poses a page, or asks for a picture.
    fn tick(&mut self) -> Task<Shot> {
        let waited = self.since.elapsed();

        match self.stage {
            // The knobs are turned once the thread is round its loop, and the
            // buffer read after them: a read that overtook the turns would
            // photograph the sound the unit powers up holding.
            // The buffer is asked for once the thread is round its loop. The
            // unit is already holding the sound, because it powered up with it,
            // so this is the ordinary read a window does when a port opens.
            Stage::Opening if waited >= SETTLE => {
                self.app.update(Message::Read);
                self.stage = Stage::Reading;
                self.since = Instant::now();
                Task::none()
            }
            // A dump is 242 values at once, so the window knowing all of them
            // is the dump having landed. Waiting for that rather than for a
            // length of time is what makes a preview the same picture on a fast
            // machine and a slow one.
            Stage::Reading if self.read() => {
                self.pose();
                Task::none()
            }
            Stage::Reading if waited >= PATIENCE => {
                eprintln!("previews: the simulated unit did not answer");
                iced::exit()
            }
            Stage::Posing if waited >= SETTLE => {
                self.stage = Stage::Taking;
                iced::window::latest()
                    .and_then(iced::window::screenshot)
                    .map(|shot| Shot::Took(Box::new(shot)))
            }
            Stage::Opening | Stage::Reading | Stage::Posing | Stage::Taking => Task::none(),
        }
    }

    /// Whether the window knows what the sound is, all of it.
    fn read(&self) -> bool {
        ParamId::ALL
            .iter()
            .all(|parameter| self.app.patch().value(*parameter).is_some())
    }

    /// Puts the window on the page that is next, and starts it settling.
    fn pose(&mut self) {
        let Some(page) = self.pages.get(self.at).copied() else {
            return;
        };
        // The surface first and then the sheet, because asking for a sheet is
        // also asking for the panel under it: doing it the other way round
        // would put every sheet back on a window that had just moved to the
        // panel anyway, and would leave a tab's page carrying the last sheet.
        self.app.update(Message::Show(page.view));
        self.app.update(Message::Ui(match page.section {
            Some(group) => control_ui::Message::Show(group),
            None => control_ui::Message::Close,
        }));
        self.stage = Stage::Posing;
        self.since = Instant::now();
    }

    /// Writes a picture out and moves on, closing the window after the last.
    fn took(&mut self, shot: &Screenshot) -> Task<Shot> {
        if let Some(page) = self.pages.get(self.at).copied() {
            let path = self.out.join(format!("{}.png", page.slug()));
            match write(&path, shot) {
                Ok(()) => println!("{}", path.display()),
                Err(trouble) => eprintln!("{}: {trouble}", path.display()),
            }
        }
        self.at += 1;
        if self.at >= self.pages.len() {
            return iced::exit();
        }
        self.pose();
        Task::none()
    }
}

/// What the pack in the picture of the librarian is called.
const SHELF: &str = "previews.syx";

/// How many programs are on it.
///
/// Enough to fill the grid several rows deep at the window's opening size, so
/// that the picture shows a shelf being read rather than a shelf with a row on
/// it. Not 128: a picture of the librarian should show the surface, and three
/// screenfuls of it below the fold are three screenfuls nobody can see in a
/// screenshot.
const SHELVED: usize = 32;

/// Builds the pack the librarian is photographed holding, from `seed`.
///
/// The same sounds the unit powers up holding, one per slot, numbered rather
/// than named: every other value in them is drawn from the seed, the category
/// among them, so the picture shows what the shelf actually draws — the slot as
/// the front panel writes it, the name, and what each program calls itself —
/// without this file inventing a pack of preset names that do not exist.
///
/// Written as bank A, because a pack is what a bank dump is and the addresses
/// down the side of the picture should be ones somebody could go and look at on
/// an instrument.
fn pack(seed: u64) -> Vec<u8> {
    let programs: Vec<Program> = (0..SHELVED)
        .map(|index| {
            let mut program = sound(seed.wrapping_add(index as u64).wrapping_add(1));
            if let Ok(name) = ProgramName::new(&format!("Preview {}", index + 1)) {
                program.set_name(name);
            }
            program
        })
        .collect();
    deepmind_midi::syx::bank_to_vec(
        DeviceId::Broadcast,
        Bank::A,
        ProgramNumber::FIRST,
        &programs,
    )
    .unwrap_or_default()
}

/// Builds the sound the simulated unit powers up holding, from `seed`.
///
/// Every parameter is put somewhere inside its *own* range, which the library
/// publishes, so nothing here can ask for a value the instrument would refuse. A
/// byte drawn across the whole of 0–255 would be a sound that mostly did not
/// move, since most of the 242 stop well short of it.
///
/// Built here and handed to the unit rather than played into it through
/// [`Panel`](deepmind_host::Panel). Two hundred and forty-two knobs turned one
/// at a time is the same sound by a road with a queue on it, and what a preview
/// wants is an instrument that is *holding* something.
fn sound(seed: u64) -> Program {
    let mut sound = Program::new(ProtocolVersion::V7);
    if let Ok(name) = ProgramName::new("Preview") {
        sound.set_name(name);
    }
    for (step, parameter) in ParamId::ALL.iter().copied().enumerate() {
        let (least, most) = (parameter.min(), parameter.max());
        let span = u32::from(most.saturating_sub(least)) + 1;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a value inside a range the library says is a u16"
        )]
        let value = u32::from(least) + draw(seed, step as u32) % span;
        // Every parameter's range fits in the byte it occupies in a dump, which
        // is the library's own note on `set_clamped`, so a value inside a range
        // is a byte and the clamp has nothing to do.
        sound.set_clamped(parameter, u8::try_from(value).unwrap_or(u8::MAX));
    }
    sound
}

/// Returns a number from `seed` and `step`, the same one every time.
///
/// The counter-through-a-hash the rest of this repository draws its scuffs from
/// (see `control_ui`'s cases) rather than a generator with state: a seed printed
/// beside a set of pictures has to be enough to get them back.
fn draw(seed: u64, step: u32) -> u32 {
    let mut word = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(step).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    word ^= word >> 30;
    word = word.wrapping_mul(0x94D0_49BB_1331_11EB);
    word ^= word >> 27;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a hash folded onto the half of itself that is wanted"
    )]
    let folded = (word >> 16) as u32;
    folded
}

/// Writes one picture out as a `PNG`.
///
/// # Errors
///
/// Returns whatever the file system or the encoder would not do.
fn write(path: &Path, shot: &Screenshot) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, shot.size.width, shot.size.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&shot.rgba)?;
    Ok(())
}

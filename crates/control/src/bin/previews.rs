//! Photographs every surface the editor has, for the documentation.
//!
//! Run by `tools/previews.fish`, which is a step of the release. See
//! [`control::preview`] for what it draws and why the sound in it is random.
//!
//! ```text
//! previews --out docs/previews --seed 1234
//! previews --list
//! ```
//!
//! `--seed` is optional: without one, a number is taken off the clock and
//! printed, because a set of pictures somebody liked has to be reproducible.
//! `--list` writes the file names a run would produce and takes no pictures,
//! which is how the release check knows what to look for without opening a
//! window.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reads the arguments and either lists the pages or takes the pictures.
fn main() -> ExitCode {
    let mut out = PathBuf::from("docs/previews");
    let mut seed = None;
    let mut listing = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--list" => listing = true,
            "--out" => match arguments.next() {
                Some(folder) => out = PathBuf::from(folder),
                None => return complain("--out wants a folder"),
            },
            "--seed" => match arguments.next().map(|given| given.parse()) {
                Some(Ok(given)) => seed = Some(given),
                _ => return complain("--seed wants a number"),
            },
            other => return complain(&format!("{other} is not one of --out, --seed, --list")),
        }
    }
    if listing {
        for page in control::preview::pages() {
            println!("{}.png", page.slug());
        }
        return ExitCode::SUCCESS;
    }
    // A number off the clock rather than nothing, and printed rather than kept:
    // the pictures are different every run, and a run that turned something up
    // is only worth having if it can be run again.
    let seed = seed.unwrap_or_else(sometime);
    println!("seed {seed}");
    match control::preview::run(out, seed) {
        Ok(()) => ExitCode::SUCCESS,
        Err(trouble) => complain(&format!("the window would not open: {trouble}")),
    }
}

/// Says what was wrong and gives up.
fn complain(what: &str) -> ExitCode {
    eprintln!("previews: {what}");
    ExitCode::FAILURE
}

/// Returns a number nobody chose.
///
/// The nanoseconds the clock is into its second, which is the one source of a
/// different answer every run that this repository already depends on. The
/// second itself is not wanted: a seed is a label on a set of pictures and a
/// nine-digit one is easier to type back in than a nineteen-digit one. A clock
/// that will not answer is a machine with bigger problems than a screenshot, so
/// the fallback is a constant rather than an error.
fn sometime() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |since| since.subsec_nanos().into())
}

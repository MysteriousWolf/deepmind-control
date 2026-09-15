//! The disk, and the two dialogs that point at it.
//!
//! The only file format is the protocol's. A patch is one program dump, a pack
//! is a run of them end to end, and the library reads and writes both from
//! bytes, so what is left here is opening the file and choosing which one.
//!
//! # The dialog is the system's
//!
//! [`rfd`] is the whole of it: the file chooser the desktop already has, through
//! the XDG portal on Linux and the platform's own on macOS and Windows. It is
//! asked for synchronously, from the window's own update, because a modal file
//! dialog is the one place in this application where waiting is what the person
//! asked for. Everything that waits on a synthesizer goes through the device
//! thread instead.
//!
//! The portal backend is chosen over the GTK one on purpose: it is Rust the
//! whole way down, so a Linux build needs no GTK development headers, and it
//! blocks on `pollster` rather than dragging an async runtime into a repository
//! that gets its frames from `thread::sleep`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rfd::FileDialog;

/// What the dialogs filter on, and what a saved file is called if nothing says
/// otherwise.
const SYSEX: &str = "syx";

/// Asks for a `.syx` file to open, and reads it.
///
/// Returns what it is called and what is in it, or nothing at all when the
/// dialog was dismissed.
///
/// # Errors
///
/// Returns whatever the file could not be read with.
pub fn open() -> Option<io::Result<(String, Vec<u8>)>> {
    let path = FileDialog::new()
        .add_filter("DeepMind SysEx", &[SYSEX])
        .set_title("Open a patch or a pack")
        .pick_file()?;
    Some(fs::read(&path).map(|bytes| (name_of(&path), bytes)))
}

/// Asks where to put a `.syx` file, and writes it.
///
/// Returns what it ended up being called, or nothing at all when the dialog was
/// dismissed.
///
/// # Errors
///
/// Returns whatever the file could not be written with.
pub fn save(suggested: &str, bytes: &[u8]) -> Option<io::Result<String>> {
    let path = FileDialog::new()
        .add_filter("DeepMind SysEx", &[SYSEX])
        .set_file_name(suggested)
        .set_title("Save as SysEx")
        .save_file()?;
    Some(fs::write(&path, bytes).map(|()| name_of(&path)))
}

/// Reads a file named on the command line.
///
/// How a desktop application is handed a document by the thing that opened it,
/// and the one way onto the shelf that does not go through a dialog.
///
/// # Errors
///
/// Returns whatever the file could not be read with.
pub fn read(path: &Path) -> io::Result<(String, Vec<u8>)> {
    fs::read(path).map(|bytes| (name_of(path), bytes))
}

/// Returns what to call a file on the screen.
///
/// The last component, because a shelf says which file it is holding and not
/// where that file lives. A path with no last component is shown whole, which is
/// stranger than it is wrong and is better than showing nothing.
fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// Returns the `.syx` files named on the command line.
///
/// Everything after the program's own name, so that `control pack.syx` opens
/// with that pack on the shelf. Only the first is opened; the rest are the
/// same file chooser away.
#[must_use]
pub fn named() -> Vec<PathBuf> {
    std::env::args_os().skip(1).map(PathBuf::from).collect()
}

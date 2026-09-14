//! The `DeepMind` editor and librarian, on the desktop.
//!
//! The window is stage 2 of [the plan]: the port picker, the identity, the edit
//! buffer, and one group of parameters editable end to end. What this binary
//! does today is say what the host crate can see, which is enough to tell a
//! machine with a synthesizer on it from a machine without one.
//!
//! [the plan]: https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/plan.md

use control_ui as _;
use deepmind_midi as _;

use deepmind_host::ports;

fn main() {
    println!("deepmind-control {}", env!("CARGO_PKG_VERSION"));
    match ports() {
        Ok(found) if found.is_empty() => {
            println!("No port on this machine can hold a conversation with a synthesizer.");
        }
        Ok(found) => {
            println!("Ports:");
            for port in found {
                println!("  {port}");
            }
            println!();
            println!("Which of these has a DeepMind on it is settled by opening one and");
            println!("seeing whether the device inquiry is answered. The window that does");
            println!("that is stage 2; see docs/plan.md.");
        }
        Err(error) => println!("No MIDI backend: {error}"),
    }
}

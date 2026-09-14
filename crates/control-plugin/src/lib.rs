//! The `DeepMind` editor as a plugin.
//!
//! Stage 6 of [the plan]: *simple* mode, which selects programs and nothing
//! else, then plugin state, then *advanced* mode, which is the desktop editing
//! surface in a plugin window. Editing is identical in both builds because it is
//! the same [`control_ui`].
//!
//! The plugin is written once, as a CLAP, and [clap-wrapper] produces the AU and
//! the VST3 from it. `nih-plug`, `baseview` and the iced adapter arrive with the
//! code that uses them rather than before it.
//!
//! Bank and librarian operations are not here and will not be: a preset pack is
//! thirty-five kilobytes of `SysEx`, and a plugin event buffer is sized for a
//! few notes.
//!
//! [the plan]: https://github.com/MysteriousWolf/deepmind-control/blob/main/docs/plan.md
//! [clap-wrapper]: https://github.com/free-audio/clap-wrapper

use control_ui as _;
use deepmind_midi as _;

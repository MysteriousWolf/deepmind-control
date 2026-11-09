//! The entry point a host looks for.
//!
//! The one place in the repository that is allowed `unsafe`, because the CLAP
//! export is an `extern "C"` function the framework's macro writes, and a plugin
//! is a library a host loads rather than a function this crate calls.
#![allow(unsafe_code)]

use crate::DeepMindControl;

nice_plug::nice_export_clap!(DeepMindControl);

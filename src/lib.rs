//! Decompile Cubism `.moc3` runtime models into readable `.cmo3` projects.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cmo3;
mod decompiler;
mod error;
pub mod moc3;

pub use decompiler::{Decompiler, Texture, decompile, decompile_to_file};
pub use error::{Error, Result};

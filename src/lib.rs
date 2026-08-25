//! Decompile Cubism `.moc3` runtime models into readable `.cmo3` projects.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
pub mod moc3;

pub use error::{Error, Result};

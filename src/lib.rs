//! Decompile Cubism `.moc3` runtime models into readable `.cmo3` projects.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod caff;
pub mod can3;
pub mod cmo3;
mod decompiler;
mod error;
pub mod moc3;
pub mod model3;
pub mod motion3;

pub use decompiler::{Decompiler, Texture, decompile, decompile_to_file};
pub use error::{Error, Result};

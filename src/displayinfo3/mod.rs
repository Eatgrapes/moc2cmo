//! Types for reading Cubism `cdi3.json` display-info files.

mod parser;
mod types;

pub use parser::DisplayInfo3;
pub use types::{DisplayInfo3Parameter, DisplayInfo3ParameterGroup, DisplayInfo3Part};

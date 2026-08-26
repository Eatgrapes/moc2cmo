//! Types for reading Cubism `model3.json` resource manifests.

mod parser;
mod types;

pub use parser::Model3;
pub use types::{Model3Group, Model3Motion, Model3References};

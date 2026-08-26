//! Types for reading Cubism `model3.json` resource manifests.

mod parser;
mod types;

pub use parser::Model3;
pub use types::{Model3Expression, Model3Group, Model3HitArea, Model3Motion, Model3References};

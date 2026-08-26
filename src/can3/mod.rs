//! Read, generate, relink, and encode Cubism `.can3` animation projects.

mod generator;
mod project;
mod xml;

pub use generator::MotionInstance;
pub use project::Can3Project;

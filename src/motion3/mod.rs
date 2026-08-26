//! Types for reading Cubism `motion3.json` animation files.

mod parser;
mod types;

pub use parser::Motion3;
pub use types::{MotionCurve, MotionMeta, MotionPoint, MotionSegment, MotionUserData};

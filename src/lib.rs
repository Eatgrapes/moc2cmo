//! Decompile Cubism `.moc3` runtime models into readable `.cmo3` projects.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod caff;
pub mod can3;
pub mod cmo3;
mod decompiler;
pub mod displayinfo3;
mod error;
pub mod expression3;
pub mod moc3;
pub mod model3;
pub mod motion3;
pub mod physics3;
pub mod pose3;
pub mod userdata3;

pub use decompiler::{
    Decompiler, Model3Decompilation, Texture, decompile, decompile_model3,
    decompile_model3_to_files, decompile_to_file,
};
pub use error::{Error, Result};

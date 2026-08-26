//! Types for reading Cubism `userdata3.json` files.

mod parser;
mod types;

pub use parser::UserData3;
pub use types::{UserData3Entry, UserData3Meta};

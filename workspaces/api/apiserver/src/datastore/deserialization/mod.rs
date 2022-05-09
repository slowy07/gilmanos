mod error;
mod pairs;

pub use error::{Error, Result};
pub use pairs::{from_map, from_map_with_prefix};
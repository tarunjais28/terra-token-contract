#[cfg(not(feature = "library"))]
extern crate arrayref;

mod error;
mod operations;
pub mod state;

pub mod contract;
pub mod msg;
#[cfg(test)]
mod tests;

pub use crate::{error::*, operations::*, state::*};
use serde::{Deserialize, Serialize};
pub type Bytes = Vec<u8>;

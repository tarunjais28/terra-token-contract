#[cfg(not(feature = "library"))]
extern crate arrayref;

mod error;
pub mod state;

pub mod contract;
pub mod msg;
#[cfg(test)]
mod tests;

pub use crate::{error::*, state::*};
use serde::{Deserialize, Serialize};
pub type Bytes = Vec<u8>;

#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod contract;
mod error;
pub mod msg;
pub mod proposal;
pub mod query;
pub mod state;
pub use crate::error::ContractError;
pub use crate::msg::ExecuteMsg;

#[cfg(test)]
pub mod testing;

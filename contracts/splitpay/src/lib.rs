#![no_std]

pub mod contract;
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

pub use contract::SplitPayContract;
pub use contract::SplitPayContractClient;

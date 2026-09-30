#![no_std]

pub mod contract;
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

pub use contract::SplitPayContract;
pub use contract::SplitPayContractClient;

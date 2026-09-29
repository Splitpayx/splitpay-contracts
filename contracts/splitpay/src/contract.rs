use crate::errors::Error;
use crate::events;
use crate::storage::{
    self, get_payment, get_pool, get_pool_members, has_config, has_member, has_payment, has_pool,
    set_config, set_member, set_payment, set_pool,
};
use crate::types::{
    ContractConfig, Distribution, Member, Payment, PaymentStatus, Pool, PoolStatus,
};
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

const BPS_DENOMINATOR: i128 = 10_000;
const MAX_BPS: u32 = 10_000;

#[contract]
pub struct SplitPayContract;

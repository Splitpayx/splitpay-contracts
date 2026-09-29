use crate::errors::Error;
use crate::types::{ContractConfig, Distribution, Member, Payment, Pool};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    Pool(u64),
    PoolMembers(u64),
    Member(u64, Address),
    Payment(u64),
    Distribution(u64, Address),
    PaymentRecipients(u64),
}

// --- Config Storage ---

pub fn has_config(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Config)
}

pub fn set_config(env: &Env, config: &ContractConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Result<ContractConfig, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

// --- Pool Storage ---

pub fn has_pool(env: &Env, pool_id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Pool(pool_id))
}

pub fn set_pool(env: &Env, pool: &Pool) {
    env.storage()
        .persistent()
        .set(&DataKey::Pool(pool.id), pool);
}

pub fn get_pool(env: &Env, pool_id: u64) -> Result<Pool, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Pool(pool_id))
        .ok_or(Error::PoolNotFound)
}

// --- Member Storage ---

pub fn get_pool_member_addresses(env: &Env, pool_id: u64) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::PoolMembers(pool_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_pool_member_addresses(env: &Env, pool_id: u64, addresses: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::PoolMembers(pool_id), addresses);
}

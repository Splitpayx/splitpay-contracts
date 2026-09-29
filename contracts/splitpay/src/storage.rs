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

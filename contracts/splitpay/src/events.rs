use crate::types::PoolStatus;
use soroban_sdk::{Address, Env, Symbol};

pub fn pool_created(env: &Env, pool_id: u64, owner: &Address, asset: &Address, created_at: u64) {
    env.events().publish(
        (Symbol::new(env, "pool_created"), pool_id),
        (owner.clone(), asset.clone(), created_at),
    );
}

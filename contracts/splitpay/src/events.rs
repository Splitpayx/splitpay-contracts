use crate::types::PoolStatus;
use soroban_sdk::{Address, Env, Symbol};

pub fn pool_created(env: &Env, pool_id: u64, owner: &Address, asset: &Address, created_at: u64) {
    env.events().publish(
        (Symbol::new(env, "pool_created"), pool_id),
        (owner.clone(), asset.clone(), created_at),
    );
}

pub fn member_added(env: &Env, pool_id: u64, address: &Address, share_bps: u32) {
    env.events().publish(
        (Symbol::new(env, "member_added"), pool_id),
        (address.clone(), share_bps),
    );
}

pub fn member_removed(env: &Env, pool_id: u64, address: &Address) {
    env.events().publish(
        (Symbol::new(env, "member_removed"), pool_id),
        address.clone(),
    );
}

pub fn share_updated(env: &Env, pool_id: u64, address: &Address, old_share: u32, new_share: u32) {
    env.events().publish(
        (Symbol::new(env, "share_updated"), pool_id),
        (address.clone(), old_share, new_share),
    );
}

pub fn pool_status_changed(env: &Env, pool_id: u64, status: PoolStatus) {
    env.events()
        .publish((Symbol::new(env, "pool_status_changed"), pool_id), status);
}

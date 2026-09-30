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

#[contractimpl]
impl SplitPayContract {
    /// Initialize the contract with an administrator address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if has_config(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();

        set_config(&env, &ContractConfig { admin });
        Ok(())
    }

    /// Create a new pool with the specified owner and payment asset.
    pub fn create_pool(
        env: Env,
        pool_id: u64,
        owner: Address,
        asset: Address,
    ) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        owner.require_auth();

        if has_pool(&env, pool_id) {
            return Err(Error::PoolAlreadyExists);
        }

        let created_at = env.ledger().timestamp();
        let pool = Pool {
            id: pool_id,
            owner: owner.clone(),
            asset: asset.clone(),
            status: PoolStatus::Active,
            created_at,
        };

        set_pool(&env, &pool);
        events::pool_created(&env, pool_id, &owner, &asset, created_at);
        Ok(())
    }
}

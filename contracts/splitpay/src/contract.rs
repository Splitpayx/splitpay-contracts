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

    /// Add a member and their percentage share in basis points (10000 = 100%).
    pub fn add_member(
        env: Env,
        pool_id: u64,
        address: Address,
        share_bps: u32,
    ) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        let pool = get_pool(&env, pool_id)?;
        pool.owner.require_auth();

        if share_bps == 0 || share_bps > MAX_BPS {
            return Err(Error::InvalidShare);
        }

        if has_member(&env, pool_id, &address) {
            return Err(Error::MemberAlreadyExists);
        }

        // Validate that total shares do not exceed 10000
        let members = get_pool_members(&env, pool_id)?;
        let mut total_shares: u32 = 0;
        for m in members.iter() {
            total_shares = total_shares
                .checked_add(m.share_bps)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        if total_shares
            .checked_add(share_bps)
            .ok_or(Error::ArithmeticOverflow)?
            > MAX_BPS
        {
            return Err(Error::InvalidTotalShares);
        }

        let member = Member {
            pool_id,
            address: address.clone(),
            share_bps,
        };
        set_member(&env, &member);

        let mut addrs = storage::get_pool_member_addresses(&env, pool_id);
        addrs.push_back(address.clone());
        storage::set_pool_member_addresses(&env, pool_id, &addrs);

        events::member_added(&env, pool_id, &address, share_bps);
        Ok(())
    }

    /// Remove a member from the pool.
    pub fn remove_member(env: Env, pool_id: u64, address: Address) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        let pool = get_pool(&env, pool_id)?;
        pool.owner.require_auth();

        if !has_member(&env, pool_id, &address) {
            return Err(Error::MemberNotFound);
        }

        storage::remove_member(&env, pool_id, &address);
        events::member_removed(&env, pool_id, &address);
        Ok(())
    }

    /// Update a member's share basis points.
    pub fn update_member_share(
        env: Env,
        pool_id: u64,
        address: Address,
        share_bps: u32,
    ) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        let pool = get_pool(&env, pool_id)?;
        pool.owner.require_auth();

        if share_bps == 0 || share_bps > MAX_BPS {
            return Err(Error::InvalidShare);
        }

        let old_member = storage::get_member(&env, pool_id, &address)?;

        // Validate that total shares with the updated value do not exceed 10000
        let members = get_pool_members(&env, pool_id)?;
        let mut total_shares: u32 = 0;
        for m in members.iter() {
            total_shares = total_shares
                .checked_add(m.share_bps)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        let base_shares = total_shares
            .checked_sub(old_member.share_bps)
            .ok_or(Error::ArithmeticOverflow)?;
        if base_shares
            .checked_add(share_bps)
            .ok_or(Error::ArithmeticOverflow)?
            > MAX_BPS
        {
            return Err(Error::InvalidTotalShares);
        }

        let updated = Member {
            pool_id,
            address: address.clone(),
            share_bps,
        };
        set_member(&env, &updated);
        events::share_updated(&env, pool_id, &address, old_member.share_bps, share_bps);
        Ok(())
    }

    /// Update the operational status of a pool (Active or Inactive).
    pub fn set_pool_status(env: Env, pool_id: u64, status: PoolStatus) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        let mut pool = get_pool(&env, pool_id)?;
        pool.owner.require_auth();

        pool.status = status;
        set_pool(&env, &pool);
        events::pool_status_changed(&env, pool_id, status);
        Ok(())
    }

    /// Retrieve pool details.
    pub fn get_pool(env: Env, pool_id: u64) -> Result<Pool, Error> {
        get_pool(&env, pool_id)
    }

    /// Retrieve a member's configuration within a pool.
    pub fn get_member(env: Env, pool_id: u64, address: Address) -> Result<Member, Error> {
        storage::get_member(&env, pool_id, &address)
    }

    /// Retrieve all configured members of a pool.
    pub fn get_pool_members(env: Env, pool_id: u64) -> Result<Vec<Member>, Error> {
        get_pool_members(&env, pool_id)
    }
}

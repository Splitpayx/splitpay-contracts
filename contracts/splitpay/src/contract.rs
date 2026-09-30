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

    /// Create a pending payment targeting a pool.
    pub fn create_payment(
        env: Env,
        payment_id: u64,
        pool_id: u64,
        payer: Address,
        amount: i128,
    ) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }
        payer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if has_payment(&env, payment_id) {
            return Err(Error::PaymentAlreadyExists);
        }

        let pool = get_pool(&env, pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(Error::InvalidPoolStatus);
        }

        // Validate that total shares equal exactly 10000 before accepting payment
        let members = get_pool_members(&env, pool_id)?;
        if members.is_empty() {
            return Err(Error::InvalidTotalShares);
        }
        let mut total_shares: u32 = 0;
        for m in members.iter() {
            total_shares = total_shares
                .checked_add(m.share_bps)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        if total_shares != MAX_BPS {
            return Err(Error::InvalidTotalShares);
        }

        let created_at = env.ledger().timestamp();
        let payment = Payment {
            id: payment_id,
            pool_id,
            payer: payer.clone(),
            asset: pool.asset.clone(),
            amount,
            status: PaymentStatus::Pending,
            created_at,
        };

        set_payment(&env, &payment);
        events::payment_created(&env, payment_id, pool_id, &payer, amount);
        Ok(())
    }

    /// Settle a payment atomically: snapshots active split, transfers funds from payer,
    /// distributes according to shares, and records permanent distribution records.
    pub fn settle_payment(env: Env, payment_id: u64) -> Result<(), Error> {
        if !has_config(&env) {
            return Err(Error::NotInitialized);
        }

        let mut payment = get_payment(&env, payment_id)?;
        if payment.status == PaymentStatus::Settled {
            return Err(Error::PaymentAlreadySettled);
        }

        payment.payer.require_auth();

        let pool = get_pool(&env, payment.pool_id)?;
        if pool.status != PoolStatus::Active {
            return Err(Error::InvalidPoolStatus);
        }

        // Snapshot current pool members and validate 10000 bps
        let members = get_pool_members(&env, payment.pool_id)?;
        if members.is_empty() {
            return Err(Error::InvalidTotalShares);
        }
        let mut total_shares: u32 = 0;
        for m in members.iter() {
            total_shares = total_shares
                .checked_add(m.share_bps)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        if total_shares != MAX_BPS {
            return Err(Error::InvalidTotalShares);
        }

        // Calculate member allocations with deterministic remainder handling
        let total_amount = payment.amount;
        let mut allocated_sum: i128 = 0;
        let mut allocations = Vec::new(&env);

        for m in members.iter() {
            let alloc = total_amount
                .checked_mul(m.share_bps as i128)
                .ok_or(Error::ArithmeticOverflow)?
                .checked_div(BPS_DENOMINATOR)
                .ok_or(Error::ArithmeticOverflow)?;
            allocated_sum = allocated_sum
                .checked_add(alloc)
                .ok_or(Error::ArithmeticOverflow)?;
            allocations.push_back(alloc);
        }

        // Remainder policy: assign remaining units deterministically to the first member (index 0)
        let remainder = total_amount
            .checked_sub(allocated_sum)
            .ok_or(Error::ArithmeticOverflow)?;
        if remainder > 0 && !allocations.is_empty() {
            let first_alloc = allocations.get(0).unwrap();
            let new_first = first_alloc
                .checked_add(remainder)
                .ok_or(Error::ArithmeticOverflow)?;
            allocations.set(0, new_first);
        }

        // Token transfers
        let token_client = token::Client::new(&env, &payment.asset);
        let contract_address = env.current_contract_address();

        // Step 1: Fund contract from payer
        token_client.transfer(&payment.payer, &contract_address, &total_amount);

        // Step 2: Distribute allocations to members and persist historical distributions
        let mut recipients = Vec::new(&env);
        for (i, m) in members.iter().enumerate() {
            let alloc = allocations.get(i as u32).unwrap();
            if alloc > 0 {
                token_client.transfer(&contract_address, &m.address, &alloc);
            }

            let dist = Distribution {
                payment_id,
                recipient: m.address.clone(),
                amount: alloc,
                share_bps: m.share_bps,
            };
            storage::set_distribution(&env, &dist);
            recipients.push_back(m.address.clone());
            events::distribution_created(&env, payment_id, &m.address, alloc, m.share_bps);
        }
        storage::set_payment_recipients(&env, payment_id, &recipients);

        // Update payment status to Settled
        payment.status = PaymentStatus::Settled;
        set_payment(&env, &payment);
        events::payment_settled(
            &env,
            payment_id,
            payment.pool_id,
            &payment.payer,
            total_amount,
        );

        Ok(())
    }

    /// Retrieve payment details.
    pub fn get_payment(env: Env, payment_id: u64) -> Result<Payment, Error> {
        get_payment(&env, payment_id)
    }

    /// Retrieve the historical distribution for a recipient in a payment.
    pub fn get_distribution(
        env: Env,
        payment_id: u64,
        recipient: Address,
    ) -> Result<Distribution, Error> {
        storage::get_distribution(&env, payment_id, &recipient)
    }

    /// Retrieve all distributions for a settled payment.
    pub fn get_distributions(env: Env, payment_id: u64) -> Result<Vec<Distribution>, Error> {
        if !has_payment(&env, payment_id) {
            return Err(Error::PaymentNotFound);
        }
        let recipients = storage::get_payment_recipients(&env, payment_id);
        let mut dists = Vec::new(&env);
        for r in recipients.iter() {
            if let Ok(d) = storage::get_distribution(&env, payment_id, &r) {
                dists.push_back(d);
            }
        }
        Ok(dists)
    }
}

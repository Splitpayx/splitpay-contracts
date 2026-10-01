#![cfg(test)]

use crate::contract::{SplitPayContract, SplitPayContractClient};
use crate::errors::Error;
use crate::types::{PaymentStatus, PoolStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

struct TestFixture<'a> {
    env: Env,
    client: SplitPayContractClient<'a>,
    _admin: Address,
    token_client: token::Client<'a>,
    stellar_asset: token::StellarAssetClient<'a>,
    asset_address: Address,
}

impl<'a> TestFixture<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(SplitPayContract, ());
        let client = SplitPayContractClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let asset_address = token_contract.address();
        let token_client = token::Client::new(&env, &asset_address);
        let stellar_asset = token::StellarAssetClient::new(&env, &asset_address);

        // Initialize contract
        client.initialize(&admin);

        Self {
            env,
            client,
            _admin: admin,
            token_client,
            stellar_asset,
            asset_address,
        }
    }
}


// ==========================================
// 1. INITIALIZATION TESTS
// ==========================================

#[test]
fn test_successful_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(SplitPayContract, ());
    let client = SplitPayContractClient::new(&env, &contract_id);

    assert_eq!(client.try_initialize(&admin), Ok(Ok(())));
}

#[test]
fn test_duplicate_initialization() {
    let fixture = TestFixture::setup();
    let another_admin = Address::generate(&fixture.env);

    let res = fixture.client.try_initialize(&another_admin);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}


// ==========================================
// 2. POOL MANAGEMENT TESTS
// ==========================================

#[test]
fn test_create_pool() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);

    fixture.env.ledger().set_timestamp(1700000000);
    assert_eq!(
        fixture
            .client
            .try_create_pool(&1, &owner, &fixture.asset_address),
        Ok(Ok(()))
    );

    let pool = fixture.client.get_pool(&1);
    assert_eq!(pool.id, 1);
    assert_eq!(pool.owner, owner);
    assert_eq!(pool.asset, fixture.asset_address);
    assert_eq!(pool.status, PoolStatus::Active);
    assert_eq!(pool.created_at, 1700000000);
}

#[test]
fn test_duplicate_pool() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    let res = fixture
        .client
        .try_create_pool(&1, &owner, &fixture.asset_address);
    assert_eq!(res, Err(Ok(Error::PoolAlreadyExists)));
}

#[test]
fn test_pool_uninitialized() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(SplitPayContract, ());
    let client = SplitPayContractClient::new(&env, &contract_id);

    let res = client.try_create_pool(&1, &owner, &asset);
    assert_eq!(res, Err(Ok(Error::NotInitialized)));
}

#[test]
fn test_set_pool_status() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    assert_eq!(fixture.client.get_pool(&1).status, PoolStatus::Active);

    fixture.client.set_pool_status(&1, &PoolStatus::Inactive);
    assert_eq!(fixture.client.get_pool(&1).status, PoolStatus::Inactive);

    fixture.client.set_pool_status(&1, &PoolStatus::Active);
    assert_eq!(fixture.client.get_pool(&1).status, PoolStatus::Active);
}


// ==========================================
// 3. MEMBER MANAGEMENT TESTS
// ==========================================

#[test]
fn test_add_member() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    assert_eq!(
        fixture.client.try_add_member(&1, &member1, &5000),
        Ok(Ok(()))
    );

    let member = fixture.client.get_member(&1, &member1);
    assert_eq!(member.pool_id, 1);
    assert_eq!(member.address, member1);
    assert_eq!(member.share_bps, 5000);

    let members = fixture.client.get_pool_members(&1);
    assert_eq!(members.len(), 1);
}

#[test]
fn test_duplicate_member() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &5000);

    let res = fixture.client.try_add_member(&1, &member1, &2000);
    assert_eq!(res, Err(Ok(Error::MemberAlreadyExists)));
}

#[test]
fn test_remove_member() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let member2 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &6000);
    fixture.client.add_member(&1, &member2, &4000);

    assert_eq!(fixture.client.get_pool_members(&1).len(), 2);

    fixture.client.remove_member(&1, &member1);
    assert_eq!(fixture.client.get_pool_members(&1).len(), 1);
    assert_eq!(
        fixture.client.try_get_member(&1, &member1),
        Err(Ok(Error::MemberNotFound))
    );
}

#[test]
fn test_remove_nonexistent_member() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    let res = fixture.client.try_remove_member(&1, &member1);
    assert_eq!(res, Err(Ok(Error::MemberNotFound)));
}

#[test]
fn test_update_member_share() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let member2 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &6000);
    fixture.client.add_member(&1, &member2, &4000);

    fixture.client.update_member_share(&1, &member1, &5000);
    assert_eq!(fixture.client.get_member(&1, &member1).share_bps, 5000);
}

#[test]
fn test_invalid_shares() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);

    // Share 0
    let res = fixture.client.try_add_member(&1, &member1, &0);
    assert_eq!(res, Err(Ok(Error::InvalidShare)));

    // Share > 10000
    let res2 = fixture.client.try_add_member(&1, &member1, &10001);
    assert_eq!(res2, Err(Ok(Error::InvalidShare)));
}

#[test]
fn test_total_shares_above_10000_rejected() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let member2 = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &6000);

    // 6000 + 4001 = 10001 > 10000
    let res = fixture.client.try_add_member(&1, &member2, &4001);
    assert_eq!(res, Err(Ok(Error::InvalidTotalShares)));
}


// ==========================================
// 4. PAYMENT CREATION TESTS
// ==========================================

#[test]
fn test_create_valid_payment() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &10000);

    assert_eq!(
        fixture.client.try_create_payment(&100, &1, &payer, &5000),
        Ok(Ok(()))
    );

    let payment = fixture.client.get_payment(&100);
    assert_eq!(payment.id, 100);
    assert_eq!(payment.pool_id, 1);
    assert_eq!(payment.payer, payer);
    assert_eq!(payment.amount, 5000);
    assert_eq!(payment.status, PaymentStatus::Pending);
}

#[test]
fn test_create_payment_zero_or_negative_amount() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &10000);

    let res = fixture.client.try_create_payment(&100, &1, &payer, &0);
    assert_eq!(res, Err(Ok(Error::InvalidAmount)));

    let res2 = fixture.client.try_create_payment(&100, &1, &payer, &-50);
    assert_eq!(res2, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_create_payment_nonexistent_pool() {
    let fixture = TestFixture::setup();
    let payer = Address::generate(&fixture.env);

    let res = fixture.client.try_create_payment(&100, &999, &payer, &5000);
    assert_eq!(res, Err(Ok(Error::PoolNotFound)));
}

#[test]
fn test_create_payment_inactive_pool() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &10000);
    fixture.client.set_pool_status(&1, &PoolStatus::Inactive);

    let res = fixture.client.try_create_payment(&100, &1, &payer, &5000);
    assert_eq!(res, Err(Ok(Error::InvalidPoolStatus)));
}

#[test]
fn test_create_payment_duplicate_id() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &10000);

    fixture.client.create_payment(&100, &1, &payer, &5000);
    let res = fixture.client.try_create_payment(&100, &1, &payer, &3000);
    assert_eq!(res, Err(Ok(Error::PaymentAlreadyExists)));
}

#[test]
fn test_create_payment_shares_below_10000_rejected() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &9999); // 99.99%

    let res = fixture.client.try_create_payment(&100, &1, &payer, &5000);
    assert_eq!(res, Err(Ok(Error::InvalidTotalShares)));
}


// ==========================================
// 5. PAYMENT SETTLEMENT TESTS
// ==========================================

#[test]
fn test_settle_one_member_100_percent() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member1 = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member1, &10000);

    // Fund payer
    fixture.stellar_asset.mint(&payer, &10000);
    assert_eq!(fixture.token_client.balance(&payer), 10000);

    fixture.client.create_payment(&100, &1, &payer, &5000);
    assert_eq!(fixture.client.try_settle_payment(&100), Ok(Ok(())));

    assert_eq!(fixture.token_client.balance(&payer), 5000);
    assert_eq!(fixture.token_client.balance(&member1), 5000);

    let payment = fixture.client.get_payment(&100);
    assert_eq!(payment.status, PaymentStatus::Settled);

    let dist = fixture.client.get_distribution(&100, &member1);
    assert_eq!(dist.amount, 5000);
    assert_eq!(dist.share_bps, 10000);
}

#[test]
fn test_settle_two_members_50_50() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let alice = Address::generate(&fixture.env);
    let bob = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &alice, &5000);
    fixture.client.add_member(&1, &bob, &5000);

    fixture.stellar_asset.mint(&payer, &2000);
    fixture.client.create_payment(&101, &1, &payer, &2000);
    fixture.client.settle_payment(&101);

    assert_eq!(fixture.token_client.balance(&alice), 1000);
    assert_eq!(fixture.token_client.balance(&bob), 1000);
    assert_eq!(fixture.token_client.balance(&payer), 0);

    let d_alice = fixture.client.get_distribution(&101, &alice);
    let d_bob = fixture.client.get_distribution(&101, &bob);
    assert_eq!(d_alice.amount + d_bob.amount, 2000);
}

#[test]
fn test_settle_multiple_members_60_40() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let alice = Address::generate(&fixture.env);
    let bob = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &alice, &6000);
    fixture.client.add_member(&1, &bob, &4000);

    fixture.stellar_asset.mint(&payer, &10000);
    fixture.client.create_payment(&102, &1, &payer, &10000);
    fixture.client.settle_payment(&102);

    assert_eq!(fixture.token_client.balance(&alice), 6000);
    assert_eq!(fixture.token_client.balance(&bob), 4000);
}

#[test]
fn test_settle_three_members_50_30_20() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let alice = Address::generate(&fixture.env);
    let bob = Address::generate(&fixture.env);
    let charlie = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &alice, &5000);
    fixture.client.add_member(&1, &bob, &3000);
    fixture.client.add_member(&1, &charlie, &2000);

    fixture.stellar_asset.mint(&payer, &1000);
    fixture.client.create_payment(&103, &1, &payer, &1000);
    fixture.client.settle_payment(&103);

    assert_eq!(fixture.token_client.balance(&alice), 500);
    assert_eq!(fixture.token_client.balance(&bob), 300);
    assert_eq!(fixture.token_client.balance(&charlie), 200);
}

#[test]
fn test_settle_uneven_amounts_and_deterministic_remainder() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let alice = Address::generate(&fixture.env);
    let bob = Address::generate(&fixture.env);
    let charlie = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    // 33.34% + 33.33% + 33.33% = 100.00%
    fixture.client.add_member(&1, &alice, &3334);
    fixture.client.add_member(&1, &bob, &3333);
    fixture.client.add_member(&1, &charlie, &3333);

    // Pay 100 stroops:
    // Alice base: 100 * 3334 / 10000 = 33
    // Bob base: 100 * 3333 / 10000 = 33
    // Charlie base: 100 * 3333 / 10000 = 33
    // Allocated: 99
    // Remainder: 1 -> deterministically assigned to first member (Alice)
    // Alice total: 34
    // Bob total: 33
    // Charlie total: 33
    // Sum = 34 + 33 + 33 = 100!
    fixture.stellar_asset.mint(&payer, &100);
    fixture.client.create_payment(&104, &1, &payer, &100);
    fixture.client.settle_payment(&104);

    let bal_alice = fixture.token_client.balance(&alice);
    let bal_bob = fixture.token_client.balance(&bob);
    let bal_charlie = fixture.token_client.balance(&charlie);

    assert_eq!(bal_alice, 34);
    assert_eq!(bal_bob, 33);
    assert_eq!(bal_charlie, 33);
    assert_eq!(bal_alice + bal_bob + bal_charlie, 100);
}

#[test]
fn test_settle_duplicate_settlement_rejected() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let member = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &member, &10000);

    fixture.stellar_asset.mint(&payer, &2000);
    fixture.client.create_payment(&105, &1, &payer, &1000);
    fixture.client.settle_payment(&105);

    // Attempt second settlement
    let res = fixture.client.try_settle_payment(&105);
    assert_eq!(res, Err(Ok(Error::PaymentAlreadySettled)));
}

#[test]
fn test_historical_distributions_remain_unchanged() {
    let fixture = TestFixture::setup();
    let owner = Address::generate(&fixture.env);
    let alice = Address::generate(&fixture.env);
    let bob = Address::generate(&fixture.env);
    let payer = Address::generate(&fixture.env);

    // Initial split: Alice 60%, Bob 40%
    fixture
        .client
        .create_pool(&1, &owner, &fixture.asset_address);
    fixture.client.add_member(&1, &alice, &6000);
    fixture.client.add_member(&1, &bob, &4000);

    fixture.stellar_asset.mint(&payer, &2000);

    // Settle Payment #1 (1000 stroops)
    fixture.client.create_payment(&1, &1, &payer, &1000);
    fixture.client.settle_payment(&1);

    assert_eq!(fixture.token_client.balance(&alice), 600);
    assert_eq!(fixture.token_client.balance(&bob), 400);

    let p1_alice = fixture.client.get_distribution(&1, &alice);
    let p1_bob = fixture.client.get_distribution(&1, &bob);
    assert_eq!(p1_alice.amount, 600);
    assert_eq!(p1_bob.amount, 400);

    // Owner reconfigures split: update Bob first to 30% so total does not exceed 10000, then Alice to 70%
    fixture.client.update_member_share(&1, &bob, &3000);
    fixture.client.update_member_share(&1, &alice, &7000);

    // Settle Payment #2 (1000 stroops)
    fixture.client.create_payment(&2, &1, &payer, &1000);
    fixture.client.settle_payment(&2);

    assert_eq!(fixture.token_client.balance(&alice), 600 + 700);
    assert_eq!(fixture.token_client.balance(&bob), 400 + 300);

    // CRITICAL PROTOCOL INVARIANT: Historical payment #1 distribution is COMPLETELY UNCHANGED
    let p1_alice_after = fixture.client.get_distribution(&1, &alice);
    let p1_bob_after = fixture.client.get_distribution(&1, &bob);
    assert_eq!(p1_alice_after.amount, 600);
    assert_eq!(p1_bob_after.amount, 400);

    // Payment #2 distribution matches new split
    let p2_alice = fixture.client.get_distribution(&2, &alice);
    let p2_bob = fixture.client.get_distribution(&2, &bob);
    assert_eq!(p2_alice.amount, 700);
    assert_eq!(p2_bob.amount, 300);
}

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

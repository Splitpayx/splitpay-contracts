use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PoolStatus {
    Active = 1,
    Inactive = 2,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PaymentStatus {
    Pending = 1,
    Settled = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub id: u64,
    pub owner: Address,
    pub asset: Address,
    pub status: PoolStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub pool_id: u64,
    pub address: Address,
    pub share_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub id: u64,
    pub pool_id: u64,
    pub payer: Address,
    pub asset: Address,
    pub amount: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Distribution {
    pub payment_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub share_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractConfig {
    pub admin: Address,
}

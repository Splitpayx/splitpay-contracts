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

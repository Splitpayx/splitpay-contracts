use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    PoolNotFound = 3,
    PaymentNotFound = 4,
    MemberNotFound = 5,
    MemberAlreadyExists = 6,
    Unauthorized = 7,
    InvalidPoolStatus = 8,
    InvalidShare = 9,
    InvalidTotalShares = 10,
    InvalidAmount = 11,
    InvalidAsset = 12,
    PaymentAlreadyExists = 13,
    PaymentAlreadySettled = 14,
    InvalidPayment = 15,
    PoolAlreadyExists = 16,
    ArithmeticOverflow = 17,
}

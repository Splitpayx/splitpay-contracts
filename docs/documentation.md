# SplitPay

## 1. Overview

SplitPay is a collaborative payment and fund distribution platform built on Stellar.

It allows individuals, teams, agencies, freelancers, and other groups to create payment pools, define how incoming funds should be distributed, receive payments in supported Stellar assets, and distribute those funds automatically according to predefined split rules.

The system consists of:

* `splitpay-web` - primary web application
* `splitpay-contract` - Soroban smart contract responsible for on-chain pool and distribution logic
* `splitpay-mobile` - React Native mobile application
* `splitpay-sdk` - shared client SDK, introduced when the contract interface stabilizes
* `splitpay-api` - optional off-chain backend for metadata, indexing, notifications and application services

The blockchain is responsible for financial rules and settlement that require verifiability.

The application layer is responsible for UX, metadata and services that do not need to live on-chain.

---

# 2. Product Vision

SplitPay should allow a group of people to receive money together without requiring one person to act as the trusted middleman.

Instead of:

```text
Client
  ↓
One person's wallet
  ↓
Manual calculations
  ↓
Manual transfers
  ↓
Team members
```

SplitPay should provide:

```text
Payer
  ↓
SplitPay Pool
  ↓
Soroban Contract
  ↓
┌──────────┬──────────┬──────────┐
Member A   Member B   Member C
```

The contract enforces the distribution rules.

---

# 3. Core Concepts

## Pool

A Pool is an on-chain collaborative payment configuration.

A pool contains:

* unique pool identifier
* owner
* members
* supported asset
* split configuration
* status
* creation timestamp
* optional metadata reference

A pool does not represent a bank account.

It represents rules for how funds should be distributed.

## Member

A member is an address participating in a pool.

Each member has a percentage share.

Example:

```text
Alice    50%
Bob      30%
Charlie  20%
```

The total must always equal exactly 100%.

## Payment

A Payment represents funds deposited into a pool for distribution.

A payment contains:

* payment identifier
* pool identifier
* payer
* asset
* gross amount
* split snapshot
* timestamp

The split configuration used for a payment must be immutable after the payment is created.

## Distribution

A Distribution represents the amounts assigned to individual members from a payment.

Example:

```text
Payment: 100 USDC

Alice    50 USDC
Bob      30 USDC
Charlie  20 USDC
```

---

# 4. Blockchain Responsibility

The Soroban contract is the source of truth for:

* pool creation
* pool membership
* split configuration
* split validation
* supported asset configuration
* payment settlement
* distribution calculations
* payment split snapshots
* authorization
* on-chain events

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
* relevant financial state

The contract must not depend on the web application to enforce financial rules.

The web application is a client of the contract.

---

# 5. Web Responsibility

`splitpay-web` is responsible for:

* authentication/onboarding
* wallet connection
* pool discovery
* pool management UI
* member management UI
* split configuration UI
* payment creation UI
* transaction status
* transaction history
* portfolio/balance presentation
* notifications
* application metadata

The web application must never be treated as the authoritative source for financial balances or split calculations.

---

# 6. Mobile Responsibility

`splitpay-mobile` will provide the same core functionality through React Native.

It should consume the same contract interface and eventually use the same SDK as the web application.

Business rules must not be duplicated between web and mobile.

---

# 7. Contract Architecture

The initial contract should be a single SplitPay protocol contract.

It should interact with Stellar assets through the Stellar Asset Contract / SEP-41 token interface.

Conceptually:

```text
User
 │
 │ authorize
 ▼
SplitPay Contract
 │
 │ token transfer
 ▼
Stellar Asset Contract
 │
 ▼
Recipient Accounts
```

The SplitPay contract is not a token contract.

It is a payment splitting and distribution contract.

---

# 8. Initial Contract Interface

The initial contract should expose functionality conceptually equivalent to:

```text
initialize(admin)

create_pool(pool_id, owner, asset)

add_member(pool_id, member, share)

remove_member(pool_id, member)

update_member_share(pool_id, member, share)

set_pool_status(pool_id, status)

get_pool(pool_id)

get_member(pool_id, member)

get_pool_members(pool_id)

create_payment(payment_id, pool_id, payer, amount)

settle_payment(payment_id)

get_payment(payment_id)

get_distribution(payment_id, member)
```

The exact interface may change during implementation if Soroban conventions or security considerations require it.

---

# 9. Pool Lifecycle

```text
CREATE
  ↓
CONFIGURE
  ↓
ACTIVE
  ↓
PAYMENT
  ↓
SETTLED
  ↓
COMPLETED
```

A pool should not accept payments until its configuration is valid.

A pool's split percentages must always equal:

```text
10000 basis points
```

Therefore:

```text
50% = 5000
25% = 2500
25% = 2500

Total = 10000
```

Basis points should be preferred over floating point percentages.


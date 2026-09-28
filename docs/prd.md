# SplitPay Contract PRD

## 1. Objective

Build the SplitPay Soroban smart contract responsible for collaborative payment pools and automatic fund distribution on Stellar.

The contract must allow a pool owner to define members and their shares, accept payments in a Stellar asset, and atomically distribute the payment according to the configured split.

The contract is the financial source of truth.

---

# 2. Technology

Use:

* Rust
* Soroban SDK
* Cargo
* Stellar CLI
* WASM
* Stellar Testnet

Do not introduce another smart contract framework.

Follow current Stellar/Soroban conventions and official documentation.

---

# 3. Core Model

Implement these concepts:

```text
Pool
Member
Payment
Distribution
ContractConfig
```

### Pool

```text
id
owner
asset
status
created_at
```

### Member

```text
pool_id
address
share_bps
```

### Payment

```text
id
pool_id
payer
asset
amount
status
created_at
```

### Distribution

```text
payment_id
recipient
amount
share_bps
```

---

# 4. Pool Rules

A pool:

* has exactly one owner
* has one configured asset
* can have multiple members
* requires total shares to equal 10000 basis points
* cannot accept payments while invalid
* cannot be mutated by unauthorized users

Example:

```text
Alice    6000
Bob      3000
Charlie  1000

Total = 10000
```

---

# 5. Payment Rules

A payment must:

* reference an existing pool
* use the pool's configured asset
* contain a positive amount
* have a unique payment ID
* be funded by an authorized payer
* snapshot the pool's current split
* settle only once

---

# 6. Settlement

Settlement should perform:

```text
1. Validate payment
2. Validate pool
3. Validate pool configuration
4. Snapshot members and shares
5. Transfer/fund the contract as required
6. Calculate allocations
7. Transfer allocations to recipients
8. Persist payment state
9. Emit settlement/distribution events
```

Settlement must be atomic.

If one required transfer fails, the complete invocation must fail.

---

# 7. Token Integration

Do not implement a custom token.

Use the Stellar Asset Contract / SEP-41 token interface through Soroban's token client.

The contract should accept an asset contract address as configuration.

The implementation should support XLM/SAC and issued Stellar assets without hard-coding an issuer.

---

# 8. Contract Interface

Implement the smallest clean public interface necessary for V1.

Expected methods:

```rust
initialize(...)
create_pool(...)
add_member(...)
remove_member(...)
update_member_share(...)
set_pool_status(...)
get_pool(...)
get_member(...)
get_pool_members(...)
create_payment(...)
settle_payment(...)
get_payment(...)
get_distribution(...)
```

The exact Rust signatures should follow Soroban SDK conventions.

Do not expose unnecessary functions.

---

# 9. Authorization

Authorization must be explicit.

Pool mutations:

```text
pool.owner.require_auth()
```

Payment funding:

```text
payer.require_auth()
```

Administrative operations:

```text
admin.require_auth()
```

Never trust caller-supplied identity arguments without checking authorization.

---

# 10. Errors

Create a typed contract error enum.

Expected errors include:

```text
AlreadyInitialized
NotInitialized
PoolNotFound
PaymentNotFound
MemberNotFound
MemberAlreadyExists
Unauthorized
InvalidPoolStatus
InvalidShare
InvalidTotalShares
InvalidAmount
InvalidAsset
PaymentAlreadyExists
PaymentAlreadySettled
InvalidPayment
```

Use contract errors consistently.

Avoid string-based error handling.

---

# 11. Events

Emit structured events for:

```text
PoolCreated
MemberAdded
MemberRemoved
ShareUpdated
PoolStatusChanged
PaymentCreated
PaymentSettled
DistributionCreated
```

Events must contain enough information for a future indexer to reconstruct activity.

---

# 12. Arithmetic

Use integer arithmetic only.

Represent percentages using basis points:

```text
10000 = 100%
```

Never use floating point.

For:

```text
amount = 1000
share = 2500
```

the allocation is:

```text
250
```

The implementation must guarantee:

```text
sum(allocations) == payment.amount
```

Define and test a deterministic remainder policy.

---

# 13. State Design

Keep storage modular.

Do not store an ever-growing list of every payment inside a single pool object.

Prefer individual storage entries keyed by IDs and addresses where appropriate.

The storage design must remain compatible with Soroban's storage and TTL model.

---

# 14. Tests

The test suite must cover:

### Pool

* initialize
* create pool
* duplicate pool
* invalid owner
* valid status
* invalid status

### Members

* add member
* duplicate member
* remove member
* update share
* unauthorized mutation
* invalid shares
* total share validation

### Payments

* valid payment
* invalid payment
* duplicate payment
* wrong asset
* zero amount
* nonexistent pool

### Settlement

* one member
* two members
* multiple members
* exact percentages
* uneven amounts
* remainder handling
* changed pool configuration
* duplicate settlement
* unauthorized settlement

### Security

* unauthorized owner mutation

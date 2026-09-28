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

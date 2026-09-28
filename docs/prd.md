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

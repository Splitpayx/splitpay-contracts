# SplitPay Smart Contracts (Stellar / Soroban)

[![Built with Soroban](https://img.shields.io/badge/Stellar-Soroban_v22-blue.svg)](https://developers.stellar.org/docs/build/smart-contracts/overview)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

V1 smart contract for **SplitPay**, a collaborative payment splitting and distribution protocol built natively on Stellar using Soroban.

SplitPay enables pool owners to define members and percentage shares in basis points, accept payment funding in any standard Stellar Asset (SEP-41 / Stellar Asset Contract), and atomically calculate and distribute allocations directly to members in a single transaction.

---

## Architecture Overview

```
                      +-----------------------------+
                      |       Pool Owner / Payer    |
                      +--------------+--------------+
                                     |
               1. create_pool()      | 4. settle_payment()
               2. add_member()       |    (Atomically transfers funds)
               3. create_payment()   |
                                     v
+--------------------------------------------------------------------------+
|                        SplitPay Contract (Soroban)                       |
|                                                                          |
|  * Pool Config: ID, Owner, Asset (SAC Address), Status (Active/Inactive)  |
|  * Members: Map of Address -> Basis Points (BPS, sum must equal 10,000)  |
|  * Payment Snapshot: ID, Pool, Payer, Asset, Amount, Status              |
|  * Settlement Engine:                                                    |
|     - Validates Pool Status & 10,000 Total BPS                           |
|     - Snapshots current Member distributions                             |
|     - Calculates deterministic integer allocations with remainder policy |
|     - Atomically calls Stellar Asset Contract `transfer_from()`          |
|     - Dispatches funds directly to member addresses                      |
|     - Records immutable Distribution records and emits events            |
+------------------------------------+-------------------------------------+
                                     |
                                     | SEP-41 Token Interface
                                     v
                      +-----------------------------+
                      | Stellar Asset Contract(SAC) |
                      |    (XLM, USDC, EURC, etc.)  |
                      +--------------+--------------+
                                     |
                 +-------------------+-------------------+
                 |                   |                   |
                 v                   v                   v
           Member 1 (BPS)      Member 2 (BPS)      Member 3 (BPS)
```

### Critical Architectural Decisions

1. **No Custom Token Contract**: SplitPay is **not** a token contract and does not maintain internal token balances. It integrates directly with Stellar assets via the official Soroban `token::Client` (SEP-41).
2. **Deterministic Integer Math & Remainder Policy**: Floating-point arithmetic is strictly prohibited. Basis points are used (`10,000 BPS = 100.00%`). To prevent unit loss during integer division (`amount * share_bps / 10,000`), the remainder difference (`payment_amount - sum_allocated`) is deterministically awarded to the first active member.
   $$\sum_{i=1}^{n} \text{distribution}_i = \text{payment\_amount}$$
3. **Immutability of Historical Distributions**: Payments take an immutable snapshot of member shares at settlement time. Subsequent changes to pool members or shares never alter past distributions.
4. **Strict Authorization**:
   - `create_pool`: Pool owner must authorize.
   - `add_member`, `remove_member`, `update_member_share`, `set_pool_status`: Pool owner must authorize.
   - `create_payment`: Payer must authorize.
   - `settle_payment`: Payer or pool owner must authorize.
   - `initialize`: Contract administrator must authorize.

---

## Contract Concepts & Storage Model

| Entity | Fields | Storage Type | Description |
| :--- | :--- | :--- | :--- |
| **ContractConfig** | `admin` | Instance Storage | Contract administrator |
| **Pool** | `id`, `owner`, `asset`, `status`, `created_at` | Persistent Storage | Payment pool configuration |
| **Member** | `pool_id`, `address`, `share_bps` | Persistent Storage | Pool member with basis point allocation |
| **Payment** | `id`, `pool_id`, `payer`, `asset`, `amount`, `status`, `created_at` | Persistent Storage | Payment record |
| **Distribution** | `payment_id`, `recipient`, `amount`, `share_bps` | Persistent Storage | Individual recipient disbursement record |

---

## Contract Interface (Public Methods)

### Administrative & Pool Management
- `initialize(admin: Address)`: Initialize contract administrator.
- `create_pool(pool_id: u64, owner: Address, asset: Address)`: Create a new pool.
- `set_pool_status(pool_id: u64, status: PoolStatus)`: Activate or deactivate a pool.
- `get_pool(pool_id: u64) -> Pool`: Query pool information.

### Member Management
- `add_member(pool_id: u64, address: Address, share_bps: u32)`: Add a member with specified basis points.
- `remove_member(pool_id: u64, address: Address)`: Remove a member from the pool.
- `update_member_share(pool_id: u64, address: Address, new_share_bps: u32)`: Update an existing member's share.
- `get_member(pool_id: u64, address: Address) -> Member`: Query a specific pool member.
- `get_pool_members(pool_id: u64) -> Vec<Member>`: List all members configured for a pool.

### Payment & Settlement
- `create_payment(payment_id: u64, pool_id: u64, payer: Address, amount: i128)`: Create an unsettled payment.
- `settle_payment(payment_id: u64)`: Atomically fund and disburse payment across members.
- `get_payment(payment_id: u64) -> Payment`: Query payment status and details.
- `get_distribution(payment_id: u64, recipient: Address) -> Distribution`: Query a recipient's distribution.
- `get_distributions(payment_id: u64) -> Vec<Distribution>`: Query all distributions for a payment.

---

## Prerequisites

- **Rust**: `1.80+` (or toolchain with `wasm32-unknown-unknown` target installed)
- **Target**: `rustup target add wasm32-unknown-unknown`
- **Stellar CLI**: `v22+` (`cargo install --locked stellar-cli --features opt`)

---

## Local Development & Testing

### Running Tests

Run the full automated test suite containing 27 unit, integration, and invariant tests:

```bash
cargo test
```

### Running Formatter & Linter

```bash
cargo fmt --all --check
```

---

## Building the Contract

Compile the release WASM bytecode:

```bash
# Using cargo rustc
cargo rustc --manifest-path contracts/splitpay/Cargo.toml --target wasm32-unknown-unknown --release --crate-type cdylib

# Or via Makefile
make build

# Or via script
bash scripts/build.sh
```

The optimized WASM artifact will be generated at:
`target/wasm32-unknown-unknown/release/splitpay.wasm`

---

## Testnet Deployment Workflow

### 1. Configure Stellar CLI Identity & Network

```bash
# Configure Testnet network
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate or import deployer identity
stellar keys generate deployer --network testnet

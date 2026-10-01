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

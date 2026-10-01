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

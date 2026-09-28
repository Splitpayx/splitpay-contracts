#!/usr/bin/env bash
set -euo pipefail

echo "==> Building SplitPay Soroban smart contract for wasm32..."
cargo rustc --manifest-path contracts/splitpay/Cargo.toml --target wasm32-unknown-unknown --release --crate-type cdylib

WASM_OUTPUT="target/wasm32-unknown-unknown/release/splitpay.wasm"
if [ -f "$WASM_OUTPUT" ]; then
    echo "==> Build successful: $WASM_OUTPUT"
    ls -lh "$WASM_OUTPUT"
else
    echo "==> Build failed: output wasm not found"
    exit 1
fi

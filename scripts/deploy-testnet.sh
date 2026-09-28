#!/usr/bin/env bash
set -euo pipefail

# SplitPay Testnet Deployment Script
# Prerequisites: Stellar CLI installed and configured with a funded testnet identity.

NETWORK="${NETWORK:-testnet}"
IDENTITY="${IDENTITY:-alice}"
WASM_PATH="target/wasm32-unknown-unknown/release/splitpay.wasm"

echo "==> Verifying wasm binary..."
if [ ! -f "$WASM_PATH" ]; then
    echo "==> Binary not found. Building..."
    ./scripts/build.sh
fi

echo "==> Deploying SplitPay contract to Stellar $NETWORK using identity '$IDENTITY'..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM_PATH" \
    --source "$IDENTITY" \
    --network "$NETWORK")

echo "==> Contract successfully deployed!"
echo "    Contract Address: $CONTRACT_ID"

# Get source address for admin initialization
ADMIN_ADDR=$(stellar keys address "$IDENTITY")
echo "==> Initializing contract with admin: $ADMIN_ADDR..."
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$ADMIN_ADDR"

echo "==> Contract initialized successfully!"
echo "==> SplitPay deployment complete."

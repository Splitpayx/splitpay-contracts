#!/usr/bin/env bash
set -euo pipefail

echo "==> Running SplitPay contract tests and invariant validation..."
cargo test "$@"
echo "==> All tests and invariants passed successfully!"

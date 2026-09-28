default: build

build:
	cargo rustc --manifest-path contracts/splitpay/Cargo.toml --target wasm32-unknown-unknown --release --crate-type cdylib


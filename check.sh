#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --workspace --all-targets
cargo check --workspace --all-features --target wasm32-unknown-unknown

cargo fmt --all

cargo clippy --workspace --all-targets --all-features --  -D warnings -W clippy::all

# tests
cargo nextest r --workspace --all-features
# this isn't a lib (yet?)
# cargo test --workspace --doc

trunk build

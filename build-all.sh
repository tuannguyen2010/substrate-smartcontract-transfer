#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path erc20/Cargo.toml
cargo +nightly contract build --manifest-path otherContract/Cargo.toml
cargo +nightly contract build
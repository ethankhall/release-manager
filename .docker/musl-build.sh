#!/bin/bash -eux

cargo fmt -- --write-mode diff
cargo check
cargo build --release
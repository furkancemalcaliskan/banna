#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repository_root"

sh scripts/repository-policy.sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release --locked --bins
target/release/banna --help >/dev/null
target/release/banna-cli --help >/dev/null
cargo package --locked


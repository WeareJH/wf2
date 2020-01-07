set -eo pipefail
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt --all -- --check

set -eo pipefail
cargo fix --allow-dirty --allow-staged && cargo fmt
cargo clippy
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt --all -- --check

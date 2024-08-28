#!/bin/bash
set -euo pipefail

# Source the function from the change_dir.sh script
source ./ci/change-dir.sh

# Call the function with parameters
change_directory "$@"

# Build
export RUSTFLAGS="-D warnings"
export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

# Build and test main crate
cargo build --locked --all-features

#!/bin/bash
set -euo pipefail

# Source the function from the change_dir.sh script
source ./ci/change-dir.sh

# Call the function with parameters
change_directory "$@"

# Stylus check
cargo stylus check

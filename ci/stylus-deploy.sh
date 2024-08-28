#!/bin/bash

# Exit the script if fails but allowing undefined variables
set -eo pipefail

# Check if PRIVATE_KEY is not set or is empty
if [ -z "${PRIVATE_KEY:-}" ]; then
    # Check if the .env file exists
    if [ -f .env ]; then
        source .env
        echo ".env file sourced successfully."
    else
        echo ".env file not found"
        exit 1
    fi
fi

# Source the function from the change_dir.sh script
source ./ci/change-dir.sh

# Call the function with parameters
change_directory "$@"

cargo stylus deploy --private-key $PRIVATE_KEY
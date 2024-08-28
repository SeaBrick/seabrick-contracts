#!/bin/bash

# Sources the enviroments
source .env

# Source the function from the change_dir.sh script
source ./ci/change-dir.sh

# Call the function with parameters
change_directory "$@"

cargo stylus deploy --private-key $PRIVATE_KEY
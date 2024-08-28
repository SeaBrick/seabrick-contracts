#!/bin/bash
set -euo pipefail

# Function to handle the --path parameter and change directory
change_directory() {
    local path=""

    while [[ "$#" -gt 0 ]]; do
        case $1 in
            --path) path="$2"; shift ;;
            -h|--help) echo "Usage: $0 --path <directory_path>"; return 0 ;;
            *) echo "Unknown parameter passed: $1"; return 1 ;;
        esac
        shift
    done

    if [ -z "$path" ]; then
        echo "Error: --path is required"
        return 1
    fi

    if cd "$path" 2>/dev/null; then
        echo "Changed directory to $path"
    else
        echo "Error: Could not change directory to $path"
        return 1
    fi
}

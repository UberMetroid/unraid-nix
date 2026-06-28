#!/bin/bash
# Unified test runner for Rust backend and WebUI template page scripts
set -e

echo "=== 1/2 Running Rust Backend Tests ==="
cargo test

if command -v php &>/dev/null; then
    echo "=== 2/2 Running WebUI Template Checks ==="
    php -f "$(dirname "$0")/web_tabs_validation.php"
else
    echo "=== 2/2 Skipping WebUI Checks (PHP not installed locally) ==="
fi

echo "🎉 All tests completed successfully!"

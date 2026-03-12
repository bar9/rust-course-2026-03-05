#!/bin/bash
set -e

echo "Running tests on host..."

# Move embedded-specific files aside
mv .cargo/config.toml .cargo/config.toml.bak
mv build.rs build.rs.bak

# Restore on exit (success or failure)
trap 'mv .cargo/config.toml.bak .cargo/config.toml; mv build.rs.bak build.rs' EXIT

# Run tests without embedded features
cargo test --lib --no-default-features

echo "Tests passed!"

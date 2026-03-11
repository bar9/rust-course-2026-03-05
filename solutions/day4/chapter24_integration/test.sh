#!/bin/bash

# Script to run tests without embedded cargo config interference

echo "🧪 Running Chapter 17 Integration Tests..."
mv .cargo .cargo.bak 2>/dev/null || true
mv build.rs build.rs.bak 2>/dev/null || true
cargo test --lib --no-default-features "$@"
mv .cargo.bak .cargo 2>/dev/null || true
mv build.rs.bak build.rs 2>/dev/null || true
echo "✅ Tests completed!"
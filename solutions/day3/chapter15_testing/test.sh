#!/bin/bash

# Script to run tests without embedded cargo config interference

echo "🧪 Running Chapter 15 Tests..."
mv .cargo .cargo.bak 2>/dev/null || true
cargo test --lib --no-default-features "$@"
mv .cargo.bak .cargo 2>/dev/null || true
echo "✅ Tests completed!"
# Chapter 19: Cargo & Dependency Management

Cargo is Rust's build system and package manager. It handles dependencies, compilation, testing, and distribution. This chapter covers dependency management, from editions and toolchains to private registries and reproducible builds.

## 1. Rust Editions

Rust editions are opt-in milestones released every three years that allow the language to evolve while maintaining stability guarantees. All editions remain fully interoperable - crates using different editions work together seamlessly.

### Available Editions

| Edition | Released | Default Resolver | Key Changes |
|---------|----------|-----------------|-------------|
| **2015** | Rust 1.0 | v1 | Original edition, `extern crate` required |
| **2018** | Rust 1.31 | v1 | Module system improvements, `async`/`await`, NLL |
| **2021** | Rust 1.56 | v2 | Disjoint captures, `into_iter()` arrays, reserved identifiers |
| **2024** | Rust 1.85 | v3 | MSRV-aware resolver, `gen` keyword, unsafe env functions |

### Key Edition Changes

**Edition 2018:**
- No more `extern crate` declarations (except for macros)
- Uniform path syntax in `use` statements
- `async`/`await` keywords reserved
- Non-lexical lifetimes (NLL)
- Module system simplification

**Edition 2021:**
- Disjoint captures in closures (only capture used fields)
- `array.into_iter()` iterates by value
- New reserved keywords: `try`
- Default to resolver v2 for Cargo
- Panic macros require format strings

**Edition 2024:**
- MSRV-aware dependency resolution (resolver v3)
- `gen` keyword for generators/coroutines
- `std::env::set_var` and `remove_var` marked unsafe
- Tail expression temporary lifetime changes
- `unsafe extern` blocks and attributes

### Configuration and Migration

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"
```

```bash
# Migrate code to next edition (modifies files)
cargo fix --edition

# Apply idiomatic style changes
cargo fix --edition --edition-idioms

# Then update Cargo.toml manually
```

### Edition Selection Strategy

| Project Type | Recommended Edition | Rationale |
|--------------|-------------------|-----------|
| New projects | Latest stable | Access to all improvements |
| Libraries | Conservative (2018/2021) | Wider compatibility |
| Applications | Latest stable | Modern features |
| Legacy code | Keep current | Migrate when beneficial |

## 2. Toolchain Channels

Rust uses a release train model with three channels:

```
Nightly (daily) → Beta (6 weeks) → Stable (6 weeks)
```

| Channel | Release Cycle | Stability | Use Case |
|---------|--------------|-----------|----------|
| **Stable** | 6 weeks | Guaranteed stable | Production |
| **Beta** | 6 weeks | Generally stable | Testing upcoming releases |
| **Nightly** | Daily | May break | Experimental features |

### Stable Channel

```bash
# Install or switch to stable
rustup default stable

# Use specific stable version
rustup install 1.82.0
rustup default 1.82.0
```

### Beta Channel

```bash
# Switch to beta
rustup default beta

# Test with beta in CI
rustup run beta cargo test
```

### Nightly Channel

```bash
# Use nightly for specific project
rustup override set nightly

# Install specific nightly
rustup install nightly-2024-11-28
```

Enabling unstable features:
```rust
// Only works on nightly
#![feature(generators)]
#![feature(type_alias_impl_trait)]
```

### Project Toolchain Configuration

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.82.0"  # Or "stable", "beta", "nightly"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

Override commands:
```bash
# Set override for current directory
rustup override set nightly

# Run command with specific toolchain
cargo +nightly build
cargo +1.82.0 test
```

### CI/CD Multi-Channel Testing

```yaml
# GitHub Actions
strategy:
  matrix:
    rust: [stable, beta, nightly]
    continue-on-error: ${{ matrix.rust == 'nightly' }}

steps:
  - uses: actions-rs/toolchain@v1
    with:
      toolchain: ${{ matrix.rust }}
      override: true
```

## 3. Dependency Resolution

### Version Requirements

Cargo uses semantic versioning (SemVer) with various requirement operators:

```toml
[dependencies]
# Caret (default) - compatible versions
serde = "1.0"        # means ^1.0.0

# Exact version
exact = "=1.0.0"

# Range
range = ">=1.2, <1.5"

# Wildcard
wildcard = "1.0.*"

# Tilde - patch updates only
tilde = "~1.0.0"
```

### Transitive Dependencies

Cargo builds a dependency graph and resolves versions using maximum version strategy:

```
Your Project
├── crate-a = "1.0"
│   └── shared = "2.1"    # Transitive dependency
└── crate-b = "2.0"
    └── shared = "2.3"    # Same dependency, different version
```

Resolution: Cargo picks `shared = "2.3"` (highest compatible version).

### Resolver Versions

| Resolver | Default For | Behavior |
|----------|------------|----------|
| **v1** | Edition 2015/2018 | Unifies features across all uses |
| **v2** | Edition 2021 | Independent feature resolution per target |
| **v3** | Edition 2024 (Rust 1.84+) | MSRV-aware dependency selection, default in 2024 |

```toml
# Explicit resolver configuration
[package]
edition = "2018"
resolver = "2"  # Opt into v2 resolver

# For workspaces
[workspace]
members = ["crate-a", "crate-b"]
resolver = "2"
```

Key v2 differences:
- Platform-specific dependencies don't affect other platforms
- Build dependencies don't share features with normal dependencies
- Dev dependencies only activate features when building tests/examples

Key v3 additions (Edition 2024 default):
- MSRV-aware dependency resolution when `rust-version` is specified
- Falls back to compatible versions when newer versions require higher MSRV
- Better support for workspaces with mixed Rust versions

## 4. Cargo.lock

The `Cargo.lock` file pins exact dependency versions for reproducible builds.

### When to Commit

| Project Type | Commit? | Reason |
|--------------|---------|--------|
| **Binary/Application** | Yes | Reproducible builds |
| **Library** | No | Allow flexible version resolution |
| **Workspace root** | Yes | Consistent versions across workspace |

### Lock File Usage

```bash
# Build with exact lock file versions
cargo build --locked

# Update all dependencies
cargo update

# Update specific dependency
cargo update -p serde

# Update to specific version
cargo update -p tokio --precise 1.21.0
```

## 5. Minimum Supported Rust Version (MSRV)

```toml
[package]
rust-version = "1.74"  # Minimum Rust version
```

### Finding and Testing MSRV

```bash
# Install cargo-msrv
cargo install cargo-msrv

# Find minimum version
cargo msrv find

# Verify declared MSRV
cargo msrv verify
```

### CI Testing

```yaml
# GitHub Actions
- name: Test MSRV
  run: |
    rustup install $(grep rust-version Cargo.toml | cut -d'"' -f2)
    cargo test --locked
```

### MSRV Policy Guidelines

| Project Type | Suggested MSRV | Rationale |
|--------------|----------------|-----------|
| Foundational libraries | 6-12 months old | Maximum compatibility |
| Application libraries | 3-6 months old | Balance features/compatibility |
| Applications | Current stable | Use latest features |
| Internal tools | Latest stable | No external users |

## 6. Workspace Management

Workspaces allow managing multiple related crates in a single repository:

```toml
# Root Cargo.toml
[workspace]
members = ["crate-a", "crate-b", "crate-c"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = "1.47"

[workspace.package]
authors = ["Your Name"]
edition = "2021"
license = "MIT"
repository = "https://github.com/user/repo"
```

Member crates inherit workspace configuration:

```toml
# crate-a/Cargo.toml
[package]
name = "crate-a"
version = "0.1.0"
authors.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
tokio.workspace = true
```

### Workspace Commands

```bash
# Build all workspace members
cargo build --workspace

# Test specific member
cargo test -p crate-a

# Run example from workspace member
cargo run -p crate-b --example demo
```

## 7. Private Registries

### Registry Options

| Solution | Type | Best For |
|----------|------|----------|
| **Kellnr** | Self-hosted registry | Small-medium teams |
| **Alexandrie** | Alternative registry | Custom deployments |
| **Panamax** | Mirror | Offline development |
| **Artifactory** | Enterprise | Large organizations |

### Kellnr Setup

```toml
# .cargo/config.toml
[registries]
kellnr = {
    index = "git://your-kellnr-host:9418/index",
    token = "your-auth-token"
}
```

Docker deployment:
```bash
docker run -p 8000:8000 \
  -e "KELLNR_ORIGIN__HOSTNAME=your-domain" \
  ghcr.io/kellnr/kellnr:latest
```

### Alexandrie Configuration

```toml
# alexandrie.toml
[database]
url = "postgresql://localhost/alexandrie"

[storage]
type = "s3"
bucket = "my-crates"
region = "us-east-1"
```

### Panamax Mirror

```bash
# Initialize mirror
panamax init my-mirror

# Sync dependencies
cargo vendor
panamax sync my-mirror vendor/

# Serve mirror
panamax serve my-mirror --port 8080
```

Client configuration:
```toml
# .cargo/config.toml
[source.my-mirror]
registry = "http://panamax.internal/crates.io-index"

[source.crates-io]
replace-with = "my-mirror"
```

### Artifactory Setup

```toml
# .cargo/config.toml
[registries]
artifactory = {
    index = "https://artifactory.company.com/artifactory/api/cargo/rust-local"
}
```

Publishing:
```bash
cargo publish --registry artifactory \
  --token "Bearer <access-token>"
```

## 8. Build Configuration

### Profiles

```toml
[profile.dev]
opt-level = 0
debug = true
overflow-checks = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.bench]
inherits = "release"

# Custom profile
[profile.production]
inherits = "release"
lto = "fat"
panic = "abort"
```

### Build Scripts

```rust
// build.rs
fn main() {
    // Link system libraries
    println!("cargo:rustc-link-lib=ssl");

    // Rerun if files change
    println!("cargo:rerun-if-changed=src/native.c");

    // Compile C code
    cc::Build::new()
        .file("src/native.c")
        .compile("native");

    // Set environment variables
    println!("cargo:rustc-env=BUILD_TIME={}",
             chrono::Utc::now().to_rfc3339());
}
```

### Build Dependencies

```toml
[build-dependencies]
cc = "1.0"
chrono = "0.4"
```

## 9. Dependencies

### Dependency Types

```toml
[dependencies]
normal = "1.0"

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"

[build-dependencies]
cc = "1.0"

[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

### Features

```toml
[dependencies]
tokio = { version = "1.47", default-features = false, features = ["rt-multi-thread", "macros"] }

[features]
default = ["std"]
std = ["serde/std"]
alloc = ["serde/alloc"]
performance = ["lto", "parallel"]
```

### Git and Path Dependencies

```toml
[dependencies]
# Git repository
from-git = { git = "https://github.com/user/repo", branch = "main" }

# Specific commit
specific = { git = "https://github.com/user/repo", rev = "abc123" }

# Local path
local = { path = "../local-crate" }

# Published with override
override = { version = "1.0", path = "../override" }
```

## 10. Documentation

### Writing Documentation

```rust
//! Module-level documentation
//!
//! This module provides utilities for working with strings.

/// Calculate factorial of n
///
/// # Examples
///
/// ```
/// assert_eq!(factorial(5), 120);
/// assert_eq!(factorial(0), 1);
/// ```
///
/// # Panics
///
/// Panics if the result would overflow.
pub fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}
```

### Documentation Commands

```bash
# Generate and open docs
cargo doc --open

# Include dependencies
cargo doc --no-deps

# Document private items
cargo doc --document-private-items

# Test documentation examples
cargo test --doc
```

## 11. Examples Directory

Structure example code for users:

```
examples/
├── basic.rs           # cargo run --example basic
├── advanced.rs        # cargo run --example advanced
└── multi-file/        # Multi-file example
    ├── main.rs
    └── helper.rs
```

```toml
# Cargo.toml
[[example]]
name = "multi-file"
path = "examples/multi-file/main.rs"
```

## 12. Benchmarking with Criterion

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_benchmark"
harness = false
```

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    for i in [20, 30, 35].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(i), i, |b, &i| {
            b.iter(|| fibonacci(black_box(i)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench

# Save baseline
cargo bench -- --save-baseline main

# Compare with baseline
cargo bench -- --baseline main
```

## 13. Security

### Dependency Auditing

```bash
# Install audit tools
cargo install cargo-audit
cargo install cargo-deny

# Check for vulnerabilities
cargo audit

# Audit with fix suggestions
cargo audit fix
```

### Deny Configuration

```toml
# deny.toml
[bans]
multiple-versions = "warn"
wildcards = "deny"
skip = []

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-3.0"]

[sources]
unknown-registry = "deny"
unknown-git = "warn"
```

```bash
cargo deny check
```

## 14. Dependency Update Strategies

### Manual Updates

```bash
# Update all dependencies
cargo update

# Update specific crate
cargo update -p serde

# See outdated dependencies
cargo install cargo-outdated
cargo outdated
```

### Automated Updates with Dependabot

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    groups:
      aws:
        patterns: ["aws-*"]
      tokio:
        patterns: ["tokio*"]
```

### Renovate Configuration

```json
{
  "extends": ["config:base"],
  "cargo": {
    "enabled": true,
    "rangeStrategy": "bump"
  },
  "packageRules": [{
    "matchManagers": ["cargo"],
    "matchPackagePatterns": ["^aws-"],
    "groupName": "AWS SDK"
  }]
}
```

## 15. Reproducible Builds

Ensure reproducibility with:

1. **Committed `Cargo.lock`** for applications
2. **Pinned toolchain** via `rust-toolchain.toml`
3. **`--locked` flag** in CI builds
4. **Vendored dependencies** for offline builds

### Docker Example

```dockerfile
FROM rust:1.82 AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

# Build application
COPY . .
RUN touch src/main.rs && cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
CMD ["app"]
```

### Vendoring Dependencies

```bash
# Vendor all dependencies
cargo vendor

# Configure to use vendored dependencies
mkdir .cargo
cat > .cargo/config.toml << EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# Build offline
cargo build --offline
```

## 16. Useful Commands

```bash
# Dependency tree
cargo tree
cargo tree -d                    # Show duplicates
cargo tree -i serde              # Inverse dependencies
cargo tree -e features           # Show features

# Workspace commands
cargo build --workspace          # Build all members
cargo test --workspace           # Test all members
cargo publish --dry-run         # Verify before publishing

# Check commands
cargo check                      # Fast compilation check
cargo clippy                     # Linting
cargo fmt                        # Format code

# Cache management
cargo clean                      # Remove target directory
cargo clean -p specific-crate   # Clean specific crate

# Package management
cargo new myproject --lib       # Create library
cargo init                      # Initialize in existing directory
cargo package                   # Create distributable package
cargo publish                   # Publish to crates.io
```

## Additional Resources

- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- [Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
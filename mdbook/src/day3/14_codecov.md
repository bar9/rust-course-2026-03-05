# Chapter 14: Code Coverage with cargo llvm-cov

Code coverage measures test effectiveness and identifies untested code paths. The `cargo llvm-cov` tool provides source-based code coverage using LLVM's instrumentation capabilities.

## 1. Installation and Setup

```bash
# Install from crates.io
cargo install cargo-llvm-cov

# Install required LLVM tools
rustup component add llvm-tools-preview

# Verify installation
cargo llvm-cov --version
```

### System Requirements

- Rust 1.60.0 or newer
- LLVM tools preview component
- Supported platforms: Linux, macOS, Windows
- LLVM versions by Rust version:
  - Rust 1.60-1.77: LLVM 14-17
  - Rust 1.78-1.81: LLVM 18
  - Rust 1.82+: LLVM 19+

## 2. Basic Usage

### Generate Coverage

```bash
# Run tests and generate coverage
cargo llvm-cov

# Clean and regenerate
cargo llvm-cov clean
cargo llvm-cov

# Generate HTML report and open
cargo llvm-cov --open

# HTML report without opening
cargo llvm-cov --html
```

### Example Output

```
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
----------------------------------------------------------------------------------------------------------------------------------------------------------------
src/calculator.rs                  12                 2    83.33%           4                 0   100.00%          45                 3    93.33%           8                 2    75.00%
src/parser.rs                      25                 5    80.00%           8                 1    87.50%         120                15    87.50%          20                 4    80.00%
src/lib.rs                          8                 0   100.00%           3                 0   100.00%          30                 0   100.00%           4                 0   100.00%
----------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                              45                 7    84.44%          15                 1    93.33%         195                18    90.77%          32                 6    81.25%
```

## 3. Report Formats

### HTML Reports

```bash
# Generate HTML report
cargo llvm-cov --html
# Output: target/llvm-cov/html/index.html

# With custom output directory
cargo llvm-cov --html --output-dir coverage
```

### JSON Format

```bash
# Generate JSON report
cargo llvm-cov --json --output-path coverage.json

```

### LCOV Format

```bash
# Generate LCOV for coverage services
cargo llvm-cov --lcov --output-path lcov.info
```

### Cobertura XML

```bash
# Generate Cobertura for CI/CD tools
cargo llvm-cov --cobertura --output-path cobertura.xml
```

### Text Summary

```bash
# Display only summary
cargo llvm-cov --summary-only

# Text report with specific format
cargo llvm-cov --text
```

## 4. Practical Example: Calculator Library

### Project Structure

```rust
// src/lib.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct Calculator {
    precision: usize,
}

impl Calculator {
    pub fn new() -> Self {
        Self { precision: 2 }
    }

    pub fn with_precision(precision: usize) -> Self {
        Self { precision }
    }

    pub fn calculate(&self, op: Operation, a: f64, b: f64) -> Result<f64, String> {
        let result = match op {
            Operation::Add => a + b,
            Operation::Subtract => a - b,
            Operation::Multiply => a * b,
            Operation::Divide => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
        };

        Ok(self.round_to_precision(result))
    }

    fn round_to_precision(&self, value: f64) -> f64 {
        let multiplier = 10_f64.powi(self.precision as i32);
        (value * multiplier).round() / multiplier
    }

    pub fn chain_operations(&self, initial: f64, operations: Vec<(Operation, f64)>) -> Result<f64, String> {
        operations.iter().try_fold(initial, |acc, (op, value)| {
            self.calculate(op.clone(), acc, *value)
        })
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
```

### Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let calc = Calculator::new();

        assert_eq!(calc.calculate(Operation::Add, 5.0, 3.0), Ok(8.0));
        assert_eq!(calc.calculate(Operation::Subtract, 5.0, 3.0), Ok(2.0));
        assert_eq!(calc.calculate(Operation::Multiply, 5.0, 3.0), Ok(15.0));
        assert_eq!(calc.calculate(Operation::Divide, 15.0, 3.0), Ok(5.0));
    }

    #[test]
    fn test_division_by_zero() {
        let calc = Calculator::new();
        assert!(calc.calculate(Operation::Divide, 5.0, 0.0).is_err());
    }

    #[test]
    fn test_precision() {
        let calc = Calculator::with_precision(3);
        assert_eq!(calc.calculate(Operation::Divide, 10.0, 3.0), Ok(3.333));
    }

    #[test]
    fn test_chain_operations() {
        let calc = Calculator::new();
        let operations = vec![
            (Operation::Add, 5.0),
            (Operation::Multiply, 2.0),
            (Operation::Subtract, 3.0),
        ];

        assert_eq!(calc.chain_operations(10.0, operations), Ok(27.0));
    }
}
```

### Coverage Analysis

```bash
# Run coverage
cargo llvm-cov

# Generate detailed HTML report
cargo llvm-cov --html --open

# Check specific test coverage
cargo llvm-cov --lib
```

## 5. Filtering and Exclusions

### Include/Exclude Patterns

```bash
# Include only library code
cargo llvm-cov --lib

# Include only binary
cargo llvm-cov --bin my-binary

# Exclude tests from coverage
cargo llvm-cov --ignore-filename-regex='tests/'
```

### Coverage Attributes

```rust
// Exclude function from coverage
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // This won't be included in coverage
}

// Use cfg_attr for conditional exclusion
#[cfg_attr(not(test), no_coverage)]
fn internal_helper() {
    // Implementation
}
```

### Configuration File

```toml
# .cargo/llvm-cov.toml
[llvm-cov]
ignore-filename-regex = ["tests/", "benches/", "examples/"]
output-dir = "coverage"
html = true
```

## 6. Workspace Coverage

### Multi-Crate Workspaces

```bash
# Coverage for entire workspace
cargo llvm-cov --workspace

# Specific workspace members
cargo llvm-cov --package crate1 --package crate2

# Exclude specific packages
cargo llvm-cov --workspace --exclude integration-tests
```

### Workspace Configuration

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["core", "utils", "app"]

[workspace.metadata.llvm-cov]
ignore-filename-regex = ["mock", "test_"]
```

### Aggregated Reports

```bash
# Generate workspace-wide HTML report
cargo llvm-cov --workspace --html

# Combined LCOV for all crates
cargo llvm-cov --workspace --lcov --output-path workspace.lcov
```

## 7. CI/CD Integration

### GitHub Actions

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true
```

### GitLab CI

```yaml
coverage:
  stage: test
  image: rust:latest
  before_script:
    - rustup component add llvm-tools-preview
    - cargo install cargo-llvm-cov
  script:
    - cargo llvm-cov --workspace --lcov --output-path lcov.info
    - cargo llvm-cov --workspace --cobertura --output-path cobertura.xml
  coverage: '/TOTAL.*\s+(\d+\.\d+)%/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: cobertura.xml
```

### Coverage Badges

```markdown
<!-- README.md -->
[![Coverage](https://codecov.io/gh/username/repo/branch/main/graph/badge.svg)](https://codecov.io/gh/username/repo)

[![Coverage](https://coveralls.io/repos/github/username/repo/badge.svg?branch=main)](https://coveralls.io/github/username/repo?branch=main)
```

## 8. Integration with Coverage Services

### Codecov

```bash
# Upload to Codecov
cargo llvm-cov --lcov --output-path lcov.info
bash <(curl -s https://codecov.io/bash) -f lcov.info
```

```yaml
# codecov.yml
coverage:
  precision: 2
  round: down
  range: "70...100"

  status:
    project:
      default:
        target: 80%
        threshold: 2%
    patch:
      default:
        target: 90%
```

### Coveralls

```yaml
# GitHub Actions with Coveralls
- name: Upload to Coveralls
  uses: coverallsapp/github-action@v2
  with:
    file: lcov.info
```

## 9. Advanced Configuration

### Custom Test Binaries

```bash
# Coverage for specific test binary
cargo llvm-cov --test integration_test

# Coverage for doc tests
cargo llvm-cov --doctests

# Coverage for examples
cargo llvm-cov --example my_example

# Integration with nextest (faster test runner)
cargo llvm-cov nextest

# Nextest with specific options
cargo llvm-cov nextest --workspace --exclude integration-tests
```

### Environment Variables

```bash
# Set custom LLVM profile directory
export CARGO_LLVM_COV_TARGET_DIR=/tmp/coverage

# Merge multiple runs
export CARGO_LLVM_COV_MERGE=1
cargo llvm-cov --no-report
cargo llvm-cov --no-run --html
```

### Profile-Guided Optimization

```bash
# Generate profile data
cargo llvm-cov --release --no-report

# Use for PGO
rustc -Cprofile-use=target/llvm-cov/*/profraw
```

## 10. Comparison with Other Tools

### cargo-tarpaulin vs cargo-llvm-cov

| Feature | cargo-tarpaulin | cargo-llvm-cov |
|---------|-----------------|----------------|
| **Coverage Type** | Line-based | Source-based |
| **Platform Support** | Linux only | Cross-platform |
| **Speed** | Slower | Faster |
| **Accuracy** | Good | More precise |
| **Report Formats** | HTML, XML, LCOV | HTML, JSON, LCOV, Cobertura |
| **Integration** | ptrace-based | LLVM-based |

### When to Use Each

- **cargo-llvm-cov**: Recommended for most projects, especially cross-platform
- **cargo-tarpaulin**: Legacy projects, specific Linux features
- **grcov**: Mozilla projects, Firefox integration

## 11. Best Practices

### Coverage Goals

```toml
# .github/coverage.toml
[coverage]
minimum_total = 80
minimum_file = 60
exclude_patterns = ["tests/*", "benches/*"]
```

### Meaningful Coverage

1. **Focus on Critical Paths**: Prioritize business logic over boilerplate
2. **Test Edge Cases**: Don't just test happy paths
3. **Avoid Coverage Gaming**: 100% coverage doesn't mean bug-free
4. **Regular Reviews**: Monitor coverage trends over time

### Coverage Improvement Strategy

```bash
# Find uncovered code
cargo llvm-cov --html
# Review HTML report for red lines

# Generate JSON for analysis
cargo llvm-cov --json --output-path coverage.json

# Parse and analyze with scripts
jq '.data[0].files[] | select(.summary.lines.percent < 80) | .filename' coverage.json
```

## 12. Troubleshooting

### Common Issues

**Issue: No coverage data generated**
```bash
# Ensure tests actually run
cargo test
# Then run coverage
cargo llvm-cov clean
cargo llvm-cov
```

**Issue: Incorrect coverage numbers**
```bash
# Clean all artifacts
cargo clean
rm -rf target/llvm-cov
cargo llvm-cov
```

**Issue: Missing functions in report**
```rust
// Ensure functions are called in tests
#[inline(never)]  // Prevent inlining
pub fn my_function() {
    // Implementation
}
```

### Performance Optimization

```bash
# Use release mode for faster execution
cargo llvm-cov --release

# Parallel test execution
cargo llvm-cov -- --test-threads=4

# Skip expensive tests
cargo llvm-cov -- --skip expensive_test
```

## 13. Real-World Example: Web Service

```rust
// src/server.rs
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: u32,
    name: String,
}

pub async fn get_user(id: web::Path<u32>) -> HttpResponse {
    // Simulate database lookup
    if *id == 0 {
        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Ok().json(User {
        id: *id,
        name: format!("User{}", id),
    })
}

pub async fn create_user(user: web::Json<User>) -> HttpResponse {
    HttpResponse::Created().json(&user.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_get_user() {
        let app = test::init_service(
            App::new().route("/user/{id}", web::get().to(get_user))
        ).await;

        let req = test::TestRequest::get()
            .uri("/user/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_user_not_found() {
        let app = test::init_service(
            App::new().route("/user/{id}", web::get().to(get_user))
        ).await;

        let req = test::TestRequest::get()
            .uri("/user/0")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}
```

### Coverage Commands for Web Service

```bash
# Run with integration tests
cargo llvm-cov --all-features

# Generate comprehensive report
cargo llvm-cov --workspace --html --open

# CI-friendly output
cargo llvm-cov --workspace --lcov --output-path lcov.info --summary-only
```

## Summary

Code coverage with `cargo llvm-cov` provides:

- **Accurate metrics** using LLVM instrumentation
- **Multiple report formats** for different use cases
- **CI/CD integration** with major platforms
- **Workspace support** for complex projects
- **Cross-platform compatibility** unlike alternatives

Remember: coverage is a tool for finding untested code, not a goal in itself. Focus on meaningful tests that verify behavior rather than achieving arbitrary coverage percentages.

## Additional Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [LLVM Coverage Mapping Format](https://llvm.org/docs/CoverageMappingFormat.html)
- [Codecov Documentation](https://docs.codecov.com/)
- [GitHub Actions for Rust](https://github.com/actions-rs)

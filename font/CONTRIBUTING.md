# Contributing to dx-font

Thank you for your interest in contributing to dx-font! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Adding New Providers](#adding-new-providers)

## Code of Conduct

This project follows the Rust Code of Conduct. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/dx`
3. Create a branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites

- Rust 1.85+ (2024 edition)
- Git Bash (on Windows)
- cargo-watch (optional): `cargo install cargo-watch`

### Building

```bash
# Build the crate
cargo build --manifest-path crates/font/Cargo.toml

# Build with all features
cargo build --manifest-path crates/font/Cargo.toml --all-features

# Build release
cargo build --manifest-path crates/font/Cargo.toml --release
```

### Running Tests

```bash
# Run unit tests
cargo test --manifest-path crates/font/Cargo.toml

# Run integration tests (requires network)
cargo test --manifest-path crates/font/Cargo.toml -- --ignored

# Run specific test
cargo test --manifest-path crates/font/Cargo.toml test_name

# Run with output
cargo test --manifest-path crates/font/Cargo.toml -- --nocapture
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --manifest-path crates/font/Cargo.toml

# Run specific benchmark
cargo bench --manifest-path crates/font/Cargo.toml bench_name
```

## Making Changes

### Branch Naming

- Feature: `feature/description`
- Bug fix: `fix/description`
- Documentation: `docs/description`
- Performance: `perf/description`

### Commit Messages

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `perf`: Performance improvements
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Maintenance tasks

Examples:
```
feat(providers): add FontShare provider
fix(cache): handle expired cache entries correctly
docs(readme): update installation instructions
```

## Testing

### Test Requirements

All contributions must include tests:

1. **Unit Tests**: For individual functions/methods
2. **Integration Tests**: For provider implementations
3. **Property Tests**: For critical algorithms (using proptest)

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

Mark network-dependent tests with `#[ignore]`:

```rust
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_real_api() {
    // Test that makes real API calls
}
```

### Property-Based Tests

Use proptest for testing invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(input in any::<String>()) {
        // Test that property holds for all inputs
        prop_assert!(invariant(input));
    }
}
```

## Code Style

### Formatting

```bash
# Format code
cargo fmt --manifest-path crates/font/Cargo.toml

# Check formatting
cargo fmt --manifest-path crates/font/Cargo.toml -- --check
```

### Linting

```bash
# Run clippy
cargo clippy --manifest-path crates/font/Cargo.toml --all-targets

# Fix clippy warnings
cargo clippy --manifest-path crates/font/Cargo.toml --all-targets --fix
```

### Style Guidelines

1. **Line Length**: Max 100 characters
2. **Indentation**: 4 spaces
3. **Imports**: Group and sort automatically
4. **Documentation**: All public items must have rustdoc comments
5. **Error Handling**: Use `Result` types, avoid `unwrap()` in production code
6. **Async**: Use `async/await`, not manual futures

### Documentation

```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error
///
/// # Examples
///
/// ```
/// use dx_font::prelude::*;
///
/// let result = function(param)?;
/// ```
pub fn function(param: Type) -> Result<ReturnType, Error> {
    // Implementation
}
```

## Submitting Changes

### Before Submitting

1. **Format**: `cargo fmt`
2. **Lint**: `cargo clippy` (no warnings)
3. **Test**: `cargo test` (all pass)
4. **Build**: `cargo build` (no errors)
5. **Documentation**: `cargo doc` (no warnings)

### Pull Request Process

1. Update CHANGELOG.md with your changes
2. Update documentation if needed
3. Ensure all tests pass
4. Create a pull request with:
   - Clear title and description
   - Reference to related issues
   - Screenshots (if UI changes)
   - Breaking changes noted

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tests pass locally
```

## Adding New Providers

To add a new font provider:

1. Create `src/providers/provider_name.rs`
2. Implement the `Provider` trait
3. Add to `src/providers/mod.rs`
4. Register in `ProviderRegistry::with_defaults()`
5. Add integration tests
6. Update documentation

### Provider Template

```rust
use crate::error::{FontError, FontResult};
use crate::models::{Font, SearchQuery};
use async_trait::async_trait;

pub struct NewProvider {
    client: reqwest::Client,
}

impl NewProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Provider for NewProvider {
    fn name(&self) -> &str {
        "New Provider"
    }

    async fn search(&self, query: &SearchQuery) -> FontResult<Vec<Font>> {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_search() {
        // Test implementation
    }
}
```

## Performance Considerations

1. **Async**: Use async for I/O operations
2. **Parallel**: Use rayon for CPU-bound parallel work
3. **Caching**: Implement caching for expensive operations
4. **Allocations**: Minimize allocations in hot paths
5. **Benchmarks**: Add benchmarks for performance-critical code

## Questions?

- Open an issue for questions
- Check existing issues and PRs
- Read the documentation

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT/Apache-2.0).

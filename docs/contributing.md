# Contributing to fastalloc

Thank you for your interest in contributing to fastalloc! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to eshanized@proton.me.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Description**: Clear description of the bug
- **Steps to Reproduce**: Minimal example that reproduces the issue
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: Rust version, OS, relevant dependencies
- **Additional Context**: Any other relevant information

### Suggesting Enhancements

Enhancement suggestions are welcome! Please include:

- **Use Case**: Why this enhancement would be useful
- **Proposed Solution**: How you envision it working
- **Alternatives**: Any alternative approaches you've considered
- **Additional Context**: Examples, mockups, etc.

### Pull Requests

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Test** your changes thoroughly
5. **Commit** with clear messages (`git commit -m 'Add amazing feature'`)
6. **Push** to your branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- Cargo
- Git

### Clone and Build

```bash
git clone https://github.com/TIVerse/fastalloc.git
cd fastalloc
cargo build
cargo test
```

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific test suite
cargo test --test basic_operations

# With miri (undefined behavior detection)
cargo +nightly miri test

# Integration tests
cargo test --tests

# Doc tests
cargo test --doc
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench allocation_speed

# Save baseline for comparison
cargo bench -- --save-baseline main
```

### Code Style

We use `rustfmt` and `clippy`:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint with clippy
cargo clippy --all-features -- -D warnings

# Pedantic lints
cargo clippy --all-features -- -W clippy::pedantic
```

## Coding Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use idiomatic Rust naming conventions
- Document all public APIs with examples
- Keep functions focused and small
- Prefer composition over inheritance

### Documentation

- All public items must have doc comments
- Include examples in doc comments
- Use `///` for item documentation
- Use `//!` for module documentation
- Add examples to demonstrate usage

Example:

```rust
/// Allocates an object from the pool.
///
/// # Examples
///
/// ```
/// use fastalloc::FixedPool;
///
/// let pool = FixedPool::new(10).unwrap();
/// let handle = pool.allocate(42).unwrap();
/// assert_eq!(*handle, 42);
/// ```
///
/// # Errors
///
/// Returns `Error::PoolExhausted` if the pool is at capacity.
pub fn allocate(&self, value: T) -> Result<OwnedHandle<T>> {
    // ...
}
```

### Testing

- Write tests for all new features
- Include edge cases and error conditions
- Use descriptive test names
- Keep tests focused (one concept per test)

```rust
#[test]
fn test_pool_exhaustion() {
    let pool = FixedPool::new(2).unwrap();
    
    let _h1 = pool.allocate(1).unwrap();
    let _h2 = pool.allocate(2).unwrap();
    
    let result = pool.allocate(3);
    assert!(result.is_err());
}
```

### Error Handling

- Use `Result` for fallible operations
- Provide descriptive error messages
- Use the `Error` enum defined in the crate
- Document error conditions in doc comments

### Performance

- Avoid allocations in hot paths
- Use `#[inline]` for small, frequently-called functions
- Benchmark performance-critical changes
- Document performance characteristics

### Safety

- Minimize use of `unsafe`
- Document all safety invariants
- Use `debug_assert!` for debug-only checks
- Prefer safe abstractions

## Project Structure

```
fastalloc/
├── src/
│   ├── lib.rs              # Crate root and public API
│   ├── error.rs            # Error types
│   ├── traits.rs           # Public traits
│   ├── utils.rs            # Utility functions
│   ├── pool/               # Pool implementations
│   ├── handle/             # Handle types
│   ├── config/             # Configuration system
│   ├── allocator/          # Internal allocators
│   └── stats/              # Statistics (optional)
├── benches/                # Benchmarks
├── examples/               # Example programs
├── tests/                  # Integration tests
└── docs/                   # Documentation
```

## Commit Messages

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Example:

```
feat(pool): add lock-free pool implementation

Implement experimental lock-free pool using crossbeam
for improved performance under high contention.

Closes #42
```

## Review Process

1. **Automated Checks**: CI must pass (tests, clippy, fmt)
2. **Code Review**: At least one maintainer must approve
3. **Documentation**: Public APIs must be documented
4. **Tests**: New features must include tests
5. **Benchmarks**: Performance-critical changes should include benchmarks

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag (`v0.x.y`)
4. Push tag to trigger CI publish workflow
5. Verify publication on crates.io

## Questions?

- Open an issue for questions
- Email: eshanized@proton.me
- Check existing documentation

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.

# Contributing to FastAlloc

Thank you for considering contributing to FastAlloc! We appreciate your time and effort in making this project better.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
   ```bash
   git clone https://github.com/your-username/fastalloc.git
   cd fastalloc
   ```
3. Install Rust using [rustup](https://rustup.rs/)
4. Install development tools:
   ```bash
   rustup component add rustfmt clippy
   ```

## Development Workflow

1. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes following the coding standards
3. Run tests and verify everything works
4. Commit your changes with a descriptive message
5. Push to your fork and open a pull request

## Coding Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent code formatting
- Run `cargo clippy` to catch common mistakes and improve code quality
- Document all public APIs with rustdoc comments
- Write unit tests for new functionality
- Keep commits focused and atomic

## Testing

Run the test suite:

```bash
# Run all tests
cargo test --all-features

# Run tests for no_std
cargo test --no-default-features

# Run benchmarks
cargo bench

# Run MIRI to detect undefined behavior
cargo +nightly miri test

# Run fuzzing (if available)
cargo +nightly fuzz run <target>
```

## Pull Request Process

1. Ensure all tests pass
2. Update documentation as needed
3. Add tests for new features or bug fixes
4. Ensure your code is properly formatted
5. Open a pull request with a clear description of changes
6. Reference any related issues
7. Wait for code review and address any feedback

## Reporting Bugs

Please open an issue on GitHub with the following information:

- Description of the bug
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment details (Rust version, OS, etc.)
- Any relevant logs or error messages

## Feature Requests

We welcome feature requests! Please open an issue describing:

- The feature you'd like to see
- Why this feature is valuable
- Any potential implementation ideas

## Documentation

Good documentation is crucial. When contributing:

- Document all public APIs
- Include code examples
- Update the README for user-facing changes
- Add or update examples in the `examples/` directory

## Community

- Join our [Discord server](https://discord.gg/your-invite-link)
- Follow us on [Twitter](https://twitter.com/your-handle)
- Read our [blog](https://your-blog-url.com)

## License

By contributing to FastAlloc, you agree that your contributions will be licensed under its [MIT/Apache-2.0 dual license](LICENSE-APACHE).

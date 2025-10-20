# Security Policy

## Supported Versions

The following versions of fastalloc are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in fastalloc, please report it responsibly.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please send an email to: **eshanized@proton.me**

Include the following information:

- Type of vulnerability
- Full description of the vulnerability
- Steps to reproduce or proof of concept
- Potential impact
- Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of submission
- **Initial Assessment**: Within 7 days
- **Status Updates**: Every 7 days until resolution
- **Fix Timeline**: We aim to release fixes for critical vulnerabilities within 30 days

### Disclosure Policy

- We request that you give us reasonable time to fix the vulnerability before public disclosure
- We will acknowledge your contribution in the security advisory (unless you prefer to remain anonymous)
- Once fixed, we will publish a security advisory

## Security Best Practices

When using fastalloc:

1. **Keep Updated**: Always use the latest version
2. **Review Dependencies**: Use `cargo audit` regularly
3. **Bounds Checking**: Enable debug assertions during development
4. **Statistics**: Use the `stats` feature to monitor for unusual allocation patterns
5. **Thread Safety**: Use appropriate pool types for your concurrency needs

## Known Security Considerations

### Memory Safety

- fastalloc minimizes `unsafe` code
- All `unsafe` blocks are documented with safety invariants
- Debug builds include additional runtime checks
- Miri testing is part of our CI pipeline

### Thread Safety

- Thread-safe pools use proper synchronization
- Thread-local pools explicitly prevent cross-thread usage
- No data races possible with safe API usage

### Resource Exhaustion

- Pool exhaustion returns errors rather than panicking
- Maximum capacity limits can prevent unbounded growth
- Statistics help detect resource exhaustion patterns

## Security Hardening

Optional features for enhanced security:

```toml
[dependencies]
fastalloc = { version = "0.1", features = ["stats"] }
```

Monitor allocation patterns to detect:
- Unusual allocation rates
- Memory exhaustion attempts
- Resource leaks

## Hall of Fame

We appreciate responsible disclosure. Contributors will be listed here:

- (No vulnerabilities reported yet)

## Contact

For security-related questions: eshanized@proton.me

For general questions: Open an issue on GitHub at https://github.com/TIVerse/fastalloc/issues

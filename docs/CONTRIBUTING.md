# Contributing to bytes-radar

Thank you for your interest in contributing to bytes-radar! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git

### Development Setup

1. **Fork the repository**

2. **Clone your fork**:

   ```bash
   git clone https://github.com/YOUR_USERNAME/bytes-radar.git
   cd bytes-radar
   ```

3. **Set up the development environment**:

   ```bash
   cargo build
   cargo test --all-features
   ```

4. **Create a new branch**:
   ```bash
   git checkout -b feature/my-new-feature
   git checkout -b fix/issue-123
   ```

## Development Workflow

### Code Style

We use standard Rust formatting and linting tools:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

### Commit Messages

Follow conventional commit format:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:

```
feat: add support for SourceForge repositories
fix: handle network timeouts gracefully
docs: update installation instructions
```

### Testing

- Write tests for new functionality
- Ensure all existing tests pass
- Include integration tests for new features

```bash
cargo test --all-features
cargo test test_name
cargo test -- --nocapture
```

## Types of Contributions

### Bug Reports

When filing a bug report, please include:

- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- System information (OS, Rust version, bytes-radar version)
- Command used and full output

### Feature Requests

For feature requests, please include:

- Clear description of the feature
- Use case and motivation
- Possible implementation approaches
- Any breaking changes

### Code Contributions

#### Adding New Platform Support

To add support for a new git platform:

1. Add URL parsing logic in `src/core/net.rs`
2. Add tests for the new platform
3. Update documentation
4. Add examples to the README

#### Adding New Output Formats

To add a new output format:

1. Add the format enum to `src/cli/args.rs`
2. Implement the formatter in `src/cli/output.rs`
3. Add tests for the new format
4. Update help text and documentation

#### Performance Improvements

For performance improvements:

1. Include benchmarks showing the improvement
2. Explain the optimization approach
3. Ensure no functionality is broken
4. Test on different repository sizes

## Testing Guidelines

### Unit Tests

- Test individual functions and modules
- Mock external dependencies
- Focus on edge cases and error conditions

### Integration Tests

- Test complete workflows
- Test with real repositories (small ones)
- Test different output formats
- Test error handling

### Performance Tests

```bash
cargo bench
cargo run --release -- user/small-repo
```

## Documentation

### Code Documentation

- Document all public APIs
- Include examples in doc comments
- Explain complex algorithms
- Keep documentation up to date

### User Documentation

- Update README for new features
- Add examples for new functionality
- Update CLI help text
- Keep installation instructions current

## Release Process

Releases are handled by maintainers:

1. Version bump in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. GitHub Actions builds and publishes

## Project Structure

```
bytes-radar/
├── .github/              # GitHub workflows and templates
├── src/
│   ├── cli/             # Command-line interface
│   ├── core/            # Core analysis logic
│   └── lib.rs           # Library entry point
├── tests/               # Integration tests
└── examples/            # Usage examples
```

## Areas for Contribution

We welcome contributions in these areas:

### High Priority

- New git platform support
- Performance optimizations
- Better error messages
- More output formats

### Medium Priority

- CLI improvements
- Documentation improvements
- More language detection
- Cross-platform testing

### Low Priority

- Code cleanup
- Minor feature additions
- Example improvements

## Getting Help

If you need help:

- Check existing [issues](https://github.com/zmh-program/bytes-radar/issues)
- Start a [discussion](https://github.com/zmh-program/bytes-radar/discussions)

## Pull Request Process

1. **Before submitting**:

   - Fork the repository
   - Create a feature branch
   - Write tests for your changes
   - Ensure all tests pass
   - Update documentation

2. **Submitting**:

   - Fill out the PR template completely
   - Reference related issues
   - Describe your changes clearly
   - Include screenshots if relevant

3. **After submitting**:

   - Respond to review feedback
   - Make requested changes
   - Keep your branch up to date

4. **Merge criteria**:
   - All tests pass
   - Code review approved
   - Documentation updated
   - No breaking changes (unless discussed)

## Recognition

Contributors are recognized in:

- Repository contributors page
- Release notes for significant contributions
- README acknowledgments section

## License

By contributing to bytes-radar, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing to bytes-radar!

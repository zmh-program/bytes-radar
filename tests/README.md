# Tests

This directory contains comprehensive tests for the bytes-radar library.

## Test Structure

### Integration Tests (`integration_tests.rs`)

Tests the main API functionality and integration between components:

- **File Metrics Creation**: Tests creating and validating file metrics
- **Project Analysis**: Tests building project analysis from multiple files
- **Language Statistics**: Tests aggregation and calculation of language-specific statistics
- **Remote Analyzer**: Tests basic analyzer configuration and usage
- **Data Validation**: Tests input validation and error handling

### Analysis Tests (`analysis_tests.rs`)

Focuses on the core analysis functionality:

- **Language Analysis**: Tests language-specific analysis and aggregation
- **Aggregate Metrics**: Tests metric accumulation and calculation
- **Merging Operations**: Tests combining analysis from multiple sources
- **Statistics Calculation**: Tests ratio calculations and derived metrics
- **Edge Cases**: Tests boundary conditions and error scenarios

### CLI Tests (`cli_tests.rs`)

Tests the command-line interface and remote operations:

- **Analyzer Configuration**: Tests timeout, authentication, and security settings
- **URL Handling**: Tests parsing and validation of repository URLs
- **Error Handling**: Tests behavior with invalid inputs
- **Network Operations**: Tests timeout and connection handling
- **Configuration Persistence**: Tests setting and updating analyzer parameters

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test Modules

```bash
# Integration tests only
cargo test integration_tests

# Analysis tests only
cargo test analysis_tests

# CLI tests only
cargo test cli_tests
```

### Run Individual Tests

```bash
# Run a specific test
cargo test test_file_metrics_creation

# Run tests matching a pattern
cargo test file_metrics

# Run with output
cargo test -- --nocapture
```

### Test Coverage

```bash
# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin
```

## Test Categories

### Unit Tests

- Test individual functions and methods in isolation
- Focus on specific functionality and edge cases
- Fast execution, no external dependencies

### Integration Tests

- Test interaction between multiple components
- Test complete workflows and use cases
- May include mock network operations

### Network Tests

- Test actual network operations (marked with `#[ignore]` by default)
- Require internet connection
- May be flaky due to network conditions

## Running Network Tests

Some tests require internet connectivity and are ignored by default:

```bash
# Run all tests including network tests
cargo test -- --ignored

# Run only network tests
cargo test network --ignored
```

## Test Data

Tests use predefined test data and mock repositories where possible to ensure:

- Consistent results across environments
- Fast test execution
- No dependency on external services

## Contributing

When adding new tests:

1. **Unit tests** should go in the same file as the code they test (using `#[cfg(test)]`)
2. **Integration tests** should go in the appropriate test file in this directory
3. **Network tests** should be marked with `#[ignore]` and documented
4. Include both positive and negative test cases
5. Test edge cases and error conditions
6. Use descriptive test names that explain what is being tested

## Test Performance

- Tests should complete quickly (< 1 second for unit tests)
- Network tests may take longer but should have reasonable timeouts
- Use `cargo test --release` for performance-sensitive tests

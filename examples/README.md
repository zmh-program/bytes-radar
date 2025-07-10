# Examples

This directory contains examples demonstrating how to use the bytes-radar library.

## Available Examples

### 1. Basic Analysis (`basic_analysis.rs`)

Demonstrates basic usage of the RemoteAnalyzer to analyze a public repository.

```bash
cargo run --example basic_analysis
```

This example analyzes the bytes-radar repository itself and displays:

- Project summary statistics
- Language breakdown
- Total lines, files, and size information

### 2. GitHub Token Analysis (`github_token_analysis.rs`)

Shows how to use GitHub authentication for private repositories.

```bash
export GITHUB_TOKEN=your_github_token_here
cargo run --example github_token_analysis
```

Make sure to:

1. Set the `GITHUB_TOKEN` environment variable
2. Update the repository URL in the example to point to your private repo
3. Ensure your token has access to the repository

### 3. Compare Repositories (`compare_repositories.rs`)

Compares multiple repositories and ranks them by various metrics.

```bash
cargo run --example compare_repositories
```

This example:

- Analyzes multiple popular repositories
- Compares them side by side
- Ranks by complexity and documentation ratios
- Demonstrates batch processing

### 4. Custom Analysis (`custom_analysis.rs`)

Demonstrates manual construction of project analysis without remote fetching.

```bash
cargo run --example custom_analysis
```

This example shows how to:

- Create FileMetrics manually
- Build a ProjectAnalysis step by step
- Use different file categories
- Display detailed file-by-file information

## Running All Examples

To run all examples in sequence:

```bash
cargo run --example basic_analysis
cargo run --example custom_analysis
cargo run --example compare_repositories

# For github_token_analysis, set your token first:
export GITHUB_TOKEN=your_token
cargo run --example github_token_analysis
```

## Notes

- Examples that analyze remote repositories require an internet connection
- GitHub token examples require valid authentication
- Some examples may take time to complete due to network requests
- Adjust timeout values if you experience connection issues

## Customizing Examples

Feel free to modify these examples:

- Change repository URLs to analyze your own projects
- Adjust timeout values for slower connections
- Add error handling for production use
- Experiment with different output formats in the CLI

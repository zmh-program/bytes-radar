# Bytes Radar

[![CI](https://github.com/zmh-program/bytes-radar/workflows/CI/badge.svg)](https://github.com/zmh-program/bytes-radar/actions)
[![Crates.io](https://img.shields.io/crates/v/bytes-radar.svg)](https://crates.io/crates/bytes-radar)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

Hyper-fast **CLOC** _(\*count lines of code)_ tool for remote repositories.

![Banner](docs/banner.jpg)

## Features

- **Asynchronous Repository Processing**: Non-blocking HTTP client with async streaming request processing for efficient remote repository fetching and decompression, optimized for **low memory usage** and **serverless environments** (always `<32MiB` runtime memory usage for large files)
- **Multi-Platform URL Resolution**: Features intelligent URL parsing engine that normalizes different Git hosting platform APIs (GitHub, GitLab, Bitbucket, Codeberg) into unified archive endpoints with branch/commit resolution
- **Streaming Archive Analysis**: Processes tar.gz archives directly in memory using streaming decompression without temporary file extraction, reducing I/O overhead and memory footprint
- **Language Detection Engine**: Implements rule-based file extension and content analysis system supporting 150+ programming languages with configurable pattern matching and statistical computation (use tokei [languages map](https://github.com/XAMPPRocky/tokei/blob/master/languages.json))
- **Real-time Progress Monitoring**: Features bandwidth-aware progress tracking with download speed calculation, ETA estimation, and adaptive UI rendering for terminal environments
- **Structured Data Serialization**: Provides multiple output format engines (Table, JSON, CSV, XML) with schema validation and type-safe serialization for integration with external tools
- **Authentication Layer**: Implements OAuth token management with secure credential handling for accessing private repositories across different hosting platforms
- **Cross-Platform Binary Distribution**: Supports native compilation targets for Linux, macOS, and Windows with platform-specific optimizations and dependency management

## Installation

### From Cargo (Recommended)

```bash
cargo install bytes-radar
```

### From Releases

Download the latest binary from [GitHub Releases](https://github.com/zmh-program/bytes-radar/releases)

### From Source

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar
cargo build --release
```

## Usage

```bash
bradar [OPTIONS] <URL>
```

### Examples

#### Basic Repository Analysis

Analyze GitHub repositories using shorthand notation:

```bash
bradar torvalds/linux
bradar microsoft/typescript
bradar rust-lang/cargo
```

#### Branch and Commit Targeting

Specify particular branches or commit hashes for analysis:

```bash
bradar microsoft/vscode@main
bradar kubernetes/kubernetes@release-1.28
bradar rust-lang/rust@abc1234567
```

#### Multi-Platform Repository Support

Analyze repositories from different Git hosting platforms:

```bash
bradar https://gitlab.com/gitlab-org/gitlab
bradar https://bitbucket.org/atlassian/stash
bradar https://codeberg.org/forgejo/forgejo
```

#### Output Format Configuration

Generate analysis results in structured data formats:

```bash
bradar -f json torvalds/linux
bradar -f csv microsoft/typescript
bradar -f xml rust-lang/cargo
```

#### Private Repository Access

Authenticate with platform tokens for private repository analysis:

```bash
bradar --token ghp_xxxxxxxxxxxxxxxxxxxx private-org/confidential-repo
bradar --token glpat-xxxxxxxxxxxxxxxxxxxx https://gitlab.com/private-group/project
```

#### Performance and Output Control

Configure analysis behavior and output verbosity:

```bash
bradar --quiet --no-progress user/repo
bradar --timeout 600 --detailed large-org/massive-repo
```

## Usage Environments

### CLI

See the CLI Options section below for command-line usage.

## Output Formats

### Table (Default)

```shell
$ bradar torvalds/linux
Analyzing: https://github.com/torvalds/linux
Analysis completed in 123.76s

================================================================================
 Project                                                  linux@main
 Total Files                                              89,639
 Total Lines                                              40,876,027
 Code Lines                                               32,848,710
 Comment Lines                                            2,877,885
 Blank Lines                                              5,149,432
 Languages                                                51
 Primary Language                                         C
 Code Ratio                                               80.4%
 Documentation                                            8.8%
================================================================================
 Language                Files        Lines     Code   Comments   Blanks   Share%
================================================================================
 C                      35,586   25,268,107 18,782,347  2,836,806 3,648,954    61.8%
 C Header               25,845   10,247,647 9,481,722          0  765,925    25.1%
 Device Tree             5,789    1,831,396 1,589,630          0  241,766     4.5%
 ReStructuredText        3,785      782,387  593,628          0  188,759     1.9%
 JSON                      961      572,657  572,655          0        2     1.4%
 Text                    5,100      566,733  499,590          0   67,143     1.4%
 YAML                    4,862      548,408  458,948          0   89,460     1.3%
 GNU Style Assembly      1,343      373,956  326,745          0   47,211     0.9%
 Shell                     960      189,965  155,974          0   33,991     0.5%
 Plain Text              1,298      128,205  105,235          0   22,970     0.3%
 Python                    293       89,285   69,449      5,770   14,066     0.2%
 Makefile                3,115       82,692   57,091     13,109   12,492     0.2%
 SVG                        82       53,409   53,316          0       93     0.1%
 Perl                       58       43,986   33,264      4,406    6,316     0.1%
 Rust                      158       39,561   19,032     16,697    3,832     0.1%
 XML                        24       22,193   20,971          0    1,222     0.1%
 PO File                     7        6,711    5,605          0    1,106     0.0%
 Happy                      10        6,078    5,352          0      726     0.0%
 Assembly                   11        5,361    4,427          0      934     0.0%
 Lex                        10        2,996    2,277        347      372     0.0%
 AWK                        12        2,611    1,777        487      347     0.0%
 C++                         7        2,267    1,932          0      335     0.0%
 ...
================================================================================
 Total                  89,639   40,876,027 32,848,710  2,877,885 5,149,432   100.0%
```

### JSON Output

```json
{
  "project_name": "linux@master",
  "summary": {
    "total_files": 75823,
    "total_lines": 28691744,
    "code_lines": 22453891,
    "comment_lines": 3891234,
    "blank_lines": 2346619
  },
  "language_statistics": [...]
}
```

## Supported Platforms

| Platform      | URL Format              | Example                           |
| ------------- | ----------------------- | --------------------------------- |
| **GitHub**    | `user/repo` or full URL | `torvalds/linux`                  |
| **GitLab**    | Full URL                | `https://gitlab.com/user/repo`    |
| **Bitbucket** | Full URL                | `https://bitbucket.org/user/repo` |
| **Codeberg**  | Full URL                | `https://codeberg.org/user/repo`  |
| **Direct**    | tar.gz URL              | `https://example.com/file.tar.gz` |

## CLI Options

```bash
bradar [OPTIONS] <URL>

ARGUMENTS:
  <URL>  URL to analyze: user/repo, user/repo@branch, or full URL

OPTIONS:
  -f, --format <FORMAT>        Output format [table|json|csv|xml]
      --detailed               Show detailed file-by-file statistics
  -d, --debug                  Enable debug output
      --token <TOKEN>          GitHub token for private repositories
      --timeout <SECONDS>      Request timeout in seconds [default: 300]
      --allow-insecure         Allow insecure HTTP connections
      --no-progress           Disable progress bar
      --quiet                 Quiet mode - minimal output
  -h, --help                  Print help
  -V, --version               Print version
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar

# Install dependencies
cargo build

# Run tests
cargo test --all-features

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features
```

## Deployment

### Cloudflare Workers

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

> [!TIP]
> The Free Tier of Cloudflare Workers has a **20s request timeout limit** (wall time). Analysis of large repositories may fail due to this limitation. Consider upgrading to Cloudflare Workers Pro or using alternative deployment methods for processing large repositories.

For detailed deployment instructions and API documentation, see [DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Usage Environments

### CLI

See the CLI Options section below for command-line usage.

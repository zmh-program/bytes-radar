use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "bradar")]
#[command(
    about = "A tool for analyzing code statistics from remote repositories"
)]
#[command(version)]
#[command(long_about = "
A professional code analysis tool for remote repositories.

SUPPORTED PLATFORMS:
  - GitHub (github.com)
  - GitLab (gitlab.com, self-hosted)
  - Bitbucket (bitbucket.org)
  - Codeberg (codeberg.org)
  - SourceForge (sourceforge.net)
  - Direct tar.gz/tgz URLs

USAGE EXAMPLES:
  bradar user/repo                    # GitHub repo (default branch)
  bradar user/repo@master             # GitHub repo with specific branch
  bradar user/repo@abc123             # GitHub repo with specific commit
  bradar https://github.com/user/repo # Full GitHub URL
  bradar https://gitlab.com/user/repo # GitLab URL
  bradar https://bitbucket.org/user/repo # Bitbucket URL
  bradar https://example.com/file.tar.gz # Direct tar.gz URL
  bradar -f json user/repo            # JSON output format
  bradar --token ghp_xxx user/repo    # With GitHub token for private repos
")]
pub struct Cli {
    #[arg(help = "URL to analyze: user/repo, user/repo@branch, or full URL")]
    pub url: Option<String>,

    #[arg(short, long, help = "Output format", value_enum)]
    pub format: Option<OutputFormat>,

    #[arg(long, help = "Show detailed file-by-file statistics")]
    pub detailed: bool,

    #[arg(short = 'd', long = "debug", help = "Enable debug output")]
    pub debug: bool,

    #[arg(long, help = "GitHub token for private repositories")]
    pub token: Option<String>,

    #[arg(long, help = "Request timeout in seconds", default_value = "300")]
    pub timeout: u64,

    #[arg(long, help = "Allow insecure HTTP connections")]
    pub allow_insecure: bool,

    #[arg(long, help = "Disable progress bar")]
    pub no_progress: bool,

    #[arg(long, help = "Quiet mode - minimal output")]
    pub quiet: bool,

    #[arg(long, help = "Enable aggressive filtering for maximum performance")]
    pub aggressive_filter: bool,

    #[arg(
        long,
        help = "Maximum file size to process in KB",
        default_value = "1024"
    )]
    pub max_file_size: u64,

    #[arg(long, help = "Include test directories")]
    pub include_tests: bool,

    #[arg(long, help = "Include documentation directories")]
    pub include_docs: bool,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "table")]
    Table,
    #[value(name = "json")]
    Json,
    #[value(name = "csv")]
    Csv,
    #[value(name = "xml")]
    Xml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

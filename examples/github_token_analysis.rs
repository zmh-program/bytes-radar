use bytes_radar::{RemoteAnalyzer, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let github_token = env::var("GITHUB_TOKEN")
        .expect("Please set GITHUB_TOKEN environment variable");

    let mut analyzer = RemoteAnalyzer::new();
    analyzer.set_github_token(&github_token);
    analyzer.set_timeout(120);

    let url = "https://github.com/your-username/your-repo";

    println!(
        "Analyzing private repository: {url}\nUsing GitHub token for authentication..."
    );

    match analyzer.analyze_url(url).await {
        Ok(project_analysis) => {
            let summary = project_analysis.get_summary();

            println!("\nPrivate Repository Analysis:");
            println!("Project: {}", summary.project_name);
            println!("Files: {}", summary.total_files);
            println!("Lines of Code: {}", summary.total_code_lines);

            println!("\nTop 5 Languages by Line Count:");
            let mut language_stats = project_analysis.get_language_statistics();
            language_stats.sort_by(|a, b| b.total_lines.cmp(&a.total_lines));

            for (i, stats) in language_stats.iter().take(5).enumerate() {
                println!(
                    "  {}. {}: {} lines ({} files)",
                    i + 1,
                    stats.language_name,
                    stats.total_lines,
                    stats.file_count
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to analyze repository: {e}");
            eprintln!("Make sure:");
            eprintln!("1. GITHUB_TOKEN is set and valid");
            eprintln!("2. Token has access to the repository");
            eprintln!("3. Repository URL is correct");
        }
    }

    Ok(())
}

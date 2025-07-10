use bytes_radar::{RemoteAnalyzer, Result};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let repositories = vec![
        "https://github.com/rust-lang/rust",
        "https://github.com/microsoft/vscode",
        "https://github.com/facebook/react",
        "https://github.com/golang/go",
    ];

    let mut analyzer = RemoteAnalyzer::new();
    analyzer.set_timeout(180);

    println!("Comparing {} repositories...\n", repositories.len());

    let mut results = HashMap::new();

    for repo in &repositories {
        println!("Analyzing: {repo}");

        match analyzer.analyze_url(repo).await {
            Ok(analysis) => {
                let summary = analysis.get_summary();
                results.insert(repo.to_string(), summary);
                println!("✓ Completed: {repo}");
            }
            Err(e) => {
                eprintln!("✗ Failed {repo}: {e}");
            }
        }
        println!();
    }

    if results.is_empty() {
        println!("No repositories were successfully analyzed.");
        return Ok(());
    }

    println!("=== Repository Comparison ===\n");

    println!(
        "{:<20} {:>10} {:>12} {:>12} {:>15}",
        "Repository", "Files", "Total Lines", "Code Lines", "Primary Lang"
    );
    println!("{}", "-".repeat(75));

    for (repo, summary) in &results {
        let repo_name = repo.split('/').last().unwrap_or(repo);
        let primary_lang =
            summary.primary_language.as_deref().unwrap_or("Unknown");

        println!(
            "{:<20} {:>10} {:>12} {:>12} {:>15}",
            repo_name,
            summary.total_files,
            summary.total_lines,
            summary.total_code_lines,
            primary_lang
        );
    }

    println!("\n=== Complexity Analysis ===\n");

    let mut complexity_ranking: Vec<_> = results.iter().collect();
    complexity_ranking.sort_by(|a, b| {
        b.1.overall_complexity_ratio
            .partial_cmp(&a.1.overall_complexity_ratio)
            .unwrap()
    });

    for (i, (repo, summary)) in complexity_ranking.iter().enumerate() {
        let repo_name = repo.split('/').last().unwrap_or(repo);
        println!(
            "{}. {}: {:.1}% code ratio",
            i + 1,
            repo_name,
            summary.overall_complexity_ratio * 100.0
        );
    }

    println!("\n=== Documentation Analysis ===\n");

    let mut doc_ranking: Vec<_> = results.iter().collect();
    doc_ranking.sort_by(|a, b| {
        b.1.overall_documentation_ratio
            .partial_cmp(&a.1.overall_documentation_ratio)
            .unwrap()
    });

    for (i, (repo, summary)) in doc_ranking.iter().enumerate() {
        let repo_name = repo.split('/').last().unwrap_or(repo);
        println!(
            "{}. {}: {:.1}% documentation ratio",
            i + 1,
            repo_name,
            summary.overall_documentation_ratio * 100.0
        );
    }

    Ok(())
}

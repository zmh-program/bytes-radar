use bytes_radar::{RemoteAnalyzer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut analyzer = RemoteAnalyzer::new();

    analyzer.set_timeout(60);

    let url = "https://github.com/zmh-program/bytes-radar";

    println!("Analyzing repository: {url}");

    let project_analysis = analyzer.analyze_url(url).await?;

    let summary = project_analysis.get_summary();

    println!("\nProject Analysis Summary:");
    println!("Project Name: {}", summary.project_name);
    println!("Total Files: {}", summary.total_files);
    println!("Total Lines: {}", summary.total_lines);
    println!("Code Lines: {}", summary.total_code_lines);
    println!("Comment Lines: {}", summary.total_comment_lines);
    println!("Blank Lines: {}", summary.total_blank_lines);
    println!("Total Size: {} bytes", summary.total_size_bytes);
    println!("Language Count: {}", summary.language_count);
    if let Some(primary_lang) = &summary.primary_language {
        println!("Primary Language: {primary_lang}");
    }
    println!(
        "Complexity Ratio: {:.2}%",
        summary.overall_complexity_ratio * 100.0
    );
    println!(
        "Documentation Ratio: {:.2}%",
        summary.overall_documentation_ratio * 100.0
    );

    println!("\nLanguage Breakdown:");
    let language_stats = project_analysis.get_language_statistics();
    for stats in language_stats {
        println!(
            "  {}: {} files, {} lines",
            stats.language_name, stats.file_count, stats.total_lines
        );
    }

    Ok(())
}

use bytes_radar::{FileCategory, FileMetrics, ProjectAnalysis, Result};

fn main() -> Result<()> {
    let mut project = ProjectAnalysis::new("my-rust-project");

    let main_rs =
        FileMetrics::new("src/main.rs", "Rust".to_string(), 50, 35, 10, 5)?
            .with_category(FileCategory::Source)
            .with_size_bytes(1200);

    let lib_rs =
        FileMetrics::new("src/lib.rs", "Rust".to_string(), 80, 60, 15, 5)?
            .with_category(FileCategory::Source)
            .with_size_bytes(2000);

    let test_rs = FileMetrics::new(
        "tests/integration_test.rs",
        "Rust".to_string(),
        45,
        35,
        5,
        5,
    )?
    .with_category(FileCategory::Test)
    .with_size_bytes(950);

    let readme_md =
        FileMetrics::new("README.md", "Markdown".to_string(), 25, 20, 0, 5)?
            .with_category(FileCategory::Documentation)
            .with_size_bytes(600);

    let cargo_toml =
        FileMetrics::new("Cargo.toml", "TOML".to_string(), 15, 12, 2, 1)?
            .with_category(FileCategory::Configuration)
            .with_size_bytes(400);

    project.add_file_metrics(main_rs)?;
    project.add_file_metrics(lib_rs)?;
    project.add_file_metrics(test_rs)?;
    project.add_file_metrics(readme_md)?;
    project.add_file_metrics(cargo_toml)?;

    let summary = project.get_summary();

    println!("=== Custom Project Analysis ===\n");
    println!("Project: {}", summary.project_name);
    println!("Total Files: {}", summary.total_files);
    println!("Total Lines: {}", summary.total_lines);
    println!("Code Lines: {}", summary.total_code_lines);
    println!("Comment Lines: {}", summary.total_comment_lines);
    println!("Blank Lines: {}", summary.total_blank_lines);
    println!("Total Size: {} bytes", summary.total_size_bytes);

    println!("\n=== Language Breakdown ===");
    for stats in project.get_language_statistics() {
        println!("{}:", stats.language_name);
        println!("  Files: {}", stats.file_count);
        println!("  Lines: {}", stats.total_lines);
        println!("  Code: {}", stats.code_lines);
        println!("  Comments: {}", stats.comment_lines);
        println!("  Complexity: {:.1}%", stats.complexity_ratio * 100.0);
        println!("  Documentation: {:.1}%", stats.documentation_ratio * 100.0);
        println!();
    }

    println!("=== File Details ===");
    for (lang_name, lang_analysis) in &project.language_analyses {
        println!("{}:", lang_name);
        for file in &lang_analysis.file_metrics {
            println!(
                "  {} ({:?}): {} lines, {} bytes",
                file.file_path,
                file.category,
                file.total_lines,
                file.size_bytes
            );
        }
        println!();
    }

    Ok(())
}

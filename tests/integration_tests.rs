use bytes_radar::{
    FileCategory, FileMetrics, ProjectAnalysis, RemoteAnalyzer, Result,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_file_metrics_creation() -> Result<()> {
        let metrics = FileMetrics::new(
            "src/main.rs",
            "Rust".to_string(),
            100,
            70,
            20,
            10,
        )?;

        assert_eq!(metrics.file_path, "src/main.rs");
        assert_eq!(metrics.language, "Rust");
        assert_eq!(metrics.total_lines, 100);
        assert_eq!(metrics.code_lines, 70);
        assert_eq!(metrics.comment_lines, 20);
        assert_eq!(metrics.blank_lines, 10);
        assert_eq!(metrics.category, FileCategory::Source);

        Ok(())
    }

    #[test]
    fn test_file_metrics_validation() {
        let result =
            FileMetrics::new("test.rs", "Rust".to_string(), 100, 60, 20, 15);

        assert!(result.is_err());
    }

    #[test]
    fn test_file_metrics_ratios() -> Result<()> {
        let metrics =
            FileMetrics::new("src/lib.rs", "Rust".to_string(), 100, 80, 15, 5)?;

        assert_eq!(metrics.complexity_ratio(), 0.8);
        assert_eq!(metrics.documentation_ratio(), 15.0 / 80.0);

        Ok(())
    }

    #[test]
    fn test_project_analysis_creation() {
        let project = ProjectAnalysis::new("test-project");
        assert_eq!(project.project_name, "test-project");
        assert!(project.language_analyses.is_empty());
    }

    #[test]
    fn test_project_analysis_add_files() -> Result<()> {
        let mut project = ProjectAnalysis::new("test-project");

        let rust_file =
            FileMetrics::new("src/main.rs", "Rust".to_string(), 50, 35, 10, 5)?;

        let js_file = FileMetrics::new(
            "script.js",
            "JavaScript".to_string(),
            30,
            25,
            3,
            2,
        )?;

        project.add_file_metrics(rust_file)?;
        project.add_file_metrics(js_file)?;

        assert_eq!(project.language_analyses.len(), 2);
        assert!(project.language_analyses.contains_key("Rust"));
        assert!(project.language_analyses.contains_key("JavaScript"));

        let summary = project.get_summary();
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_lines, 80);
        assert_eq!(summary.total_code_lines, 60);

        Ok(())
    }

    #[test]
    fn test_language_statistics() -> Result<()> {
        let mut project = ProjectAnalysis::new("multi-lang-project");

        project.add_file_metrics(FileMetrics::new(
            "main.rs",
            "Rust".to_string(),
            100,
            80,
            15,
            5,
        )?)?;

        project.add_file_metrics(FileMetrics::new(
            "lib.rs",
            "Rust".to_string(),
            50,
            40,
            8,
            2,
        )?)?;

        project.add_file_metrics(FileMetrics::new(
            "app.js",
            "JavaScript".to_string(),
            75,
            60,
            10,
            5,
        )?)?;

        let stats = project.get_language_statistics();
        assert_eq!(stats.len(), 2);

        let rust_stats =
            stats.iter().find(|s| s.language_name == "Rust").unwrap();
        assert_eq!(rust_stats.file_count, 2);
        assert_eq!(rust_stats.total_lines, 150);
        assert_eq!(rust_stats.code_lines, 120);

        let js_stats = stats
            .iter()
            .find(|s| s.language_name == "JavaScript")
            .unwrap();
        assert_eq!(js_stats.file_count, 1);
        assert_eq!(js_stats.total_lines, 75);
        assert_eq!(js_stats.code_lines, 60);

        Ok(())
    }

    #[test]
    fn test_file_categories() -> Result<()> {
        let mut project = ProjectAnalysis::new("categorized-project");

        let source_file =
            FileMetrics::new("src/main.rs", "Rust".to_string(), 50, 40, 8, 2)?
                .with_category(FileCategory::Source);

        let test_file = FileMetrics::new(
            "tests/test.rs",
            "Rust".to_string(),
            30,
            25,
            3,
            2,
        )?
        .with_category(FileCategory::Test);

        let doc_file = FileMetrics::new(
            "README.md",
            "Markdown".to_string(),
            20,
            18,
            0,
            2,
        )?
        .with_category(FileCategory::Documentation);

        project.add_file_metrics(source_file)?;
        project.add_file_metrics(test_file)?;
        project.add_file_metrics(doc_file)?;

        assert_eq!(project.get_summary().total_files, 3);

        Ok(())
    }

    #[test]
    fn test_remote_analyzer_creation() {
        let _analyzer = RemoteAnalyzer::new();

        let _default_analyzer = RemoteAnalyzer::default();

        assert!(true);
    }

    #[test]
    fn test_analyzer_configuration() {
        let mut analyzer = RemoteAnalyzer::new();

        analyzer.set_timeout(120);
        analyzer.set_allow_insecure(true);
        analyzer.set_github_token("test-token");

        assert!(true);
    }

    #[tokio::test]
    async fn test_project_summary_primary_language() -> Result<()> {
        let mut project = ProjectAnalysis::new("lang-test");

        project.add_file_metrics(FileMetrics::new(
            "main.py",
            "Python".to_string(),
            200,
            160,
            30,
            10,
        )?)?;

        project.add_file_metrics(FileMetrics::new(
            "config.json",
            "JSON".to_string(),
            10,
            8,
            0,
            2,
        )?)?;

        let summary = project.get_summary();
        assert_eq!(summary.primary_language, Some("Python".to_string()));

        Ok(())
    }
}

use bytes_radar::{AggregateMetrics, FileMetrics, LanguageAnalysis, Result};

#[cfg(test)]
mod analysis_tests {
    use super::*;

    #[test]
    fn test_language_analysis_creation() {
        let analysis = LanguageAnalysis::new("Rust".to_string());

        assert_eq!(analysis.language_name, "Rust");
        assert!(analysis.file_metrics.is_empty());
        assert_eq!(analysis.aggregate_metrics.total_lines, 0);
    }

    #[test]
    fn test_language_analysis_add_files() -> Result<()> {
        let mut analysis = LanguageAnalysis::new("Rust".to_string());

        let file1 =
            FileMetrics::new("main.rs", "Rust".to_string(), 100, 80, 15, 5)?;

        let file2 =
            FileMetrics::new("lib.rs", "Rust".to_string(), 50, 40, 8, 2)?;

        analysis.add_file_metrics(file1)?;
        analysis.add_file_metrics(file2)?;

        assert_eq!(analysis.file_metrics.len(), 2);
        assert_eq!(analysis.aggregate_metrics.total_lines, 150);
        assert_eq!(analysis.aggregate_metrics.code_lines, 120);
        assert_eq!(analysis.aggregate_metrics.comment_lines, 23);
        assert_eq!(analysis.aggregate_metrics.blank_lines, 7);
        assert_eq!(analysis.aggregate_metrics.file_count, 2);

        Ok(())
    }

    #[test]
    fn test_language_analysis_wrong_language() -> Result<()> {
        let mut analysis = LanguageAnalysis::new("Rust".to_string());

        let wrong_file = FileMetrics::new(
            "script.js",
            "JavaScript".to_string(),
            50,
            40,
            5,
            5,
        )?;

        let result = analysis.add_file_metrics(wrong_file);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_language_analysis_merge() -> Result<()> {
        let mut analysis1 = LanguageAnalysis::new("Python".to_string());
        let mut analysis2 = LanguageAnalysis::new("Python".to_string());

        analysis1.add_file_metrics(FileMetrics::new(
            "main.py",
            "Python".to_string(),
            100,
            80,
            15,
            5,
        )?)?;

        analysis2.add_file_metrics(FileMetrics::new(
            "utils.py",
            "Python".to_string(),
            75,
            60,
            10,
            5,
        )?)?;

        analysis1.merge(analysis2)?;

        assert_eq!(analysis1.file_metrics.len(), 2);
        assert_eq!(analysis1.aggregate_metrics.total_lines, 175);
        assert_eq!(analysis1.aggregate_metrics.code_lines, 140);

        Ok(())
    }

    #[test]
    fn test_language_analysis_merge_different_languages() -> Result<()> {
        let mut rust_analysis = LanguageAnalysis::new("Rust".to_string());
        let mut js_analysis = LanguageAnalysis::new("JavaScript".to_string());

        rust_analysis.add_file_metrics(FileMetrics::new(
            "main.rs",
            "Rust".to_string(),
            50,
            40,
            8,
            2,
        )?)?;

        js_analysis.add_file_metrics(FileMetrics::new(
            "app.js",
            "JavaScript".to_string(),
            30,
            25,
            3,
            2,
        )?)?;

        let result = rust_analysis.merge(js_analysis);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_language_statistics_calculation() -> Result<()> {
        let mut analysis = LanguageAnalysis::new("TypeScript".to_string());

        analysis.add_file_metrics(FileMetrics::new(
            "component.ts",
            "TypeScript".to_string(),
            120,
            100,
            15,
            5,
        )?)?;

        analysis.add_file_metrics(FileMetrics::new(
            "service.ts",
            "TypeScript".to_string(),
            80,
            65,
            10,
            5,
        )?)?;

        let stats = analysis.calculate_statistics();

        assert_eq!(stats.language_name, "TypeScript");
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.total_lines, 200);
        assert_eq!(stats.code_lines, 165);
        assert_eq!(stats.comment_lines, 25);
        assert_eq!(stats.blank_lines, 10);

        assert_eq!(stats.average_file_size, 100.0);

        assert!((stats.complexity_ratio - 0.825).abs() < 0.001);
        assert!((stats.documentation_ratio - (25.0 / 165.0)).abs() < 0.001);

        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_initialization() {
        let metrics = AggregateMetrics::default();

        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.code_lines, 0);
        assert_eq!(metrics.comment_lines, 0);
        assert_eq!(metrics.blank_lines, 0);
        assert_eq!(metrics.total_size_bytes, 0);
        assert_eq!(metrics.file_count, 0);
    }

    #[test]
    fn test_aggregate_metrics_incorporate() -> Result<()> {
        let mut aggregate = AggregateMetrics::default();

        let file1 =
            FileMetrics::new("test1.rs", "Rust".to_string(), 50, 40, 8, 2)?
                .with_size_bytes(1000);

        let file2 =
            FileMetrics::new("test2.rs", "Rust".to_string(), 30, 25, 3, 2)?
                .with_size_bytes(600);

        aggregate.incorporate(&file1);
        aggregate.incorporate(&file2);

        assert_eq!(aggregate.total_lines, 80);
        assert_eq!(aggregate.code_lines, 65);
        assert_eq!(aggregate.comment_lines, 11);
        assert_eq!(aggregate.blank_lines, 4);
        assert_eq!(aggregate.total_size_bytes, 1600);
        assert_eq!(aggregate.file_count, 2);

        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_ratios() -> Result<()> {
        let mut aggregate = AggregateMetrics::default();

        let file =
            FileMetrics::new("test.rs", "Rust".to_string(), 100, 80, 15, 5)?;

        aggregate.incorporate(&file);

        assert_eq!(aggregate.complexity_ratio(), 0.8);
        assert_eq!(aggregate.documentation_ratio(), 15.0 / 80.0);

        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_empty_ratios() {
        let aggregate = AggregateMetrics::default();

        assert_eq!(aggregate.complexity_ratio(), 0.0);
        assert_eq!(aggregate.documentation_ratio(), 0.0);
    }

    #[test]
    fn test_file_metrics_edge_cases() -> Result<()> {
        let empty_file =
            FileMetrics::new("empty.rs", "Rust".to_string(), 0, 0, 0, 0)?;

        assert_eq!(empty_file.complexity_ratio(), 0.0);
        assert_eq!(empty_file.documentation_ratio(), 0.0);

        let no_comments =
            FileMetrics::new("simple.rs", "Rust".to_string(), 10, 8, 0, 2)?;

        assert_eq!(no_comments.documentation_ratio(), 0.0);

        Ok(())
    }

    #[test]
    fn test_file_metrics_validation_edge_cases() -> Result<()> {
        let empty_path = FileMetrics::new("", "Rust".to_string(), 10, 8, 1, 1)?;
        assert!(empty_path.validate().is_err());

        let empty_language =
            FileMetrics::new("test.rs", "".to_string(), 10, 8, 1, 1)?;
        assert!(empty_language.validate().is_err());

        Ok(())
    }
}

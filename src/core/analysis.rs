use crate::error::{AnalysisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    Source,
    Documentation,
    Configuration,
    Data,
    Binary,
    Test,
    Build,
}

impl Default for FileCategory {
    fn default() -> Self {
        Self::Source
    }
}

impl Display for FileCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let category = match self {
            Self::Source => "Source",
            Self::Documentation => "Documentation",
            Self::Configuration => "Configuration",
            Self::Data => "Data",
            Self::Binary => "Binary",
            Self::Test => "Test",
            Self::Build => "Build",
        };
        write!(f, "{category}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file_path: String,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub category: FileCategory,
    pub language: String,
    pub size_bytes: u64,
}

impl FileMetrics {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        language: String,
        total_lines: usize,
        code_lines: usize,
        comment_lines: usize,
        blank_lines: usize,
    ) -> Result<Self> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();

        if total_lines != code_lines + comment_lines + blank_lines {
            return Err(AnalysisError::invalid_statistics(format!(
                "Line count mismatch: total={}, sum of parts={}",
                total_lines,
                code_lines + comment_lines + blank_lines
            )));
        }

        Ok(Self {
            file_path: path_str,
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            category: FileCategory::default(),
            language,
            size_bytes: 0,
        })
    }

    pub fn with_category(mut self, category: FileCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_size_bytes(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    pub fn complexity_ratio(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            self.code_lines as f64 / self.total_lines as f64
        }
    }

    pub fn documentation_ratio(&self) -> f64 {
        if self.code_lines == 0 {
            0.0
        } else {
            self.comment_lines as f64 / self.code_lines as f64
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.file_path.is_empty() {
            return Err(AnalysisError::invalid_statistics(
                "File path cannot be empty",
            ));
        }

        if self.language.is_empty() {
            return Err(AnalysisError::invalid_statistics(
                "Language cannot be empty",
            ));
        }

        if self.total_lines
            != self.code_lines + self.comment_lines + self.blank_lines
        {
            return Err(AnalysisError::invalid_statistics(
                "Line counts don't add up",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysis {
    pub language_name: String,
    pub file_metrics: Vec<FileMetrics>,
    pub aggregate_metrics: AggregateMetrics,
}

impl LanguageAnalysis {
    pub fn new(language_name: String) -> Self {
        Self {
            language_name,
            file_metrics: Vec::new(),
            aggregate_metrics: AggregateMetrics::default(),
        }
    }

    pub fn add_file_metrics(&mut self, metrics: FileMetrics) -> Result<()> {
        metrics.validate()?;

        if metrics.language != self.language_name {
            return Err(AnalysisError::invalid_statistics(format!(
                "Language mismatch: expected '{}', got '{}'",
                self.language_name, metrics.language
            )));
        }

        self.aggregate_metrics.incorporate(&metrics);
        self.file_metrics.push(metrics);
        Ok(())
    }

    pub fn merge(&mut self, other: LanguageAnalysis) -> Result<()> {
        if self.language_name != other.language_name {
            return Err(AnalysisError::aggregation(format!(
                "Cannot merge different languages: '{}' and '{}'",
                self.language_name, other.language_name
            )));
        }

        for metrics in other.file_metrics {
            self.add_file_metrics(metrics)?;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> LanguageStatistics {
        LanguageStatistics {
            language_name: self.language_name.clone(),
            file_count: self.file_metrics.len(),
            total_lines: self.aggregate_metrics.total_lines,
            code_lines: self.aggregate_metrics.code_lines,
            comment_lines: self.aggregate_metrics.comment_lines,
            blank_lines: self.aggregate_metrics.blank_lines,
            total_size_bytes: self.aggregate_metrics.total_size_bytes,
            average_file_size: if self.file_metrics.is_empty() {
                0.0
            } else {
                self.aggregate_metrics.total_lines as f64
                    / self.file_metrics.len() as f64
            },
            complexity_ratio: self.aggregate_metrics.complexity_ratio(),
            documentation_ratio: self.aggregate_metrics.documentation_ratio(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub total_size_bytes: u64,
    pub file_count: usize,
}

impl AggregateMetrics {
    pub fn incorporate(&mut self, metrics: &FileMetrics) {
        self.total_lines += metrics.total_lines;
        self.code_lines += metrics.code_lines;
        self.comment_lines += metrics.comment_lines;
        self.blank_lines += metrics.blank_lines;
        self.total_size_bytes += metrics.size_bytes;
        self.file_count += 1;
    }

    pub fn complexity_ratio(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            self.code_lines as f64 / self.total_lines as f64
        }
    }

    pub fn documentation_ratio(&self) -> f64 {
        if self.code_lines == 0 {
            0.0
        } else {
            self.comment_lines as f64 / self.code_lines as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStatistics {
    pub language_name: String,
    pub file_count: usize,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub total_size_bytes: u64,
    pub average_file_size: f64,
    pub complexity_ratio: f64,
    pub documentation_ratio: f64,
}

impl Display for LanguageStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} files, {} lines ({} code, {} comments, {} blank) - {:.1}% complexity, {:.1}% documented",
            self.language_name,
            self.file_count,
            self.total_lines,
            self.code_lines,
            self.comment_lines,
            self.blank_lines,
            self.complexity_ratio * 100.0,
            self.documentation_ratio * 100.0
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub project_name: String,
    pub language_analyses: HashMap<String, LanguageAnalysis>,
    pub global_metrics: AggregateMetrics,
}

impl ProjectAnalysis {
    pub fn new<N: Into<String>>(project_name: N) -> Self {
        Self {
            project_name: project_name.into(),
            language_analyses: HashMap::new(),
            global_metrics: AggregateMetrics::default(),
        }
    }

    pub fn add_file_metrics(&mut self, metrics: FileMetrics) -> Result<()> {
        metrics.validate()?;

        let language_analysis = self
            .language_analyses
            .entry(metrics.language.clone())
            .or_insert_with(|| LanguageAnalysis::new(metrics.language.clone()));

        language_analysis.add_file_metrics(metrics.clone())?;
        self.global_metrics.incorporate(&metrics);

        Ok(())
    }

    pub fn get_language_statistics(&self) -> Vec<LanguageStatistics> {
        let mut stats: Vec<_> = self
            .language_analyses
            .values()
            .map(|analysis| analysis.calculate_statistics())
            .collect();

        stats.sort_by(|a, b| b.total_lines.cmp(&a.total_lines));
        stats
    }

    pub fn get_summary(&self) -> ProjectSummary {
        let language_stats = self.get_language_statistics();

        ProjectSummary {
            project_name: self.project_name.clone(),
            total_files: self.global_metrics.file_count,
            total_lines: self.global_metrics.total_lines,
            total_code_lines: self.global_metrics.code_lines,
            total_comment_lines: self.global_metrics.comment_lines,
            total_blank_lines: self.global_metrics.blank_lines,
            total_size_bytes: self.global_metrics.total_size_bytes,
            language_count: self.language_analyses.len(),
            primary_language: language_stats
                .first()
                .map(|s| s.language_name.clone()),
            overall_complexity_ratio: self.global_metrics.complexity_ratio(),
            overall_documentation_ratio: self
                .global_metrics
                .documentation_ratio(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub project_name: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size_bytes: u64,
    pub language_count: usize,
    pub primary_language: Option<String>,
    pub overall_complexity_ratio: f64,
    pub overall_documentation_ratio: f64,
}

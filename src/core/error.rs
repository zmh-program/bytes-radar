use std::io;
use thiserror::Error;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DownloadUrlError {
    pub url: String,
    pub error_message: String,
    pub error_type: String,
    pub http_status_code: Option<u16>,
    pub retry_count: u32,
}

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to read file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Unsupported file extension: {extension}")]
    UnsupportedExtension { extension: String },

    #[error("Language not found: {language}")]
    LanguageNotFound { language: String },

    #[error("Invalid file statistics: {reason}")]
    InvalidStatistics { reason: String },

    #[error("Directory traversal failed: {path}")]
    DirectoryTraversalError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Language detection failed for file: {file_path}")]
    LanguageDetectionError { file_path: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("JSON serialization error")]
    JsonSerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("XML serialization error: {message}")]
    XmlSerializationError { message: String },

    #[error("Aggregation error: {operation}")]
    AggregationError { operation: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Archive processing error: {message}")]
    ArchiveError { message: String },

    #[error("URL parsing error: {url}")]
    UrlParsingError { url: String },
}

pub type Result<T> = std::result::Result<T, AnalysisError>;

impl AnalysisError {
    pub fn file_read<P: AsRef<str>>(path: P, source: io::Error) -> Self {
        Self::FileReadError {
            path: path.as_ref().to_string(),
            source,
        }
    }

    pub fn unsupported_extension<E: AsRef<str>>(extension: E) -> Self {
        Self::UnsupportedExtension {
            extension: extension.as_ref().to_string(),
        }
    }

    pub fn language_not_found<L: AsRef<str>>(language: L) -> Self {
        Self::LanguageNotFound {
            language: language.as_ref().to_string(),
        }
    }

    pub fn invalid_statistics<R: AsRef<str>>(reason: R) -> Self {
        Self::InvalidStatistics {
            reason: reason.as_ref().to_string(),
        }
    }

    pub fn directory_traversal<P: AsRef<str>>(
        path: P,
        source: io::Error,
    ) -> Self {
        Self::DirectoryTraversalError {
            path: path.as_ref().to_string(),
            source,
        }
    }

    pub fn language_detection<P: AsRef<str>>(file_path: P) -> Self {
        Self::LanguageDetectionError {
            file_path: file_path.as_ref().to_string(),
        }
    }

    pub fn configuration<M: AsRef<str>>(message: M) -> Self {
        Self::ConfigurationError {
            message: message.as_ref().to_string(),
        }
    }

    pub fn aggregation<O: AsRef<str>>(operation: O) -> Self {
        Self::AggregationError {
            operation: operation.as_ref().to_string(),
        }
    }

    pub fn network<M: AsRef<str>>(message: M) -> Self {
        Self::NetworkError {
            message: message.as_ref().to_string(),
        }
    }

    pub fn archive<M: AsRef<str>>(message: M) -> Self {
        Self::ArchiveError {
            message: message.as_ref().to_string(),
        }
    }

    pub fn url_parsing<U: AsRef<str>>(url: U) -> Self {
        Self::UrlParsingError {
            url: url.as_ref().to_string(),
        }
    }

    pub fn xml_serialization<M: AsRef<str>>(message: M) -> Self {
        Self::XmlSerializationError {
            message: message.as_ref().to_string(),
        }
    }
}

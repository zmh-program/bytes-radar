use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentFilter {
    pub max_file_size: u64,
    pub ignore_hidden: bool,
    pub ignore_binary: bool,
    pub ignore_build_dirs: bool,
    pub ignore_package_dirs: bool,
    pub ignore_test_dirs: bool,
    pub ignore_docs_dirs: bool,
    pub custom_ignore_patterns: Vec<String>,
    pub allowed_extensions: Option<Vec<String>>,
}

impl Default for IntelligentFilter {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024,
            ignore_hidden: true,
            ignore_binary: true,
            ignore_build_dirs: true,
            ignore_package_dirs: true,
            ignore_test_dirs: false,
            ignore_docs_dirs: false,
            custom_ignore_patterns: Vec::new(),
            allowed_extensions: None,
        }
    }
}

impl IntelligentFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn aggressive() -> Self {
        Self {
            max_file_size: 512 * 1024,
            ignore_hidden: true,
            ignore_binary: true,
            ignore_build_dirs: true,
            ignore_package_dirs: true,
            ignore_test_dirs: true,
            ignore_docs_dirs: true,
            custom_ignore_patterns: vec![
                "*.lock".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.cache".to_string(),
            ],
            allowed_extensions: Some(vec![
                "rs".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "py".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "hpp".to_string(),
                "go".to_string(),
                "php".to_string(),
                "rb".to_string(),
                "cs".to_string(),
                "swift".to_string(),
                "kt".to_string(),
                "scala".to_string(),
                "dart".to_string(),
                "vue".to_string(),
                "jsx".to_string(),
                "tsx".to_string(),
            ]),
        }
    }

    pub fn should_process_file(&self, file_path: &str, file_size: u64) -> bool {
        let path = Path::new(file_path);

        if file_size > self.max_file_size {
            return false;
        }

        if self.ignore_hidden && self.is_hidden_file(path) {
            return false;
        }

        if self.ignore_binary && self.is_binary_file(path) {
            return false;
        }

        if self.ignore_build_dirs && self.is_in_build_directory(path) {
            return false;
        }

        if self.ignore_package_dirs && self.is_in_package_directory(path) {
            return false;
        }

        if self.ignore_test_dirs && self.is_in_test_directory(path) {
            return false;
        }

        if self.ignore_docs_dirs && self.is_in_docs_directory(path) {
            return false;
        }

        if self.matches_custom_ignore_patterns(file_path) {
            return false;
        }

        if let Some(ref allowed_exts) = self.allowed_extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !allowed_exts.iter().any(|e| e.to_lowercase() == ext_str) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn is_hidden_file(&self, path: &Path) -> bool {
        path.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
    }

    fn is_binary_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "exe"
                    | "dll"
                    | "so"
                    | "dylib"
                    | "bin"
                    | "obj"
                    | "o"
                    | "a"
                    | "lib"
                    | "jpg"
                    | "jpeg"
                    | "png"
                    | "gif"
                    | "bmp"
                    | "ico"
                    | "svg"
                    | "webp"
                    | "tiff"
                    | "mp3"
                    | "mp4"
                    | "wav"
                    | "avi"
                    | "mov"
                    | "wmv"
                    | "flv"
                    | "mkv"
                    | "webm"
                    | "zip"
                    | "tar"
                    | "gz"
                    | "bz2"
                    | "xz"
                    | "7z"
                    | "rar"
                    | "jar"
                    | "war"
                    | "pdf"
                    | "doc"
                    | "docx"
                    | "xls"
                    | "xlsx"
                    | "ppt"
                    | "pptx"
                    | "ttf"
                    | "otf"
                    | "woff"
                    | "woff2"
                    | "eot"
            )
        } else {
            false
        }
    }

    fn is_in_build_directory(&self, path: &Path) -> bool {
        path.components().any(|component| {
            let component_str =
                component.as_os_str().to_string_lossy().to_lowercase();
            component_str == "target"
                || component_str == "build"
                || component_str == "dist"
                || component_str == "out"
                || component_str == ".cargo"
                || component_str.starts_with("cmake-build-")
        })
    }

    fn is_in_package_directory(&self, path: &Path) -> bool {
        path.components().any(|component| {
            let component_str =
                component.as_os_str().to_string_lossy().to_lowercase();
            component_str == "node_modules"
                || component_str == "vendor"
                || component_str == ".nuget"
                || component_str == "packages"
                || component_str == ".pub-cache"
                || component_str == "bower_components"
        })
    }

    fn is_in_test_directory(&self, path: &Path) -> bool {
        path.components().any(|component| {
            let component_str =
                component.as_os_str().to_string_lossy().to_lowercase();
            component_str == "tests"
                || component_str == "test"
                || component_str == "__tests__"
                || component_str == "spec"
                || component_str == "__pycache__"
        })
    }

    fn is_in_docs_directory(&self, path: &Path) -> bool {
        path.components().any(|component| {
            let component_str =
                component.as_os_str().to_string_lossy().to_lowercase();
            component_str == "docs"
                || component_str == "doc"
                || component_str == "documentation"
                || component_str == ".github"
                || component_str == "examples"
        })
    }

    fn matches_custom_ignore_patterns(&self, file_path: &str) -> bool {
        for pattern in &self.custom_ignore_patterns {
            if self.glob_match(pattern, file_path) {
                return true;
            }
        }
        false
    }

    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                text.starts_with(prefix) && text.ends_with(suffix)
            } else {
                false
            }
        } else {
            text == pattern
        }
    }
}

pub struct FilterStats {
    pub total_entries: usize,
    pub filtered_out: usize,
    pub processed: usize,
    pub bytes_saved: u64,
}

impl FilterStats {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            filtered_out: 0,
            processed: 0,
            bytes_saved: 0,
        }
    }

    pub fn record_entry(&mut self, file_size: u64, was_filtered: bool) {
        self.total_entries += 1;
        if was_filtered {
            self.filtered_out += 1;
            self.bytes_saved += file_size;
        } else {
            self.processed += 1;
        }
    }

    pub fn filter_ratio(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.filtered_out as f64 / self.total_entries as f64
        }
    }

    pub fn format_bytes_saved(&self) -> String {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB"];
        let mut size = self.bytes_saved as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", self.bytes_saved, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = IntelligentFilter::default();

        assert!(filter.should_process_file("src/main.rs", 1000));
        assert!(!filter.should_process_file(".hidden/file.rs", 1000));
        assert!(!filter.should_process_file("target/debug/main", 1000));
        assert!(!filter.should_process_file("file.exe", 1000));
        assert!(!filter.should_process_file("large_file.rs", 2 * 1024 * 1024));
    }

    #[test]
    fn test_aggressive_filter() {
        let filter = IntelligentFilter::aggressive();

        assert!(filter.should_process_file("src/main.rs", 1000));
        assert!(!filter.should_process_file("README.md", 1000));
        assert!(!filter.should_process_file("tests/test.rs", 1000));
        assert!(!filter.should_process_file("docs/guide.rs", 1000));
    }

    #[test]
    fn test_build_directory_detection() {
        let filter = IntelligentFilter::default();

        assert!(!filter.should_process_file("target/debug/main.rs", 1000));
        assert!(!filter.should_process_file("build/output/file.rs", 1000));
        assert!(!filter.should_process_file("dist/bundle.js", 1000));
        assert!(filter.should_process_file("src/target_parser.rs", 1000));
    }

    #[test]
    fn test_package_directory_detection() {
        let filter = IntelligentFilter::default();

        assert!(
            !filter.should_process_file("node_modules/package/index.js", 1000)
        );
        assert!(!filter.should_process_file("vendor/package/lib.php", 1000));
        assert!(filter.should_process_file("src/vendor_api.rs", 1000));
    }
}

use crate::core::{filter::IntelligentFilter, net::RemoteAnalyzer};
use wasm_bindgen::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalysisOptions {
    pub ignore_hidden: bool,
    pub ignore_gitignore: bool,
    pub max_file_size: i64,
    pub aggressive_filtering: Option<bool>,
    pub custom_filter: Option<IntelligentFilter>,
}

#[wasm_bindgen]
pub async fn analyze_url(
    url: String,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let opts: AnalysisOptions = serde_wasm_bindgen::from_value(options)?;

    web_sys::console::log_1(
        &format!("Starting analysis for URL: {}", url).into(),
    );

    let mut analyzer = RemoteAnalyzer::new();

    if let Some(custom_filter) = opts.custom_filter {
        analyzer.set_filter(custom_filter);
    } else if let Some(aggressive) = opts.aggressive_filtering {
        analyzer.set_aggressive_filtering(aggressive);
    } else {
        let mut filter = IntelligentFilter::default();
        filter.max_file_size = opts.max_file_size as u64;
        filter.ignore_hidden = opts.ignore_hidden;
        analyzer.set_filter(filter);
    }

    match analyzer.analyze_url(&url).await {
        Ok(analysis) => {
            web_sys::console::log_1(
                &format!(
                    "Analysis completed successfully for project: {} ({} files, {} languages)",
                    analysis.project_name,
                    analysis.global_metrics.file_count,
                    analysis.language_analyses.len()
                )
                .into(),
            );

            #[derive(serde::Serialize)]
            struct WASMAnalysisResult {
                project_name: String,
                summary: crate::core::analysis::ProjectSummary,
                language_statistics:
                    Vec<crate::core::analysis::LanguageStatistics>,
                wasm_debug_info: WASMDebugInfo,
            }

            #[derive(serde::Serialize, Clone)]
            struct WASMDebugInfo {
                total_languages: usize,
                total_files: usize,
            }

            let language_statistics = analysis.get_language_statistics();
            let summary = analysis.get_summary();

            let largest_file = analysis
                .language_analyses
                .values()
                .flat_map(|lang| &lang.file_metrics)
                .max_by_key(|file| file.total_lines)
                .map(|file| file.file_path.clone());

            let wasm_debug_info = WASMDebugInfo {
                total_languages: analysis.language_analyses.len(),
                total_files: analysis.global_metrics.file_count,
            };

            let result = WASMAnalysisResult {
                project_name: analysis.project_name.clone(),
                summary,
                language_statistics,
                wasm_debug_info: wasm_debug_info.clone(),
            };

            web_sys::console::log_1(
                &format!(
                    "WASM Analysis Debug - Languages: {}, Files: {}",
                    wasm_debug_info.total_languages,
                    wasm_debug_info.total_files,
                )
                .into(),
            );

            Ok(serde_wasm_bindgen::to_value(&result)?)
        }
        Err(e) => {
            let error_msg = format!("Analysis failed: {}", e);
            web_sys::console::log_1(&error_msg.into());
            web_sys::console::log_1(
                &format!("Error details - URL: {}, Error: {:?}", url, e).into(),
            );

            #[derive(serde::Serialize)]
            struct WASMErrorResult {
                error: String,
                error_type: String,
                url: String,
            }

            let error_result = WASMErrorResult {
                error: format!("{}", e),
                error_type: match e {
                    crate::core::error::AnalysisError::NetworkError {
                        ..
                    } => "NetworkError".to_string(),
                    _ => "AnalysisError".to_string(),
                },
                url: url.clone(),
            };

            Err(serde_wasm_bindgen::to_value(&error_result)?)
        }
    }
}

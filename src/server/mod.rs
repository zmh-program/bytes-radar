use crate::core::RemoteAnalyzer;
use wasm_bindgen::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalysisOptions {
    pub ignore_hidden: bool,
    pub ignore_gitignore: bool,
    pub max_file_size: i64,
}

#[cfg(feature = "worker")]
#[wasm_bindgen]
pub async fn analyze_url_server(url: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let _opts: AnalysisOptions = serde_wasm_bindgen::from_value(options)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse options: {}", e)))?;

    let mut analyzer = RemoteAnalyzer::new();
    analyzer.set_timeout(300);

    match analyzer.analyze_url(url).await {
        Ok(analysis) => {
            web_sys::console::log_1(
                &format!(
                    "Server: Successfully analyzed project: {}",
                    analysis.project_name
                )
                .into(),
            );
            serde_wasm_bindgen::to_value(&analysis).map_err(|e| {
                JsValue::from_str(&format!("Failed to serialize analysis result: {}", e))
            })
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Server: Analysis failed: {}", e).into());
            Err(JsValue::from_str(&format!("Analysis failed: {}", e)))
        }
    }
}

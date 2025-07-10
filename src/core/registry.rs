use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LanguageType {
    Programming,
    Markup,
    Data,
    Configuration,
    Documentation,
    Other,
}

impl Default for LanguageType {
    fn default() -> Self {
        LanguageType::Programming
    }
}

impl Display for LanguageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageType::Programming => write!(f, "Programming"),
            LanguageType::Markup => write!(f, "Markup"),
            LanguageType::Data => write!(f, "Data"),
            LanguageType::Configuration => write!(f, "Configuration"),
            LanguageType::Documentation => write!(f, "Documentation"),
            LanguageType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineCommentPosition {
    Any,
    Start,
}

impl Default for LineCommentPosition {
    fn default() -> Self {
        LineCommentPosition::Any
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filenames: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shebangs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mime_types: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        alias = "line_comment"
    )]
    pub line_comments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multi_line_comments: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nested_comments: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doc_quotes: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quotes: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verbatim_quotes: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub important_syntax: Vec<String>,
    #[serde(default)]
    pub language_type: LanguageType,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_literate: bool,
    #[serde(default, skip_serializing_if = "is_false", alias = "nested")]
    pub is_nested: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_blank: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub case_sensitive: bool,
    #[serde(default)]
    pub line_comment_position: LineCommentPosition,
}

fn is_false(b: &bool) -> bool {
    !b
}

fn is_true(b: &bool) -> bool {
    *b
}

fn default_true() -> bool {
    true
}

pub struct LanguageRegistry;

impl LanguageRegistry {
    pub fn get_language(name: &str) -> Option<&'static LanguageDefinition> {
        LANGUAGE_MAP.get(name)
    }

    pub fn detect_by_extension(
        extension: &str,
    ) -> Option<&'static LanguageDefinition> {
        let ext = extension.to_lowercase();
        EXTENSION_MAP
            .get(&ext)
            .and_then(|name| LANGUAGE_MAP.get(name))
    }

    pub fn detect_by_filename(
        filename: &str,
    ) -> Option<&'static LanguageDefinition> {
        let lower_filename = filename.to_lowercase();
        FILENAME_MAP
            .get(&lower_filename)
            .and_then(|name| LANGUAGE_MAP.get(name))
    }

    pub fn detect_by_path<P: AsRef<Path>>(
        path: P,
    ) -> Option<&'static LanguageDefinition> {
        let path = path.as_ref();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(lang) = Self::detect_by_filename(filename) {
                return Some(lang);
            }
        }

        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            return Self::detect_by_extension(extension);
        }

        None
    }

    pub fn all_languages() -> impl Iterator<Item = &'static LanguageDefinition>
    {
        LANGUAGE_MAP.values()
    }

    pub fn languages_by_type(
        lang_type: LanguageType,
    ) -> impl Iterator<Item = &'static LanguageDefinition> {
        LANGUAGE_MAP
            .values()
            .filter(move |lang| lang.language_type == lang_type)
    }
}

fn create_languages() -> HashMap<String, LanguageDefinition> {
    const LANGUAGES_JSON: &str = include_str!("../languages.json");
    let mut languages: HashMap<String, LanguageDefinition> =
        serde_json::from_str(LANGUAGES_JSON)
            .expect("Failed to parse languages.json");

    for (key, lang_def) in languages.iter_mut() {
        if lang_def.name.is_empty() {
            lang_def.name = key.clone();
        }
    }

    languages
}

fn create_extension_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (name, lang) in LANGUAGE_MAP.iter() {
        for ext in &lang.extensions {
            map.insert(ext.clone(), name.clone());
        }
    }
    map
}

fn create_filename_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (name, lang) in LANGUAGE_MAP.iter() {
        for filename in &lang.filenames {
            map.insert(filename.to_lowercase(), name.clone());
        }
    }
    map
}

static LANGUAGE_MAP: Lazy<HashMap<String, LanguageDefinition>> =
    Lazy::new(create_languages);
static EXTENSION_MAP: Lazy<HashMap<String, String>> =
    Lazy::new(create_extension_map);
static FILENAME_MAP: Lazy<HashMap<String, String>> =
    Lazy::new(create_filename_map);

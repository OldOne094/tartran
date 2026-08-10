use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub workspace_path: String,
    pub ui_language: String,
    pub theme: String,
    pub remove_tashkeel: bool,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    pub target_lang: String,
    #[serde(default)]
    pub source_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub title: Option<String>,
    pub author: Option<String>,
    pub target_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsInput {
    pub workspace_path: Option<String>,
    pub ui_language: Option<String>,
    pub theme: Option<String>,
    pub remove_tashkeel: Option<bool>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub author: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: String,
    pub updated_at: String,
    pub chapter_count: i64,
    pub translated_count: i64,
    pub reviewed_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrupted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterSummary {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub word_count: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDetail {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub source_text: String,
    pub translation: String,
    pub word_count: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterMemory {
    pub chapter_id: String,
    pub chapter_number: i64,
    pub summary: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChapterInput {
    pub number: Option<i64>,
    pub title: String,
    pub source_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChapterInput {
    pub title: Option<String>,
    pub source_text: Option<String>,
    pub translation: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportChaptersInput {
    pub text: String,
    #[serde(default = "default_split_mode")]
    pub split_by: String,
}

fn default_split_mode() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportChaptersResult {
    pub imported: usize,
    pub skipped: usize,
    pub chapters: Vec<ChapterSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterSearchResult {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossaryEntry {
    pub id: String,
    pub zh: String,
    pub en: String,
    pub ar: String,
    pub category: String,
    pub notes: String,
    pub aliases: Vec<String>,
    pub locked: bool,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGlossaryInput {
    pub zh: String,
    #[serde(default)]
    pub en: String,
    #[serde(default)]
    pub ar: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGlossaryInput {
    pub zh: Option<String>,
    pub en: Option<String>,
    pub ar: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub locked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossarySearchResult {
    pub id: String,
    pub zh: String,
    pub en: String,
    pub ar: String,
    pub category: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub id: String,
    pub chapter_id: String,
    pub zh: String,
    pub en: String,
    pub ar: String,
    pub category: String,
    pub notes: String,
    pub context: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSuggestionInput {
    pub chapter_id: String,
    pub zh: String,
    #[serde(default)]
    pub en: String,
    #[serde(default)]
    pub ar: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSuggestionInput {
    pub status: Option<String>,
    pub zh: Option<String>,
    pub en: Option<String>,
    pub ar: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateChapterInput {
    pub chapter_id: String,
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResult {
    pub chapter_id: String,
    pub translation: String,
    pub suggestions: Vec<Suggestion>,
    pub model: String,
    pub duration_ms: u64,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

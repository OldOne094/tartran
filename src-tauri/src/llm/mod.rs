use crate::models::ModelInfo;
use crate::storage::key_store::{KeyStore, OsKeyring};

pub mod rate_limiter;

const KEYRING_SERVICE: &str = "tartran";
const DEFAULT_KEY_ID: &str = "default";

pub const DEFAULT_MODEL: &str = "gemini-3.6-flash";

pub fn available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gemini-3.6-flash".into(),
            label: "Gemini 3.6 Flash".into(),
            description: "Default. Free-tier, 1M context.".into(),
        },
        ModelInfo {
            id: "gemini-3.1-flash-lite".into(),
            label: "Gemini 3.1 Flash Lite".into(),
            description: "Budget. Lower latency, good for bulk.".into(),
        },
        ModelInfo {
            id: "gemini-2.5-pro".into(),
            label: "Gemini 2.5 Pro".into(),
            description: "Paid. Highest quality.".into(),
        },
    ]
}

pub struct TranslateRequest<'a> {
    pub model: &'a str,
    pub system_prompt: &'a str,
    pub user_prompt: &'a str,
}

pub struct TranslateResponse {
    pub text: String,
    pub usage_tokens: u64,
}

pub enum LlmError {
    RateLimited(String),
    Network(String),
    Response(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RateLimited(m) => write!(f, "{m}"),
            LlmError::Network(m) => write!(f, "Network error: {m}"),
            LlmError::Response(m) => write!(f, "{m}"),
        }
    }
}

pub struct GeminiProvider {
    api_key: String,
    client: ureq::Agent,
}

impl GeminiProvider {
    pub fn from_keyring() -> Option<Self> {
        let store = OsKeyring::new(KEYRING_SERVICE);
        let api_key = store.get(DEFAULT_KEY_ID)?;
        Some(GeminiProvider::new(api_key))
    }

    pub fn new(api_key: String) -> Self {
        let client = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(180)))
            .http_status_as_error(false)
            .build()
            .new_agent();
        GeminiProvider { api_key, client }
    }

    pub fn translate(&self, req: &TranslateRequest<'_>) -> Result<TranslateResponse, LlmError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            req.model
        );
        let payload = serde_json::json!({
            "system_instruction": { "parts": [{ "text": req.system_prompt }] },
            "contents": [{ "parts": [{ "text": req.user_prompt }] }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 65000
            }
        });
        let resp = self
            .client
            .post(&url)
            .query("key", &self.api_key)
            .send_json(payload)
            .map_err(classify_error)?;
        let status = resp.status().as_u16();
        let body = resp
            .into_body()
            .read_to_string()
            .map_err(|e| LlmError::Network(e.to_string()))?;
        if status != 200 {
            let msg = extract_error_message(&body).unwrap_or_else(|| format!("Gemini API returned status {status}"));
            if status == 429 {
                return Err(LlmError::RateLimited(msg));
            }
            return Err(LlmError::Response(msg));
        }
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| LlmError::Response(e.to_string()))?;
        let text = extract_text(&v);
        let usage_tokens = extract_usage_tokens(&v);
        match text {
            Some(t) => Ok(TranslateResponse { text: t, usage_tokens }),
            None => Err(LlmError::Response("Empty response from Gemini".into())),
        }
    }
}

fn classify_error(e: ureq::Error) -> LlmError {
    match e {
        ureq::Error::StatusCode(429) => LlmError::RateLimited("Rate limit hit. Wait a moment and try again.".into()),
        ureq::Error::StatusCode(code) => LlmError::Response(format!("Gemini API returned status {code}")),
        ureq::Error::Timeout(_) => LlmError::Network("Request timed out".into()),
        ureq::Error::HostNotFound => LlmError::Network("Could not resolve Gemini host".into()),
        other => LlmError::Network(other.to_string()),
    }
}

fn extract_error_message(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.pointer("/error/message")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}

fn extract_text(v: &serde_json::Value) -> Option<String> {
    let candidates = v.get("candidates")?.as_array()?;
    let first = candidates.first()?;
    let parts = first.pointer("/content/parts")?.as_array()?;
    let mut out = String::new();
    for part in parts {
        if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
            out.push_str(t);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Sum of prompt + candidate tokens from `usageMetadata`, used to report free-tier consumption.
fn extract_usage_tokens(v: &serde_json::Value) -> u64 {
    let Some(meta) = v.get("usageMetadata") else {
        return 0;
    };
    let prompt = meta.get("promptTokenCount").and_then(|x| x.as_u64()).unwrap_or(0);
    let candidates = meta
        .get("candidatesTokenCount")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    prompt + candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text_from_response() {
        let v = serde_json::json!({
            "candidates": [{
                "content": { "parts": [{ "text": "الترجمة الأولى" }, { "text": " والثانية" }] }
            }]
        });
        assert_eq!(extract_text(&v).unwrap(), "الترجمة الأولى والثانية");
    }

    #[test]
    fn parses_error_message() {
        let body = r#"{"error":{"code":429,"message":"Quota exceeded"}}"#;
        assert_eq!(extract_error_message(body).unwrap(), "Quota exceeded");
    }

    #[test]
    fn extracts_usage_tokens() {
        let v = serde_json::json!({
            "candidates": [],
            "usageMetadata": { "promptTokenCount": 120, "candidatesTokenCount": 80 }
        });
        assert_eq!(extract_usage_tokens(&v), 200);
    }

    #[test]
    fn usage_is_zero_when_missing() {
        let v = serde_json::json!({ "candidates": [] });
        assert_eq!(extract_usage_tokens(&v), 0);
    }
}

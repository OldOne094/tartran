use chrono::Utc;
use regex::Regex;
use serde_json::{Map, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_STRING_LEN: usize = 300;

fn secret_regex() -> Regex {
    Regex::new(r#"(?i)AIza[0-9A-Za-z_-]{20,}|api[_-]?key[\s"':=]+[^\s"'&,}]{8,}"#).unwrap()
}

fn redact(re: &Regex, s: &str) -> String {
    re.replace_all(s, "[REDACTED]").to_string()
}

fn truncate_value(re: &Regex, value: &Value) -> Value {
    match value {
        Value::String(s) => {
            if s.chars().count() > MAX_STRING_LEN {
                Value::String(format!("[string {} chars (truncated)]", s.chars().count()))
            } else {
                Value::String(redact(re, s))
            }
        }
        Value::Array(items) => Value::Array(items.iter().map(|v| truncate_value(re, v)).collect()),
        Value::Object(map) => {
            let out: Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), truncate_value(re, v)))
                .collect();
            Value::Object(out)
        }
        other => other.clone(),
    }
}

pub struct Logger {
    dir: PathBuf,
    _lock: Mutex<()>,
}

impl Logger {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Logger {
            dir,
            _lock: Mutex::new(()),
        }
    }

    pub fn debug(&self, msg: &str, ctx: Option<&Value>) {
        self.write("debug", msg, ctx)
    }
    pub fn info(&self, msg: &str, ctx: Option<&Value>) {
        self.write("info", msg, ctx)
    }
    pub fn warn(&self, msg: &str, ctx: Option<&Value>) {
        self.write("warn", msg, ctx)
    }
    pub fn error(&self, msg: &str, ctx: Option<&Value>) {
        self.write("error", msg, ctx)
    }

    fn write(&self, level: &str, msg: &str, ctx: Option<&Value>) {
        let re = secret_regex();
        let mut entry = Map::new();
        entry.insert("t".into(), Value::String(Utc::now().to_rfc3339()));
        entry.insert("level".into(), Value::String(level.into()));
        entry.insert("msg".into(), Value::String(redact(&re, msg)));
        if let Some(c) = ctx {
            entry.insert("ctx".into(), truncate_value(&re, c));
        }
        let line = Value::Object(entry).to_string();
        let day = Utc::now().format("%Y-%m-%d").to_string();
        let file = self.dir.join(format!("app-{day}.log"));
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(file) {
            let _ = writeln!(f, "{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_keys() {
        let re = secret_regex();
        assert_eq!(redact(&re, "key=AIzaSyLongSecretKeyValue123"), "key=[REDACTED]");
    }

    #[test]
    fn truncates_long_strings() {
        let re = secret_regex();
        let long = "x".repeat(400);
        let out = truncate_value(&re, &Value::String(long));
        assert!(out.as_str().unwrap().starts_with("[string 400 chars"));
    }
}

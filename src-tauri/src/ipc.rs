pub fn ok<T: serde::Serialize>(data: T) -> serde_json::Value {
    serde_json::json!({ "ok": true, "data": data })
}

pub fn err(code: &str, message: &str) -> serde_json::Value {
    serde_json::json!({ "ok": false, "error": { "code": code, "message": message } })
}

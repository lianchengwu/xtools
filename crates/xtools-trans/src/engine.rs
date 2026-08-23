use std::time::Duration;

use serde::Deserialize;

pub trait TranslateEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn translate(&self, text: &str, src: &str, dst: &str) -> Result<String, TranslateError>;
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TranslateError {
    #[error("先输入要翻译的文字。")]
    Empty,
    #[error("网络请求失败。")]
    Network,
    #[error("{0}")]
    Engine(String),
}

/// Official no-key MyMemory API. No credentials in the repo.
pub struct MyMemoryEngine;

impl TranslateEngine for MyMemoryEngine {
    fn name(&self) -> &'static str {
        "MyMemory"
    }

    fn translate(&self, text: &str, src: &str, dst: &str) -> Result<String, TranslateError> {
        if text.trim().is_empty() {
            return Err(TranslateError::Empty);
        }
        let src = if src == "auto" { "Autodetect" } else { src };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();
        let mut response = agent
            .get("https://api.mymemory.translated.net/get")
            .query("q", text)
            .query("langpair", format!("{src}|{dst}"))
            .call()
            .map_err(|_| TranslateError::Network)?;
        let body: MemoryResponse = response
            .body_mut()
            .read_json()
            .map_err(|_| TranslateError::Engine("引擎返回无法解析。".into()))?;
        body.into_text()
    }
}

#[derive(Debug, Deserialize)]
struct MemoryResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MemoryData>,
    #[serde(rename = "responseStatus")]
    response_status: serde_json::Value,
    #[serde(rename = "responseDetails")]
    response_details: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MemoryData {
    #[serde(rename = "translatedText")]
    translated_text: Option<String>,
}

impl MemoryResponse {
    fn into_text(self) -> Result<String, TranslateError> {
        let status_ok = match &self.response_status {
            serde_json::Value::Number(n) => n.as_u64() == Some(200),
            serde_json::Value::String(s) => s == "200",
            _ => false,
        };
        let text = self
            .response_data
            .and_then(|d| d.translated_text)
            .map(|s| html_unescape(&s))
            .filter(|s| !s.trim().is_empty());
        if status_ok {
            text.ok_or_else(|| TranslateError::Engine("引擎返回空译文。".into()))
        } else {
            let detail = self
                .response_details
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "翻译失败。".into());
            Err(TranslateError::Engine(detail))
        }
    }
}

fn html_unescape(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;

    impl TranslateEngine for Echo {
        fn name(&self) -> &'static str {
            "echo"
        }

        fn translate(&self, text: &str, src: &str, dst: &str) -> Result<String, TranslateError> {
            if text.trim().is_empty() {
                return Err(TranslateError::Empty);
            }
            Ok(format!("{src}->{dst}:{text}"))
        }
    }

    #[test]
    fn trait_is_swappable() {
        let engine: Box<dyn TranslateEngine> = Box::new(Echo);
        assert_eq!(engine.name(), "echo");
        assert_eq!(
            engine.translate("hi", "en", "zh-CN").unwrap(),
            "en->zh-CN:hi"
        );
        assert_eq!(
            engine.translate("  ", "en", "zh-CN"),
            Err(TranslateError::Empty)
        );
    }

    #[test]
    fn parses_success_payload() {
        let body = MemoryResponse {
            response_data: Some(MemoryData {
                translated_text: Some("你好".into()),
            }),
            response_status: serde_json::json!(200),
            response_details: None,
        };
        assert_eq!(body.into_text().unwrap(), "你好");
    }

    #[test]
    fn parses_engine_error() {
        let body = MemoryResponse {
            response_data: None,
            response_status: serde_json::json!("403"),
            response_details: Some("QUERY LENGTH LIMIT EXCEEDED".into()),
        };
        assert_eq!(
            body.into_text(),
            Err(TranslateError::Engine("QUERY LENGTH LIMIT EXCEEDED".into()))
        );
    }
}

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

/// Baidu Translation Open Platform API.
pub struct BaiduEngine {
    pub appid: String,
    pub key: String,
}

impl BaiduEngine {
    pub fn new(appid: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            appid: appid.into(),
            key: key.into(),
        }
    }
}

impl TranslateEngine for BaiduEngine {
    fn name(&self) -> &'static str {
        "百度翻译"
    }

    fn translate(&self, text: &str, src: &str, dst: &str) -> Result<String, TranslateError> {
        if text.trim().is_empty() {
            return Err(TranslateError::Empty);
        }
        if self.appid.trim().is_empty() || self.key.trim().is_empty() {
            return Err(TranslateError::Engine(
                "请先配置百度翻译 AppID 和 密钥 (Key)。".into(),
            ));
        }

        let from_lang = to_baidu_lang(src);
        let to_lang = to_baidu_lang(dst);

        let salt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(123456)
            .to_string();

        let sign_input = format!("{}{}{}{}", self.appid.trim(), text, salt, self.key.trim());
        let sign = md5_hex(sign_input.as_bytes());

        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();

        let mut response = agent
            .post("https://fanyi-api.baidu.com/api/trans/vip/translate")
            .send_form([
                ("q", text),
                ("from", from_lang),
                ("to", to_lang),
                ("appid", self.appid.trim()),
                ("salt", &salt),
                ("sign", &sign),
            ])
            .map_err(|_| TranslateError::Network)?;

        let body: BaiduResponse = response
            .body_mut()
            .read_json()
            .map_err(|_| TranslateError::Engine("百度翻译返回无法解析。".into()))?;

        body.into_text()
    }
}

/// Maps generic language codes to Baidu language codes.
pub fn to_baidu_lang(lang: &str) -> &'static str {
    match lang {
        "auto" => "auto",
        "zh" | "zh-CN" | "zh-Hans" => "zh",
        "en" => "en",
        "ja" | "jp" => "jp",
        "ko" | "kor" => "kor",
        "fr" | "fra" => "fra",
        "de" => "de",
        "es" | "spa" => "spa",
        "ru" => "ru",
        other => {
            if other.starts_with("zh") {
                "zh"
            } else {
                "auto"
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct BaiduResponse {
    #[serde(default)]
    trans_result: Option<Vec<BaiduTransItem>>,
    #[serde(default)]
    error_code: Option<serde_json::Value>,
    #[serde(default)]
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BaiduTransItem {
    dst: String,
}

impl BaiduResponse {
    fn into_text(self) -> Result<String, TranslateError> {
        if let Some(ref code_val) = self.error_code {
            let code_str = match code_val {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => String::new(),
            };
            if !code_str.is_empty() && code_str != "52000" {
                let msg = match code_str.as_str() {
                    "52001" => "请求超时，请重试".to_string(),
                    "52002" => "百度系统错误，请稍后重试".to_string(),
                    "52003" => "未授权用户：请检查 AppID 是否正确或开通服务".to_string(),
                    "54000" => "必填参数为空".to_string(),
                    "54001" => "签名错误：请检查 AppID 与 密钥 (Key) 是否匹配".to_string(),
                    "54003" => "访问频率受限，请稍后重试".to_string(),
                    "54004" => "账户余额不足".to_string(),
                    "54005" => "长请求频繁，请稍后重试".to_string(),
                    "58000" => "客户端 IP 非法".to_string(),
                    "58001" => "译文语言方向不支持".to_string(),
                    "58002" => "服务当前已关闭".to_string(),
                    _ => {
                        let detail = self.error_msg.unwrap_or_else(|| "未知错误".into());
                        format!("百度翻译错误 [{}]: {}", code_str, detail)
                    }
                };
                return Err(TranslateError::Engine(msg));
            }
        }

        if let Some(items) = self.trans_result {
            if !items.is_empty() {
                let text = items
                    .into_iter()
                    .map(|it| it.dst)
                    .collect::<Vec<_>>()
                    .join("\n");
                if !text.trim().is_empty() {
                    return Ok(text);
                }
            }
        }

        Err(TranslateError::Engine("百度翻译返回空译文。".into()))
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

/// Standard RFC 1321 MD5 calculation.
pub fn md5_hex(input: &[u8]) -> String {
    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let s: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    let k: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut msg = input.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes(chunk[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                48..=63 => (c ^ (b | (!d)), (7 * i) % 16),
                _ => unreachable!(),
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                (a.wrapping_add(f).wrapping_add(k[i]).wrapping_add(m[g])).rotate_left(s[i]),
            );
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&a0.to_le_bytes());
    result[4..8].copy_from_slice(&b0.to_le_bytes());
    result[8..12].copy_from_slice(&c0.to_le_bytes());
    result[12..16].copy_from_slice(&d0.to_le_bytes());

    let mut hex = String::with_capacity(32);
    for byte in result {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    hex
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

    #[test]
    fn rfc1321_md5_test_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"a"), "0cc175b9c0f1b6a831c399e269772661");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            md5_hex(b"message digest"),
            "f96b697d7cb7938d525a2f31aaf161d0"
        );
        assert_eq!(
            md5_hex(b"abcdefghijklmnopqrstuvwxyz"),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
    }

    #[test]
    fn parses_baidu_success_payload() {
        let body = BaiduResponse {
            trans_result: Some(vec![
                BaiduTransItem {
                    dst: "你好".into(),
                },
                BaiduTransItem {
                    dst: "世界".into(),
                },
            ]),
            error_code: None,
            error_msg: None,
        };
        assert_eq!(body.into_text().unwrap(), "你好\n世界");
    }

    #[test]
    fn parses_baidu_error_payload() {
        let body = BaiduResponse {
            trans_result: None,
            error_code: Some(serde_json::json!("52003")),
            error_msg: Some("UNAUTHORIZED USER".into()),
        };
        assert_eq!(
            body.into_text(),
            Err(TranslateError::Engine(
                "未授权用户：请检查 AppID 是否正确或开通服务".into()
            ))
        );
    }

    #[test]
    fn baidu_lang_mapping() {
        assert_eq!(to_baidu_lang("auto"), "auto");
        assert_eq!(to_baidu_lang("zh-CN"), "zh");
        assert_eq!(to_baidu_lang("ja"), "jp");
        assert_eq!(to_baidu_lang("ko"), "kor");
        assert_eq!(to_baidu_lang("fr"), "fra");
        assert_eq!(to_baidu_lang("es"), "spa");
    }
}

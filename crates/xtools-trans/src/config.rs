use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateConfig {
    #[serde(default = "default_engine")]
    pub engine_index: i32, // 0 = MyMemory, 1 = Baidu
    #[serde(default)]
    pub baidu_appid: String,
    #[serde(default)]
    pub baidu_key: String,
}

fn default_engine() -> i32 {
    0
}

impl Default for TranslateConfig {
    fn default() -> Self {
        let mut cfg = Self {
            engine_index: 0,
            baidu_appid: String::new(),
            baidu_key: String::new(),
        };
        if let Ok(val) = std::env::var("BAIDU_TRANSLATE_APPID") {
            cfg.baidu_appid = val;
        }
        if let Ok(val) = std::env::var("BAIDU_TRANSLATE_KEY") {
            cfg.baidu_key = val;
        }
        cfg
    }
}

impl TranslateConfig {
    pub fn load() -> Self {
        let mut cfg = Self::default();
        if let Some(path) = config_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<TranslateConfig>(&content) {
                    cfg = loaded;
                }
            }
        }
        if cfg.baidu_appid.is_empty() {
            if let Ok(val) = std::env::var("BAIDU_TRANSLATE_APPID") {
                cfg.baidu_appid = val;
            }
        }
        if cfg.baidu_key.is_empty() {
            if let Ok(val) = std::env::var("BAIDU_TRANSLATE_KEY") {
                cfg.baidu_key = val;
            }
        }
        cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = config_file_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(&path, json)?;
        }
        Ok(())
    }
}

fn config_file_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|p| p.join("xtools").join("translate.json"))
    }
    #[cfg(unix)]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("xtools").join("translate.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_engine_zero() {
        let cfg = TranslateConfig::default();
        assert_eq!(cfg.engine_index, 0);
    }
}

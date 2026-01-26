use std::collections::HashMap;
use std::fs;
use std::env;

pub struct Config {
    pub discord_webhook: String,
}

impl Config {
    pub fn load() -> Self {
        let mut map = HashMap::new();
        // 獲取執行檔同目錄下的 cw.cfg
        let mut cfg_path = env::current_exe().unwrap_or_default();
        cfg_path.pop();
        cfg_path.push("cw.cfg");

        if let Ok(content) = fs::read_to_string(cfg_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() { continue; } // 跳過註釋
                if let Some((key, value)) = line.split_once('=') {
                    map.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self {
            discord_webhook: map.get("discord_webhook").cloned().unwrap_or_default(),
        }
    }
}

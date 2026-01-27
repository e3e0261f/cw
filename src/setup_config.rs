use std::collections::HashMap;
use std::fs;
use std::env;

pub struct Config {
    pub discord_webhook: String,
    pub phrase_mode: bool,
    pub verbosity: u32,
    pub auto_discord: bool,
    pub log_directory: String,
    pub log_file_prefix: String,
    pub log_level: String,
    pub log_max_size_mb: u64,
    pub log_backup_count: u32,
}

impl Config {
    pub fn load() -> Self {
        let mut map = HashMap::new();
        let mut cfg_path = env::current_exe().unwrap_or_default();
        cfg_path.pop();
        cfg_path.push("cw.cfg");

        if let Ok(content) = fs::read_to_string(cfg_path) {
            for line in content.lines() {
                let line_clean = line.split('#').next().unwrap_or("").trim();
                if line_clean.is_empty() { continue; }
                if let Some((key, value)) = line_clean.split_once('=') {
                    let clean_value = value.trim().trim_matches('"').to_string();
                    map.insert(key.trim().to_string(), clean_value);
                }
            }
        }

        let max_size_raw = map.get("log_max_size").cloned().unwrap_or_else(|| "10".to_string());
        let max_size_mb = max_size_raw.replace("MB", "").trim().parse().unwrap_or(10);

        Self {
            discord_webhook: map.get("discord_webhook").cloned().unwrap_or_default(),
            phrase_mode: map.get("phrase_mode").map(|v| v == "true").unwrap_or(false),
            verbosity: map.get("verbosity").and_then(|v| v.parse().ok()).unwrap_or(1),
            auto_discord: map.get("auto_discord").map(|v| v == "true").unwrap_or(false),
            log_directory: map.get("log_directory").cloned().unwrap_or_else(|| "/tmp".to_string()),
            log_file_prefix: map.get("log_file_prefix").cloned().unwrap_or_else(|| "cw".to_string()),
            log_level: map.get("log_level").cloned().unwrap_or_else(|| "INFO".to_string()),
            log_max_size_mb: max_size_mb,
            log_backup_count: map.get("log_backup_count").and_then(|v| v.parse().ok()).unwrap_or(5),
        }
    }
}

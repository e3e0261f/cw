use std::collections::HashMap;
use std::fs;
use std::env;
use chrono::Local; // 使用 chrono 库来获取当前日期

pub struct Config {
    pub discord_webhook: String,
    pub log_directory: String,
    pub log_file_prefix: String,
    pub log_file_date_format: String,
    pub log_level: String,
    pub log_max_size: String,
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
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() { continue; }
                if let Some((key, value)) = line.split_once('=') {
                    map.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self {
            discord_webhook: map.get("discord_webhook").cloned().unwrap_or_default(),
            log_directory: map.get("log_directory").cloned().unwrap_or("/tmp".to_string()),
            log_file_prefix: map.get("log_file_prefix").cloned().unwrap_or("cw".to_string()),
            log_file_date_format: map.get("log_file_date_format").cloned().unwrap_or("%Y-%m-%d".to_string()),
            log_level: map.get("log_level").cloned().unwrap_or("INFO".to_string()),
            log_max_size: map.get("log_max_size").cloned().unwrap_or("10MB".to_string()),
            log_backup_count: map.get("log_backup_count").and_then(|v| v.parse().ok()).unwrap_or(5),
        }
    }

    // 获取日志文件的完整路径
    pub fn log_file_path(&self) -> String {
        let current_date = Local::now().format(&self.log_file_date_format).to_string();
        format!("{}/{}-{}.log", self.log_directory, self.log_file_prefix, current_date)
    }
}


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
    pub log_file_date_format: String,
    pub log_level: String,
    pub log_max_size_mb: u64,
    pub log_backup_count: u32,
    pub mention_id: String,
    pub discord_interval: u64,
    pub translate_error: bool,
    pub show_stats: bool,
    pub discord_show_errors: bool,
}

impl Config {
    pub fn load() -> Self {
        let mut exe_path = env::current_exe().unwrap_or_default();
        exe_path.pop();
        let cfg_path = exe_path.join("cw.cfg");

        let mut map = HashMap::new();
        if let Ok(content) = fs::read_to_string(&cfg_path) {
            for line in content.lines() {
                let clean = line.split('#').next().unwrap_or("").trim();
                if let Some((k, v)) = clean.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
                }
            }
        }

        Self {
            discord_webhook: map.get("discord_webhook").cloned().unwrap_or_default(),
            phrase_mode: map.get("phrase_mode").map(|v| v == "true").unwrap_or(false),
            verbosity: map.get("verbosity").and_then(|v| v.parse().ok()).unwrap_or(1),
            auto_discord: map.get("auto_discord").map(|v| v == "true").unwrap_or(false),
            log_directory: map.get("log_directory").cloned().unwrap_or_else(|| "./logs".to_string()),
            log_file_prefix: map.get("log_file_prefix").cloned().unwrap_or_else(|| "cw".to_string()),
            log_file_date_format: map.get("log_file_date_format").cloned().unwrap_or_else(|| "%Y-%m-%d".to_string()),
            log_level: map.get("log_level").cloned().unwrap_or_else(|| "INFO".to_string()),
            log_max_size_mb: map.get("log_max_size").and_then(|v| v.replace("MB","").trim().parse().ok()).unwrap_or(10),
            log_backup_count: map.get("log_backup_count").and_then(|v| v.parse().ok()).unwrap_or(5),
            mention_id: map.get("mention_id").cloned().unwrap_or_default(),
            discord_interval: map.get("discord_interval").and_then(|v| v.parse().ok()).unwrap_or(2),
            translate_error: map.get("translate_error").map(|v| v == "true").unwrap_or(true),
            show_stats: map.get("show_stats").map(|v| v == "true").unwrap_or(false),
            discord_show_errors: map.get("discord_show_errors").map(|v| v == "true").unwrap_or(false),
        }
    }

    pub fn generate_default() -> std::io::Result<()> {
        let mut path = env::current_exe().unwrap_or_default();
        path.pop();
        let cfg_path = path.join("cw.cfg");
        let template = "# CW 專業字幕工程工作站 - 配置文件\nphrase_mode = false\nverbosity = 1\ndiscord_webhook = \"\"\nauto_discord = false\nmention_id = \"\"\ndiscord_show_errors = false\nshow_stats = false\ndiscord_interval = 2\ntranslate_error = true\nlog_directory = \"./logs\"\nlog_file_prefix = \"cw\"\nlog_file_date_format = \"%Y-%m-%d\"\nlog_level = \"INFO\"\nlog_max_size = 10MB\nlog_backup_count = 5\n";
        fs::write(cfg_path, template)?;
        Ok(())
    }
}

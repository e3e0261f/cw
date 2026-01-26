mod models;
mod utils;
mod converter;
mod auditor;

use opencc_rust::*;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use chrono::Local;

use models::{TypoData, FileReport, ResultStatus, Config};
use converter::run_conversion_full_view;
use auditor::process_audit;

// 輔助函式：將訊息同時印到螢幕並寫入日誌
fn log_info(log_path: &PathBuf, msg: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, msg);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn main() -> io::Result<()> {
    // 1. 初始化日誌與時間 (修復報錯的核心)
    let temp_log = env::temp_dir().join(format!("cw_{}.log", Local::now().format("%Y%m%d")));
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 2. 獲取參數
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        utils::print_help();
        return Ok(());
    }

    // 3. 判定標籤
    let use_phrase_mode = args.iter().any(|arg| arg == "-p");
    let wants_broadcast = args.iter().any(|arg| arg == "-b");
    let is_audit_mode = args.iter().any(|arg| arg == "-a");

    // 4. 提取路徑 (排除以 - 開頭的參數)
    let paths: Vec<PathBuf> = args.iter().skip(1)
        .filter(|a| !a.starts_with('-'))
        .map(|a| Path::new(a).to_path_buf())
        .collect();

    // 5. 讀取 Webhook 設定
    let mut webhook_url: Option<String> = None;
    if wants_broadcast {
        let config_abs_path = "/home/lee/BOok/PJct/cw/config.json";
        if let Ok(config_str) = fs::read_to_string(config_abs_path) {
            if let Ok(conf) = serde_json::from_str::<Config>(&config_str) {
                webhook_url = Some(conf.webhook_url);
            }
        }
    }

    // 6. 載入引擎
    let (ac_engine, typo_map, patterns, regex_rules) = load_typo_engine(&temp_log);
    let opencc_config = if use_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(opencc_config).expect("OpenCC 啟動失敗");

    let mut reports = Vec::new();

    // 7. 邏輯分支：審核模式 vs 轉換模式
    if is_audit_mode {
        if paths.len() < 2 {
            println!("\x1b[1;31m❌ 錯誤：審核模式需要提供原文與譯文兩個路徑。\x1b[0m");
            println!("用法: cw -a <原文檔案> <譯文檔案>");
            return Ok(());
        }
        let original = paths[0].to_string_lossy().to_string();
        let translated = paths[1].to_string_lossy().to_string();

        println!("\x1b[1;33m🔍 審核對比模式啟動...\x1b[0m");
        let (err_count, _) = process_audit(
            &original, &translated, &temp_log, &ac_engine, &typo_map, &patterns, true, opencc_config
        ).unwrap_or((0, vec![]));

        reports.push(FileReport {
            input_name: original,
            status: if err_count == 0 { ResultStatus::Success } else { ResultStatus::Warning },
            issues_summary: vec![format!("對比完成，發現 {} 處差異", err_count)]
        });
    } else {
        let final_path = match paths.get(0) {
            Some(p) if p.exists() => p,
            _ => {
                println!("\x1b[1;31m❌ 錯誤：未指定有效的檔案路徑。\x1b[0m");
                return Ok(());
            }
        };
        let path_str = final_path.to_string_lossy().to_string();
        let out_name = final_path.with_extension("txt").to_str().unwrap().to_string();

        println!("\x1b[1;34m🎯 目標確認：\x1b[0m {}", path_str);
        println!("\x1b[1;34m📂 轉換開始...\x1b[0m");

        match run_conversion_full_view(&converter, &path_str, &out_name, &regex_rules, use_phrase_mode) {
            Ok(_) => {
                let (err_count, _) = process_audit(
                    &path_str, &out_name, &temp_log, &ac_engine, &typo_map, &patterns, false, opencc_config
                ).unwrap_or((0, vec![]));

                reports.push(FileReport {
                    input_name: path_str.clone(),
                    status: if err_count == 0 { ResultStatus::Success } else { ResultStatus::Warning },
                    issues_summary: vec![format!("共發現 {} 處差異", err_count)]
                });
            },
            Err(e) => {
                reports.push(FileReport {
                    input_name: path_str.clone(),
                    status: ResultStatus::Error,
                    issues_summary: vec![format!("失敗: {}", e)]
                });
            }
        }
    }

    // 8. 輸出總結
    print_final_summary(reports.clone(), &temp_log, &current_time);

    // 9. 發送 Discord
    if wants_broadcast {
        if let (Some(url), Some(report)) = (webhook_url, reports.get(0)) {
            println!("📡 正在嘗試發送 Discord 報告...");
            let (status_msg, color) = match report.status {
                ResultStatus::Success => ("✅ 處理成功".to_string(), 3066993),
                ResultStatus::Warning => (report.issues_summary[0].clone(), 15105570),
                ResultStatus::Error => ("❌ 處理出錯".to_string(), 15158332),
            };
            utils::send_discord_report(&url, &report.input_name, &status_msg, color);
            // 短暫等待確保發送成功
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    Ok(())
}

fn load_typo_engine(_log_path: &PathBuf) -> (AhoCorasick, HashMap<String, String>, Vec<String>, Vec<(Regex, String)>) {
    let typo_path = "/home/lee/BOok/PJct/cw/typos.json";
    let data: TypoData = fs::read_to_string(typo_path)
        .map(|s| serde_json::from_str(&s).unwrap())
        .unwrap_or_else(|_| TypoData { typos: HashMap::new(), regex_overrides: HashMap::new() });

    let patterns: Vec<String> = data.typos.keys().cloned().collect();
    let ac = AhoCorasick::new(&patterns).unwrap();
    let regex_rules = data.regex_overrides.into_iter()
        .filter_map(|(k, v)| Regex::new(&k).ok().map(|re| (re, v)))
        .collect();

    (ac, data.typos, patterns, regex_rules)
}

fn print_final_summary(reports: Vec<FileReport>, _log: &PathBuf, time: &str) {
    println!("\n\x1b[1;36m━━━━━━━━━━━━━━━━ 總結報告 ({}) ━━━━━━━━━━━━━━━━\x1b[0m", time);
    for r in reports {
        let (icon, color) = match r.status {
            ResultStatus::Success => ("✓ 合格", "32"),
            ResultStatus::Warning => ("⚠ 警告", "33"),
            ResultStatus::Error   => ("✗ 失敗", "31"),
        };
        println!(" \x1b[{}m[{}] {}\x1b[0m", color, icon, r.input_name);
    }
}

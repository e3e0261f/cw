mod report_format;
mod ui_style;
mod rules_stay_raw;
mod engine_translate;
mod checker;
mod audit;
mod setup_config;
mod mode_a_compare;
mod downloader;
mod mode_b_discord;

use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;
use chrono::Local; // 【修正】：補上 Local 引用
use report_format::{FileReport, ResultStatus};
use opencc_rust::*;
use rules_stay_raw::RawGuard;

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();

    // 1. Stdin 管道模式檢測
    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }

    if args.len() < 2 { 
        ui_style::print_help(); 
        return Ok(()); 
    }

    // 2. 解析參數
    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    
    let task_url = args.iter().position(|r| r == "--task").and_then(|i| args.get(i + 1)).cloned();
    let task_text_raw = args.iter().position(|r| r == "--text").and_then(|i| args.get(i + 1)).cloned();
    let task_text = task_text_raw.as_ref().map(|val| {
        if Path::new(val).exists() {
            fs::read_to_string(val).unwrap_or_else(|_| val.clone())
        } else {
            val.clone()
        }
    });
    let mention_id = args.iter().position(|r| r == "--id")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| config.mention_id.clone());

    let mut file_paths: Vec<String> = args.into_iter()
        .skip(1)
        .filter(|arg| {
            !arg.starts_with("-") && 
            !arg.starts_with("--") && 
            Some(arg) != task_url.as_ref() && 
            Some(arg) != task_text_raw.as_ref() &&
            arg != &mention_id
        })
        .collect();

    // --- 自動化下載環節 ---
    if let Some(ref url) = task_url {
        println!("\n\x1b[1;36m🛸 偵測到任務連結，啟動下載...\x1b[0m");
        let dl_dir = Path::new(&config.log_directory).join("cw_tasks");
        let _ = fs::create_dir_all(&dl_dir);
        if let Ok(target) = downloader::MegaDownloader::scout_target(url) {
            println!("  🎯 鎖定檔案: {}", target);
            if let Ok(local) = downloader::MegaDownloader::fetch_file(url, &target, &dl_dir) {
                file_paths.push(local.to_string_lossy().to_string());
            }
        }
    }

    if is_compare_mode {
        if file_paths.len() >= 2 {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        } else {
            ui_style::print_check_err("對比模式需要兩個檔案路徑。");
        }
    } else {
        // --- 儀表板 ---
        let mode_desc = if is_phrase_mode { "S2TWP (本土化強化)" } else { "S2T (標準對等)" };
        let discord_status = if config.discord_webhook.is_empty() { "未設定" } else { "已就緒" };

        println!("\n\x1b[1;36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m");
        println!("\x1b[1;36m┃ 🚀 CW 任務啟動 | 模式: {}┃\x1b[0m", mode_desc);
        println!("\x1b[1;36m┃ Discord : {} | 提及 ID: {}┃\x1b[0m", discord_status, if mention_id.is_empty() { "無" } else { &mention_id });
        println!("\x1b[1;36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m");
        
        let mut reports: Vec<FileReport> = Vec::new();

        for (idx, path_str) in file_paths.iter().enumerate() {
            // 直接發送模式
            if path_str.to_lowercase().ends_with(".txt") && is_discord_mode {
                println!("\n\x1b[1;33mℹ 直接發送模式: {}\x1b[0m", path_str);
                reports.push(FileReport {
                    input_name: path_str.clone(), output_name: path_str.clone(), 
                    temp_log_path: std::path::PathBuf::new(), status: ResultStatus::Success, 
                    verif_errors: vec![], translated_pairs: vec![], 
                    duration: std::time::Duration::from_secs(0),
                });
                continue;
            }

            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            
            // 【命名修復】：動態生成日誌名稱 (前綴_日期_原名.log)
            let date_str = Local::now().format(&config.log_file_date_format).to_string();
            let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
            let log_file_name = format!("{}_{}_{}.log", config.log_file_prefix, date_str, stem);
            let abs_temp_log = Path::new(&config.log_directory).join(log_file_name);
            
            let fix = checker::needs_trailing_newline_fix(path_str);
            let mut v_errs = Vec::new();
            if fix { v_errs.push("原檔不規範：末尾遺失空行。已修復。".to_string()); }

            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name, fix) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    let status = if fix { ResultStatus::VerifWarning } else { ResultStatus::Success };
                    let _ = audit::create_detailed_log(path_str, &out_name, &abs_temp_log, &status, config.log_max_size_mb, config.log_backup_count);
                    ui_style::print_check_ok(&format!("處理完成 | 日誌: {}", ui_style::format_abs_path_link(&abs_temp_log)));
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log,
                        status, verif_errors: v_errs, translated_pairs: pairs, duration: file_start.elapsed(),
                    });
                }
                Err(e) => ui_style::print_check_err(&format!("失敗: {}", e)),
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());

        if is_discord_mode && !config.discord_webhook.is_empty() && !reports.is_empty() {
            let _ = mode_b_discord::execute(
                &config.discord_webhook, 
                task_text.as_deref(), 
                &mention_id, 
                config.discord_interval, 
                &reports
            );
        }
    }
    Ok(())
}

fn run_stdin_mode(is_phrase: bool) {
    let config = if is_phrase { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let conv = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            println!("{}", engine_translate::translate_single_line(&conv, &guard, &l, ""));
        }
    }
}

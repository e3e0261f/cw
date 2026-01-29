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
use std::path::{Path, PathBuf};
use std::time::Instant;
use report_format::{FileReport, ResultStatus};
use opencc_rust::*;
use rules_stay_raw::RawGuard;

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();

    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }
    if args.len() < 2 { ui_style::print_help(); return Ok(()); }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    let task_url = args.iter().position(|r| r == "--task").and_then(|i| args.get(i + 1)).cloned();
    let task_text_raw = args.iter().position(|r| r == "--text").and_then(|i| args.get(i + 1)).cloned();
    let task_text = task_text_raw.as_ref().map(|val| {
        if Path::new(val).exists() { fs::read_to_string(val).unwrap_or_else(|_| val.clone()) } else { val.clone() }
    });
    let mention_id = args.iter().position(|r| r == "--id").and_then(|i| args.get(i + 1)).cloned().unwrap_or_else(|| config.mention_id.clone());

    let mut file_paths: Vec<String> = args.into_iter().skip(1)
        .filter(|arg| !arg.starts_with("-") && !arg.starts_with("--") && Some(arg) != task_url.as_ref() && Some(arg) != task_text_raw.as_ref() && arg != &mention_id)
        .collect();

    if let Some(ref url) = task_url {
        let dl_dir = Path::new(&config.log_directory).join("cw_tasks");
        let _ = fs::create_dir_all(&dl_dir);
        if let Ok(target) = downloader::MegaDownloader::scout_target(url) {
            if let Ok(local) = downloader::MegaDownloader::fetch_file(url, &target, &dl_dir) {
                file_paths.push(local.to_string_lossy().to_string());
            }
        }
    }

    if is_compare_mode {
        if file_paths.len() >= 2 {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        }
    } else {
        let log_dir = Path::new(&config.log_directory);
        if !log_dir.exists() { let _ = fs::create_dir_all(log_dir); }
        let abs_log_dir = fs::canonicalize(log_dir).unwrap_or_else(|_| PathBuf::from(log_dir));
        
        println!("\n\x1b[1;36m============================================================\x1b[0m");
        println!("\x1b[1;36m🚀 CW 任務啟動 | 模式: {}\x1b[0m", if is_phrase_mode { "S2TWP (本土化)" } else { "S2T (標準)" });
        println!("\x1b[1;36mDiscord : {} | 提及: {}\x1b[0m", if config.discord_webhook.is_empty() { "未設定" } else { "已就緒" }, if mention_id.is_empty() { "無" } else { &mention_id });
        println!("\x1b[1;36m日誌等級: {} | 目錄: {}\x1b[0m", config.log_level, abs_log_dir.display());
        println!("\x1b[1;36m============================================================\x1b[0m");
        
        let mut reports = Vec::new();
        for (idx, path_str) in file_paths.iter().enumerate() {
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            let log_file_name = format!("{}_{}_{}.log", config.log_file_prefix, chrono::Local::now().format(&config.log_file_date_format), Path::new(path_str).file_stem().unwrap().to_str().unwrap());
            let abs_temp_log = abs_log_dir.join(log_file_name);
            let fix = checker::needs_trailing_newline_fix(path_str);
            let mut v_errs = Vec::new();
            if fix { v_errs.push("原檔格式異常：末尾無空行。系統已自動補完。".to_string()); }

            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name, fix) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    let status = if fix { ResultStatus::VerifWarning } else { ResultStatus::Success };
                    let _ = audit::create_detailed_log(path_str, &out_name, &abs_temp_log, &status, config.log_max_size_mb, config.log_backup_count);
                    ui_style::print_check_ok(&format!("處理完成 | 日誌: {}", ui_style::format_abs_path_link(&abs_temp_log)));
                    reports.push(FileReport { input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log, status, verif_errors: v_errs, translated_pairs: pairs, duration: total_start.elapsed() });
                }
                Err(e) => ui_style::print_check_err(&format!("失敗: {}", e)),
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());
        if is_discord_mode && !config.discord_webhook.is_empty() && !reports.is_empty() {
            let _ = mode_b_discord::execute(&config.discord_webhook, task_text.as_deref(), &mention_id, config.discord_interval, &reports);
        }
    }
    Ok(())
}

fn run_stdin_mode(is_phrase: bool) {
    let config = if is_phrase { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let conv = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() { if let Ok(l) = line { println!("{}", engine_translate::translate_single_line(&conv, &guard, &l, "")); } }
}

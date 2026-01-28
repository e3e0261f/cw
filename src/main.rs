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
use std::time::{Instant, Duration}; // 【修正】：補上了 Duration 導入
use report_format::{FileReport, ResultStatus};
use opencc_rust::*;
use rules_stay_raw::RawGuard;

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();

    // 1. Stdin 管道模式
    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }

    if args.len() < 2 { 
        ui_style::print_help(); 
        return Ok(()); 
    }

    // 2. 解析參數 (使用 cloned 避免 borrow 衝突)
    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    
    let task_url = args.iter().position(|r| r == "--task").and_then(|i| args.get(i + 1)).cloned();
    let task_text = args.iter().position(|r| r == "--text").and_then(|i| args.get(i + 1)).cloned();
    let mention_id = args.iter().position(|r| r == "--id").and_then(|i| args.get(i + 1)).cloned()
                        .unwrap_or_else(|| config.mention_id.clone());

    let mut file_paths: Vec<String> = args.into_iter()
        .skip(1)
        .filter(|arg| !arg.starts_with("-") && !arg.starts_with("--") && 
                Some(arg) != task_url.as_ref() && Some(arg) != task_text.as_ref())
        .collect();

    // --- 自動化下載任務 ---
    if let Some(ref url) = task_url {
        println!("\n\x1b[1;36m🛸 偵測到任務連結，啟動下載...\x1b[0m");
        let dl_dir = Path::new(&config.log_directory).join("cw_tasks");
        let _ = fs::create_dir_all(&dl_dir);
        match downloader::MegaDownloader::scout_target(url) {
            Ok(target) => {
                println!("  🎯 鎖定檔案: {}", target);
                if let Ok(local) = downloader::MegaDownloader::fetch_file(url, &target, &dl_dir) {
                    file_paths.push(local.to_string_lossy().to_string());
                }
            }
            Err(e) => ui_style::print_check_err(&format!("下載失敗: {}", e)),
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
        let mode_desc = if is_phrase_mode { "S2TWP (本土化強化)" } else { "S2T (標準模式)" };
        let discord_status = if config.discord_webhook.is_empty() { "未設定" } else { "已就緒" };

        println!("\n\x1b[1;36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m");
        println!("\x1b[1;36m┃ 🚀 CW 任務啟動 | 模式: {}┃\x1b[0m", mode_desc);
        println!("\x1b[1;36m┃ Discord : {} | 等級: {}┃\x1b[0m", discord_status, config.log_level);
        println!("\x1b[1;36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m");
        
        let mut reports: Vec<FileReport> = Vec::new();

        for (idx, path_str) in file_paths.iter().enumerate() {
            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
            let log_file_name = format!("{}_{}.log", config.log_file_prefix, stem);
            let abs_temp_log = Path::new(&config.log_directory).join(log_file_name);
            
            let fix_needed = checker::needs_trailing_newline_fix(path_str);
            let mut v_errs = Vec::new();
            if fix_needed { v_errs.push("原檔不規範：末尾遺失空行。已自動修復。".to_string()); }

            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name, fix_needed) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    if fix_needed { println!("  \x1b[1;33m⚠️  提醒：原檔結尾無空行，已自動校正。\x1b[0m"); }

                    let status = if fix_needed { ResultStatus::VerifWarning } else { ResultStatus::Success };
                    let _ = audit::create_detailed_log(path_str, &out_name, &abs_temp_log, &status, config.log_max_size_mb, config.log_backup_count);
                    
                    let log_link = ui_style::format_abs_path_link(&abs_temp_log);
                    ui_style::print_check_ok(&format!("完成 ({:?}) | 日誌: {}", file_start.elapsed(), log_link));
                    
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log,
                        status, verif_errors: v_errs, translated_pairs: pairs, duration: file_start.elapsed(),
                    });
                }
                Err(e) => {
                    ui_style::print_check_err(&format!("失敗: {}", e));
                    // 激活使用 ConvertError 狀態
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: "N/A".to_string(),
                        temp_log_path: std::path::PathBuf::new(),
                        status: ResultStatus::ConvertError,
                        verif_errors: vec![e.to_string()],
                        translated_pairs: vec![],
                        duration: Duration::from_secs(0), // 這裡現在能正確辨識 Duration 了
                    });
                }
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());

        if is_discord_mode && !config.discord_webhook.is_empty() && !reports.is_empty() {
            println!("\n\x1b[1;33m📡 正在發送成果至 Discord...\x1b[0m");
            match mode_b_discord::execute(&config.discord_webhook, task_text.as_deref(), &mention_id, &reports) {
                Ok(_) => println!("\x1b[1;32m ✔ 成功：已送達 Discord 頻道。\x1b[0m"),
                Err(e) => ui_style::print_check_err(&format!("Discord 發送失敗: {}", e)),
            }
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

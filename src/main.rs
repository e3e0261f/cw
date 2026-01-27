mod report_format;
mod ui_style;
mod rules_stay_raw;
mod engine_translate;
mod checker;
mod setup_config;
mod mode_a_compare;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use report_format::{FileReport, ResultStatus};

fn main() -> std::io::Result<()> {
    // 1. 載入配置
    let config = setup_config::Config::load(); 
    
    // 2. 獲取命令行參數
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { 
        ui_style::print_help(); 
        return Ok(()); 
    }

    // 3. 核心決策邏輯
    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    
    // 【實質使用 is_discord_mode】：決定 Discord 的狀態顯示
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    
    let file_paths: Vec<String> = args.into_iter()
        .skip(1)
        .filter(|arg| arg != "-p" && arg != "-a" && arg != "-b")
        .collect();

    if is_compare_mode {
        if file_paths.len() >= 2 {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        } else {
            ui_style::print_check_err("對比模式需要兩個檔案路徑。");
        }
    } else {
        // --- 處理日誌目錄的絕對路徑 ---
        let log_dir = Path::new(&config.log_directory);
        if !log_dir.exists() { let _ = fs::create_dir_all(log_dir); }
        let abs_log_dir = fs::canonicalize(log_dir).unwrap_or_else(|_| PathBuf::from(log_dir));

        // --- 儀表板顯示 (解決警告) ---
        let mode_desc = if is_phrase_mode { "S2TWP (本土化強化)" } else { "S2T (標準對等)" };
        
        // 使用 is_discord_mode 決定狀態文字
        let discord_status = if config.discord_webhook.is_empty() { 
            "\x1b[1;31m未設定\x1b[0m" 
        } else if is_discord_mode { 
            "\x1b[1;32m已就緒 (自動發送)\x1b[0m" 
        } else { 
            "\x1b[1;32m已就緒 (手動)\x1b[0m" 
        };

        println!("\n\x1b[1;36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m");
        println!("\x1b[1;36m┃ 🚀 CW 任務啟動 | 模式: {}┃\x1b[0m", mode_desc);
        println!("\x1b[1;36m┃ Discord : {}┃\x1b[0m", discord_status);
        // 【實質使用 log_level】：顯示在介面上
        println!("\x1b[1;36m┃ 日誌等級: {} | 目錄: {}\x1b[0m", config.log_level, abs_log_dir.display());
        println!("\x1b[1;36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m");
        
        let mut reports: Vec<FileReport> = Vec::new();

        for (idx, path_str) in file_paths.iter().enumerate() {
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
            let log_file_name = format!("{}_{}.log", config.log_file_prefix, stem);
            let abs_temp_log = abs_log_dir.join(log_file_name);
            
            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    let errors = checker::check_integrity(&out_name);
                    
                    let log_hint = ui_style::format_abs_path_link(&abs_temp_log);
                    ui_style::print_check_ok(&format!("處理完成 | 日誌: {}", log_hint));
                    
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log,
                        status: ResultStatus::Success, verif_errors: errors, translated_pairs: pairs,
                    });
                }
                Err(e) => ui_style::print_check_err(&format!("失敗: {}", e)),
            }
        }
        ui_style::print_summary(&reports);

        // 最後再次確認是否執行 Discord 發送 (未來邏輯接入口)
        if is_discord_mode && !config.discord_webhook.is_empty() {
            println!("\n\x1b[1;33m📡 Discord 發送模組待命中 (準備彙整資料...)\x1b[0m");
        }
    }
    Ok(())
}

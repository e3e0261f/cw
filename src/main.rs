mod report_format;
mod ui_style;
mod rules_stay_raw;
mod engine_translate;
mod checker;
mod audit;
mod setup_config;
mod mode_a_compare;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use report_format::{FileReport, ResultStatus};

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 { 
        ui_style::print_help(); 
        return Ok(()); 
    }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
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
        let log_dir = Path::new(&config.log_directory);
        if !log_dir.exists() { let _ = fs::create_dir_all(log_dir); }
        let abs_log_dir = fs::canonicalize(log_dir).unwrap_or_else(|_| PathBuf::from(log_dir));

        let mode_desc = if is_phrase_mode { "S2TWP (本土化強化)" } else { "S2T (標準對等)" };
        let discord_status = if config.discord_webhook.is_empty() { "未設定" } else { "已就緒" };

        // 使用簡約的分割線
        println!("\n\x1b[1;36m============================================================\x1b[0m");
        println!("\x1b[1;36m🚀 CW 任務啟動 | 模式: {}\x1b[0m", mode_desc);
        println!("\x1b[1;36mDiscord : {} | 等級: {}\x1b[0m", discord_status, config.log_level);
        println!("\x1b[1;36m日誌目錄: {}\x1b[0m", abs_log_dir.display());
        println!("\x1b[1;36m============================================================\x1b[0m");
        
        let mut reports: Vec<FileReport> = Vec::new();

        for (idx, path_str) in file_paths.iter().enumerate() {
            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            
            let out_name = format!("{}.txt", path_str);
            let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
            let log_file_name = format!("{}_{}.log", config.log_file_prefix, stem);
            let abs_temp_log = std::path::Path::new(&config.log_directory).join(log_file_name);
            
            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    
                    let errors = checker::check_integrity(&out_name);
                    let status = if errors.is_empty() { ResultStatus::Success } else { ResultStatus::VerifWarning };
                    
                    // 生成日誌 (追加模式)
                    let _ = audit::create_detailed_log(
                        path_str, &out_name, &abs_temp_log, &status, 
                        config.log_max_size_mb, config.log_backup_count
                    );
                    
                    let duration = file_start.elapsed();
                    let log_hint = ui_style::format_abs_path_link(&abs_temp_log);
                    ui_style::print_check_ok(&format!("處理完成 ({:?}) | 日誌: {}", duration, log_hint));
                    
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log,
                        status, verif_errors: errors, translated_pairs: pairs, duration,
                    });
                }
                Err(e) => {
                    // 【核心修正】：實質使用 ConvertError 欄位
                    ui_style::print_check_err(&format!("失敗: {}", e));
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: "N/A".to_string(),
                        temp_log_path: std::path::PathBuf::new(),
                        status: ResultStatus::ConvertError, // 這裡正式激活了！
                        verif_errors: vec![e.to_string()],
                        translated_pairs: vec![],
                        duration: std::time::Duration::from_secs(0),
                    });
                }
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());

        // 雖然發送模組還沒寫，但我們使用了 is_discord_mode 消除警告
        if is_discord_mode && !config.discord_webhook.is_empty() {
            println!("\n\x1b[1;33m📡 Discord 發送模組準備中 (共 {} 份報告)...\x1b[0m", reports.len());
        }
    }
    Ok(())
}

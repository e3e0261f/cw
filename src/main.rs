mod report_format;
mod ui_style;
mod rules_stay_raw;
mod engine_translate;
mod checker;
mod setup_config;
mod mode_a_compare;

use std::env;
use std::path::Path;
use report_format::{FileReport, ResultStatus};

fn main() -> std::io::Result<()> {
    // 1. 載入配置（消除 unused 警告，並顯示狀態）
    let config = setup_config::Config::load(); 
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { ui_style::print_help(); return Ok(()); }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p");
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let file_paths: Vec<String> = args.into_iter().skip(1)
        .filter(|arg| arg != "-p" && arg != "-a" && arg != "-b").collect();

    if is_compare_mode {
        if file_paths.len() >= 2 {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        }
    } else {
        // 在啟動時確認 config 讀取
        let discord_status = if config.discord_webhook.is_empty() { "未設定" } else { "已就緒" };
        println!("\n\x1b[1;36m🚀 翻譯任務啟動 | Discord: {}\x1b[0m", discord_status);
        
        let mut reports = Vec::new();
        for (idx, path_str) in file_paths.iter().enumerate() {
            println!("\x1b[1;35m➔ 檔案 [{}/{}] : {}\x1b[0m", idx + 1, file_paths.len(), path_str);
            let out_name = Path::new(path_str).with_extension("txt").to_str().unwrap().to_string();
            let stem = Path::new(path_str).file_stem().unwrap().to_str().unwrap();
            let temp_log = env::temp_dir().join(format!("cntw_{}.log", stem));
            
            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name) {
                Ok(pairs) => {
                    ui_style::print_translated_preview(&pairs);
                    
                    // 【關鍵】啟用完整性檢查，確保不會閹割功能
                    let errors = checker::check_integrity(&out_name);
                    ui_style::print_check_ok("處理完成");
                    
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: out_name,
                        temp_log_path: temp_log,
                        status: ResultStatus::Success,
                        verif_errors: errors,
                        translated_pairs: pairs,
                    });
                }
                Err(e) => println!("  \x1b[1;31m✘ 失敗: {}\x1b[0m", e),
            }
        }
        ui_style::print_summary(&reports);
    }
    Ok(())
}

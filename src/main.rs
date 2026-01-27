mod report_format;
mod ui_style;
mod rules_stay_raw;
mod engine_translate;
mod checker;
mod audit;
mod setup_config;
mod mode_a_compare;
mod downloader;

use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::Instant;
use report_format::{FileReport, ResultStatus};
use opencc_rust::*;
use rules_stay_raw::RawGuard;

fn main() -> io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();

    // 1. 解析參數
    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    
    let task_url = args.iter().position(|r| r == "--task").and_then(|i| args.get(i + 1));
    let task_text = args.iter().position(|r| r == "--text").and_then(|i| args.get(i + 1));
    // --- 新增：自定義 ID 參數 ---
    let mention_id = args.iter().position(|r| r == "--id").and_then(|i| args.get(i + 1))
                        .map(|s| s.as_str()).unwrap_or("845089536765853766");

    // 2. 判斷是否為 Stdin 管道輸入 (例如: echo "..." | cw)
    // 如果沒有傳入任何檔案路徑，且 stdin 不是終端機
    if args.len() == 1 || (args.len() == 2 && is_phrase_mode) {
        if atty::isnt(atty::Stream::Stdin) {
            run_stdin_mode(is_phrase_mode);
            return Ok(());
        }
    }

    if args.len() < 2 { ui_style::print_help(); return Ok(()); }

    let mut file_paths: Vec<String> = args.into_iter()
        .skip(1)
        .filter(|arg| !arg.starts_with("-") && arg != task_url.unwrap_or(&"".to_string()) && arg != task_text.unwrap_or(&"".to_string()) && arg != mention_id)
        .collect();

    // --- 自動化下載環節 ---
    if let Some(url) = task_url {
        println!("\n\x1b[1;36m🛸 任務連結已就緒，開始下載...\x1b[0m");
        let dl_dir = Path::new(&config.log_directory).join("cw_tasks");
        let _ = fs::create_dir_all(&dl_dir);
        match downloader::MegaDownloader::scout_target(url) {
            Ok(target) => {
                if let Ok(local) = downloader::MegaDownloader::fetch_file(url, &target, &dl_dir) {
                    file_paths.push(local.to_str().unwrap().to_string());
                }
            }
            Err(e) => ui_style::print_check_err(&format!("下載失敗：{}", e)),
        }
    }

    if is_compare_mode {
        // --- 對比模式 ---
        if file_paths.len() >= 2 {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        }
    } else {
        // --- 翻譯模式 ---
        println!("\n\x1b[1;36m🚀 任務啟動 | 模式: {} | Discord ID: {}\x1b[0m", 
                 if is_phrase_mode { "S2TWP" } else { "S2T" }, mention_id);
        
        let mut reports = Vec::new();
        for (idx, path_str) in file_paths.iter().enumerate() {
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
            let abs_temp_log = Path::new(&config.log_directory).join(format!("{}_{}.log", config.log_file_prefix, stem));
            
            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs); }
                    let status = ResultStatus::Success;
                    let _ = audit::create_detailed_log(path_str, &out_name, &abs_temp_log, &status);
                    ui_style::print_check_ok(&format!("完成 ({:?})", Instant::now().duration_since(total_start)));
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: out_name, temp_log_path: abs_temp_log,
                        status, verif_errors: vec![], translated_pairs: pairs, duration: total_start.elapsed(),
                    });
                }
                Err(e) => ui_style::print_check_err(&format!("失敗: {}", e)),
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());

        if is_discord_mode && !config.discord_webhook.is_empty() {
            println!("\x1b[1;33m📡 準備推送至 Discord... 提及 ID: <@{}>\x1b[0m", mention_id);
        }
    }
    Ok(())
}

/// 【新功能】管道模式：直接翻譯 stdin 並輸出
fn run_stdin_mode(is_phrase: bool) {
    let config = if is_phrase { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let conv = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();
    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            // 直接以普通文本模式翻譯並噴出
            let out = engine_translate::translate_single_line(&conv, &guard, &l, "");
            println!("{}", out);
        }
    }
}

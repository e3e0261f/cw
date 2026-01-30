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
use std::time::{Instant, Duration};
use report_format::{FileReport, ResultStatus};
use opencc_rust::{OpenCC, DefaultConfig};
use rules_stay_raw::RawGuard;

fn main() -> io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();

    // 系統初始化指令
    if args.iter().any(|arg| arg == "--init") {
        return setup_config::Config::generate_default();
    }

    // Stdin 模式
    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }
    if args.len() < 2 { ui_style::print_help(); return Ok(()); }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    let is_inplace = args.iter().any(|arg| arg == "-d");
    
    let task_url = args.iter().position(|r| r == "--task").and_then(|i| args.get(i + 1)).cloned();
    let task_text_raw = args.iter().position(|r| r == "--text").and_then(|i| args.get(i + 1)).cloned();
    let task_text = task_text_raw.as_ref().map(|val| {
        if Path::new(val).exists() { fs::read_to_string(val).unwrap_or_else(|_| val.clone()) } else { val.clone() }
    });
    let mention_id = args.iter().position(|r| r == "--id").and_then(|i| args.get(i + 1)).cloned().unwrap_or_else(|| config.mention_id.clone());

    let mut file_paths: Vec<String> = args.into_iter()
        .skip(1)
        .filter(|arg| !arg.starts_with("-") && !arg.starts_with("--") && Some(arg) != task_url.as_ref() && Some(arg) != task_text_raw.as_ref())
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
        if file_paths.len() >= 2 { mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]); }
    } else {
        println!("\n\x1b[1;36m🚀 CW 1.9.1 啟動 | 模式: {} | 日誌: {}\x1b[0m", 
                 if is_phrase_mode { "S2TWP" } else { "S2T" }, config.log_level);
        
        let mut reports = Vec::new();
        for (idx, path_str) in file_paths.iter().enumerate() {
            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            
            let temp_out = format!("{}.tmp", path_str);
            let fix = checker::needs_trailing_newline_fix(path_str);
            let issues = checker::diagnose_file(path_str, config.translate_error);

            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &temp_out, fix) {
                Ok(pairs) => {
                    if config.verbosity >= 1 { ui_style::print_translated_preview(&pairs, &issues); }
                    ui_style::print_footnotes(&issues);

                    let out_name = if is_inplace { 
                        fs::rename(&temp_out, path_str)?; path_str.to_string() 
                    } else { 
                        let final_name = format!("{}.txt", path_str);
                        let _ = fs::rename(&temp_out, &final_name); final_name
                    };

                    let stem = Path::new(path_str).file_stem().unwrap_or_default().to_str().unwrap_or("log");
                    let abs_temp_log = Path::new(&config.log_directory).join(format!("{}_{}.log", config.log_file_prefix, stem));
                    let status = if issues.is_empty() { ResultStatus::Success } else { ResultStatus::VerifWarning };
                    let _ = audit::create_detailed_log_with_issues(path_str, &out_name, &abs_temp_log, &status, config.log_max_size_mb, config.log_backup_count, &issues.iter().map(|i| i.message.clone()).collect::<Vec<_>>());

                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: if is_inplace { "(已覆蓋)".to_string() } else { out_name },
                        temp_log_path: abs_temp_log, status, issues, translated_pairs: pairs, duration: file_start.elapsed()
                    });
                }
                Err(e) => {
                    ui_style::print_check_err(&format!("失敗: {}", e));
                    reports.push(FileReport {
                        input_name: path_str.clone(), output_name: "N/A".to_string(), 
                        temp_log_path: std::path::PathBuf::new(), status: ResultStatus::ConvertError, 
                        issues: vec![], translated_pairs: vec![], duration: Duration::from_secs(0)
                    });
                }
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());
        if is_discord_mode && !config.discord_webhook.is_empty() {
            let _ = mode_b_discord::execute(&config.discord_webhook, task_text.as_deref(), &mention_id, config.discord_interval, config.show_stats, config.discord_show_errors, &reports);
        }
    }
    Ok(())
}

fn run_stdin_mode(is_phrase: bool) {
    let config = if is_phrase { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let conv = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines().map_while(Result::ok) {
        println!("{}", engine_translate::translate_single_line(&conv, &guard, &line, ""));
    }
}

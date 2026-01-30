mod mode_a_compare;
mod mode_b_discord;
mod ui_style;

use cw::core;
use cw::report_format::{FileReport, ResultStatus};
use std::env;
use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = core::Config::load();
    let args: Vec<String> = env::args().collect();

    // 系統初始化
    if args.iter().any(|arg| arg == "--init") {
        return core::Config::generate_default();
    }

    // 管道模式檢測
    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }

    if args.len() < 2 {
        ui_style::print_help();
        return Ok(());
    }

    let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_a = args.iter().any(|arg| arg == "-a");
    let is_b = args.iter().any(|arg| arg == "-b") || config.auto_discord;
    let is_d = args.iter().any(|arg| arg == "-d");

    let task_url = args
        .iter()
        .position(|r| r == "--task")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let mut paths: Vec<String> = args
        .into_iter()
        .skip(1)
        .filter(|a| !a.starts_with("-"))
        .collect();

    // 直接使用 core 內部的下載器
    if let Some(ref url) = task_url {
        let dl_dir = Path::new(&config.log_directory).join("cw_tasks");
        let _ = fs::create_dir_all(&dl_dir);
        if let Ok(target) = core::MegaDownloader::scout_target(url) {
            if let Ok(local) = core::MegaDownloader::fetch_file(url, &target, &dl_dir) {
                paths.push(local.to_string_lossy().to_string());
            }
        }
    }

    if is_a {
        if paths.len() >= 2 {
            mode_a_compare::run_detailed_compare(is_p, &paths[0], &paths[1]);
        }
    } else {
        println!(
            "\n\x1b[1;36m🚀 CW 1.9.3 | 模式: {} | 日誌等級: {}\x1b[0m",
            if is_p { "S2TWP" } else { "S2T" },
            config.log_level
        );
        let mut reports = Vec::new();
        for (idx, path_str) in paths.iter().enumerate() {
            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, paths.len(), path_str);
            let fix = core::needs_trailing_newline_fix(path_str);
            let issues = core::diagnose_file(path_str, config.translate_error);

            match core::run_safe_translate(is_p, path_str, &format!("{}.tmp", path_str), fix) {
                Ok(pairs) => {
                    ui_style::print_translated_preview(&pairs, config.full_preview, &issues);
                    let out_name = if is_d {
                        fs::rename(format!("{}.tmp", path_str), path_str)?;
                        path_str.to_string()
                    } else {
                        let n = format!("{}.txt", path_str);
                        let _ = fs::rename(format!("{}.tmp", path_str), &n);
                        n
                    };
                    let log_p = Path::new(&config.log_directory)
                        .join(format!("{}.log", config.log_file_prefix));
                    let _ = core::create_log(
                        path_str,
                        &out_name,
                        &log_p,
                        &ResultStatus::Success,
                        config.log_max_size_mb,
                        config.log_backup_count,
                        &issues,
                    );
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: out_name,
                        temp_log_path: log_p,
                        status: ResultStatus::Success,
                        issues,
                        translated_pairs: pairs,
                        duration: file_start.elapsed(),
                    });
                    ui_style::print_check_ok("處理完成");
                }
                Err(e) => ui_style::print_check_err(&format!("失敗: {}", e)),
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());
        if is_b && !config.discord_webhook.is_empty() {
            let _ = mode_b_discord::execute(
                &config.discord_webhook,
                None,
                &config.mention_id,
                config.discord_interval,
                config.show_stats,
                config.discord_show_errors,
                &reports,
            );
        }
    }
    Ok(())
}

fn run_stdin_mode(is_phrase: bool) {
    let config = if is_phrase {
        opencc_rust::DefaultConfig::S2TWP
    } else {
        opencc_rust::DefaultConfig::S2T
    };
    let conv = opencc_rust::OpenCC::new(config).unwrap();
    let guard = core::RawGuard::new();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines().map_while(Result::ok) {
        println!("{}", core::translate_single_line(&conv, &guard, &line, ""));
    }
}

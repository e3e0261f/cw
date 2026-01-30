mod audit;
mod checker;
mod downloader;
mod engine_translate;
mod mode_a_compare;
mod mode_b_discord;
mod report_format;
mod rules_stay_raw;
mod setup_config;
mod ui_style;

use chrono::Local;
use opencc_rust::*;
use report_format::{FileReport, ResultStatus};
use rules_stay_raw::RawGuard;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let config = setup_config::Config::load();
    let args: Vec<String> = env::args().collect();

    if !atty::is(atty::Stream::Stdin) {
        let is_p = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
        run_stdin_mode(is_p);
        return Ok(());
    }
    if args.len() < 2 {
        ui_style::print_help();
        return Ok(());
    }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p") || config.phrase_mode;
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let is_discord_mode = args.iter().any(|arg| arg == "-b") || config.auto_discord;

    let task_url = args
        .iter()
        .position(|r| r == "--task")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let task_text_raw = args
        .iter()
        .position(|r| r == "--text")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let task_text = task_text_raw.as_ref().map(|val| {
        if Path::new(val).exists() {
            fs::read_to_string(val).unwrap_or_else(|_| val.clone())
        } else {
            val.clone()
        }
    });
    let mention_id = args
        .iter()
        .position(|r| r == "--id")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| config.mention_id.clone());

    let mut file_paths: Vec<String> = args
        .into_iter()
        .skip(1)
        .filter(|arg| {
            !arg.starts_with("-")
                && !arg.starts_with("--")
                && Some(arg) != task_url.as_ref()
                && Some(arg) != task_text_raw.as_ref()
        })
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

    println!(
        "\n全量预览: {}",
        if config.full_preview {
            "开启"
        } else {
            "关闭"
        } // 增加这一行进行确认
    );

    if is_compare_mode {
        if file_paths.len() >= 2 {
            // ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        }
    } else {
        let discord_status = if config.discord_webhook.is_empty() {
            "未設定"
        } else if is_discord_mode {
            "已就緒 (自動發送)"
        } else {
            "已就緒"
        };
        // println!("\n\x1b[1;36m============================================================\x1b[0m");
        println!("翻译模式: {}", if is_phrase_mode { "TW2SP" } else { "T2S" });
        println!(
            "Discord : {}  \n日志等級: {}",
            discord_status, config.log_level
        );
        println!("------------------------------------------------------------");

        let mut reports = Vec::new();
        for (idx, path_str) in file_paths.iter().enumerate() {
            let file_start = Instant::now();
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = format!("{}.txt", path_str);
            let date_str = Local::now()
                .format(&config.log_file_date_format)
                .to_string();
            let stem = Path::new(path_str)
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("log");
            let log_file_name = format!("{}_{}_{}.log", config.log_file_prefix, date_str, stem);
            let abs_temp_log = Path::new(&config.log_directory).join(log_file_name);

            let original_issues = checker::diagnose_file(path_str, config.translate_error);
            let fix = checker::needs_trailing_newline_fix(path_str);

            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name, fix) {
                Ok(pairs) => {
                    if config.verbosity >= 1 {
                        // 同步傳入 original_issues
                        ui_style::print_translated_preview(
                            &pairs,
                            config.full_preview,
                            &original_issues,
                        );
                    }

                    let status = if fix || !original_issues.is_empty() {
                        ResultStatus::VerifWarning
                    } else {
                        ResultStatus::Success
                    };
                    // ... 后面逻辑保持不变 ...

                    let _ = audit::create_detailed_log_with_issues(
                        path_str,
                        &out_name,
                        &abs_temp_log,
                        &status,
                        config.log_max_size_mb,
                        config.log_backup_count,
                        &original_issues,
                    );

                    // 只有在原檔真的缺少空行時才印黃色提醒
                    if fix {
                        println!("  \x1b[1;33m⚠️  提醒：原檔結尾缺少空行，系統已為成品檔補全。 (已修復)\x1b[0m");
                    }

                    let log_link = ui_style::format_abs_path_link(&abs_temp_log);
                    ui_style::print_check_ok(&format!("處理完成 | 日誌: {}", log_link));

                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: out_name,
                        temp_log_path: abs_temp_log,
                        status,
                        verif_errors: vec![],
                        original_issues,
                        translated_pairs: pairs,
                        duration: file_start.elapsed(),
                    });
                }
                Err(e) => {
                    ui_style::print_check_err(&format!("失敗: {}", e));
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: "N/A".to_string(),
                        temp_log_path: std::path::PathBuf::new(),
                        status: ResultStatus::ConvertError,
                        verif_errors: vec![e.to_string()],
                        original_issues: vec![],
                        translated_pairs: vec![],
                        duration: Duration::from_secs(0),
                    });
                }
            }
        }
        ui_style::print_summary(&reports, total_start.elapsed());
        if is_discord_mode && !config.discord_webhook.is_empty() && !reports.is_empty() {
            let _ = mode_b_discord::execute(
                &config.discord_webhook,
                task_text.as_deref(),
                &mention_id,
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
        DefaultConfig::S2TWP
    } else {
        DefaultConfig::S2T
    };
    let conv = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();
    let stdin = io::stdin();
    for l in stdin.lock().lines().map_while(Result::ok) {
        println!(
            "{}",
            engine_translate::translate_single_line(&conv, &guard, &l, "")
        );
    }
}

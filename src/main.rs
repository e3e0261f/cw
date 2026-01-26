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
    let _config = setup_config::Config::load(); 
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { ui_style::print_help(); return Ok(()); }

    let is_phrase_mode = args.iter().any(|arg| arg == "-p");
    let is_compare_mode = args.iter().any(|arg| arg == "-a");
    let file_paths: Vec<String> = args.into_iter().skip(1)
        .filter(|arg| arg != "-p" && arg != "-a" && arg != "-b").collect();

    if is_compare_mode {
        if file_paths.len() < 2 {
            println!("\x1b[1;31m❌ 錯誤：對比模式需要兩個檔案路徑。\x1b[0m");
        } else {
            ui_style::print_compare_header(&file_paths[0], &file_paths[1]);
            mode_a_compare::run_detailed_compare(is_phrase_mode, &file_paths[0], &file_paths[1]);
        }
    } else {
        println!("\n\x1b[1;36m🚀 翻譯任務啟動...\x1b[0m");
        let mut reports = Vec::new();

        for (idx, path_str) in file_paths.iter().enumerate() {
            ui_style::print_file_header(idx + 1, file_paths.len(), path_str);
            let out_name = Path::new(path_str).with_extension("txt").to_str().unwrap().to_string();
            let stem = Path::new(path_str).file_stem().unwrap().to_str().unwrap();
            let temp_log = env::temp_dir().join(format!("cntw_{}.log", stem));
            
            match engine_translate::run_safe_translate(is_phrase_mode, path_str, &out_name) {
                Ok(pairs) => {
                    ui_style::print_translated_preview(&pairs);
                    let errors = checker::check_integrity(&out_name);
                    
                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: out_name,
                        temp_log_path: temp_log,
                        status: ResultStatus::Success,
                        verif_errors: errors,
                        translated_pairs: pairs,
                    });
                }
                Err(e) => {
                    println!("  \x1b[1;31m✘ 處理失敗: {}\x1b[0m", e);
                }
            }
        }
        ui_style::print_summary(&reports);
    }
    Ok(())
}

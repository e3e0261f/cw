mod models;
mod audit;

use opencc_rust::*;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use models::{TypoData, FileReport, ResultStatus};
use audit::*;

fn main() -> io::Result<()> {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let is_compare_mode = raw_args.iter().any(|arg| arg == "-a");
    let file_paths: Vec<String> = raw_args.into_iter().filter(|arg| arg != "-a").collect();

    if file_paths.is_empty() {
        println!("用法: cw *.srt 或 cw -a A.srt B.srt");
        return Ok(());
    }

    // 初始化引擎
    let (ac, typo_map, patterns, regex_rules) = load_typo_engine();

    if is_compare_mode {
        if file_paths.len() != 2 {
            println!("錯誤: 對比模式需要兩個檔案路徑。");
            return Ok(());
        }
        run_comparison_live(&file_paths[0], &file_paths[1], &ac, &typo_map, &patterns, &regex_rules)?;
    } else {
        let total = file_paths.len();
        println!("\n\x1b[1;36m🚀 啟動批次任務：共處理 {} 個檔案\x1b[0m", total);
        
        let mut reports = Vec::new();
        let converter = OpenCC::new(DefaultConfig::S2TWP).expect("OpenCC 啟動失敗");

        for (i, path_str) in file_paths.iter().enumerate() {
            let path = Path::new(&path_str);
            if path.is_dir() { continue; }

            let out_name = path.with_extension("txt").to_str().unwrap().to_string();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let temp_log = env::temp_dir().join(format!("cntw_{}.log", stem));

            println!("\n\x1b[1;35m➔ 檔案 [{}/{}] : {}\x1b[0m", i + 1, total, path_str);
            println!("  \x1b[1;34m[1/3] 正在執行簡繁翻譯...\x1b[0m");

            match run_conversion(&converter, path_str, &out_name) {
                Ok(_) => {
                    println!("  \x1b[1;34m[2/3] 正在執行內容稽核...\x1b[0m");
                    let (v_errs, advices) = process_audit(path_str, &out_name, &temp_log, &ac, &typo_map, &patterns, &regex_rules)?;
                    
                    let status = if v_errs.is_empty() { ResultStatus::Success } else { ResultStatus::VerifWarning };
                    
                    if v_errs.is_empty() {
                        println!("  \x1b[1;32m ✔ 轉換與格式校驗通過\x1b[0m");
                    } else {
                        println!("  \x1b[1;31m ✘ 格式發現 {} 處錯誤\x1b[0m", v_errs.len());
                    }

                    reports.push(FileReport {
                        input_name: path_str.clone(),
                        output_name: out_name,
                        temp_log_path: temp_log,
                        status,
                        verif_errors: v_errs,
                        quality_advices: advices,
                    });
                }
                Err(_) => {
                    println!("  \x1b[1;31m ✘ 讀寫失敗\x1b[0m");
                }
            }
        }
        print_final_summary(reports);
    }
    Ok(())
}

fn load_typo_engine() -> (AhoCorasick, HashMap<String, String>, Vec<String>, Vec<(Regex, String)>) {
    let mut json_path = env::current_exe().expect("無法獲取路徑");
    json_path.pop();
    json_path.push("typos.json");

    let default_json = r#"{"typos": {"比列": "比例"}, "regex": {}}"#;
    let data: TypoData = fs::read_to_string(&json_path)
        .map(|s| serde_json::from_str(&s).unwrap())
        .unwrap_or_else(|_| serde_json::from_str(default_json).unwrap());

    let patterns: Vec<String> = data.typos.keys().cloned().collect();
    let ac = AhoCorasick::new(&patterns).unwrap();

    let mut regex_rules = Vec::new();
    for (re_str, tip) in &data.regex {
        if let Ok(re) = Regex::new(re_str) {
            regex_rules.push((re, tip.clone()));
        }
    }

    (ac, data.typos, patterns, regex_rules)
}

fn run_conversion(converter: &OpenCC, input: &str, output: &str) -> io::Result<()> {
    let reader = BufReader::new(File::open(input)?);
    let mut writer = File::create(output)?;
    for line in reader.lines() {
        let l = line?;
        if is_srt_structure(&l) { writeln!(writer, "{}", l)?; }
        else { writeln!(writer, "{}", converter.convert(&l))?; }
    }
    Ok(())
}

fn run_comparison_live(path_a: &str, path_b: &str, ac: &AhoCorasick, typo_map: &HashMap<String, String>, patterns: &[String], regex_rules: &[(Regex, String)]) -> io::Result<()> {
    let temp_log = env::temp_dir().join("manual_compare.log");
    let (v_errs, advices) = process_audit(path_a, path_b, &temp_log, ac, typo_map, patterns, regex_rules)?;
    for e in v_errs { println!("\x1b[1;31m  ❌ 結構錯誤: {}\x1b[0m", e); }
    for a in advices { println!("\x1b[1;34m  💡 內容稽核: {}\x1b[0m", a); }
    Ok(())
}

fn print_final_summary(reports: Vec<FileReport>) {
    let (mut s, mut w, mut f) = (0, 0, 0);
    for r in &reports {
        match r.status {
            ResultStatus::Success => s += 1,
            ResultStatus::VerifWarning => w += 1,
            ResultStatus::ConvertError => f += 1,
        }
    }

    let line = "=".repeat(60);
    println!("\n\x1b[1;36m{}\x1b[0m", line);
    println!("\x1b[1;36m📋 詳細處理清單\x1b[0m");
    println!("\x1b[1;36m{}\x1b[0m", line);

    for r in &reports {
        match r.status {
            ResultStatus::Success => {
                println!("\x1b[1;32m[OK]\x1b[0m {} -> {}", r.input_name, r.output_name);
                if !r.quality_advices.is_empty() {
                    println!("     \x1b[1;34m└─ 內容稽核 ({} 條提示):\x1b[0m", r.quality_advices.len());
                    for adv in r.quality_advices.iter().take(5) { println!("        • {}", adv); }
                }
                println!("     └─ 詳細日誌: {}", r.temp_log_path.display());
            }
            ResultStatus::VerifWarning => {
                println!("\x1b[1;33m[⚠]\x1b[0m {}", r.input_name);
                if !r.verif_errors.is_empty() {
                    println!("     \x1b[1;31m└─ 格式錯誤: {:?}\x1b[0m", r.verif_errors);
                }
                if !r.quality_advices.is_empty() {
                    println!("     \x1b[1;34m└─ 內容稽核 ({} 條提示):\x1b[0m", r.quality_advices.len());
                    for adv in r.quality_advices.iter() { println!("        • {}", adv); }
                }
                println!("     └─ 詳細日誌: {}", r.temp_log_path.display());
            }
            ResultStatus::ConvertError => {
                println!("\x1b[1;31m[✘]\x1b[0m {} (失敗)", r.input_name);
            }
        }
    }

    println!("\x1b[1;36m{}\x1b[0m", line);
    println!("\x1b[1;36m🎯 任務總結報表\x1b[0m");
    println!("總數: {} | \x1b[1;32m通過: {}\x1b[0m | \x1b[1;33m警告: {}\x1b[0m | \x1b[1;31m失敗: {}\x1b[0m", reports.len(), s, w, f);
    if f == 0 && !reports.is_empty() {
        println!("\x1b[1;32m✨ 所有檔案均已處理完成且校驗通過\x1b[0m");
    }
    println!("\x1b[1;36m{}\x1b[0m\n", line);
}

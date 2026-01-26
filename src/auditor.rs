use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use aho_corasick::AhoCorasick;
use std::collections::HashMap;
use opencc_rust::*;

pub fn process_audit(
    original_path: &str,
    translated_path: &str,
    log_path: &PathBuf,
    ac: &AhoCorasick,
    typo_map: &HashMap<String, String>,
    patterns: &[String],
    verbose: bool,
    opencc_config: DefaultConfig,
) -> io::Result<(usize, Vec<String>)> {
    let converter = OpenCC::new(opencc_config).expect("OpenCC 啟動失敗");
    
    let f_orig = fs::File::open(original_path)?;
    let f_trans = fs::File::open(translated_path)?;
    
    let reader_orig = io::BufReader::new(f_orig);
    let reader_trans = io::BufReader::new(f_trans);
    
    let mut orig_lines = reader_orig.lines();
    let mut trans_lines = reader_trans.lines();
    
    let mut error_count = 0;
    let mut issues = Vec::new();
    let mut line_num = 0;

    if verbose {
        // 使用模式匹配代替 == 來判斷模式
        let mode_name = match opencc_config {
            DefaultConfig::S2TWP => "詞彙修正模式",
            _ => "純字體模式",
        };
        println!("\n🧐 啟動對比 (標準: {})", mode_name);
        println!(" 行號 | 狀態      | SRT 原文摘要               | 對比結果");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    loop {
        line_num += 1;
        let l_orig = orig_lines.next();
        let l_trans = trans_lines.next();

        if l_orig.is_none() && l_trans.is_none() { break; }

        let s_orig = l_orig.unwrap_or(Ok(String::new())).unwrap_or_default();
        let s_trans = l_trans.unwrap_or(Ok(String::new())).unwrap_or_default();

        // 1. 預期轉換結果 (OpenCC + Typo)
        let expected_cc = converter.convert(&s_orig);
        let mut expected_final = expected_cc.clone();
        for mat in ac.find_iter(&expected_cc) {
            let word = &patterns[mat.pattern()];
            if let Some(fix) = typo_map.get(word) {
                expected_final = expected_final.replace(word, fix);
            }
        }

        // 2. 判斷是否一致
        if s_trans != expected_final {
            error_count += 1;
            issues.push(format!("第 {} 行不匹配", line_num));

            if verbose {
                // 錯誤行：紅色 [✗ ERR]
                println!(
                    "\x1b[31m{:04} | [✗ ERR] | {:<25} | {}\x1b[0m",
                    line_num, 
                    truncate_str(&expected_final, 25), 
                    s_trans
                );
            }
        } else if verbose {
            // 成功行：綠色 [✓ OK ]
            println!(
                "\x1b[32m{:04} | [✓ OK ] | {:<25} | {}\x1b[0m",
                line_num, 
                truncate_str(&expected_final, 25), 
                s_trans
            );
        }
    }

    if verbose {
        println!("\n對照結束。總計發現 {} 個不同處。日誌: {}", error_count, log_path.to_string_lossy());
    }

    Ok((error_count, issues))
}

fn truncate_str(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_len {
        let mut truncated: String = chars.into_iter().take(max_len - 3).collect();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

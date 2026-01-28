use std::fs::File;
use std::io::{BufRead, BufReader};
use unicode_width::UnicodeWidthStr;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use crate::rules_stay_raw::RawGuard;
use crate::engine_translate::translate_single_line;

const COL_WIDTH: usize = 48; // 每一列的視覺寬度

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let file_a = BufReader::new(File::open(path_a).expect("找不到 A 檔"));
    let file_b = BufReader::new(File::open(path_b).expect("找不到 B 檔"));
    
    let config = if is_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    // 表頭：手動計算對齊
    let head_a = format_to_width("原始參考 (A)", COL_WIDTH);
    let head_b = format_to_width("翻譯成果 (B)", COL_WIDTH);
    println!("\x1b[1;37m{:>4} │ {:^7} │ {} │ {}\x1b[0m", "行號", "狀態", head_a, head_b);
    println!("{}", "━".repeat(115));

    let lines_a: Vec<String> = file_a.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    let lines_b: Vec<String> = file_b.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    
    let max_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    for i in 0..max_lines {
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);
        let line_num = i + 1;

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') { current_section = a.trim().to_string(); }
                let expected = translate_single_line(&converter, &guard, a, &current_section);
                
                if b == &expected {
                    // OK 行
                    let fa = format_to_width(a, COL_WIDTH);
                    let fb = format_to_width(b, COL_WIDTH);
                    println!("{:>4} │ \x1b[1;32m[ OK  ]\x1b[0m │ \x1b[2m{} │ {}\x1b[0m", line_num, fa, fb);
                } else {
                    // ERR 行
                    print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                    print_github_diff(&expected, b);
                    println!();
                }
            },
            (Some(a), None) => {
                let fa = format_to_width(a, COL_WIDTH);
                let err_msg = format_to_width("(( 缺少尾部空行 srt格式錯誤 ))", COL_WIDTH);
                println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ {} │ \x1b[1;31m{}\x1b[0m", line_num, fa, err_msg);
            },
            (None, Some(b)) => {
                let err_msg = format_to_width("(( 缺少尾部空行 srt格式錯誤 ))", COL_WIDTH);
                let fb = format_to_width(b, COL_WIDTH);
                println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m{}\x1b[0m │ {}", line_num, err_msg, fb);
            },
            (None, None) => break,
        }
    }
    println!("{}", "━".repeat(115));
}

/// 修正後的對齊工具：width() 直接回傳 usize
fn format_to_width(s: &str, width: usize) -> String {
    let mut res = String::new();
    let mut curr_w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str()); // 這裡回傳 usize
        if curr_w + cw > width {
            if !res.is_empty() { res.pop(); }
            res.push('…');
            curr_w = width;
            break;
        }
        res.push(c);
        curr_w += cw;
    }
    res + &" ".repeat(width - curr_w)
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
    
    // 左側 A
    let mut w_a = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Delete {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_a + cw <= COL_WIDTH { print!("\x1b[1;31m{}\x1b[0m", disp); w_a += cw; }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_a + cw <= COL_WIDTH { print!("{}", v); w_a += cw; }
        }
    }
    if w_a < COL_WIDTH { print!("{}", " ".repeat(COL_WIDTH - w_a)); }
    print!(" │ ");

    // 右側 B
    let mut w_b = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Insert {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_b + cw <= COL_WIDTH { print!("\x1b[1;37;41m{}\x1b[0m", disp); w_b += cw; }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_b + cw <= COL_WIDTH { print!("{}", v); w_b += cw; }
        }
    }
}

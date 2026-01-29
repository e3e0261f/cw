use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use unicode_width::UnicodeWidthStr;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use crate::rules_stay_raw::RawGuard;
use crate::engine_translate::translate_single_line;

const COL_WIDTH: usize = 48;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let mut fa = File::open(path_a).expect("找不到 A");
    let mut fb = File::open(path_b).expect("找不到 B");

    // 1. 物理偵測：直接讀取最後 4 個位元組，判定是否具備標準空行結尾 (\n\n)
    let a_has_tail = check_physical_blank_line(&mut fa);
    let b_has_tail = check_physical_blank_line(&mut fb);

    // 2. 讀取內容
    let _ = fa.seek(SeekFrom::Start(0));
    let _ = fb.seek(SeekFrom::Start(0));
    
    // 過濾掉結尾的純空行，由後面的物理邏輯統一處理顯示
    let lines_a: Vec<String> = BufReader::new(fa).lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty()) 
        .collect();
    let lines_b: Vec<String> = BufReader::new(fb).lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty())
        .collect();
    
    let config = if is_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    println!("\x1b[1;37m{:>4} │ {:^7} │ {:<width$} │ {:<width$}\x1b[0m", "行號", "狀態", "原始參考 (A)", "翻譯成果 (B)", width = COL_WIDTH);
    println!("{}", "-------------------------------------------------------------------------------------------------------------");

    let text_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    // 處理 1-8 行（文字內容）
    for i in 0..text_lines {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') { current_section = a.trim().to_string(); }
                let expected = translate_single_line(&converter, &guard, a, &current_section);
                if b == &expected || b == a {
                    println!("{}{:>4} │ [ OK  ] │ {} │ {}\x1b[0m", zebra, line_num, format_to_width(a, COL_WIDTH), format_to_width(b, COL_WIDTH));
                } else {
                    print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                    print_github_diff(&expected, b);
                    println!();
                }
            },
            (Some(a), None) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ {} │ \x1b[1;31m(( 缺少此行 ))\x1b[0m", line_num, format_to_width(a, COL_WIDTH)),
            (None, Some(b)) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m(( 缺少此行 ))\x1b[0m │ {}", line_num, format_to_width(b, COL_WIDTH)),
            (None, None) => break,
        }
    }

    // --- 第 9 行：專屬物理邊界診斷 ---
    let footer_num = text_lines + 1;
    match (a_has_tail, b_has_tail) {
        (true, true) => {
            // 雙方都標準，顯示一條清爽的 OK 空行
            println!("{:>4} │ \x1b[1;32m[ OK  ]\x1b[0m │ {:<width$} │ {:<width$}", footer_num, "", "", width = COL_WIDTH);
        },
        (false, true) => {
            // A 沒、B 有：這是修復成功的證明
            println!("{:>4} │ \x1b[1;33m[ FIX ]\x1b[0m │ \x1b[1;31m{} │ \x1b[1;32m{}\x1b[0m", 
                footer_num, format_to_width("缺少空行", COL_WIDTH), format_to_width("系統已補全", COL_WIDTH));
        },
        (false, false) => {
            // 雙方都沒：雙紅警告
            println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m{} │ \x1b[1;31m{}\x1b[0m", 
                footer_num, format_to_width("缺少空行", COL_WIDTH), format_to_width("缺少空行", COL_WIDTH));
        },
        (true, false) => {
            // A 有、B 沒：這是嚴重的退化錯誤
            println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;32m{} │ \x1b[1;31m{}\x1b[0m", 
                footer_num, format_to_width("正常", COL_WIDTH), format_to_width("缺少空行", COL_WIDTH));
        }
    }

    println!("{}", "=============================================================================================================");
}

/// 物理檢查優化：不只看最後一個字節，要看是否以 \n\n 結尾
fn check_physical_blank_line(file: &mut File) -> bool {
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < 2 { return false; }
    
    let mut buf = [0u8; 4]; // 讀取最後 4 個位元組以相容 CRLF
    let seek_pos = if len >= 4 { len - 4 } else { 0 };
    let _ = file.seek(SeekFrom::Start(seek_pos));
    let read_len = file.read(&mut buf).unwrap_or(0);
    let tail = &buf[..read_len];

    // 檢查結尾是否為 \n\n 或 \r\n\r\n (SRT 的標準空行結尾)
    let s = String::from_utf8_lossy(tail);
    s.ends_with("\n\n") || s.ends_with("\n\r\n") || s.ends_with("\r\n\r\n")
}

fn format_to_width(s: &str, width: usize) -> String {
    let mut res = String::new();
    let mut curr_w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if curr_w + cw > width { if !res.is_empty() { res.pop(); } res.push('…'); curr_w = width; break; }
        res.push(c); curr_w += cw;
    }
    res + &" ".repeat(width - curr_w)
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
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

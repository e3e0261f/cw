use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use unicode_width::UnicodeWidthStr;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use crate::rules_stay_raw::RawGuard;
use crate::engine_translate::translate_single_line;

const COL_WIDTH: usize = 48;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let file_a_res = File::open(path_a);
    let file_b_res = File::open(path_b);
    if file_a_res.is_err() || file_b_res.is_err() { return; }

    let mut fa = file_a_res.unwrap();
    let mut fb = file_b_res.unwrap();

    // 1. 偵測檔案末尾是否有換行符 (Byte 級別)
    let a_has_newline = check_last_byte_is_nl(&mut fa);
    let b_has_newline = check_last_byte_is_nl(&mut fb);

    // 2. 讀取內容
    let _ = fa.seek(SeekFrom::Start(0));
    let _ = fb.seek(SeekFrom::Start(0));
    let reader_a = BufReader::new(fa);
    let reader_b = BufReader::new(fb);

    let lines_a: Vec<String> = reader_a.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    let lines_b: Vec<String> = reader_b.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    
    let config = if is_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    println!("\x1b[1;37m{:>4} │ {:^7} │ {:<width$} │ {:<width$}\x1b[0m", "行號", "狀態", "原始參考 (A)", "翻譯成果 (B)", width = COL_WIDTH);
    println!("{}", "-------------------------------------------------------------------------------------------------------------");

    let max_len = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    for i in 0..max_len {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') { current_section = a.trim().to_string(); }
                let expected = translate_single_line(&converter, &guard, a, &current_section);
                if b == &expected {
                    println!("{}{:>4} │ [ OK  ] │ {} │ {}\x1b[0m", zebra, line_num, format_to_width(a, COL_WIDTH), format_to_width(b, COL_WIDTH));
                } else {
                    print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                    print_github_diff(&expected, b);
                    println!();
                }
            },
            (Some(a), None) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ {} │ \x1b[1;31m(( 成果檔遭截斷 ))\x1b[0m", line_num, format_to_width(a, COL_WIDTH)),
            (None, Some(b)) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m(( 原始檔缺失此行 ))\x1b[0m │ {}", line_num, format_to_width(b, COL_WIDTH)),
            (None, None) => break,
        }
    }

    // 3. 【特別診斷】：處理你最在意的第 9 行 (末尾空行)
    if !a_has_newline && b_has_newline {
        let line_num = max_len + 1;
        println!("{:>4} │ \x1b[1;33m[ FIX ]\x1b[0m │ \x1b[1;31m(( 遺失末尾空行 ))\x1b[0m           │ \x1b[1;32m(( 系統已自動補完 ))\x1b[0m", line_num);
    }

    println!("{}", "=============================================================================================================");
}

fn check_last_byte_is_nl(file: &mut File) -> bool {
    let meta = file.metadata().unwrap();
    if meta.len() == 0 { return false; }
    let _ = file.seek(SeekFrom::End(-1));
    let mut b = [0u8; 1];
    if file.read_exact(&mut b).is_ok() { return b[0] == b'\n'; }
    false
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

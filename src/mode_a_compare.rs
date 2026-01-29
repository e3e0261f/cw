use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use unicode_width::UnicodeWidthStr;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use crate::rules_stay_raw::RawGuard;
use crate::engine_translate::translate_single_line;

const COL_WIDTH: usize = 48;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let file_a = BufReader::new(File::open(path_a).expect("找不到 A"));
    let file_b = BufReader::new(File::open(path_b).expect("找不到 B"));
    let config = if is_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    println!("\x1b[1;37m{:>4} │ {:^7} │ {:<width$} │ {:<width$}\x1b[0m", "行號", "狀態", "原始 A", "成果 B", width = COL_WIDTH);
    println!("{}", "-------------------------------------------------------------------------------------------------------------");

    let lines_a: Vec<String> = file_a.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    let lines_b: Vec<String> = file_b.lines().map(|l| l.unwrap_or_default().replace('\u{feff}', "")).collect();
    let max_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    for i in 0..max_lines {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" }; // 斑馬紋淡化奇數行
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
            (Some(a), None) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ {} │ \x1b[1;31m{}\x1b[0m", line_num, format_to_width(a, COL_WIDTH), format_to_width("(( 缺少尾部空行 srt格式錯誤 ))", COL_WIDTH)),
            (None, Some(b)) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m{}\x1b[0m │ {}", line_num, format_to_width("(( 缺少尾部空行 srt格式錯誤 ))", COL_WIDTH), format_to_width(b, COL_WIDTH)),
            (None, None) => break,
        }
    }

    // 末尾靈魂檢測：檢查 Byte 級別的換行符
    check_final_newline(path_a, path_b);
    println!("{}", "=============================================================================================================");
}

fn check_final_newline(path_a: &str, path_b: &str) {
    let check = |p: &str| -> bool {
        if let Ok(mut f) = File::open(p) {
            let _ = f.seek(SeekFrom::End(-1));
            let mut b = [0u8; 1];
            if f.read_exact(&mut b).is_ok() { return b[0] == b'\n'; }
        }
        false
    };
    if check(path_b) && !check(path_a) {
        println!("\x1b[1;33m💡 末尾狀態: A 檔缺少換行，B 檔已由系統自動修復補完。\x1b[0m");
    }
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

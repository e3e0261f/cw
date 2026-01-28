use std::fs::File;
use std::io::{BufRead, BufReader};
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
                    println!("{:>4} │ \x1b[1;32m[ OK  ]\x1b[0m │ \x1b[2m{:<width$} │ {:<width$}\x1b[0m", 
                             line_num, truncate(a, COL_WIDTH), truncate(b, COL_WIDTH), width = COL_WIDTH);
                } else {
                    print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                    print_github_diff(&expected, b);
                    println!();
                }
            },
            (Some(a), None) => {
                // 【優化處】：原檔有內容，成果檔沒了（通常是少了最後的空行）
                print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                print!("{}", truncate(a, COL_WIDTH)); // 左邊印出原檔有的內容
                print!(" │ ");
                // 右邊用純紅字（不帶紅底）印出提示
                print!("\x1b[1;31m{}\x1b[0m", truncate("(( 缺少尾部空行 srt格式错误 ))", COL_WIDTH));
                println!();
            },
            (None, Some(b)) => {
                // 【優化處】：成果檔多了內容
                print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                // 左邊用純紅字印出提示
                print!("\x1b[1;31m{}\x1b[0m", truncate("(( 缺少尾部空行 srt格式错误 ))", COL_WIDTH));
                print!(" │ ");
                print!("{}", truncate(b, COL_WIDTH)); // 右邊印出多出來的內容
                println!();
            },
            (None, None) => break,
        }
    }
    println!("{}", "━".repeat(115));
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
    let mut w_a = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Delete {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_a + cw <= COL_WIDTH { print!("\x1b[1;31m{}\x1b[0m", disp); w_a += cw; } // 刪除處標紅
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
            if w_b + cw <= COL_WIDTH { print!("\x1b[1;37;41m{}\x1b[0m", disp); w_b += cw; } // 新增/錯誤標紅底
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_b + cw <= COL_WIDTH { print!("{}", v); w_b += cw; }
        }
    }
}

fn truncate(s: &str, width: usize) -> String {
    let mut res = String::new();
    let mut curr_w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if curr_w + cw > width { break; }
        res.push(c);
        curr_w += cw;
    }
    res + &" ".repeat(width - curr_w)
}

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
    let mut current_section = String::new();

    for i in 0..std::cmp::max(lines_a.len(), lines_b.len()) {
        let a = lines_a.get(i).cloned().unwrap_or_default();
        let b = lines_b.get(i).cloned().unwrap_or_default();
        if a.trim().starts_with('[') { current_section = a.trim().to_string(); }
        let expected = translate_single_line(&converter, &guard, &a, &current_section);

        if b == expected {
            println!("{:>4} │ \x1b[1;32m[ OK  ]\x1b[0m │ \x1b[2m{:<width$} │ {:<width$}\x1b[0m", i+1, truncate(&a, COL_WIDTH), truncate(&b, COL_WIDTH), width = COL_WIDTH);
        } else {
            print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", i+1);
            print_github_diff(&expected, &b);
            println!();
        }
    }
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
    let mut w_a = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Delete {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_a + cw <= COL_WIDTH { print!("\x1b[1;32m{}\x1b[0m", disp); w_a += cw; }
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

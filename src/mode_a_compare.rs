use std::fs::File;
use std::io::{BufRead, BufReader};
use cw::core; 
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use unicode_width::UnicodeWidthStr;
use colored::Colorize; 

const COL: usize = 42;

pub fn run_detailed_compare(_is_phrase: bool, path_a: &str, path_b: &str) {
    let fa = File::open(path_a).expect("找不到 A");
    let fb = File::open(path_b).expect("找不到 B");
    let lines_a: Vec<String> = BufReader::new(fa).lines().map(|l| l.unwrap_or_default()).collect();
    let lines_b: Vec<String> = BufReader::new(fb).lines().map(|l| l.unwrap_or_default()).collect();
    
    let issues = core::diagnose_file(path_a, true);
    let conv_s2t = OpenCC::new(DefaultConfig::S2T).unwrap();
    let conv_s2twp = OpenCC::new(DefaultConfig::S2TWP).unwrap();
    let guard = core::RawGuard::new();

    crate::ui_style::print_compare_header(path_a, path_b);
    let max = std::cmp::max(lines_a.len(), lines_b.len());
    let mut section = String::new();

    for i in 0..max {
        let l_idx = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        let issue_idx = issues.iter().position(|iss| iss.line == l_idx);
        let status = if let Some(idx) = issue_idx {
            format!("[ ! {:02} ]", idx + 1).red().bold().to_string()
        } else {
            crate::ui_style::status_info()
        };

        if let (Some(a), Some(b)) = (opt_a, opt_b) {
            if a.trim().starts_with('[') { section = a.trim().to_string(); }
            let exp_s2t = core::translate_single_line(&conv_s2t, &guard, a, &section);
            let exp_s2twp = core::translate_single_line(&conv_s2twp, &guard, a, &section);
            if b == &exp_s2t || b == &exp_s2twp || b == a {
                println!("{}{:>4} │ {} │ {} │ {}\x1b[0m", zebra, l_idx, status, crate::ui_style::format_to_width(a, COL), crate::ui_style::format_to_width(b, COL));
            } else {
                print!("{:>4} │ [ DIFF ] │ ", l_idx);
                print_diff(a, b);
                println!();
            }
        }
    }
    if core::needs_trailing_newline_fix(path_a) {
        // 【修正點】：補齊了括號，讓 status_fixd() 能對應到 {}
        println!("{:>4} │ {} │ {} │ {}", 
                 max + 1, 
                 crate::ui_style::status_fixd(), 
                 crate::ui_style::format_to_width("缺少空行", COL), 
                 crate::ui_style::format_to_width("系統已補全", COL));
    }
    crate::ui_style::print_footnotes(&issues);
}

fn print_diff(a: &str, b: &str) {
    let diff = TextDiff::from_chars(a, b);
    let mut w_a = 0;
    for c in diff.iter_all_changes() {
        if c.tag() == ChangeTag::Delete {
            print!("{}", c.value().red().on_white());
            w_a += UnicodeWidthStr::width(c.value());
        } else if c.tag() == ChangeTag::Equal {
            print!("{}", c.value());
            w_a += UnicodeWidthStr::width(c.value());
        }
    }
    if w_a < COL { print!("{}", " ".repeat(COL - w_a)); }
    print!(" │ ");
    for c in diff.iter_all_changes() {
        if c.tag() == ChangeTag::Insert {
            print!("{}", c.value().green().bold());
        } else if c.tag() == ChangeTag::Equal {
            print!("{}", c.value());
        }
    }
}

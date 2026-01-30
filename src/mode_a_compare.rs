use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::{rules_stay_raw::RawGuard, engine_translate::translate_single_line};
use crate::ui_style::{status_info, status_fixd, format_to_width, print_footnotes};
use colored::Colorize;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use unicode_width::UnicodeWidthStr;

const COL: usize = 42;

pub fn run_detailed_compare(is_phrase: bool, path_a: &str, path_b: &str) {
    let fa = File::open(path_a).expect("找不到 A");
    let fb = File::open(path_b).expect("找不到 B");
    let lines_a: Vec<String> = BufReader::new(fa).lines().map(|l| l.unwrap_or_default()).collect();
    let lines_b: Vec<String> = BufReader::new(fb).lines().map(|l| l.unwrap_or_default()).collect();
    
    let issues = crate::checker::diagnose_file(path_a, true);
    let conv = OpenCC::new(if is_phrase { DefaultConfig::S2TWP } else { DefaultConfig::S2T }).unwrap();
    let guard = RawGuard::new();

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
            let label = if idx < 99 { format!("! {:02}", idx + 1) } else { "! ++".to_string() };
            format!("[ {} ]", label).red().bold().to_string()
        } else {
            status_info()
        };

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') { section = a.trim().to_string(); }
                let exp = translate_single_line(&conv, &guard, a, &section);
                if b == &exp || b == a {
                    println!("{}{:>4} │ {} │ {} │ {}\x1b[0m", zebra, l_idx, status, format_to_width(a, COL), format_to_width(b, COL));
                } else {
                    print!("{:>4} │ {} │ ", l_idx, "[ DIFF ]".red().bold());
                    print_diff(a, b);
                    println!();
                }
            },
            _ => {} 
        }
    }

    if crate::checker::needs_trailing_newline_fix(path_a) {
        println!("{:>4} │ {} │ {} │ {}", max + 1, status_fixd(), format_to_width("缺少空行", COL), format_to_width("系統已補全", COL));
    }
    print_footnotes(&issues);
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
        if c.tag() == ChangeTag::Insert { print!("{}", c.value().green().bold()); }
        else if c.tag() == ChangeTag::Equal { print!("{}", c.value()); }
    }
}

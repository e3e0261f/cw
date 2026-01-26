use std::fs::File;
use std::io::{self, BufReader};

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains("-->") { return true; }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10 { return true; }
    false
}

pub fn check_integrity(_path: &str) -> Vec<String> {
    let issues = Vec::new();
    // 如果未來需要實質檢查，再使用 path 變數
    issues
}

pub fn is_chinese(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

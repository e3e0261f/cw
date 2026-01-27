pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains("-->") { return true; }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10 { return true; }
    false
}

pub fn check_integrity(path: &str) -> Vec<String> {
    let mut issues = Vec::new();
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() == 0 { issues.push("檔案長度為零".to_string()); }
    }
    issues
}

// 加上此標籤，告訴編譯器這是備用工具，不需報警告
#[allow(dead_code)]
pub fn is_chinese(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

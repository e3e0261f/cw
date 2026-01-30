use std::fs;
use skrt;
use crate::report_format::SubtitleIssue;

pub fn diagnose_file(path: &str, translate: bool) -> Vec<SubtitleIssue> {
    let mut issues = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(s) => s.replace('\u{feff}', ""),
        Err(_) => return vec![SubtitleIssue { line: 0, message: "讀取失敗".to_string() }],
    };

    if path.to_lowercase().ends_with(".ass") { return issues; }

    if needs_trailing_newline_fix(path) {
        issues.push(SubtitleIssue { line: 0, message: "檔案末端損壞：缺少 SRT 規範空行".to_string() });
    }

    match skrt::Srt::try_parse(&content) {
        Ok(srt) => {
            for sub in srt.subtitles() {
                if sub.start() > sub.end() {
                    issues.push(SubtitleIssue { line: sub.number() as usize, message: "時間邏輯錯誤：結束早於開始".to_string() });
                }
            }
        },
        Err(e) => {
            issues.push(SubtitleIssue { line: 0, message: if translate { format!("語法異常: {:?}", e) } else { format!("{:?}", e) } });
        }
    }
    issues
}

pub fn needs_trailing_newline_fix(path: &str) -> bool {
    if let Ok(data) = fs::read(path) {
        if data.is_empty() { return true; }
        let len = data.len();
        return len < 2 || data[len-1] != b'\n' || (data[len-2] != b'\n' && data[len-2] != b'\r');
    }
    false
}

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    t.is_empty() || t.contains("-->") || (t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10)
}

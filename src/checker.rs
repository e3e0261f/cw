use std::fs;
use skrt;
use crate::report_format::SubtitleIssue;

/// 核心功能塊：執行全量格式診斷
pub fn diagnose_file(path: &str, translate: bool) -> Vec<SubtitleIssue> {
    let mut issues = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(s) => s.replace('\u{feff}', ""),
        Err(_) => return vec![SubtitleIssue { line: 0, message: "讀取失敗".to_string() }],
    };

    if path.to_lowercase().ends_with(".ass") { return issues; }

    // 1. 物理末端檢查
    if needs_trailing_newline_fix(path) {
        issues.push(SubtitleIssue { line: 0, message: "檔案末端損壞：缺少 SRT 規範空行".to_string() });
    }

    // 2. 語法檢查 (使用 skrt)
    match skrt::Srt::try_parse(&content) {
        Ok(srt) => {
            for (idx, sub) in srt.subtitles().iter().enumerate() {
                if sub.start() > sub.end() {
                    issues.push(SubtitleIssue { line: idx + 1, message: "時間邏輯錯誤：結束早於開始".to_string() });
                }
            }
        },
        Err(e) => {
            issues.push(SubtitleIssue { 
                line: 0, 
                message: if translate { translate_skrt_error(e) } else { format!("{:?}", e) } 
            });
        }
    }
    issues
}

fn translate_skrt_error(err: skrt::SrtError) -> String {
    match err {
        skrt::SrtError::InvalidTimestamp { position } => format!("L{}: 時間戳格式錯誤。", position),
        skrt::SrtError::UnexpectedEof => "檔案非預期結束".to_string(),
        _ => format!("{:?}", err),
    }
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

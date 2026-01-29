use std::fs;
use skrt;

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains("-->") { return true; }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10 { return true; }
    false
}

/// 使用 skrt 0.1.1 進行深度診斷
pub fn diagnose_file(path: &str, translate: bool) -> Vec<String> {
    let mut issues = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(s) => s.replace('\u{feff}', ""), 
        Err(_) => return vec!["[錯誤] 無法讀取檔案內容".to_string()],
    };

    if path.to_lowercase().ends_with(".ass") { return issues; }

    match skrt::Srt::try_parse(&content) {
        Ok(srt_obj) => {
            // 【解決 number 報錯】：使用 enumerate 產生序號，不依賴 sub.number()
            // 【解決 start/end 報錯】：使用括號調用方法 .start() 和 .end()
            for (idx, sub) in srt_obj.subtitles().iter().enumerate() {
                if sub.start() > sub.end() {
                    issues.push(format!("第 {} 組字幕: 時間邏輯錯誤 (結束早於開始)", idx + 1));
                }
            }
        },
        Err(e) => {
            let err_msg = if translate { translate_skrt_error(e) } else { format!("{:?}", e) };
            issues.push(err_msg);
        }
    }
    issues
}

/// 對應 skrt::SrtError (v0.1.1) 實際存在的變體
fn translate_skrt_error(err: skrt::SrtError) -> String {
    match err {
        // 【修正】：只匹配 0.1.1 版中確實存在的變體
        skrt::SrtError::InvalidTimestamp { position } => 
            format!("! L{}: 時間戳格式錯誤。請檢查是否符合 [時:分:秒,毫秒] 規範。", position),
        
        skrt::SrtError::UnexpectedEof => 
            "! 檔案非預期結束：檔案結尾不完整或格式損毀。".to_string(),
            
        // 使用萬用匹配處理其他可能的內部錯誤
        _ => format!("! 語法異常: {:?}", err),
    }
}

pub fn needs_trailing_newline_fix(path: &str) -> bool {
    if !path.to_lowercase().ends_with(".srt") { return false; }
    if let Ok(data) = fs::read(path) {
        if data.is_empty() { return true; }
        let len = data.len();
        if len < 2 || data[len-1] != b'\n' || (data[len-2] != b'\n' && data[len-2] != b'\r') {
            return true;
        }
    }
    false
}

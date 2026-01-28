use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// 判定是否為 SRT/ASS 的結構行
pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains("-->") { return true; }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10 { return true; }
    false
}

/// 【核心功能】檢測原始 SRT 是否遺失末尾空行
/// 返回 true 代表需要修復
pub fn needs_trailing_newline_fix(path: &str) -> bool {
    if !path.to_lowercase().ends_with(".srt") { return false; }
    
    if let Ok(mut file) = File::open(path) {
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if len < 2 { return true; } // 檔案太小或為空，肯定需要補

        let _ = file.seek(SeekFrom::End(-2));
        let mut buffer = [0; 2];
        if file.read_exact(&mut buffer).is_ok() {
            // SRT 標準結尾應該是 \n\n (或 \r\n\r\n)
            // 如果最後一個不是 \n，或是倒數第二個也不是換行符，則判定為損壞
            if buffer[1] != b'\n' || (buffer[0] != b'\n' && buffer[0] != b'\r') {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
pub fn is_chinese(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

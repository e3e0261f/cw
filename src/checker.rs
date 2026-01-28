use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains("-->") { return true; }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10 { return true; }
    false
}

pub fn needs_trailing_newline_fix(path: &str) -> bool {
    if !path.to_lowercase().ends_with(".srt") { return false; }
    if let Ok(mut file) = File::open(path) {
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if len < 2 { return true; }
        let _ = file.seek(SeekFrom::End(-2));
        let mut buffer = [0; 2];
        if file.read_exact(&mut buffer).is_ok() {
            if buffer[1] != b'\n' || (buffer[0] != b'\n' && buffer[0] != b'\r') {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
pub fn check_integrity(_path: &str) -> Vec<String> { Vec::new() }

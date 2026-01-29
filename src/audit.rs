use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::report_format::ResultStatus;
use crate::checker;
use chrono::Local;

/// 【核心功能】生成詳盡日誌，包含原檔的所有稽核錯誤
pub fn create_detailed_log_with_issues(
    path_a: &str,
    path_b: &str,
    log_path: &PathBuf,
    status: &ResultStatus,
    max_mb: u64,
    backup_count: u32,
    issues: &[String], // 接收全量的錯誤清單
) -> io::Result<()> {
    // 自動滾動日誌
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() > max_mb * 1024 * 1024 { rotate_logs(log_path, backup_count)?; }
    }

    let mut log_f = OpenOptions::new().create(true).append(true).open(log_path)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let reader_a = BufReader::new(File::open(path_a)?);
    let reader_b = BufReader::new(File::open(path_b)?);

    writeln!(log_f, "\n[ 任務批次：{} ]", now)?;
    writeln!(log_f, "原始檔案：{}\n輸出檔案：{}", path_a, path_b)?;
    
    // --- 寫入全量診斷建議 (不管是 10 個還是 1000 個都記下來) ---
    if !issues.is_empty() {
        writeln!(log_f, "🛠️ 原檔診斷報告 (共 {} 處)：", issues.len())?;
        for issue in issues {
            writeln!(log_f, "   • {}", issue)?;
        }
    }
    
    writeln!(log_f, "------------------------------------------------------------")?;

    for (idx, (l_a, l_b)) in reader_a.lines().zip(reader_b.lines()).enumerate() {
        let a = l_a.unwrap_or_default();
        let b = l_b.unwrap_or_default();
        if a != b {
            let ts = Local::now().format("%H:%M:%S%.3f");
            let tag = if checker::is_srt_structure(&a) { "[結構]" } else { "[內容]" };
            writeln!(log_f, "[{}] L{:03} {} 變動 | 原: {} | 譯: {}", ts, idx + 1, tag, a, b)?;
        }
    }
    writeln!(log_f, "[ 狀態：{:?} ]\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", status)?;
    Ok(())
}

fn rotate_logs(log_path: &PathBuf, count: u32) -> io::Result<()> {
    for i in (1..count).rev() {
        let old = log_path.with_extension(format!("log.{}", i));
        let new = log_path.with_extension(format!("log.{}", i + 1));
        if old.exists() { let _ = fs::rename(old, new); }
    }
    let first = log_path.with_extension("log.1");
    if log_path.exists() { fs::rename(log_path, first)?; }
    Ok(())
}

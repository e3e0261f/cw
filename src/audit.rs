use std::fs::{self, OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::report_format::{ResultStatus, SubtitleIssue};
use crate::checker;
use chrono::Local;

pub fn create_detailed_log_with_issues(path_a: &str, path_b: &str, log_p: &PathBuf, status: &ResultStatus, max_mb: u64, count: u32, issues: &[SubtitleIssue]) -> std::io::Result<()> {
    if let Ok(m) = fs::metadata(log_p) { if m.len() > max_mb * 1024 * 1024 { rotate(log_p, count)?; } }
    let mut f = OpenOptions::new().create(true).append(true).open(log_p)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(f, "\n[ 任務批次：{} ]\n原檔：{}\n成果：{}", now, path_a, path_b)?;
    for issue in issues { writeln!(f, "🛠️ 診斷：L{:03} {}", issue.line, issue.message)?; }
    let r_a = BufReader::new(File::open(path_a)?);
    let r_b = BufReader::new(File::open(path_b)?);
    for (i, (l_a, l_b)) in r_a.lines().zip(r_b.lines()).enumerate() {
        let (a, b) = (l_a?, l_b?);
        if a != b { writeln!(f, "[{}] L{:03} 變動 | 原: {} | 譯: {}", Local::now().format("%H:%M:%S%.3f"), i+1, a, b)?; }
    }
    writeln!(f, "[ 狀態：{:?} ]\n{}", status, "━".repeat(40))?;
    Ok(())
}

fn rotate(p: &PathBuf, c: u32) -> std::io::Result<()> {
    for i in (1..c).rev() {
        let old = p.with_extension(format!("log.{}", i));
        let new = p.with_extension(format!("log.{}", i+1));
        if old.exists() { fs::rename(old, new)?; }
    }
    if p.exists() { fs::rename(p, p.with_extension("log.1"))?; }
    Ok(())
}

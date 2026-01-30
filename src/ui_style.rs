use crate::report_format::{FileReport, ResultStatus, SubtitleIssue};
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

const DIVIDER: &str = "============================================================";

pub fn status_info() -> String { "[ INFO ]".green().to_string() }
pub fn status_warn() -> String { "[ WARN ]".yellow().bold().to_string() }
pub fn status_fail() -> String { "[ FAIL ]".red().bold().to_string() }
pub fn status_fixd() -> String { "[ FIXD ]".yellow().bold().to_string() }

pub fn print_help() {
    println!("\n{}🚀 CW 字幕工作站 v1.9.1{}", "\x1b[1;36m", "\x1b[0m");
    println!("{}", DIVIDER);
    println!("用法: cw <檔案.srt> [-p 專業] [-d 覆寫] [-b 傳送]");
    println!("系統: --init (生成預設 cw.cfg)");
}

pub fn print_translated_preview(pairs: &[(usize, String, String)], issues: &[SubtitleIssue]) {
    println!("{}", "--- 翻譯對照預覽 ---".dimmed());
    for (n, o, t) in pairs.iter().take(10) {
        let has_err = issues.iter().any(|i| i.line == *n);
        let label = if has_err { format!("L{:03}!", n).red().bold() } else { format!("L{:03} ", n).dimmed() };
        println!("  {} 原: {}\n        譯: {}", label, o.trim(), t.trim().green());
    }
}

pub fn print_footnotes(issues: &[SubtitleIssue]) {
    if issues.is_empty() { return; }
    println!("{}", "--- 異常細節報告 ---".red().bold());
    for (idx, iss) in issues.iter().enumerate().take(10) {
        let line_tag = if iss.line == 0 { "末端".to_string() } else { format!("L{:03}", iss.line) };
        println!("  {} {}: {}", format!("!{:02}", idx+1).red(), line_tag, iss.message.red());
    }
}

pub fn print_summary(reports: &[FileReport], dur: std::time::Duration) {
    println!("\n{}\n📋 任務處理明細報告", DIVIDER);
    let mut s = 0;
    for r in reports {
        let icon = if r.status == ResultStatus::Success { s+=1; "[OK]".green() } else { "[⚠]".yellow() };
        println!("{} {} -> {}\n     ├─ 變動: {} 行 | 異常: {} 處 | 耗時: {:?}", icon, r.input_name, r.output_name, r.translated_pairs.len(), r.issues.len(), r.duration);
        println!("     └─ 日誌: \x1b[4m{}\x1b[0m", r.temp_log_path.display());
    }
    println!("------------------------------------------------------------\n🎯 成功: {}/{} | 總耗時: {:?}\n{}", s, reports.len(), dur, DIVIDER);
}

pub fn format_to_width(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w > width { s.chars().take(width-1).collect::<String>() + "…" }
    else { s.to_string() + &" ".repeat(width - w) }
}

pub fn print_compare_header(a: &str, b: &str) {
    println!("\n🔍 對比模式\n{}\nA: {}\nB: {}", "=".repeat(60), a, b);
}

pub fn print_check_ok(m: &str) { println!("  \x1b[1;32m✔ {}\x1b[0m", m); }
pub fn print_check_err(m: &str) { println!("  \x1b[1;31m✘ {}\x1b[0m", m); }
pub fn print_file_header(idx: usize, total: usize, name: &str) { println!("\x1b[1;35m➔ [{}/{}] {}\x1b[0m", idx, total, name); }

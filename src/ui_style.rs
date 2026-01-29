use crate::report_format::{FileReport, ResultStatus};
use similar::{ChangeTag, TextDiff};

const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";
const DIVIDER_HEAVY: &str = "============================================================";
const DIVIDER_LIGHT: &str = "------------------------------------------------------------";

pub fn print_help() {
    println!("\n{}🚀 CW 專業字幕工程工作站 v1.8.2{}", BLUE, RESET);
    println!("{}", DIVIDER_HEAVY);
    println!("用法: cw <檔案.srt> [--task URL] [--text MSG]");
    println!("專業: cw -p <檔案> (本土化強化模式)");
    println!("對比: cw -a <原始> <成果>");
    println!("{}", DIVIDER_HEAVY);
}

pub fn print_file_header(idx: usize, total: usize, name: &str) {
    println!("\n\x1b[1;35m[{}/{}] 處理檔案: {}\x1b[0m", idx, total, name);
}

pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() { return; }
    println!("{}翻譯對照預覽:{}", DIM, RESET);
    for (line_num, origin, trans) in pairs.iter().take(15) {
        let diff = TextDiff::from_chars(origin, trans);
        print!("  {}L{:03} 原:{} ", DIM, line_num, RESET);
        for change in diff.iter_all_changes() {
            if change.tag() == ChangeTag::Delete { print!("{}{}{}", RED, change.value(), RESET); }
            else if change.tag() == ChangeTag::Equal { print!("{}", change.value()); }
        }
        println!();
        print!("       {}譯:{} ", GREEN, RESET);
        for change in diff.iter_all_changes() {
            if change.tag() == ChangeTag::Insert { print!("{}{}{}", GREEN, change.value(), RESET); }
            else if change.tag() == ChangeTag::Equal { print!("{}", change.value()); }
        }
        println!();
    }
}

pub fn print_summary(reports: &[FileReport], total_duration: std::time::Duration) {
    println!("\n{}", DIVIDER_HEAVY);
    println!("📋 任務處理明細報告");
    println!("{}", DIVIDER_LIGHT);
    let mut s_count = 0;
    for r in reports {
        let icon = match r.status {
            ResultStatus::Success => { s_count += 1; format!("{}[OK]{}", GREEN, RESET) },
            _ => format!("{}[⚠]{}", "\x1b[1;33m", RESET),
        };
        println!("{} {} -> {}", icon, r.input_name, r.output_name);
        for err in &r.verif_errors { println!("     \x1b[1;33m└─ 🛠  {}\x1b[0m", err); }
        println!("     └─ 日誌: {}", r.temp_log_path.display());
    }
    println!("{}", DIVIDER_LIGHT);
    println!("🎯 統計: 通過 {} / 總計 {} | 總耗時: {:?}", s_count, reports.len(), total_duration);
    println!("{}", DIVIDER_HEAVY);
}

pub fn print_check_ok(msg: &str) { println!("  {} ✔ {}{}", GREEN, msg, RESET); }
pub fn print_check_err(msg: &str) { println!("  {} ✘ {}{}", RED, msg, RESET); }
pub fn format_abs_path_link(path: &std::path::Path) -> String { format!("{}{}{}", UNDERLINE, path.display(), RESET) }

pub fn print_compare_header(path_a: &str, path_b: &str) {
    println!("\n{}", DIVIDER_HEAVY);
    println!("🔍 深度內容對比校對 (斑馬紋模式 / 字元級標紅)");
    println!("{}", DIVIDER_LIGHT);
    println!("A: {}\nB: {}", path_a, path_b);
    println!("{}", DIVIDER_HEAVY);
}

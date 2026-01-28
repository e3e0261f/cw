use crate::report_format::{FileReport, ResultStatus};
use unicode_width::UnicodeWidthStr;
use similar::{ChangeTag, TextDiff};

const UI_WIDTH: usize = 70;
const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

fn print_row(text: &str) {
    let text_width = UnicodeWidthStr::width(text);
    let padding = if UI_WIDTH > text_width + 4 { UI_WIDTH - text_width - 4 } else { 0 };
    println!("{}┃{} {} {}{}┃{}", BLUE, RESET, text, " ".repeat(padding), BLUE, RESET);
}

pub fn print_help() {
    println!("\n{}┏{}┓{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    print_row("🚀 CW 專業字幕工程工作站 v1.6.8");
    println!("{}┣{}┫{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    print_row("用法: cw <檔案.srt> [--task URL] [--text MSG]");
    println!("{}┗{}┛{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
}

pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() { return; }
    println!("  {}────────────── 翻譯對照預覽 ──────────────{}", "\x1b[2m", RESET);
    for (line_num, origin, trans) in pairs.iter().take(10) {
        let diff = TextDiff::from_chars(origin, trans);
        print!("  \x1b[2mL{:03} 原:\x1b[0m ", line_num);
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
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("📋 任務處理詳細明細報告");
    println!("{}┣{}┫", BLUE, line);
    
    let mut s_count = 0;
    for r in reports {
        let icon = match r.status {
            ResultStatus::Success => { s_count += 1; format!("{}[OK]{}", GREEN, RESET) },
            ResultStatus::VerifWarning => format!("{}[⚠]{}", YELLOW, RESET),
            ResultStatus::ConvertError => format!("{}[✘]{}", RED, RESET),
        };
        print_row(&format!("{} {} -> {}", icon, r.input_name, r.output_name));
        
        // 醒目提示原檔損壞
        if !r.verif_errors.is_empty() {
            for err in &r.verif_errors {
                print_row(&format!("     \x1b[1;33m└─ 🛠  {}\x1b[0m", err));
            }
        }
        print_row(&format!("     └─ 變動: {} 行 | 耗時: {:?}", r.translated_pairs.len(), r.duration));
    }
    
    println!("{}┣{}┫{}", BLUE, line, RESET);
    let summary = format!("🎯 統計: 通過 {} / 總計 {} | 總耗時: {:?}", s_count, reports.len(), total_duration);
    print_row(&summary);
    println!("{}┗{}┛{}", BLUE, line, RESET);
}

pub fn print_file_header(idx: usize, total: usize, name: &str) {
    println!("\n\x1b[1;35m➔ 檔案 [{}/{}] : {}\x1b[0m", idx, total, name);
}
pub fn print_check_ok(msg: &str) { println!("  {} ✔ {}{}", GREEN, msg, RESET); }
pub fn print_check_err(msg: &str) { println!("  {} ✘ {}{}", RED, msg, RESET); }
pub fn print_compare_header(path_a: &str, path_b: &str) {
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("🔍 深度內容對比校對模式 (字元級標紅)");
    println!("{}┣{}┫", BLUE, line);
    print_row(&format!("A: {}", path_a));
    print_row(&format!("B: {}", path_b));
    println!("{}┗{}┛{}", BLUE, line, RESET);
}
pub fn format_abs_path_link(path: &std::path::Path) -> String { format!("\x1b[4m{}\x1b[0m", path.display()) }

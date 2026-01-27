use crate::report_format::{FileReport, ResultStatus};
use unicode_width::UnicodeWidthStr;
use similar::{ChangeTag, TextDiff};

const UI_WIDTH: usize = 70;
const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";
const UNDERLINE: &str = "\x1b[4m";

fn print_row(text: &str) {
    let text_width = UnicodeWidthStr::width(text);
    let padding = if UI_WIDTH > text_width + 4 { UI_WIDTH - text_width - 4 } else { 0 };
    println!("{}┃{} {} {}{}┃{}", BLUE, RESET, text, " ".repeat(padding), BLUE, RESET);
}

pub fn print_help() {
    println!("\n{}┏{}┓{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    print_row("🚀 CW 專業字幕工程工作站 v1.2.0");
    println!("{}┣{}┫{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    print_row("用法: cw <檔案.srt> 或 cw *.ass");
    print_row("專業: cw -p <檔案> (本土化強化模式)");
    print_row("對比: cw -a <原始> <對標>");
    println!("{}┗{}┛{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
}

pub fn print_file_header(idx: usize, total: usize, name: &str) {
    println!("\n\x1b[1;35m➔ 檔案 [{}/{}] : {}\x1b[0m", idx, total, name);
}

pub fn print_compare_header(path_a: &str, path_b: &str) {
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("🔍 深度內容對比校對模式 (GitHub 字元級標紅)");
    println!("{}┣{}┫", BLUE, line);
    print_row(&format!("A: {}", path_a));
    print_row(&format!("B: {}", path_b));
    println!("{}┗{}┛{}", BLUE, line, RESET);
}

pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() { return; }
    println!("  {}────────────── 翻譯對照預覽 (僅變動行) ──────────────{}", "\x1b[2m", RESET);
    for (line_num, origin, trans) in pairs.iter().take(15) {
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

pub fn print_summary(reports: &[FileReport]) {
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("📋 任務處理詳細明細報表");
    println!("{}┣{}┫", BLUE, line);
    let mut s_count = 0;
    for r in reports {
        let icon = if r.status == ResultStatus::Success { s_count += 1; format!("{}[OK]{}", GREEN, RESET) } else { format!("{}[✘]{}", RED, RESET) };
        print_row(&format!("{} {} -> {}", icon, r.input_name, r.output_name));
        // 總結裡也印出絕對路徑
        if r.status == ResultStatus::Success {
            print_row(&format!("     └─ 日誌: {}", r.temp_log_path.display()));
        }
    }
    println!("{}┣{}┫", BLUE, line);
    print_row(&format!("🎯 統計: 通過 {} / 總計 {}", s_count, reports.len()));
    println!("{}┗{}┛{}", BLUE, line, RESET);
}

pub fn print_check_ok(msg: &str) {
    println!("  {} ✔ {}{}", GREEN, msg, RESET);
}

// 補齊缺失的函式，修復 E0425 錯誤
pub fn print_check_err(msg: &str) {
    println!("  {} ✘ {}{}", RED, msg, RESET);
}

// 供絕對路徑顯示使用的格式化
pub fn format_abs_path_link(path: &std::path::Path) -> String {
    format!("{}{}{}", UNDERLINE, path.display(), RESET)
}

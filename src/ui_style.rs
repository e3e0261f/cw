use crate::report_format::{FileReport, ResultStatus};
use unicode_width::UnicodeWidthStr;

const UI_WIDTH: usize = 70;
const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

pub fn print_help() {
    println!("\n{}🚀 CW 專業字幕工作站 v1.1.0{}", BLUE, RESET);
    println!("用法: cw <檔案> [-p專業模式] [-a對比模式]");
}

pub fn print_file_header(idx: usize, total: usize, name: &str) {
    println!("\n\x1b[1;35m➔ 檔案 [{}/{}] : {}\x1b[0m", idx, total, name);
}

// 供 -a 模式使用的標題
pub fn print_compare_header(path_a: &str, path_b: &str) {
    println!("\n{}┏{}┓{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    println!("{}┃ 🔍 深度內容對比校對模式 (字元級標紅) {}", BLUE, " ".repeat(28), RESET);
    println!("{}┣{}┫{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
    println!("{}┃ A: {}{}", BLUE, path_a, " ".repeat(UI_WIDTH - 6 - UnicodeWidthStr::width(path_a)));
    println!("{}┃ B: {}{}", BLUE, path_b, " ".repeat(UI_WIDTH - 6 - UnicodeWidthStr::width(path_b)));
    println!("{}┗{}┛{}", BLUE, "━".repeat(UI_WIDTH - 2), RESET);
}

pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() { 
        println!("  {}無任何文字變動（不含結構行）{}", "\x1b[2m", RESET);
        return; 
    }
    println!("  {}────────────── 翻譯對照預覽 (僅顯示變動行) ──────────────{}", "\x1b[2m", RESET);
    for (line_num, origin, trans) in pairs.iter().take(15) {
        println!("  \x1b[2mL{:03} 原:\x1b[0m {}", line_num, origin.trim());
        println!("       \x1b[1;32m譯:\x1b[0m {}", trans.trim());
    }
    if pairs.len() > 15 {
        println!("  {}... 還有 {} 行變動已存入日誌檔案{}", "\x1b[2m", pairs.len() - 15, RESET);
    }
}

pub fn print_summary(reports: &[FileReport]) {
    let line_str = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓{}", BLUE, line_str, RESET);
    
    let mut s_count = 0;
    let mut f_count = 0;

    for r in reports {
        let icon = if r.status == ResultStatus::Success { 
            s_count += 1; "[OK]" 
        } else { 
            f_count += 1; "[✘]" 
        };
        let text = format!("{} {} -> {}", icon, r.input_name, r.output_name);
        let text_width = UnicodeWidthStr::width(text.as_str());
        let padding = if UI_WIDTH > text_width + 4 { UI_WIDTH - text_width - 4 } else { 0 };
        println!("{}┃{} {} {}{}┃{}", BLUE, RESET, text, " ".repeat(padding), BLUE, RESET);
    }
    
    println!("{}┣{}┫{}", BLUE, line_str, RESET);
    let summary = format!("🎯 統計: 通過 {} | 失敗 {}", s_count, f_count);
    let s_width = UnicodeWidthStr::width(summary.as_str());
    println!("{}┃{} {} {}{}┃{}", BLUE, RESET, summary, " ".repeat(UI_WIDTH - s_width - 4), BLUE, RESET);
    println!("{}┗{}┛{}", BLUE, line_str, RESET);
}

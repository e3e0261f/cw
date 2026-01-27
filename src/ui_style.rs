use crate::report_format::{FileReport, ResultStatus};
use unicode_width::UnicodeWidthStr;

const UI_WIDTH: usize = 70;
const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

fn print_row(text: &str) {
    let text_width = UnicodeWidthStr::width(text);
    let padding = if UI_WIDTH > text_width + 4 { UI_WIDTH - text_width - 4 } else { 0 };
    println!("{}┃{} {} {}{}┃{}", BLUE, RESET, text, " ".repeat(padding), BLUE, RESET);
}

pub fn print_help() {
    println!("\n{}🚀 CW 專業字幕工作站 v1.2.0{}", BLUE, RESET);
    println!("用法: cw <檔案> [-p專業模式] [-a對比模式]");
}

pub fn print_compare_header(path_a: &str, path_b: &str) {
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("🔍 深度內容對比校對模式 (字元級標紅)");
    println!("{}┣{}┫", BLUE, line);
    print_row(&format!("A: {}", path_a));
    print_row(&format!("B: {}", path_b));
    println!("{}┗{}┛{}", BLUE, line, RESET);
}

pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() { 
        println!("  {}無任何文字變動{}", "\x1b[2m", RESET);
        return; 
    }
    println!("  {}────────────── 翻譯對照預覽 (僅變動行) ──────────────{}", "\x1b[2m", RESET);
    for (line_num, origin, trans) in pairs.iter().take(15) {
        println!("  \x1b[2mL{:03} 原:\x1b[0m {}", line_num, origin.trim());
        println!("       {}譯:{} {}", GREEN, RESET, trans.trim()); // 這裡用到了 GREEN
    }
}

pub fn print_summary(reports: &[FileReport]) {
    let line = "━".repeat(UI_WIDTH - 2);
    println!("\n{}┏{}┓", BLUE, line);
    print_row("📋 任務處理詳細明細報表");
    println!("{}┣{}┫", BLUE, line);
    
    let mut s_count = 0;
    for r in reports {
        let icon = if r.status == ResultStatus::Success { 
            s_count += 1; 
            format!("{}[OK]{}", GREEN, RESET) // 這裡用到了 GREEN
        } else { 
            format!("{}[✘]{}", RED, RESET)    // 這裡用到了 RED
        };
        print_row(&format!("{} {} -> {}", icon, r.input_name, r.output_name));
    }
    
    println!("{}┣{}┫{}", BLUE, line, RESET);
    print_row(&format!("🎯 統計: 通過 {} / 總計 {}", s_count, reports.len()));
    println!("{}┗{}┛{}", BLUE, line, RESET);
}

pub fn print_check_ok(msg: &str) {
    println!("  {} ✔ {}{}", GREEN, msg, RESET);
}

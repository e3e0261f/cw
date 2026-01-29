use crate::report_format::{FileReport, ResultStatus};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};

const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";
const DIVIDER_HEAVY: &str = "============================================================";
const DIVIDER_LIGHT: &str = "------------------------------------------------------------";

pub fn print_help() {
    println!("\n{}🚀 CW 專業字幕工程工作站 v1.8.4{}", BLUE, RESET);
    println!("{}", DIVIDER_HEAVY);
    println!("用法: cw <檔案.srt> [--task URL] [--text MSG]");
    println!("專業: cw -p <檔案> (本土化強化模式)");
    println!("對比: cw -a <原始> <成果>");
    println!("{}", DIVIDER_HEAVY);
}

pub fn print_summary(reports: &[FileReport], total_duration: std::time::Duration) {
    println!("\n{}", DIVIDER_HEAVY);
    println!("📋 任務處理明細報告");
    println!("{}", DIVIDER_LIGHT);
    let mut s_count = 0;
    for r in reports {
        let icon = match r.status {
            ResultStatus::Success => {
                s_count += 1;
                format!("{}[OK]{}", GREEN, RESET)
            }
            _ => format!("{}[⚠]{}", YELLOW, RESET),
        };
        println!("{} {} -> {}", icon, r.input_name, r.output_name);
        for err in &r.verif_errors {
            println!("     \x1b[1;33m├─ 🛠  提示: {}{}", err, RESET);
        }
        for issue in &r.original_issues {
            println!("     \x1b[1;33m├─ ⚠️  原檔問題: {}{}", issue, RESET);
        }
        println!(
            "     ├─ 變動: {} 行 | 耗時: {:?}",
            r.translated_pairs.len(),
            r.duration
        );
        println!("     └─ 日誌: {}", r.temp_log_path.display());
    }
    println!("{}", DIVIDER_LIGHT);
    println!(
        "🎯 統計: 通過 {} / 總計 {} | 總耗時: {:?}",
        s_count,
        reports.len(),
        total_duration
    );
    println!("{}", DIVIDER_HEAVY);
}

pub fn print_file_header(idx: usize, total: usize, name: &str) {
    println!("\n\x1b[1;35m[{}/{}] 處理檔案: {}\x1b[0m", idx, total, name);
}
pub fn print_translated_preview(pairs: &[(usize, String, String)]) {
    if pairs.is_empty() {
        return;
    }
    println!("{}翻譯對照預覽:{}", DIM, RESET);
    for (line_num, origin, trans) in pairs.iter().take(15) {
        let diff = TextDiff::from_chars(origin, trans);
        print!("  {}L{:03} 原:{} ", DIM, line_num, RESET);
        for change in diff.iter_all_changes() {
            if change.tag() == ChangeTag::Delete {
                print!("{}{}{}", RED, change.value(), RESET);
            } else if change.tag() == ChangeTag::Equal {
                print!("{}", change.value());
            }
        }
        println!();
        print!("       {}譯:{} ", GREEN, RESET);
        for change in diff.iter_all_changes() {
            if change.tag() == ChangeTag::Insert {
                print!("{}{}{}", GREEN, change.value(), RESET);
            } else if change.tag() == ChangeTag::Equal {
                print!("{}", change.value());
            }
        }
        println!();
    }
}
pub fn print_check_ok(msg: &str) {
    println!("  {} ✔ {}{}", GREEN, msg, RESET);
}
pub fn print_check_err(msg: &str) {
    println!("  {} ✘ {}{}", RED, msg, RESET);
}
pub fn format_abs_path_link(path: &std::path::Path) -> String {
    format!("{}{}{}", UNDERLINE, path.display(), RESET)
}
pub fn print_compare_header(path_a: &str, path_b: &str) {
    println!("\n{}", DIVIDER_HEAVY);
    println!("🔍 深度內容對比校對 (斑馬紋模式 / 檔案修復偵測)");
    println!("{}", DIVIDER_LIGHT);
    println!("A: {}\nB: {}", path_a, path_b);
    println!("{}", DIVIDER_HEAVY);
}

pub fn status_ok(msg: &str) -> String {
    format!("[ OK  ] {}", msg).green().to_string()
}

pub fn status_warn(msg: &str) -> String {
    format!("[ WARN ] {}", msg).yellow().bold().to_string()
}
#[allow(dead_code)]
pub fn status_err(msg: &str) -> String {
    format!("[ ERR  ] {}", msg).red().bold().to_string()
}
#[allow(dead_code)]
pub fn status_fix(msg: &str) -> String {
    format!("[ FIX  ] {}", msg).yellow().bold().to_string() // 或用你原本的顏色
}

// 可選：報告標題
pub fn report_title(title: &str) -> String {
    format!("完整性檢查報告（{}）：", title)
        .yellow()
        .bold()
        .to_string()
}

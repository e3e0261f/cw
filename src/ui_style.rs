use crate::report_format::{FileReport, ResultStatus};
use colored::Colorize;
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

const BLUE: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const RESET: &str = "\x1b[0m";
const UNDERLINE: &str = "\x1b[4m";
const DIVIDER_HEAVY: &str = "============================================================";
const DIVIDER_LIGHT: &str = "-------------------------------------------------------------------------------------------------------------";

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
/// 終極全量預覽：順序輸出 + 庫函數顏色 + 遠端錯誤捕獲
pub fn print_translated_preview(
    pairs: &[(usize, String, String)],
    full_preview: bool,
    issues: &[String],
) {
    if pairs.is_empty() && issues.is_empty() {
        return;
    }

    println!("{}", "翻譯對照預覽:".bold().underline());

    // 用於追蹤哪些錯誤已經在循環中印過了
    let mut printed_issue_indices = HashSet::new();

    for (line_num, orig, trans) in pairs {
        let is_changed = orig.trim() != trans.trim();
        let line_tag = format!("L{}:", line_num);

        // 查找是否有屬於這一行的錯誤
        let current_issue = issues
            .iter()
            .enumerate()
            .find(|(_, msg)| msg.contains(&line_tag));

        // 邏輯：全量模式 OR 有變動 OR 有錯誤，就印出來
        if full_preview || is_changed || current_issue.is_some() {
            // 如果有錯誤，印出醒目的紅色錯誤標籤
            if let Some((idx, msg)) = current_issue {
                println!("  {}", msg.bright_red().bold());
                printed_issue_indices.insert(idx);
            }

            if is_changed {
                // 變動行：亮白色原文，綠色加粗譯文
                println!("  L{:03} 原: {}", line_num, orig.white());
                println!("       譯: {}", trans.green().bold());
            } else {
                // 未變動行：使用 dimmed() 變暗，保持行號連續
                println!("  L{:03} 原: {}", line_num, orig.dimmed());
                println!("       譯: {}", trans.dimmed());
            }
        }
    }

    // --- 關鍵修復：處理像 L239 這種超出文本範圍的遠端錯誤 ---
    let mut printed_remote_header = false;
    for (idx, msg) in issues.iter().enumerate() {
        if !printed_issue_indices.contains(&idx) {
            if !printed_remote_header {
                println!(
                    "  {}",
                    "--------------------------------------------------".dimmed()
                );
                println!(
                    "  {}",
                    "⚠️  偵測到超出文本範圍的異常 (遠端行):".bright_yellow()
                );
                printed_remote_header = true;
            }
            println!("  {}", msg.bright_red());
        }
    }
    println!();
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
// pub fn print_compare_header(path_a: &str, path_b: &str) {
//     println!("\n{}", DIVIDER_HEAVY);
//     println!("🔍 深度內容對比校對 (斑馬紋模式 / 檔案修復偵測)");
//     println!("A: {}                     B: {}", path_a, path_b);
//     println!("{}", DIVIDER_LIGHT);
//     println!("{}", DIVIDER_HEAVY);
// }

#[allow(dead_code)]
pub fn status_warn() -> String {
    "[ WARN ]".yellow().bold().to_string()
}

#[allow(dead_code)]
pub fn status_info() -> String {
    "[ INFO ]".green().to_string()
}

#[allow(dead_code)]
pub fn status_fail() -> String {
    "[ FAIL ]".red().bold().to_string()
}

#[allow(dead_code)]
pub fn status_fixd() -> String {
    "[ FIXD ]".yellow().bold().to_string() // 或用你原本的顏色
}

#[allow(dead_code)]
pub fn report_title(title: &str) -> String {
    format!("完整性檢查報告（{}）：", title)
        .yellow()
        .bold()
        .to_string()
}

// 加上 pub，让全项目都能用这个“尺子”
pub fn format_to_width(s: &str, width: usize) -> String {
    let mut res = String::new();
    let mut curr_w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if curr_w + cw > width {
            if !res.is_empty() {
                res.pop();
            }
            res.push('…');
            curr_w = width;
            break;
        }
        res.push(c);
        curr_w += cw;
    }
    res + &" ".repeat(width - curr_w)
}
// 在 ui_style.rs 中修改/添加
pub fn print_compare_header_dynamic(path_a: &str, path_b: &str, width: usize) {
    // 保留旧函数里有用的“仪式感”
    println!("\n{}", DIVIDER_HEAVY);
    println!("🔍 -a 深度內容對比校對");
    println!("{}", DIVIDER_LIGHT);

    let head_a = format_to_width(path_a, width);
    let head_b = format_to_width(path_b, width);

    // 打印你的“完美对齐”动态表头
    println!(
        " \x1b[1;37m{:>4}  {:^8}  {}  {}\x1b[0m",
        "行號",
        "狀態",
        head_a.cyan().bold(),
        head_b.cyan().bold()
    );
}

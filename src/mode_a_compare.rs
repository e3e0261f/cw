use crate::engine_translate::translate_single_line;
use crate::rules_stay_raw::RawGuard;
use crate::ui_style::{status_fail, status_info, status_warn};
use colored::Colorize;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use unicode_width::UnicodeWidthStr;

const COL_WIDTH: usize = 40;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let mut fa = File::open(path_a).expect("找不到 A");
    let mut fb = File::open(path_b).expect("找不到 B");

    // 1. 物理偵測：直接讀取最後 4 個位元組，判定是否具備標準空行結尾 (\n\n)
    let a_has_tail = check_physical_blank_line(&mut fa);
    let b_has_tail = check_physical_blank_line(&mut fb);

    // 2. 讀取內容
    let _ = fa.seek(SeekFrom::Start(0));
    let _ = fb.seek(SeekFrom::Start(0));

    // 過濾掉結尾的純空行，由後面的物理邏輯統一處理顯示
    let lines_a: Vec<String> = BufReader::new(fa)
        .lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty())
        .collect();
    let lines_b: Vec<String> = BufReader::new(fb)
        .lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty())
        .collect();

    let config = if is_phrase_mode {
        DefaultConfig::S2TWP
    } else {
        DefaultConfig::S2T
    };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    println!(
        "\x1b[1;37m{:>4} │ {:^7} │ {:<width$} │ {:<width$}\x1b[0m",
        "行號",
        "狀態",
        "原始參考 (A)",
        "翻譯成果 (B)",
        width = COL_WIDTH
    );

    println!("-------------------------------------------------------------------------------------------------------------");

    let text_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    // 處理 1-8 行（文字內容）
    for i in 0..text_lines {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') {
                    current_section = a.trim().to_string();
                }
                let expected = translate_single_line(&converter, &guard, a, &current_section);
                if b == &expected || b == a {
                    println!(
                        "{}{:>4} │ {} │ {} │ {}\x1b[0m",
                        zebra,
                        line_num,
                        status_info(),
                        format_to_width(a, COL_WIDTH),
                        format_to_width(b, COL_WIDTH)
                    );
                } else {
                    print!("{:>4} │ \x1b[1;31m{}\x1b[0m │ ", line_num, status_fail());
                    print_github_diff(&expected, b);
                    println!();
                }
            }
            (Some(a), None) => println!(
                "{:>4} │ \x1b[1;31m{}\x1b[0m │ {} │ \x1b[1;31m(( 缺少此行 ))\x1b[0m",
                line_num,
                status_fail(),
                format_to_width(a, COL_WIDTH)
            ),
            (None, Some(b)) => println!(
                "{:>4} │ \x1b[1;31m{}\x1b[0m │ \x1b[1;31m(( 缺少此行 ))\x1b[0m │ {}",
                line_num,
                status_fail(),
                format_to_width(b, COL_WIDTH)
            ),
            (None, None) => break,
        }
    }

    // 專屬物理邊界診斷
    // --- 檔尾空行診斷（整合到表格最後一行，美化對齊） ---
    let footer_num = text_lines + 1;

    // A/B 細節文字（無 A: B: 前綴）
    let a_detail = if a_has_tail {
        "正常".green().to_string()
    } else {
        "缺少空行（可自動修復）".yellow().bold().to_string()
    };
    let b_detail = if b_has_tail {
        "正常".green().to_string()
    } else {
        "缺少空行（可自動修復）".yellow().bold().to_string()
    };

    // 整體狀態標籤（固定寬度 8 字符，補空格等寬）
    let overall_label = if a_has_tail && b_has_tail {
        status_info()
    } else {
        status_warn()
    };

    // 格式化 A/B 欄（強制寬度對齊）
    let a_formatted = format_to_width(&a_detail, COL_WIDTH);
    let b_formatted = format_to_width(&b_detail, COL_WIDTH);

    println!(
        "{:>4} │ {} │ {} │ {}",
        footer_num, overall_label, a_formatted, b_formatted
    );

    println!("=============================================================================================================");
}

/// 物理檢查優化：不只看最後一個字節，要看是否以 \n\n 結尾
fn check_physical_blank_line(file: &mut File) -> bool {
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < 2 {
        return false;
    }

    let mut buf = [0u8; 4]; // 讀取最後 4 個位元組以相容 CRLF
    let seek_pos = len.saturating_sub(4);
    let _ = file.seek(SeekFrom::Start(seek_pos));
    let read_len = file.read(&mut buf).unwrap_or(0);
    let tail = &buf[..read_len];

    // 檢查結尾是否為 \n\n 或 \r\n\r\n (SRT 的標準空行結尾)
    let s = String::from_utf8_lossy(tail);
    s.ends_with("\n\n") || s.ends_with("\n\r\n") || s.ends_with("\r\n\r\n")
}

fn format_to_width(s: &str, width: usize) -> String {
    // 1. 定义清理 ANSI 颜色代码的正则（不用引入 regex 库也能手动过滤，但为了稳妥建议用正则）
    // 如果不想引入 regex 库，可以直接操作，这里提供一个直接能跑的版本
    let strip_ansi = |text: &str| -> String {
        let mut result = String::new();
        let mut skipping = false;
        for c in text.chars() {
            if c == '\x1b' {
                skipping = true;
            }
            if !skipping {
                result.push(c);
            }
            if skipping && c == 'm' {
                skipping = false;
            }
        }
        result
    };

    let clean_text = strip_ansi(s);
    let visible_width = UnicodeWidthStr::width(clean_text.as_str());

    // 2. 根据视觉宽度进行处理
    if visible_width > width {
        // 如果太长，简单截断（这里保留原始字符串 s，但截断逻辑要小心）
        // 建议：直接返回带颜色的原始串，或者在这里做更复杂的截断
        s.chars().take(width).collect()
    } else {
        // 3. 关键点：补齐的空格数 = 目标宽度 - 视觉宽度
        let padding = width - visible_width;
        format!("{}{}", s, " ".repeat(padding))
    }
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
    let mut w_a = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Delete {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_a + cw <= COL_WIDTH {
                print!("\x1b[1;31m{}\x1b[0m", disp);
                w_a += cw;
            }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_a + cw <= COL_WIDTH {
                print!("{}", v);
                w_a += cw;
            }
        }
    }
    if w_a < COL_WIDTH {
        print!("{}", " ".repeat(COL_WIDTH - w_a));
    }
    print!(" │ ");
    let mut w_b = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Insert {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_b + cw <= COL_WIDTH {
                print!("\x1b[1;37;41m{}\x1b[0m", disp);
                w_b += cw;
            }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_b + cw <= COL_WIDTH {
                print!("{}", v);
                w_b += cw;
            }
        }
    }
}

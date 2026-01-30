use crate::engine_translate::translate_single_line;
use crate::rules_stay_raw::RawGuard;
use crate::ui_style::{status_fail, status_info, status_warn};
use colored::Colorize;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

const COL_WIDTH: usize = 40;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    // 1. 提取文件名
    let name_a = Path::new(path_a)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path_a);
    let name_b = Path::new(path_b)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path_b);

    // 2. 调用 UI 函数打印动态表头
    crate::ui_style::print_compare_header_dynamic(name_a, name_b, COL_WIDTH);

    let mut fa = File::open(path_a).expect("找不到 A");
    let mut fb = File::open(path_b).expect("找不到 B");

    // 3. 物理检测尾部空行
    let a_has_tail = check_physical_blank_line(&mut fa);
    let b_has_tail = check_physical_blank_line(&mut fb);

    let _ = fa.seek(SeekFrom::Start(0));
    let _ = fb.seek(SeekFrom::Start(0));

    // 4. 读取内容（过滤掉纯空行，方便文本对比）
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

    // 5. 获取审计地图（基于字符串内容匹配，无视空行导致的偏移）
    let errors_a = crate::checker::audit_timeline_errors(path_a);
    let errors_b = crate::checker::audit_timeline_errors(path_b);

    println!("-------------------------------------------------------------------------------------------------------------");

    let text_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    // 6. 核心循环：处理文字与时间轴审计
    for i in 0..text_lines {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                // --- 逻辑审计注入点 ---
                let err_a = if a.contains("-->") {
                    errors_a.get(a.trim())
                } else {
                    None
                };
                let err_b = if b.contains("-->") {
                    errors_b.get(b.trim())
                } else {
                    None
                };

                if err_a.is_some() || err_b.is_some() {
                    let disp_a = match err_a {
                        Some(msg) => format_to_width(msg, COL_WIDTH).red().to_string(),
                        None => format_to_width(a, COL_WIDTH),
                    };
                    let disp_b = match err_b {
                        Some(msg) => format_to_width(msg, COL_WIDTH).red().to_string(),
                        None => format_to_width(b, COL_WIDTH),
                    };

                    println!(
                        "{}{:>4} │ {} │ {} │ {}\x1b[0m",
                        zebra,
                        line_num,
                        status_fail(),
                        disp_a,
                        disp_b
                    );
                    continue; // 发生逻辑错误，跳过后续文本对比
                }

                // --- 正常对比逻辑 ---
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

    // 7. 档尾物理边界诊断
    let footer_num = text_lines + 1;
    let overall_label = if a_has_tail && b_has_tail {
        status_info()
    } else {
        status_warn()
    };

    let a_raw = if a_has_tail {
        "正常"
    } else {
        "原始文件格式错误-缺少空行"
    };
    let b_raw = if b_has_tail {
        "正常"
    } else {
        "缺少空行-SRT格式规范警告"
    };

    let a_padded = format_to_width(a_raw, COL_WIDTH);
    let b_padded = format_to_width(b_raw, COL_WIDTH);

    let a_final = if a_has_tail {
        a_padded.green()
    } else {
        a_padded.yellow().bold()
    };
    let b_final = if b_has_tail {
        b_padded.green()
    } else {
        b_padded.yellow().bold()
    };

    println!(
        "{:>4} │ {} │ {} │ {}",
        footer_num, overall_label, a_final, b_final
    );

    println!("=============================================================================================================");
}

/// 物理检查：判定是否以 \n\n 结尾
fn check_physical_blank_line(file: &mut File) -> bool {
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < 2 {
        return false;
    }
    let mut buf = [0u8; 4];
    let seek_pos = len.saturating_sub(4);
    let _ = file.seek(SeekFrom::Start(seek_pos));
    let read_len = file.read(&mut buf).unwrap_or(0);
    let tail = &buf[..read_len];
    let s = String::from_utf8_lossy(tail);
    s.ends_with("\n\n") || s.ends_with("\n\r\n") || s.ends_with("\r\n\r\n")
}

fn format_to_width(s: &str, width: usize) -> String {
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
    if visible_width > width {
        s.chars().take(width).collect()
    } else {
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

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use unicode_width::UnicodeWidthStr;
use opencc_rust::*;
use similar::{ChangeTag, TextDiff};
use crate::rules_stay_raw::RawGuard;
use crate::engine_translate::translate_single_line;

const COL_WIDTH: usize = 48;

pub fn run_detailed_compare(is_phrase_mode: bool, path_a: &str, path_b: &str) {
    let mut fa = File::open(path_a).expect("找不到 A");
    let mut fb = File::open(path_b).expect("找不到 B");

    // 1. 物理偵測：這份檔案是否以標準 SRT 的「空行(\n\n)」結尾
    let a_is_standard = check_srt_physical_end(&mut fa);
    let b_is_standard = check_srt_physical_end(&mut fb);

    // 2. 讀取內容（過濾掉末尾可能干擾的純空行，由物理偵測統一接管）
    let _ = fa.seek(SeekFrom::Start(0));
    let _ = fb.seek(SeekFrom::Start(0));
    
    // 這裡我們只取「有內容」的行進行對比，末尾的規範交給最後一行的邏輯
    let lines_a: Vec<String> = BufReader::new(fa).lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty()) // 過濾空行，避免重複報錯
        .collect();
    let lines_b: Vec<String> = BufReader::new(fb).lines()
        .map(|l| l.unwrap_or_default().replace('\u{feff}', ""))
        .filter(|l| !l.trim().is_empty())
        .collect();
    
    let config = if is_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).unwrap();
    let guard = RawGuard::new();

    let head_a = format_to_width("原始參考 (A)", COL_WIDTH);
    let head_b = format_to_width("現有成果 (B)", COL_WIDTH);
    println!("\x1b[1;37m{:>4} │ {:^7} │ {} │ {}\x1b[0m", "行號", "狀態", head_a, head_b);
    println!("{}", "-------------------------------------------------------------------------------------------------------------");

    let text_lines = std::cmp::max(lines_a.len(), lines_b.len());
    let mut current_section = String::new();

    // --- 第一部分：文字內容對比 ---
    for i in 0..text_lines {
        let line_num = i + 1;
        let zebra = if i % 2 == 0 { "" } else { "\x1b[2m" };
        let opt_a = lines_a.get(i);
        let opt_b = lines_b.get(i);

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                if a.trim().starts_with('[') { current_section = a.trim().to_string(); }
                let expected = translate_single_line(&converter, &guard, a, &current_section);
                if b == &expected || b == a {
                    println!("{}{:>4} │ [ OK  ] │ {} │ {}\x1b[0m", zebra, line_num, format_to_width(a, COL_WIDTH), format_to_width(b, COL_WIDTH));
                } else {
                    print!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ ", line_num);
                    print_github_diff(&expected, b);
                    println!();
                }
            },
            (Some(a), None) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ {} │ \x1b[1;31m{}\x1b[0m", 
                line_num, format_to_width(a, COL_WIDTH), format_to_width("(( 缺少此行 ))", COL_WIDTH)),
            (None, Some(b)) => println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m{}\x1b[0m │ {}", 
                line_num, format_to_width("(( 缺少此行 ))", COL_WIDTH), format_to_width(b, COL_WIDTH)),
            (None, None) => break,
        }
    }

    // --- 第二部分：SRT 規範行 (結尾空行檢查) ---
    // 這一行會緊接在文字行號之後
    let footer_row_num = text_lines + 1;
    let zebra_footer = if text_lines % 2 == 0 { "" } else { "\x1b[2m" };

    match (a_is_standard, b_is_standard) {
        (true, true) => {
            // 兩邊都規範，顯示最後一個 OK 空行
            println!("{}{:>4} │ [ OK  ] │ {} │ {}\x1b[0m", 
                zebra_footer, footer_row_num, format_to_width("", COL_WIDTH), format_to_width("", COL_WIDTH));
        },
        (false, true) => {
            // A 沒、B 有：顯示 FIX
            println!("{:>4} │ \x1b[1;33m[ FIX ]\x1b[0m │ \x1b[1;31m{} │ \x1b[1;32m{}\x1b[0m", 
                footer_row_num, format_to_width("缺少空行", COL_WIDTH), format_to_width("系統已補全", COL_WIDTH));
        },
        (false, false) => {
            // 兩邊都沒：顯示雙重 ERR
            println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;31m{} │ \x1b[1;31m{}\x1b[0m", 
                footer_row_num, format_to_width("缺少空行", COL_WIDTH), format_to_width("缺少空行", COL_WIDTH));
        },
        (true, false) => {
            // A 有、B 沒：顯示成果損壞
            println!("{:>4} │ \x1b[1;31m[ ERR ]\x1b[0m │ \x1b[1;32m{} │ \x1b[1;31m{}\x1b[0m", 
                footer_row_num, format_to_width("正常", COL_WIDTH), format_to_width("缺失空行", COL_WIDTH));
        }
    }

    println!("{}", "=============================================================================================================");
}

/// 物理檢查：檔案是否以 \n\n (或 \r\n\r\n) 結尾
fn check_srt_physical_end(file: &mut File) -> bool {
    let meta = file.metadata().unwrap();
    if meta.len() < 2 { return false; }
    let _ = file.seek(SeekFrom::End(-2));
    let mut buf = [0u8; 2];
    if file.read_exact(&mut buf).is_ok() {
        return buf[1] == b'\n' && (buf[0] == b'\n' || buf[0] == b'\r');
    }
    false
}

fn format_to_width(s: &str, width: usize) -> String {
    let mut res = String::new();
    let mut curr_w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if curr_w + cw > width { if !res.is_empty() { res.pop(); } res.push('…'); curr_w = width; break; }
        res.push(c); curr_w += cw;
    }
    res + &" ".repeat(width - curr_w)
}

fn print_github_diff(expected: &str, actual: &str) {
    let diff = TextDiff::from_chars(expected, actual);
    let mut w_a = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Delete {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_a + cw <= COL_WIDTH { print!("\x1b[1;31m{}\x1b[0m", disp); w_a += cw; }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_a + cw <= COL_WIDTH { print!("{}", v); w_a += cw; }
        }
    }
    if w_a < COL_WIDTH { print!("{}", " ".repeat(COL_WIDTH - w_a)); }
    print!(" │ ");
    let mut w_b = 0;
    for change in diff.iter_all_changes() {
        if change.tag() == ChangeTag::Insert {
            let v = change.value();
            let disp = if v == " " { "·" } else { v };
            let cw = UnicodeWidthStr::width(disp);
            if w_b + cw <= COL_WIDTH { print!("\x1b[1;37;41m{}\x1b[0m", disp); w_b += cw; }
        } else if change.tag() == ChangeTag::Equal {
            let v = change.value();
            let cw = UnicodeWidthStr::width(v);
            if w_b + cw <= COL_WIDTH { print!("{}", v); w_b += cw; }
        }
    }
}

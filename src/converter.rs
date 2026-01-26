use opencc_rust::*;
use std::fs::File;
use std::io::{self, Write, Read};
use regex::Regex;
use crate::utils::is_srt_structure; // 引入隔壁 utils 的功能

// 轉換並印出上下對齊排版
pub fn run_conversion_full_view(
    converter: &OpenCC, 
    input: &str, 
    output: &str, 
    regex_rules: &[(Regex, String)], 
    phrase_mode: bool
) -> io::Result<()> {
    let mut f = File::open(input)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    let content = String::from_utf8(buffer).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "非 UTF-8"))?.replace('\u{feff}', "");
    let mut writer = File::create(output)?;
    
    let mode_desc = if phrase_mode { "詞彙模式 (S2TWP)" } else { "純字體模式 (S2T)" };
    println!("\n\x1b[1;36m📂 轉換開始 [{}]: {}\x1b[0m", mode_desc, input);
    println!("{}", "\x1b[90m━\x1b[0m".repeat(80));

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        if is_srt_structure(line) {
            println!("\x1b[90m{:04} | [ OK ] |\x1b[0m {}", line_num, line);
            writeln!(writer, "{}", line)?;
        } else {
            let mut converted = converter.convert(line);
            for (re, replacement) in regex_rules { 
                converted = re.replace_all(&converted, replacement).to_string(); 
            }
            println!("\x1b[90m{:04} | [ OK ] |\x1b[0m 原文: {}", line_num, line);
            println!("\x1b[90m     |        |\x1b[0m 譯文: {}", highlight_diff_blue(line, &converted));
            println!("\x1b[90m     |        └──────────────────────────────────\x1b[0m");
            writeln!(writer, "{}", converted)?;
        }
    }
    Ok(())
}

// 藍色高亮
pub fn highlight_diff_blue(src: &str, dst: &str) -> String {
    let src_chars: Vec<char> = src.chars().collect();
    dst.chars().enumerate().map(|(i, c)| {
        if i < src_chars.len() && c == src_chars[i] { c.to_string() }
        else { format!("\x1b[34m{}\x1b[0m", c) }
    }).collect()
}

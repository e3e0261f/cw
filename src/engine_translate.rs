use crate::checker;
use crate::rules_stay_raw::RawGuard;
use opencc_rust::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run_safe_translate(
    use_phrase_mode: bool,
    input: &str,
    output: &str,
    apply_fix: bool,
) -> io::Result<Vec<(usize, String, String)>> {
    let config = if use_phrase_mode {
        DefaultConfig::S2TWP
    } else {
        DefaultConfig::S2T
    };
    let converter = OpenCC::new(config).expect("OpenCC 啟動失敗");
    let guard = RawGuard::new();
    let reader = BufReader::new(File::open(input)?);
    let mut writer = File::create(output)?;
    let mut translated_pairs = Vec::new();
    let mut current_section = String::new();

    for (idx, line) in reader.lines().enumerate() {
        let mut l = line?;
        if l.starts_with('\u{feff}') {
            l = l.replace('\u{feff}', "");
        }

        // 處理 Section 標題 (如 ASS 的 [Events])
        if guard.section_re.is_match(l.trim()) {
            current_section = l.trim().to_string();
            writeln!(writer, "{}", l)?;
            // 同樣存入 pairs 確保行號連續
            translated_pairs.push((idx + 1, l.clone(), l.clone()));
            continue;
        } // <--- 這裡結束 section 處理

        // 執行翻譯
        let translated = translate_single_line(&converter, &guard, &l, &current_section);

        // --- 核心修改：徹底全量收集，不加任何 if 过滤 ---
        translated_pairs.push((idx + 1, l.clone(), translated.clone()));

        // 寫入檔案
        writeln!(writer, "{}", translated)?;
    } // <--- 這裡結束 for 循環

    if apply_fix {
        writeln!(writer)?;
    }
    Ok(translated_pairs)
} // <--- 這裡結束函數

pub fn translate_single_line(conv: &OpenCC, guard: &RawGuard, line: &str, section: &str) -> String {
    if guard.is_forbidden_zone(line, section) || checker::is_srt_structure(line) {
        return line.to_string();
    }
    if (line.starts_with("Dialogue:") || line.starts_with("Comment:")) && section == "[Events]" {
        let (meta, content) = guard.split_ass_line(line);
        return format!("{}{}", meta, translate_content(conv, guard, content));
    }
    translate_content(conv, guard, line)
}

fn translate_content(conv: &OpenCC, guard: &RawGuard, text: &str) -> String {
    let mut last_end = 0;
    let mut result = String::new();
    for cap in guard.tag_re.find_iter(text) {
        let pre = &text[last_end..cap.start()];
        if !pre.is_empty() {
            result.push_str(&conv.convert(pre));
        }
        result.push_str(cap.as_str());
        last_end = cap.end();
    }
    result.push_str(&conv.convert(&text[last_end..]));
    result
}

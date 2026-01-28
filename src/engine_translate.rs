use opencc_rust::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use crate::checker;
use crate::rules_stay_raw::RawGuard;

pub fn run_safe_translate(use_phrase_mode: bool, input: &str, output: &str, apply_fix: bool) -> io::Result<Vec<(usize, String, String)>> {
    let config = if use_phrase_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T };
    let converter = OpenCC::new(config).expect("OpenCC 啟動失敗");
    let guard = RawGuard::new();
    let reader = BufReader::new(File::open(input)?);
    let mut writer = File::create(output)?;
    let mut translated_pairs = Vec::new();
    let mut current_section = String::new();

    for (idx, line) in reader.lines().enumerate() {
        let mut l = line?;
        if l.starts_with('\u{feff}') { l = l.replace('\u{feff}', ""); }
        if guard.section_re.is_match(l.trim()) {
            current_section = l.trim().to_string();
            writeln!(writer, "{}", l)?;
            continue;
        }
        let translated = translate_single_line(&converter, &guard, &l, &current_section);
        if translated != l && !checker::is_srt_structure(&l) {
            translated_pairs.push((idx + 1, l.clone(), translated.clone()));
        }
        writeln!(writer, "{}", translated)?;
    }
    if apply_fix { writeln!(writer)?; }
    Ok(translated_pairs)
}

pub fn translate_single_line(conv: &OpenCC, guard: &RawGuard, line: &str, section: &str) -> String {
    if guard.is_forbidden_zone(line, section) || checker::is_srt_structure(line) { return line.to_string(); }
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
        if !pre.is_empty() { result.push_str(&conv.convert(pre)); }
        result.push_str(cap.as_str());
        last_end = cap.end();
    }
    result.push_str(&conv.convert(&text[last_end..]));
    result
}

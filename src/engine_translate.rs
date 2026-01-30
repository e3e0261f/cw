use opencc_rust::*;
use std::fs;
use std::io::{self, Write};
use crate::{checker, rules_stay_raw::RawGuard};
use chardetng::EncodingDetector;

pub fn run_safe_translate(p_mode: bool, input: &str, output: &str, fix: bool) -> io::Result<Vec<(usize, String, String)>> {
    let conv = OpenCC::new(if p_mode { DefaultConfig::S2TWP } else { DefaultConfig::S2T }).unwrap();
    let guard = RawGuard::new();
    
    let raw_bytes = fs::read(input)?;
    let mut detector = EncodingDetector::new();
    detector.feed(&raw_bytes, true);
    let encoding = detector.guess(None, true);
    let (content, _, _) = encoding.decode(&raw_bytes);

    let mut writer = fs::File::create(output)?;
    let mut pairs = Vec::new();
    let mut section = String::new();

    for (i, line) in content.lines().enumerate() {
        let mut l = line.replace('\u{feff}', "");
        l = l.trim_end().to_string(); 
        if guard.section_re.is_match(l.trim()) { section = l.trim().to_string(); }
        let trans = translate_single_line(&conv, &guard, &l, &section);
        pairs.push((i + 1, l.clone(), trans.clone()));
        writeln!(writer, "{}", trans)?;
    }
    if fix { writeln!(writer)?; }
    Ok(pairs)
}

pub fn translate_single_line(conv: &OpenCC, guard: &RawGuard, l: &str, s: &str) -> String {
    if guard.is_forbidden_zone(l, s) || checker::is_srt_structure(l) { return l.to_string(); }
    if (l.starts_with("Dialogue:") || l.starts_with("Comment:")) && s == "[Events]" {
        let (m, c) = guard.split_ass_line(l);
        return format!("{}{}", m, translate_content(conv, guard, c));
    }
    translate_content(conv, guard, l)
}

fn translate_content(conv: &OpenCC, guard: &RawGuard, text: &str) -> String {
    let mut last = 0;
    let mut res = String::new();
    for cap in guard.tag_re.find_iter(text) {
        res.push_str(&conv.convert(&text[last..cap.start()]));
        res.push_str(cap.as_str());
        last = cap.end();
    }
    res.push_str(&conv.convert(&text[last..]));
    res
}

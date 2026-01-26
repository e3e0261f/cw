use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    t.is_empty() || t.contains("-->") || t.chars().all(|c| c.is_ascii_digit())
}

pub fn is_chinese(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

pub fn strip_chinese(s: &str) -> String {
    s.chars().filter(|&c| !is_chinese(c)).collect()
}

pub fn is_similar_one_char(s1: &str, s2: &str) -> bool {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    if v1.len() != v2.len() { return false; }
    v1.iter().zip(v2.iter()).filter(|(a, b)| a != b).count() == 1
}

pub fn process_audit(
    path_a: &str,
    path_b: &str,
    log_p: &PathBuf,
    ac: &AhoCorasick,
    typo_map: &HashMap<String, String>,
    patterns: &[String],
    regex_rules: &[(Regex, String)],
) -> io::Result<(Vec<String>, Vec<String>)> {
    let reader_a = BufReader::new(File::open(path_a)?);
    let reader_b = BufReader::new(File::open(path_b)?);
    let mut log_f = File::create(log_p)?;
    let mut v_errs = Vec::new();
    let mut advices = Vec::new();

    let mut zh_terms: HashMap<String, usize> = HashMap::new();
    let mut en_names: HashMap<String, usize> = HashMap::new();
    
    // 讀取檔案
    let lines_a: Vec<String> = reader_a.lines().map(|l| l.unwrap()).collect();
    let lines_b: Vec<String> = reader_b.lines().map(|l| l.unwrap()).collect();

    let mut subtitle_groups: Vec<Vec<String>> = Vec::new();
    let mut current_group: Vec<String> = Vec::new();

    writeln!(log_f, "--- CW 詳細稽核日誌 ---\n")?;

    for (i, b_line) in lines_b.iter().enumerate() {
        if i >= lines_a.len() { break; }
        let a_line = &lines_a[i];
        let line_num = i + 1;

        if is_srt_structure(b_line) {
            if !current_group.is_empty() {
                subtitle_groups.push(current_group.clone());
                current_group.clear();
            }
            if a_line != b_line { v_errs.push(format!("L{} 結構不匹配", line_num)); }
        } else {
            current_group.push(b_line.clone());
            if strip_chinese(a_line) != strip_chinese(b_line) {
                v_errs.push(format!("L{} 非中文內容變動", line_num));
            }

            // 1. 錯字檢查
            for mat in ac.find_iter(b_line) {
                let wrong = &patterns[mat.pattern().as_usize()];
                let right = typo_map.get(wrong).unwrap();
                advices.push(format!("L{} [錯字] '{}' -> '{}'", line_num, wrong, right));
            }

            // 2. Regex 規則
            for (re, tip) in regex_rules {
                if re.is_match(b_line) {
                    advices.push(format!("L{} [匹配] {} -> {}", line_num, re.as_str(), tip));
                }
            }

            // 3. 一致性收集
            let chars: Vec<char> = b_line.chars().filter(|&c| is_chinese(c)).collect();
            for len in 3..=4 {
                for win in chars.windows(len) {
                    let term: String = win.iter().collect();
                    *zh_terms.entry(term).or_insert(0) += 1;
                }
            }
        }
    }

    // 4. 冗餘偵測
    if !subtitle_groups.is_empty() {
        for candidate in &subtitle_groups[0] {
            if !candidate.trim().is_empty() && subtitle_groups.iter().all(|g| g.contains(candidate)) {
                advices.push(format!("[警告] 全片冗餘: '{}'", candidate));
            }
        }
    }

    // 5. 中文相似度
    let zh_list: Vec<&String> = zh_terms.keys().collect();
    for i in 0..zh_list.len() {
        for j in i + 1..zh_list.len() {
            if is_similar_one_char(zh_list[i], zh_list[j]) {
                advices.push(format!("[一致性] 中文譯名不一: '{}' / '{}'", zh_list[i], zh_list[j]));
            }
        }
    }

    Ok((v_errs, advices))
}

use regex::Regex;

pub struct RawGuard {
    pub tag_re: Regex,
    pub section_re: Regex,
}

impl RawGuard {
    pub fn new() -> Self {
        Self {
            // 保護 \N, \h, {控制代碼}, <標籤>
            tag_re: Regex::new(r"(\\N|\\h|\{.*?\}|<.*?>)").unwrap(),
            section_re: Regex::new(r"^\[.*\]$").unwrap(),
        }
    }

    pub fn is_forbidden_zone(&self, line: &str, current_section: &str) -> bool {
        let t = line.trim();
        if t.is_empty() || t.starts_with(';') { return true; } // 保護註釋
        if current_section == "[V4+ Styles]" && t.starts_with("Style:") { return true; }
        if current_section == "[Script Info]" || current_section == "[Aegisub Project Garbage]" { return true; }
        false
    }

    pub fn split_ass_line<'a>(&self, line: &'a str) -> (&'a str, &'a str) {
        let mut commas = 0;
        let mut split_idx = 0;
        for (i, c) in line.char_indices() {
            if c == ',' {
                commas += 1;
                if commas == 9 { split_idx = i + 1; break; }
            }
        }
        if split_idx == 0 { (line, "") } else { (&line[..split_idx], &line[split_idx..]) }
    }
}

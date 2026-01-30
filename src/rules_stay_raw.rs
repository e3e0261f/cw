use regex::Regex;
pub struct RawGuard { pub tag_re: Regex, pub section_re: Regex }
impl RawGuard {
    pub fn new() -> Self {
        Self {
            tag_re: Regex::new(r"(\\N|\\h|\{.*?\}|<.*?>)").unwrap(),
            section_re: Regex::new(r"^\[.*\]$").unwrap(),
        }
    }
    pub fn is_forbidden_zone(&self, l: &str, s: &str) -> bool {
        let t = l.trim();
        t.is_empty() || t.starts_with(';') || (s == "[V4+ Styles]" && t.starts_with("Style:")) || s == "[Script Info]"
    }
    pub fn split_ass_line<'a>(&self, l: &'a str) -> (&'a str, &'a str) {
        let mut c = 0;
        for (i, ch) in l.char_indices() {
            if ch == ',' { c += 1; if c == 9 { return (&l[..i+1], &l[i+1..]); } }
        }
        (l, "")
    }
}

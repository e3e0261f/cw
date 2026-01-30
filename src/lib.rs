// ==========================================
// CW Subtitle Brain Module (v1.9.3)
// ==========================================

pub mod report_format {
    use std::path::PathBuf;
    use std::time::Duration;

    #[derive(PartialEq, Debug, Clone)]
    pub enum ResultStatus {
        Success,
        VerifWarning,
        ConvertError,
    }

    #[derive(Debug, Clone)]
    pub struct SubtitleIssue {
        pub line: usize,
        pub message: String,
    }

    #[derive(Debug)]
    pub struct FileReport {
        pub input_name: String,
        pub output_name: String,
        pub temp_log_path: PathBuf,
        pub status: ResultStatus,
        pub issues: Vec<SubtitleIssue>,
        pub translated_pairs: Vec<(usize, String, String)>,
        pub duration: Duration,
    }
}

pub mod core {
    use crate::report_format::{ResultStatus, SubtitleIssue};
    use chardetng::EncodingDetector;
    use chrono::Local;
    use opencc_rust::{DefaultConfig, OpenCC};
    use regex::Regex;
    use std::collections::HashMap;
    use std::env;
    use std::fs::{self, File, OpenOptions};
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    // --- [ 功能塊: 配置 ] ---
    pub struct Config {
        pub discord_webhook: String,
        pub phrase_mode: bool,
        pub verbosity: u32,
        pub auto_discord: bool,
        pub log_directory: String,
        pub log_file_prefix: String,
        pub log_file_date_format: String,
        pub log_level: String,
        pub log_max_size_mb: u64,
        pub log_backup_count: u32,
        pub mention_id: String,
        pub discord_interval: u64,
        pub translate_error: bool,
        pub show_stats: bool,
        pub discord_show_errors: bool,
        pub full_preview: bool,
    }

    impl Config {
        pub fn load() -> Self {
            let mut exe_path = env::current_exe().unwrap_or_default();
            exe_path.pop();
            let cfg_path = exe_path.join("cw.cfg");
            let mut map = HashMap::new();
            if let Ok(content) = fs::read_to_string(&cfg_path) {
                for line in content.lines() {
                    let clean = line.split('#').next().unwrap_or("").trim();
                    if let Some((k, v)) = clean.split_once('=') {
                        map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
                    }
                }
            }
            Self {
                discord_webhook: map.get("discord_webhook").cloned().unwrap_or_default(),
                phrase_mode: map.get("phrase_mode").map(|v| v == "true").unwrap_or(false),
                verbosity: map
                    .get("verbosity")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1),
                auto_discord: map
                    .get("auto_discord")
                    .map(|v| v == "true")
                    .unwrap_or(false),
                log_directory: map
                    .get("log_directory")
                    .cloned()
                    .unwrap_or_else(|| "./logs".to_string()),
                log_file_prefix: map
                    .get("log_file_prefix")
                    .cloned()
                    .unwrap_or_else(|| "cw".to_string()),
                log_file_date_format: map
                    .get("log_file_date_format")
                    .cloned()
                    .unwrap_or_else(|| "%Y-%m-%d".to_string()),
                log_level: map
                    .get("log_level")
                    .cloned()
                    .unwrap_or_else(|| "INFO".to_string()),
                log_max_size_mb: map
                    .get("log_max_size")
                    .and_then(|v| v.replace("MB", "").trim().parse().ok())
                    .unwrap_or(10),
                log_backup_count: map
                    .get("log_backup_count")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                mention_id: map.get("mention_id").cloned().unwrap_or_default(),
                discord_interval: map
                    .get("discord_interval")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2),
                translate_error: map
                    .get("translate_error")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                show_stats: map.get("show_stats").map(|v| v == "true").unwrap_or(false),
                discord_show_errors: map
                    .get("discord_show_errors")
                    .map(|v| v == "true")
                    .unwrap_or(false),
                full_preview: map
                    .get("full_preview")
                    .map(|v| v == "true")
                    .unwrap_or(false),
            }
        }

        pub fn generate_default() -> io::Result<()> {
            let mut path = env::current_exe().unwrap_or_default();
            path.pop();
            let cfg_path = path.join("cw.cfg");
            let template = include_str!("../default_cw.cfg");
            fs::write(cfg_path, template)?;
            println!("\x1b[1;32m✨ 已生成預設 cw.cfg\x1b[0m");
            Ok(())
        }
    }

    // --- [ 功能塊: 翻譯核心 ] ---
    pub struct RawGuard {
        pub tag_re: Regex,
        pub section_re: Regex,
    }
    impl Default for RawGuard {
        fn default() -> Self {
            Self::new()
        }
    }
    impl RawGuard {
        pub fn new() -> Self {
            Self {
                tag_re: Regex::new(r"(\\N|\\h|\{.*?\}|<.*?>)").unwrap(),
                section_re: Regex::new(r"^\[.*\]$").unwrap(),
            }
        }
        pub fn is_forbidden_zone(&self, l: &str, s: &str) -> bool {
            let t = l.trim();
            t.is_empty()
                || t.starts_with(';')
                || (s == "[V4+ Styles]" && t.starts_with("Style:"))
                || s == "[Script Info]"
        }
        pub fn split_ass_line<'a>(&self, l: &'a str) -> (&'a str, &'a str) {
            let mut c = 0;
            for (i, ch) in l.char_indices() {
                if ch == ',' {
                    c += 1;
                    if c == 9 {
                        return (&l[..i + 1], &l[i + 1..]);
                    }
                }
            }
            (l, "")
        }
    }

    pub fn run_safe_translate(
        p_mode: bool,
        input: &str,
        output: &str,
        fix: bool,
    ) -> io::Result<Vec<(usize, String, String)>> {
        let conv = OpenCC::new(if p_mode {
            DefaultConfig::S2TWP
        } else {
            DefaultConfig::S2T
        })
        .unwrap();
        let guard = RawGuard::new();
        let raw_bytes = fs::read(input)?;
        let mut detector = EncodingDetector::new();
        detector.feed(&raw_bytes, true);
        let encoding = detector.guess(None, true);
        let (content, _, _) = encoding.decode(&raw_bytes);
        let mut writer = File::create(output)?;
        let mut pairs = Vec::new();
        let mut section = String::new();
        for (i, line) in content.lines().enumerate() {
            let l = line.replace('\u{feff}', "").trim_end().to_string();
            if guard.section_re.is_match(l.trim()) {
                section = l.trim().to_string();
            }
            let trans = translate_single_line(&conv, &guard, &l, &section);
            pairs.push((i + 1, l, trans.clone()));
            writeln!(writer, "{}", trans)?;
        }
        if fix {
            writeln!(writer)?;
        }
        Ok(pairs)
    }

    pub fn translate_single_line(conv: &OpenCC, guard: &RawGuard, l: &str, s: &str) -> String {
        if guard.is_forbidden_zone(l, s) || is_srt_structure(l) {
            return l.to_string();
        }
        if (l.starts_with("Dialogue:") || l.starts_with("Comment:")) && s == "[Events]" {
            let (m, c) = guard.split_ass_line(l);
            let mut last = 0;
            let mut res = String::new();
            for cap in guard.tag_re.find_iter(c) {
                res.push_str(&conv.convert(&c[last..cap.start()]));
                res.push_str(cap.as_str());
                last = cap.end();
            }
            res.push_str(&conv.convert(&c[last..]));
            return format!("{}{}", m, res);
        }
        let mut last = 0;
        let mut res = String::new();
        for cap in guard.tag_re.find_iter(l) {
            res.push_str(&conv.convert(&l[last..cap.start()]));
            res.push_str(cap.as_str());
            last = cap.end();
        }
        res.push_str(&conv.convert(&l[last..]));
        res
    }

    // --- [ 功能塊: 診斷 ] ---
    pub fn is_srt_structure(l: &str) -> bool {
        let t = l.trim();
        t.is_empty() || t.contains("-->") || (t.chars().all(|c| c.is_ascii_digit()) && t.len() < 10)
    }

    pub fn diagnose_file(path: &str, _translate: bool) -> Vec<SubtitleIssue> {
        let mut issues = Vec::new();
        let content = fs::read_to_string(path)
            .unwrap_or_default()
            .replace('\u{feff}', "");
        if path.to_lowercase().ends_with(".ass") {
            return issues;
        }
        if needs_trailing_newline_fix(path) {
            issues.push(SubtitleIssue {
                line: 0,
                message: "檔案末端損壞：缺少 SRT 規範空行".to_string(),
            });
        }
        if let Ok(srt) = skrt::Srt::try_parse(&content) {
            for (idx, sub) in srt.subtitles().iter().enumerate() {
                if sub.start() > sub.end() {
                    issues.push(SubtitleIssue {
                        line: idx + 1,
                        message: "時間邏輯錯誤：結束早於開始".to_string(),
                    });
                }
            }
        }
        issues
    }

    pub fn needs_trailing_newline_fix(path: &str) -> bool {
        if let Ok(data) = fs::read(path) {
            if data.is_empty() {
                return true;
            }
            let len = data.len();
            return len < 2
                || data[len - 1] != b'\n'
                || (data[len - 2] != b'\n' && data[len - 2] != b'\r');
        }
        false
    }

    pub fn create_log(
        p_a: &str,
        p_b: &str,
        log_p: &PathBuf,
        status: &ResultStatus,
        _max: u64,
        _count: u32,
        issues: &[SubtitleIssue],
    ) -> io::Result<()> {
        let mut f = OpenOptions::new().create(true).append(true).open(log_p)?;
        writeln!(
            f,
            "\n批次：{} | 原檔：{} | 成果：{}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            p_a,
            p_b
        )?;
        for iss in issues {
            writeln!(f, "🛠️ L{:03} {}", iss.line, iss.message)?;
        }
        writeln!(f, "[ 狀態：{:?} ]\n{}", status, "-".repeat(40))?;
        Ok(())
    }

    // --- [ 功能塊: 下載器 ] ---
    pub struct MegaDownloader;
    impl MegaDownloader {
        pub fn scout_target(url: &str) -> Result<String, String> {
            let output = Command::new("megals")
                .arg(url)
                .output()
                .map_err(|e: io::Error| e.to_string())?;
            let list = String::from_utf8_lossy(&output.stdout);
            let mut candidates: Vec<String> = Vec::new();
            for line in list.lines() {
                let l = line.to_lowercase();
                if l.ends_with(".srt") || l.ends_with(".ass") {
                    candidates.push(line.to_string());
                }
            }
            if candidates.is_empty() {
                return Err("無字幕檔".to_string());
            }
            Ok(candidates
                .iter()
                .find(|c| c.to_lowercase().contains("cn"))
                .unwrap_or(&candidates[0])
                .clone())
        }
        pub fn fetch_file(url: &str, target: &str, dest: &Path) -> Result<PathBuf, String> {
            let s = Command::new("megadl")
                .arg("--path")
                .arg(dest)
                .arg(url)
                .status()
                .map_err(|e: io::Error| e.to_string())?;
            if !s.success() {
                return Err("下載失敗".to_string());
            }
            Ok(dest.join(Path::new(target).file_name().unwrap()))
        }
    }
}

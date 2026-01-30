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

pub struct FileReport {
    pub input_name: String,
    pub output_name: String,
    pub temp_log_path: PathBuf,
    pub status: ResultStatus,
    pub issues: Vec<SubtitleIssue>, // 統一所有錯誤與建議
    pub translated_pairs: Vec<(usize, String, String)>,
    pub duration: Duration,
}

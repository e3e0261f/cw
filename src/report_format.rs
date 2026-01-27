use std::path::PathBuf;
use std::time::Duration;

#[derive(PartialEq, Debug)]
pub enum ResultStatus {
    Success,
    VerifWarning,
    ConvertError, // 轉換失敗狀態
}

pub struct FileReport {
    pub input_name: String,
    pub output_name: String,
    pub temp_log_path: PathBuf,
    pub status: ResultStatus,
    pub verif_errors: Vec<String>,
    pub translated_pairs: Vec<(usize, String, String)>,
    pub duration: Duration,
}

use std::path::PathBuf;

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum ResultStatus {
    Success,
    VerifWarning,
    ConvertError,
}

#[allow(dead_code)] // 這裡保護整個結構體，確保數據欄位不報警告
pub struct FileReport {
    pub input_name: String,
    pub output_name: String,
    pub temp_log_path: PathBuf,
    pub status: ResultStatus,
    pub verif_errors: Vec<String>,
    pub translated_pairs: Vec<(usize, String, String)>,
}

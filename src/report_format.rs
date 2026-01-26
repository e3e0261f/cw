use std::path::PathBuf;

#[derive(PartialEq)]
pub enum ResultStatus {
    Success,
    VerifWarning,
    ConvertError,
}

pub struct FileReport {
    pub input_name: String,
    pub output_name: String,
    pub temp_log_path: PathBuf,
    pub status: ResultStatus,
    pub verif_errors: Vec<String>,
    pub translated_pairs: Vec<(usize, String, String)>, // 存儲變動行
}

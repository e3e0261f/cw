use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

// 這裡定義從 typos.json 讀取的結構
#[derive(Deserialize)]
pub struct TypoData {
    pub typos: HashMap<String, String>,
    #[serde(default)] // 如果 json 沒寫這項，就給空列表
    pub regex: HashMap<String, String>,
}

// 這裡定義每個檔案的處理報告
pub struct FileReport {
    pub input_name: String,
    pub output_name: String,
    pub temp_log_path: PathBuf,
    pub status: ResultStatus,
    pub verif_errors: Vec<String>,
    pub quality_advices: Vec<String>,
}

// 處理狀態
pub enum ResultStatus {
    Success,
    ConvertError,
    VerifWarning,
}

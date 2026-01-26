use serde::{Deserialize, Serialize}; // 統一在這裡導入一次即可
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub webhook_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TypoData {
    pub typos: HashMap<String, String>,
    pub regex_overrides: HashMap<String, String>,
}

#[derive(Clone)]
pub struct FileReport {
    pub input_name: String,
    pub status: ResultStatus,
    pub issues_summary: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ResultStatus {
    Success,
    Warning,
    Error,
}

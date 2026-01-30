use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MegaDownloader;

impl MegaDownloader {
    pub fn scout_target(url: &str) -> Result<String, String> {
        let output = Command::new("megals").arg(url).output()
            .map_err(|e| format!("執行 megals 失敗: {}", e))?;
        let list = String::from_utf8_lossy(&output.stdout);
        let mut candidates: Vec<String> = Vec::new();
        for line in list.lines() {
            let l = line.to_lowercase();
            if l.ends_with(".srt") || l.ends_with(".ass") { candidates.push(line.to_string()); }
        }
        if candidates.is_empty() { return Err("Mega 連結中無字幕檔案".to_string()); }
        let target = candidates.iter().find(|c| c.to_lowercase().contains("cn")).unwrap_or(&candidates[0]);
        Ok(target.clone())
    }

    pub fn fetch_file(url: &str, target_path_in_mega: &str, dest_dir: &Path) -> Result<PathBuf, String> {
        let status = Command::new("megadl").arg("--path").arg(dest_dir).arg(url).status()
            .map_err(|e| format!("啟動 megadl 失敗: {}", e))?;
        if !status.success() { return Err("megadl 下載失敗".to_string()); }
        let file_name = Path::new(target_path_in_mega).file_name().ok_or("無法解析檔名")?;
        Ok(dest_dir.join(file_name))
    }
}

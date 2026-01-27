use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::report_format::ResultStatus;
use crate::checker; // 調用檢查員
use chrono::Local;

pub fn create_detailed_log(
    path_a: &str,
    path_b: &str,
    log_path: &PathBuf,
    status: &ResultStatus,
) -> io::Result<()> {
    let file_a = File::open(path_a)?;
    let file_b = File::open(path_b)?;
    let reader_a = BufReader::new(file_a);
    let reader_b = BufReader::new(file_b);
    
    let mut log_f = File::create(log_path)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    writeln!(log_f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    writeln!(log_f, "🚀 CW 字幕稽核詳細日誌 | 生成時間：{}", now)?;
    writeln!(log_f, "原始檔案：{}\n輸出檔案：{}", path_a, path_b)?;
    writeln!(log_f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")?;

    for (idx, (l_a, l_b)) in reader_a.lines().zip(reader_b.lines()).enumerate() {
        let a = l_a.unwrap_or_default();
        let b = l_b.unwrap_or_default();
        let line_num = idx + 1;
        let ts = Local::now().format("%H:%M:%S%.3f");

        // 使用 checker 的邏輯
        let tag = if checker::is_srt_structure(&a) { "[結構]" } else { "[內容]" };

        if a == b {
            writeln!(log_f, "[{}] L{:03} {} 一致", ts, line_num, tag)?;
        } else {
            writeln!(log_f, "[{}] L{:03} {} 【發現變動】", ts, line_num, tag)?;
            writeln!(log_f, "      原: {}", a)?;
            writeln!(log_f, "      譯: {}", b)?;
        }
    }
    
    writeln!(log_f, "\n[ 最終狀態: {:?} ]", status)?;
    Ok(())
}

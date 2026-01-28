use reqwest::blocking::{multipart, Client};
use std::{fs, thread, time::Duration};
use std::path::Path;
use crate::report_format::{FileReport, ResultStatus};

const DISCORD_LIMIT: usize = 1900; // 保守限制在 1900 字

pub fn execute(
    webhook_url: &str, 
    intro_text: Option<&str>, 
    mention_id: &str, 
    interval: u64,
    reports: &[FileReport]
) -> Result<(), String> {
    let client = Client::new();

    // 1. 準備完整的長文字內容
    let mut full_content = String::new();
    if !mention_id.is_empty() {
        full_content.push_str(&format!("🔔 **任務提醒**：<@{}>\n", mention_id));
    }
    if let Some(text) = intro_text {
        full_content.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        full_content.push_str(text);
        full_content.push_str("\n");
    }
    full_content.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    full_content.push_str("✅ **處理清單總結**：\n");
    for r in reports {
        let emoji = if r.status == ResultStatus::Success { "🔹" } else { "🔸" };
        full_content.push_str(&format!("{} `{}` (變動: {} 行)\n", emoji, r.input_name, r.translated_pairs.len()));
    }

    // 2. 執行智慧切分
    let chunks = split_content_safely(&full_content);
    let total_chunks = chunks.len();

    // 3. 分段發送
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == total_chunks - 1;
        let mut form = multipart::Form::new().text("content", chunk.clone());

        // 只有最後一棒才掛載附件 (最多 10 個)
        if is_last {
            let mut count = 0;
            for r in reports {
                if r.status != ResultStatus::ConvertError {
                    let path = Path::new(&r.output_name);
                    if path.exists() {
                        if let Ok(data) = fs::read(path) {
                            let name = path.file_name().unwrap().to_string_lossy().to_string();
                            form = form.part(format!("file{}", count), multipart::Part::bytes(data).file_name(name));
                            count += 1;
                        }
                    }
                }
                if count >= 10 { break; }
            }
        }

        // 執行 POST
        let resp = client.post(webhook_url).multipart(form).send()
            .map_err(|e| format!("網路連線失敗: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Discord 拒絕 (代碼: {})", resp.status()));
        }

        // 模擬人手速間隔
        if !is_last {
            thread::sleep(Duration::from_secs(interval));
        }
    }

    Ok(())
}

/// 智慧切分：換行 > 空格 > URL 避讓
fn split_content_safely(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.chars().count() > DISCORD_LIMIT {
        let mut split_pos = DISCORD_LIMIT;
        let current_chunk = remaining.chars().take(DISCORD_LIMIT).collect::<String>();

        // 1. 找最後一個換行
        if let Some(pos) = current_chunk.rfind('\n') {
            split_pos = pos;
        } 
        // 2. 找最後一個空格
        else if let Some(pos) = current_chunk.rfind(' ') {
            split_pos = pos;
        }

        // 3. URL 避讓邏輯：檢查切割點是否正在切開 http...
        let temp_cut = &remaining[..split_pos];
        if let Some(url_start) = temp_cut.rfind("http") {
            // 如果從 http 到切口之間沒有空格，說明 URL 被切斷了
            if !remaining[url_start..split_pos].contains(' ') {
                split_pos = url_start; // 將整段 URL 移到下一塊
            }
        }

        // 執行切割
        let (part, rest) = remaining.split_at(split_pos);
        chunks.push(part.trim().to_string());
        remaining = rest.trim();
    }
    
    if !remaining.is_empty() {
        chunks.push(remaining.to_string());
    }
    chunks
}

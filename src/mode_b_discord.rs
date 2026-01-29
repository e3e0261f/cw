use reqwest::blocking::{multipart, Client};
use std::{fs, thread, time::Duration};
use std::path::Path;
use crate::report_format::{FileReport, ResultStatus};

const DISCORD_LIMIT: usize = 1900;

pub fn execute(
    webhook_url: &str, 
    intro_text: Option<&str>, 
    mention_id: &str, 
    interval: u64,
    reports: &[FileReport]
) -> Result<(), String> {
    let client = Client::new();
    let mut full_content = String::new();

    if let Some(text) = intro_text {
        full_content.push_str("\n");
        full_content.push_str(text);
        full_content.push_str("\n");
    if !mention_id.is_empty() { full_content.push_str(&format!("<@{}>", mention_id)); }
    }
    full_content.push_str("\n");
 //    for r in reports {
 //       let emoji = if r.status == ResultStatus::Success { "🔹" } else { "🔸" };
 //       full_content.push_str(&format!("{} `{}` (變動: {} 行)\n", emoji, r.input_name, r.translated_pairs.len()));
 //   }

    let chunks = split_content_safely(&full_content);
    println!("\n📡 正在啟動智慧傳送車...");
    println!("   訊息總長：{} 字元 | 預計分段：{} 段", full_content.chars().count(), chunks.len());

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        let mut form = multipart::Form::new().text("content", chunk.clone());

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

        let has_url = chunk.contains("http");
        print!("[發送中] 第 {}/{} 段 ({} 字元) {}...", i+1, chunks.len(), chunk.chars().count(), if has_url {"(帶URL)"} else {""});
        
        let resp = client.post(webhook_url).multipart(form).send()
            .map_err(|e| format!("網路連線失敗: {}", e))?;

        println!(" [HTTP {}]", resp.status().as_u16());

        if !is_last {
            println!("   [延時] 等待 {} 秒以模擬人工手速...", interval);
            thread::sleep(Duration::from_secs(interval));
        }
    }
    Ok(())
}

fn split_content_safely(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut remaining = text;
    while remaining.chars().count() > DISCORD_LIMIT {
        let mut split_pos = DISCORD_LIMIT;
        let current_chunk = remaining.chars().take(DISCORD_LIMIT).collect::<String>();
        if let Some(pos) = current_chunk.rfind('\n') { split_pos = pos; } 
        else if let Some(pos) = current_chunk.rfind(' ') { split_pos = pos; }
        
        let temp_cut = &remaining[..split_pos];
        if let Some(url_start) = temp_cut.rfind("http") {
            if !remaining[url_start..split_pos].contains(' ') { split_pos = url_start; }
        }
        let (part, rest) = remaining.split_at(split_pos);
        chunks.push(part.trim().to_string());
        remaining = rest.trim();
    }
    if !remaining.is_empty() { chunks.push(remaining.to_string()); }
    chunks
}

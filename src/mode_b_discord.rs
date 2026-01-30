use reqwest::blocking::{multipart, Client};
use std::{fs, thread, time::Duration};
use std::path::Path;
use crate::report_format::{FileReport, ResultStatus};

const DISCORD_LIMIT: usize = 1950;

pub fn execute(
    webhook_url: &str, 
    intro_text: Option<&str>, 
    mention_id: &str, 
    interval: u64,
    show_stats: bool,
    show_errors: bool,
    reports: &[FileReport]
) -> Result<(), String> {
    let client = Client::new();
    let mut full_content = String::new();

    if let Some(text) = intro_text {
        full_content.push_str(text);
        full_content.push('\n');
    }

    if show_stats {
        for r in reports {
            full_content.push_str(&format!("`{}` (變動: {} 行)\n", r.input_name, r.translated_pairs.len()));
        }
    }

    if show_errors {
        for r in reports {
            // 修正：讀取 SubtitleIssue 的 message
            for issue in &r.issues {
                full_content.push_str(&format!("! {}\n", issue.message));
            }
        }
    }

    if !mention_id.is_empty() {
        full_content.push_str(&format!("<@{}>", mention_id));
    }

    let chunks = split_content_safely(&full_content);
    let chunks_to_send = if chunks.is_empty() { vec!["".to_string()] } else { chunks };

    for (i, chunk) in chunks_to_send.iter().enumerate() {
        let is_last = i == chunks_to_send.len() - 1;
        let mut form = multipart::Form::new().text("content", chunk.clone());

        if is_last {
            let mut count = 0;
            for r in reports {
                if r.status != ResultStatus::ConvertError {
                    if let Ok(data) = fs::read(&r.output_name) {
                        let name = Path::new(&r.output_name).file_name().unwrap().to_string_lossy().to_string();
                        form = form.part(format!("file{}", count), multipart::Part::bytes(data).file_name(name));
                        count += 1;
                    }
                }
                if count >= 10 { break; }
            }
        }
        let _ = client.post(webhook_url).multipart(form).send().map_err(|e| e.to_string())?;
        if !is_last { thread::sleep(Duration::from_secs(interval)); }
    }
    Ok(())
}

fn split_content_safely(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut remaining = text;
    if remaining.is_empty() { return chunks; }
    while remaining.chars().count() > DISCORD_LIMIT {
        let mut split_pos = DISCORD_LIMIT;
        let current_chunk = remaining.chars().take(DISCORD_LIMIT).collect::<String>();
        if let Some(pos) = current_chunk.rfind('\n') { split_pos = pos; } 
        else if let Some(pos) = current_chunk.rfind(' ') { split_pos = pos; }
        let (part, rest) = remaining.split_at(split_pos);
        chunks.push(part.to_string());
        remaining = rest.trim_start();
    }
    if !remaining.is_empty() { chunks.push(remaining.to_string()); }
    chunks
}

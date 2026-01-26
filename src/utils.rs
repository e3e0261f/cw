use reqwest::blocking::Client;
use serde_json::json;

pub fn is_srt_structure(l: &str) -> bool {
    let t = l.trim();
    t.is_empty() || t.contains("-->") || t.chars().all(|c| c.is_ascii_digit())
}

pub fn print_help() {
    println!("\n\x1b[1;36m🚀 CN-TW 助手 (單一檔案模式)\x1b[0m");
    println!("使用方法:");
    println!("  \x1b[32mcw <檔案>\x1b[0m                純字體轉換");
    println!("  \x1b[32mcw -p <檔案>\x1b[0m             開啟詞彙修正");
    println!("  \x1b[35mcw -b <檔案>\x1b[0m             轉換並發送 Discord (讀取 config.json)");
    println!("  \x1b[33mcw -a <原文> <譯文>\x1b[0m      對比兩檔案差異");
}

pub fn send_discord_report(webhook_url: &str, file_name: &str, status_text: &str, color: i32) {
    let client = Client::new();
    let payload = json!({
        "username": "CN-TW 助手",
        "embeds": [{
            "title": format!("🎬 處理報告: {}", file_name),
            "description": status_text,
            "color": color,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });
    let _ = client.post(webhook_url).json(&payload).send();
}


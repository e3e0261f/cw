use reqwest::blocking::{multipart, Client};
use std::fs;
use crate::report_format::{FileReport, ResultStatus};

pub fn execute(
    webhook_url: &str, 
    intro_text: Option<&str>, 
    mention_id: &str, 
    reports: &[FileReport]
) -> Result<(), String> {
    let client = Client::new();

    // 1. 組裝文字訊息 (支援 Discord 的 <@ID> 語法)
    let mut content = format!("🔔 <@{}>\n", mention_id);
    content.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    if let Some(text) = intro_text {
        content.push_str(text);
        content.push_str("\n");
    }
    content.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    // content.push_str("✅ 翻譯任務已由 CW 自動化流程處理完畢。");

    // 2. 準備 Multipart 表單
    let mut form = multipart::Form::new().text("content", content);

    // 3. 附加成功翻譯的檔案 (最多 10 個)
    let mut attached_count = 0;
    for r in reports {
        if r.status == ResultStatus::Success {
            if let Ok(file_data) = fs::read(&r.output_name) {
                let part = multipart::Part::bytes(file_data)
                    .file_name(r.output_name.clone());
                form = form.part(format!("file{}", attached_count), part);
                attached_count += 1;
            }
        }
        if attached_count >= 10 { break; }
    }

    if attached_count == 0 {
        return Err("找不到可發送的成功檔案附件".to_string());
    }

    // 4. 發送請求
    let response = client.post(webhook_url)
        .multipart(form)
        .send()
        .map_err(|e| format!("網路傳輸失敗: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Discord 拒絕 (代碼: {})", response.status()))
    }
}

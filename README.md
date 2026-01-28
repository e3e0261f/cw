# CW - (Production-Grade Localization Tool)

#核心目標：在不觸碰任何非文字結構（如 SRT 時間軸、Markdown 標記）的前提下，實現極高精準度的繁簡轉換並與discord頻道內轉發 和 表情互動。

- 結構保護技術：內建精密規則引擎，確保字幕時間軸、程式碼塊、專有名詞在轉換過程中完整如初。

- 視覺化校對流程：獨創顏色高亮對照模式，翻譯變動一目瞭然，讓二次人工校對效率提升 300%。

- 團隊非同步協作：一鍵打通 Discord Webhook 交付鏈路，實現從本地處理到雲端同步的無縫銜接。

- 工業級效能：由 Rust 驅動，輕鬆處理萬行級文字，拒絕崩潰與延遲。

- SRT 格式对空格、数字、箭头 (-->) 和换行极其敏感。普通的翻译工具（如网页翻译）经常会把时间轴数字翻错，或者把 --> 换成中文箭头。

### 下載地址
* **Linux 版本**: [點此下載最新版 (tar.gz)](https://github.com/e3e0261f/cw/releases/latest/download/cw-linux-x64.tar.gz)
* **Windows 版本**: [點此下載最新版 (zip)](https://github.com/e3e0261f/cw/releases/latest/download/cw-windows-x64.zip)

## 構建安裝

- Rust 1.60+，Cargo

```bash
# 從原始碼安裝
git clone https://github.com/e3e0261f/cw.git
cd cw
cargo install --path .
```

或者直接使用 cargo run：
```bash
cargo run -- test1.srt
```

使用方式
基本用法（翻譯單個檔案）：
```bash
cw test1.srt
```

## 依賴

- opencc-rust（簡繁轉換）
- colored（終端彩色）
- clap（命令列引數）
- unicode-width（中文寬度計算）
- 其他：aho-corasick, rayon, regex 等

## TODO
- 未來的 -b 無感流程：只需一個指令：會自動：下載 -> 翻譯 -> 稽核 -> 傳送。
```Bash
cw --task "MEGA_URL" --text "這是今天的疫苗報告" -b
```
- 對命令列的標準輸入 直接翻譯
- 智慧篩選：它會幫你從 Mega 連結的一堆檔案裡，自動揪出那個帶有 cn 的字幕，這省去了人工挑選的麻煩。

## 貢獻
- 歡迎 PR / Issue！
- 如果你在使用中發現 bug 或有功能建議，直接開 issue 告訴我。

## 許可證
- MIT License
- Made with ❤️ in Rust

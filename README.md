<div align="center">

# 🧙‍♂️ Convert Wizard (CW)
### 極速、精準的專業字幕繁簡轉換工具

[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-blue?style=for-the-badge)](https://github.com/e3e0261f/cw/releases)

<p>
  基於 OpenCC 核心，為追求極致體驗的字幕組與翻譯者打造。
</p>

---
</div>

<!-- TODO_START -->
# 🚀 CW 專案進度表

### ✅ 已完成 (地基穩固)
- [x] 多檔案模組化拆分 (main, audit, rules, ui_style, config, compare, downloader)
- [x] 標籤保護翻譯邏輯
- [x] GitHub 風格紅綠底色對比 (-a)
- [x] 0 警告編譯

### 🌟 未來遠景
- [ ] 與 Discord Bot 對接
- [ ] 自動偵測檔案編碼 (GBK/UTF-8)

### 🛠 待修復的小問題 (精力恢復後再動手)
- [ ] 檔名生成的路徑邏輯優化
- [ ] 配置文件路徑在不同目錄下的穩定性
- [ ] Discord 傳送模組的附件大小限制檢查
- [ ] log path fix
- [ ] err left print
- [ ] $ the space
- [ ] SRT 修復:檢查原檔結尾是否有換行符號
- [x] 缩进错乱
<!-- TODO_END -->

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


## 貢獻
- 歡迎 PR / Issue！
- 如果你在使用中發現 bug 或有功能建議，直接開 issue 告訴我。

## 許可證
- MIT License
- Made with ❤️ in Rust

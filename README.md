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


## 标准用法-支持多参数输入
```bash
❯ cw test1.srt
[1/1] 標準簡繁-處理檔案: test1.srt -> test1.srt.txt
翻譯對照預覽:
  L003 原: 网络安全这件事  得想办法解决一下
       譯: 網絡安全這件事  得想辦法解決一下
  L004 原: 软件、程序、代码
       譯: 軟件、程序、代碼
  L005 原: 硬件、硬盘、屏幕、显卡
       譯: 硬件、硬盤、屏幕、顯卡
  L006 原: 互联网、服务器、带宽
       譯: 互聯網、服務器、帶寬
  L007 原: 数据、数据库、信息
       譯: 數據、數據庫、信息
  L008 原: 优化、支持、起子
       譯: 優化、支持、起子
[OK]尾部空行
------------------------------------------------------------
變動: 6 行
 日誌: /tmp/cw_2026-01_test1.log
🎯 統計: 通過 1 / 總計 1 | 總耗時: 55.956894ms
------------------------------------------------------------
```

## 管道用法-标准输入输出
```bash
❯ echo "這個软件的程序數據需要優化" | cw
這個軟件的程序數據需要優化
❯ echo "這個软件的程序數據需要優化" | cw -p
這個軟體的程式資料需要最佳化
```


<!-- TODO_START -->
# 🚀 CW 專案進度表 (2026-01-29)

### ✅ 已完成 (地基穩固)
- [x] 多檔案模組化拆分 (main, audit, rules, ui_style, config, compare, downloader)
- [x] 標籤保護翻譯邏輯
- [x] GitHub 風格紅綠底色對比 (-a)
- [x] 0 警告編譯
- [x] 檔名生成的路徑邏輯優化
- [x] 配置文件路徑在不同目錄下的穩定性
- [x] Discord 傳送模組的附件大小限制檢查
- [x] log path fix
- [x] err left print
- [x] 缩进错乱
- [x] add skrt

### 🌟 未來遠景
- [ ] 與 Discord Bot 對接
- [ ] 自動偵測檔案編碼 (GBK/UTF-8)
- [ ] MEGA Auto Download cn srt

### 🛠 待修復的小問題 (精力恢復後再動手)
- [ ] 原始srt文件缺少尾部空行 进行报错 翻译后修复
- [ ] -b 推送信息中 夹杂错误修正建议
- [ ] 生成默认 cw.cfg
- [ ] 自動診斷報告系統
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

# CW - 简繁中文字幕/文本翻译 & 纠错工具

一个高效的 Rust CLI 工具，专注于**简体中文 → 繁体中文**的自动翻译与校对，支持字幕文件 (.srt/.lrc/.txt) 和普通文本文件。

核心使用 OpenCC 进行简繁转换，并支持自定义规则优化、深度对比预览、Discord 推送等功能。适合字幕组、翻译爱好者、个人校对使用。

## 功能亮点

- 自动转换简体部分（基于 OpenCC）
- 专业词汇优化模式 (基于 OpenCC）
- 深度内容对比校对模式 (-a)
- 翻译结果统计（通过/失败、日誌路径）
- 变动的行预览（原文 vs 译文）
- -b参数 推送结果 到 Discord webhook
- 彩色终端输出 + UI 框线

## 安装

需要 Rust 1.60+ 环境：

##TODO
- 未來的-b操作流程可能是這樣：
無感流程：你現在只需下一個指令：
電腦就會自動：下載 -> 翻譯 -> 稽核 -> 發送。這就是你說的「工作站」。
```Bash
cw --task "MEGA_URL" --text "這是今天的疫苗報告" -b
```
- 对命令行的标准输入 直接翻译
- 智慧篩選：它會幫你從 Mega 連結的一堆檔案裡，自動揪出那個帶有 cn 的字幕，這省去了人工挑選的麻煩。
```bash
# 从源代码安装
git clone https://github.com/e3e0261f/cw.git
cd cw
cargo install --path .
```

或者直接使用 cargo run：
```bash
cargo run -- test1.srt
```

使用方式
基本用法（翻译单个文件）：
```bash
cw test1.srt
```

##依赖

- opencc-rust（简繁转换）
- colored（终端彩色）
- clap（命令行参数）
- unicode-width（中文宽度计算）
- 其他：aho-corasick, rayon, regex 等

##构建要求
- Rust 1.60+，Cargo

##贡献
- 欢迎 PR / Issue！
- 如果你在使用中发现 bug 或有功能建议，直接开 issue 告诉我。

##许可证
- MIT License
- Made with ❤️ in Rust

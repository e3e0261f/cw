# CW - 简繁中文字幕/文本翻译 & 纠错工具

一个高效的 Rust CLI 工具，专注于**简体中文 → 繁体中文**的自动翻译与校对，支持字幕文件 (.srt/.lrc/.txt) 和普通文本文件。

核心使用 OpenCC 进行简繁转换，并支持自定义规则优化、深度对比预览、Discord 推送等功能。适合字幕组、翻译爱好者、个人校对使用。

## 功能亮点

- 自动简繁转换（基于 OpenCC）
- 专业词汇优化模式 (-p)
- 深度内容对比校对模式 (-a)
- 翻译结果统计（通过/失败、日誌路径）
- 变动的行预览（原文 vs 译文）
- 支持 Discord webhook 推送结果 (-b)
- 彩色终端输出 + 美观 UI 框线

## 安装

需要 Rust 1.60+ 环境：

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
# 输出 → test1.srt.txt（翻译结果）

常用选项：
# 专业优化模式（OpenCC 高级词汇处理）
cw -p test1.srt

# 深度对比模式（显示原文 vs 译文差异）
cw -a test1.srt test2.srt

# 发送结果到 Discord
cw -b test1.srt

查看帮助：
cw --help

```

行变动已存入日志档案: /path/to/translation_log.txt

✓ 翻译完成


依赖

opencc-rust（简繁转换）
colored（终端彩色）
clap（命令行参数）
unicode-width（中文宽度计算）
其他：aho-corasick, rayon, regex 等

构建要求
Rust 1.60+，Cargo

贡献
欢迎 PR / Issue！
如果你在使用中发现 bug 或有功能建议，直接开 issue 告诉我。

许可证
MIT License
Made with ❤️ in Rust



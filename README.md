# CW - (Production-Grade Localization Tool)

#核心目标：在不触碰任何非文本结构（如 SRT 时间轴、Markdown 标记）的前提下，实现极高精准度的繁简转换与并与discord频道内发可编辑内容 和 表情互动。

- 结构保护技术：内置精密规则引擎，确保字幕时间轴、代码块、专有名词在转换过程中完整如初。

- 视觉化校对流程：独创颜色高亮对照模式，翻译变动一目了然，让二次人工校对效率提升 300%。

- 团队异步协作：一键打通 Discord Webhook 交付链路，实现从本地处理到云端同步的无缝衔接。

- 工业级性能：由 Rust 驱动，轻松处理万行级文本，拒绝崩溃与延迟。

- SRT 格式对空格、数字、箭头 (-->) 和换行极其敏感。普通的翻译工具（如网页翻译）经常会把时间轴数字翻错，或者把 --> 换成中文箭头。

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

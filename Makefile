# 預設執行：編譯、測試翻譯、測試對比、讀取日誌
all:  clippy test build sync translate compare

# 1. 編譯（release 模式）
build:
	@echo "正在編譯 release 版本..."
	cargo build --release

# 2. 測試單一檔案翻譯
translate:
	@echo "正在測試翻譯功能...----------------------------------------"
	echo "這個软件的程序數據需要優化" | cw
	echo "這個软件的程序數據需要優化" | cw -p
	./target/release/cw -b ./deps/test2.srt.txt
# 3. 測試對比模式
compare:
	@echo "正在測試對比模式...----------------------------------------"
	./target/release/cw -a ./deps/test1.srt ./deps/test1.srt.txt
	./target/release/cw -a ./deps/test2.srt ./deps/test2.srt.txt

# 4. 直接查看最新日誌
log:
	@echo "讀取稽核日誌...--------------------------------------------"
	cat /tmp/cw_260024.log

# 5. 清理所有測試產生的垃圾
clean:
	@echo "清理環境...------------------------------------------------"
	rm -f ./target/release/*.txt
#	rm -f /tmp/cw_*.log

# 同步 TODO 到 README
# TODAY = $(shell date +%Y-%m-%d)
sync:
	@echo "正在同步 TODO 到 README..."
	@sed -i '/<!-- TODO_START -->/,/<!-- TODO_END -->/{ /<!-- TODO_START -->/b; /<!-- TODO_END -->/b; d }' README.md
	@sed -i '/<!-- TODO_START -->/r TODO.md' README.md

VERSION = $(shell grep '^version =' Cargo.toml | cut -d '"' -f 2)
release: build
	@echo "準備發布版本 v$(VERSION)..."
	git add .
	git commit -m "Release v$(VERSION)" || echo "無變動需提交"
	git tag -a v$(VERSION) -m "Version $(VERSION)"
	git push origin main --tags
	@echo "🚀 版本 v$(VERSION) 已發送至 GitHub！"

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test -- --nocapture

build:
	cargo build --release

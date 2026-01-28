# 預設執行：編譯、測試翻譯、測試對比、讀取日誌
all: build translate compare log

# 1. 編譯（release 模式）
build:
	@echo "正在編譯..."
	cargo build --release

# 2. 測試單一檔案翻譯
translate:
	@echo "正在測試翻譯功能..."
	./target/release/cw test1.srt

# 3. 測試對比模式
compare:
	@echo "正在測試對比模式..."
	./target/release/cw -a test1.srt test1.srt.txt

# 4. 直接查看最新日誌
log:
	@echo "讀取稽核日誌..."
	cat /tmp/cw_260024.log

# 5. 清理所有測試產生的垃圾
clean:
	@echo "清理環境..."
	rm -f *.txt
	rm -f /tmp/cw_*.log

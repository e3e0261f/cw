# 預設執行：編譯、測試翻譯、測試對比、讀取日誌
all: build translate compare log

# 1. 編譯（release 模式）
build:
	@echo "正在編譯..."
	cargo build --release

# 2. 測試單一檔案翻譯
translate:
	@echo "正在測試翻譯功能..."
	echo "這個软件的程序數據需要優化" | cw
#	./target/release/cw test1.srt
	echo "這個软件的程序數據需要優化" | cw -p
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
	rm -f ./target/release/*.txt
	rm -f /tmp/cw_*.log

# 同步 TODO 到 README
sync:
	@echo "正在同步 TODO 到 README..."
	@# 使用 sed 將 TODO.md 的內容嵌入 README.md 兩個標記之間
	@sed -i '/<!-- TODO_START -->/,/<!-- TODO_END -->/{ /<!-- TODO_START -->/b; /<!-- TODO_END -->/b; d }' README.md
	@sed -i '/<!-- TODO_START -->/r TODO.md' README.md
	@echo "同步完成！"

# 修改你原本的 build 或 test，讓它順便同步
all: build sync translate compare

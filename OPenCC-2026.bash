s2tw-srt = ''
        if test (count $argv) -eq 0
            echo "🔍 正在遞迴掃描 ~/下載 (台式在地化專用)..."
            find "$HOME/下載" -type f -name "*.srt" | while read -l f
                if string match -q "*.srt.txt" "$f"
                    continue
                end
                
                # 💡 亮點：使用 s2twp.json (Simple to Traditional Taiwanese Phrases)
                opencc -i "$f" -o "$f.txt" -c s2twp.json
                echo "✨ 在地化轉繁成功: $f.txt"
            end
        else
            set f $argv[1]
            if test -f "$f"
                opencc -i "$f" -o "$f.txt" -c s2twp.json
                echo "✨ 在地化轉繁成功: $f.txt"
            else
                echo "❌ 找不到檔案: $f"
            end
        end
        echo "🎉 搞定！"
      '';

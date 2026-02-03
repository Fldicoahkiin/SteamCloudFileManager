echo "正在更新 Steam Cloud File Manager..."

echo "等待程序退出..."
for i in {1..30}; do
	if ! pgrep -x "SteamCloudFileManager" >/dev/null; then
		break
	fi
	sleep 1
done

sleep 1

echo "替换应用程序..."
rm -rf "{{CURRENT_APP}}"
cp -R "{{NEW_APP}}" "{{CURRENT_APP}}"

echo "清理临时文件..."
rm -rf "{{TEMP_DIR}}"
rm -f "{{DOWNLOAD_PATH}}"

echo "更新完成！正在启动程序..."
open "{{CURRENT_APP}}"

rm -f "$0"

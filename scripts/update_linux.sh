#!/bin/bash
# Steam Cloud File Manager Update Script (Linux)

echo "正在更新 Steam Cloud File Manager..."

# 等待原进程退出
echo "等待程序退出..."
for i in {1..30}; do
    if ! pgrep -f "steam-cloud-file-manager" > /dev/null && ! pgrep -f "SteamCloudFileManager" > /dev/null; then
        break
    fi
    sleep 1
done

sleep 1

# 替换文件
echo "替换程序文件..."
cp -f "{{NEW_EXE}}" "{{CURRENT_EXE}}"
chmod +x "{{CURRENT_EXE}}"

if [ -f "{{NEW_SO}}" ]; then
    cp -f "{{NEW_SO}}" "{{CURRENT_SO}}"
    echo "已更新: libsteam_api.so"
fi

# 清理
echo "清理临时文件..."
rm -rf "{{TEMP_DIR}}"
rm -f "{{DOWNLOAD_PATH}}"

echo "更新完成！正在启动程序..."
"{{CURRENT_EXE}}" &

# 删除自身
rm -f "$0"

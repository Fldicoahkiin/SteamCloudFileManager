# Steam Cloud File Manager Update Script
$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "正在更新 Steam Cloud File Manager..." -ForegroundColor Cyan
Write-Host ""

# 等待原进程退出
Write-Host "等待程序退出..."
$maxWait = 30
$waited = 0
while ($waited -lt $maxWait) {
    $proc = Get-Process -Name "SteamCloudFileManager" -ErrorAction SilentlyContinue
    if (-not $proc) {
        break
    }
    Start-Sleep -Seconds 1
    $waited++
}

if ($waited -ge $maxWait) {
    Write-Host "警告: 等待超时，尝试强制继续..." -ForegroundColor Yellow
}

Write-Host "程序已退出，开始更新..."
Start-Sleep -Seconds 1

# 复制新文件
try {
    Copy-Item -Path "{{NEW_EXE}}" -Destination "{{CURRENT_EXE}}" -Force
    Write-Host "已更新: SteamCloudFileManager.exe" -ForegroundColor Green
    
    if (Test-Path "{{NEW_DLL}}") {
        Copy-Item -Path "{{NEW_DLL}}" -Destination "{{CURRENT_DLL}}" -Force
        Write-Host "已更新: steam_api64.dll" -ForegroundColor Green
    }
} catch {
    Write-Host "更新失败: $_" -ForegroundColor Red
    Write-Host "请手动替换程序文件。" -ForegroundColor Yellow
    Read-Host "按任意键退出"
    exit 1
}

# 清理临时文件
Write-Host "清理临时文件..."
Remove-Item -Path "{{TEMP_DIR}}" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "{{DOWNLOAD_PATH}}" -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "更新完成！正在启动程序..." -ForegroundColor Green
Start-Sleep -Seconds 1

# 启动新版本
Start-Process -FilePath "{{CURRENT_EXE}}"

# 删除自身
Remove-Item -Path $MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue

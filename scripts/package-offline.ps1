# Shine Helper 离线便携版打包脚本
# 使用方法: powershell -ExecutionPolicy Bypass -File package-offline.ps1

param(
    [string]$OutputDir = ".\dist\portable"
)

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Green
Write-Host "Shine Helper 离线便携版打包工具" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""

# 检查 Node.js
Write-Host "[1/5] 检查环境..." -ForegroundColor Cyan
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Host "错误: 需要安装 Node.js 22.x" -ForegroundColor Red
    Write-Host "请访问 https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}
Write-Host "Node.js: $nodeVersion" -ForegroundColor Green

# 检查 npm
$npmVersion = npm --version 2>$null
if (-not $npmVersion) {
    Write-Host "错误: npm 未找到" -ForegroundColor Red
    exit 1
}
Write-Host "npm: $npmVersion" -ForegroundColor Green

# 创建输出目录
Write-Host ""
Write-Host "[2/5] 准备输出目录..." -ForegroundColor Cyan
if (Test-Path $OutputDir) {
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\resources\openclaw\data" -Force | Out-Null

# 复制 Tauri 构建产物
Write-Host ""
Write-Host "[3/5] 复制应用文件..." -ForegroundColor Cyan
$targetExe = Get-ChildItem -Path "src-tauri\target\release" -Filter "shine_helper.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($targetExe) {
    Copy-Item $targetExe.FullName "$OutputDir\shine_helper.exe"
    Write-Host "  已复制: shine_helper.exe" -ForegroundColor Green
} else {
    Write-Host "  警告: 未找到 Tauri 构建产物" -ForegroundColor Yellow
    Write-Host "  请先运行: npm run tauri build" -ForegroundColor Yellow
    Write-Host "  或者手动复制 shine_helper.exe 到输出目录" -ForegroundColor Yellow
}

# 复制 resources 文件
if (Test-Path "resources\openclaw") {
    Copy-Item "resources\openclaw\*" "$OutputDir\resources\openclaw\" -Recurse -Force
    Write-Host "  已复制: resources/openclaw/" -ForegroundColor Green
}

# 复制启动脚本
Write-Host ""
Write-Host "[4/5] 复制启动脚本..." -ForegroundColor Cyan
if (Test-Path "start.bat") {
    Copy-Item "start.bat" "$OutputDir\"
    Write-Host "  已复制: start.bat" -ForegroundColor Green
} else {
    Write-Host "  警告: 未找到 start.bat" -ForegroundColor Yellow
}

# 创建 README
$readme = @"
# Shine Helper 便携版

## 使用说明

1. 解压到任意目录
2. 双击 start.bat 启动应用
3. 首次使用需要配置 OpenClaw（如果未预打包）

## 文件说明

- start.bat - 启动脚本（双击运行）
- shine_helper.exe - 主程序
- resources/openclaw/ - OpenClaw 运行时（需自行配置）

## 离线打包步骤

如需在离线环境使用，需要预打包 OpenClaw：

1. 下载 Node.js 22.x Windows x64:
   https://nodejs.org/dist/v22.x.x/node-v22.x.x-win-x64.zip

2. 解压到 resources/openclaw/node/

3. 安装 OpenClaw:
   在联网环境运行: npm install -g openclaw@latest
   导出: npm pack openclaw
   解压 tarball 到 resources/openclaw/openclaw/

4. 在 resources/openclaw/openclaw/ 运行:
   npm install --production

## 故障排除

如果启动失败，请检查：
1. resources/openclaw/node/node.exe 是否存在
2. resources/openclaw/openclaw/gateway.js 是否存在
3. 端口 18789 是否被占用
"@

$readme | Out-File "$OutputDir\README.txt" -Encoding UTF8
Write-Host "  已创建: README.txt" -ForegroundColor Green

# 打包为 zip
Write-Host ""
Write-Host "[5/5] 生成压缩包..." -ForegroundColor Cyan
$zipPath = ".\dist\shine_helper_portable.zip"
if (-not (Test-Path ".\dist")) {
    New-Item -ItemType Directory -Path ".\dist" -Force | Out-Null
}
if (Test-Path $zipPath) { 
    Remove-Item $zipPath -Force 
}
Compress-Archive -Path "$OutputDir\*" -DestinationPath $zipPath -Force

Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host "打包完成！" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""
Write-Host "输出目录: $OutputDir" -ForegroundColor Cyan
Write-Host "压缩包: $zipPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "提示: 压缩包可直接分发，无需安装" -ForegroundColor Yellow
# Shine Helper OpenClaw 集成实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 OpenClaw 完全集成到 Shine Helper 便携版中，实现一键启动，用户无需感知 OpenClaw 存在

**Architecture:** 预打包 Node.js + OpenClaw 到 resources 目录，启动脚本自动管理 OpenClaw 进程生命周期，Tauri 应用通过 localhost:18789 与 OpenClaw 通信

**Tech Stack:** Node.js 22.x, OpenClaw, Tauri 2.x, Rust, Batch Script

---

## Task 1: 创建目录结构和资源模板

**Files:**
- Create: `resources/openclaw/README.txt`

**Step 1: 创建资源目录模板说明**

创建 `resources/openclaw/README.txt` 说明离线打包步骤：

```text
# OpenClaw 离线打包说明

本目录用于存放 OpenClaw 便携版运行时。

## 打包步骤（在有网络环境执行）：

1. 下载 Node.js 22.x Windows x64 二进制：
   https://nodejs.org/dist/v22.x.x/node-v22.x.x-win-x64.zip
   解压到本目录下的 node/ 文件夹

2. 安装 OpenClaw：
   在联网环境运行: npm install -g openclaw@latest
   然后运行: npm pack openclaw
   将生成的 tarball 解压到本目录下的 openclaw/ 文件夹

3. 在 openclaw 目录运行:
   npm install --production

4. 首次运行后会在 data/ 目录生成配置文件

## 目录结构：
resources/
└── openclaw/
    ├── node/           # Node.js 运行时
    ├── openclaw/       # OpenClaw 代码 + node_modules
    ├── data/           # OpenClaw 工作目录
    └── README.txt      # 本文件
```

**Step 2: 提交**

```bash
git add resources/openclaw/README.txt
git commit -m "docs: add OpenClaw offline packaging template"
```

---

## Task 2: 创建启动脚本 start.bat

**Files:**
- Create: `start.bat`

**Step 1: 创建启动脚本**

创建 `start.bat`：

```batch
@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"
set "OPENCLAW_DIR=%SCRIPT_DIR%\resources\openclaw"
set "NODE_DIR=%OPENCLAW_DIR%\node"
set "OPENCLAW_APP_DIR=%OPENCLAW_DIR%\openclaw"
set "SHINE_EXE=%SCRIPT_DIR%\shine_helper.exe"

echo [Shine Helper] 检查运行环境...

REM 检查必要文件
if not exist "%NODE_DIR%\node.exe" (
    echo [错误] 未找到 Node.js 运行时
    echo 请确保 resources\openclaw\node 目录存在 Node.js
    pause
    exit /b 1
)

if not exist "%OPENCLAW_APP_DIR%\package.json" (
    echo [错误] 未找到 OpenClaw 应用
    echo 请确保 resources\openclaw\openclaw 目录存在
    pause
    exit /b 1
)

if not exist "%SHINE_EXE%" (
    echo [错误] 未找到 shine_helper.exe
    pause
    exit /b 1
)

echo [Shine Helper] 检查 OpenClaw 服务状态...

REM 检查端口 18789 是否已被占用
netstat -ano | findstr ":18789 " > nul
if %errorlevel% equ 0 (
    echo [Shine Helper] OpenClaw 服务已在运行
) else (
    echo [Shine Helper] 启动 OpenClaw 服务...
    
    if not exist "%OPENCLAW_DIR%\data" (
        mkdir "%OPENCLAW_DIR%\data"
    )
    
    cd /d "%OPENCLAW_APP_DIR%"
    start "OpenClaw" /b "%NODE_DIR%\node.exe" "%OPENCLAW_APP_DIR%\gateway.js" --port 18789 --data "%OPENCLAW_DIR%\data"
    
    echo [Shine Helper] 等待服务启动...
    timeout /t 8 /nobreak > nul
    
    REM 再次检查
    netstat -ano | findstr ":18789 " > nul
    if %errorlevel% neq 0 (
        echo [警告] OpenClaw 可能启动失败，继续尝试启动应用...
    ) else (
        echo [Shine Helper] OpenClaw 服务已就绪
    )
)

echo [Shine Helper] 启动桌面应用...
start "" "%SHINE_EXE%"

echo [Shine Helper] 启动完成
exit
```

**Step 2: 验证脚本语法**

检查脚本是否有语法错误（Windows 会自动检查）

**Step 3: 提交**

```bash
git add start.bat
git commit -m "feat: add start.bat launcher script"
```

---

## Task 3: 更新 Rust 配置添加集成选项

**Files:**
- Modify: `src-tauri/src/config.rs`

**Step 1: 更新 OpenClaw 配置结构体**

在 `src-tauri/src/config.rs` 中更新 `OpenClawConfig`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub url: String,           // 默认: "http://localhost:18789"
    pub use_local: bool,       // 使用本地 OpenClaw
    pub auto_start: bool,      // 集成模式：自动启动 OpenClaw
}
```

更新 Default 实现：

```rust
impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:18789".to_string(),
            use_local: true,
            auto_start: true,
        }
    }
}
```

**Step 2: 提交**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: update OpenClaw config for integrated mode"
```

---

## Task 4: 在 Tauri 启动时添加进程检查

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 更新 lib.rs 添加进程管理函数**

在 `src-tauri/src/lib.rs` 中添加 OpenClaw 进程管理：

```rust
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;

pub fn check_openclaw_running() -> bool {
    // 检查端口 18789 是否被占用
    TcpStream::connect_timeout(
        &"127.0.0.1:18789".parse().unwrap(),
        Duration::from_secs(1)
    ).is_ok()
}

pub fn start_openclaw_process(openclaw_dir: &str, node_dir: &str) -> Result<(), String> {
    let gateway_js = std::path::Path::new(openclaw_dir).join("gateway.js");
    let data_dir = std::path::Path::new(openclaw_dir).join("data");
    let node_exe = std::path::Path::new(node_dir).join("node.exe");
    
    if !node_exe.exists() {
        return Err("Node.js executable not found".to_string());
    }
    
    if !gateway_js.exists() {
        return Err("OpenClaw gateway.js not found".to_string());
    }
    
    // 创建 data 目录
    std::fs::create_dir_all(&data_dir).ok();
    
    // 启动 OpenClaw
    let mut cmd = Command::new(&node_exe);
    cmd.arg(gateway_js.to_str().unwrap())
       .arg("--port")
       .arg("18789")
       .arg("--data")
       .arg(data_dir.to_str().unwrap())
       .spawn()
       .map_err(|e| format!("Failed to start OpenClaw: {}", e))?;
    
    // 等待服务就绪
    std::thread::sleep(Duration::from_secs(5));
    
    Ok(())
}
```

**Step 2: 更新 main.rs 在启动时检查**

在 `src-tauri/src/main.rs` 的 `main` 函数开头添加：

```rust
fn main() {
    // 获取资源目录路径
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    let resources_dir = exe_dir.join("resources").join("openclaw");
    let node_dir = resources_dir.join("node");
    let openclaw_dir = resources_dir.join("openclaw");
    
    // 检查并启动 OpenClaw
    if resources_dir.exists() && !check_openclaw_running() {
        println!("[Shine Helper] Starting OpenClaw...");
        if let Err(e) = start_openclaw_process(
            openclaw_dir.to_str().unwrap_or(""),
            node_dir.to_str().unwrap_or("")
        ) {
            eprintln!("[Shine Helper] Warning: Failed to start OpenClaw: {}", e);
        }
    }
    
    // 继续 Tauri 应用启动
    tauri::Builder::default()
        .run(...)
}
```

**Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

**Step 4: 提交**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs
git commit -m "feat: add OpenClaw process management on startup"
```

---

## Task 5: 更新前端配置界面隐藏 OpenClaw 配置

**Files:**
- Modify: `src/views/ConfigView.vue`

**Step 1: 更新配置界面**

在 `ConfigView.vue` 中，将 OpenClaw 连接配置标记为内部集成（可选隐藏或简化）：

```vue
<!-- OpenClaw 配置 - 简化显示 -->
<div class="mb-8">
  <h2 class="text-lg font-semibold mb-4">AI 服务</h2>
  <div class="text-sm text-gray-500 mb-4">
    已集成 OpenClaw AI 助手
  </div>
  <!-- 保留 API Key 配置，但默认隐藏详细信息 -->
  <div class="space-y-4">
    <div>
      <label class="block text-sm font-medium mb-1">API Key (可选)</label>
      <input
        v-model="config.openclaw.api_key"
        type="password"
        placeholder="用于 OpenAI 等模型认证"
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
      />
    </div>
  </div>
</div>
```

**Step 2: 验证构建**

```bash
npm run build
```

**Step 3: 提交**

```bash
git add src/views/ConfigView.vue
git commit -m "feat: simplify OpenClaw config UI for integrated mode"
```

---

## Task 6: 创建离线打包脚本（可选）

**Files:**
- Create: `scripts/package-offline.ps1`

**Step 1: 创建打包脚本**

创建 PowerShell 脚本辅助打包：

```powershell
# scripts/package-offline.ps1
# Shine Helper 离线便携版打包脚本

param(
    [string]$OutputDir = ".\dist\portable"
)

$ErrorActionPreference = "Stop"

Write-Host "Shine Helper 离线便携版打包工具" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

# 检查 Node.js
Write-Host "`n[1/5] 检查环境..."
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Host "错误: 需要安装 Node.js 22.x" -ForegroundColor Red
    exit 1
}
Write-Host "Node.js: $nodeVersion"

# 创建输出目录
if (Test-Path $OutputDir) {
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $OutputDir | Out-Null

# 复制 Tauri 构建产物
Write-Host "`n[2/5] 复制应用文件..."
$targetExe = Get-ChildItem -Path "src-tauri\target\release" -Filter "*.exe" | Select-Object -First 1
if ($targetExe) {
    Copy-Item $targetExe.FullName "$OutputDir\shine_helper.exe"
} else {
    Write-Host "警告: 未找到 Tauri 构建产物，请先运行构建" -ForegroundColor Yellow
}

# 创建 resources 目录
New-Item -ItemType Directory -Path "$OutputDir\resources\openclaw\data" -Force | Out-Null

# 复制 resources 文件
if (Test-Path "resources\openclaw") {
    Copy-Item "resources\openclaw\*" "$OutputDir\resources\openclaw\" -Recurse -Force
}

# 复制启动脚本
Write-Host "`n[3/5] 复制启动脚本..."
Copy-Item "start.bat" "$OutputDir\"

# 打包说明
@"
# Shine Helper 便携版

## 使用说明

1. 解压到任意目录
2. 双击 start.bat 启动应用
3. 首次使用需要配置 OpenClaw（如果未预打包）

## 离线打包步骤

如需重新打包 OpenClaw：
1. 下载 Node.js 22.x Windows x64
2. 解压到 resources/openclaw/node/
3. 运行: npm install -g openclaw@latest
4. 复制到 resources/openclaw/openclaw/
"@ | Out-File "$OutputDir\README.txt" -Encoding UTF8

Write-Host "`n[4/5] 生成便携版..."
# 可选：创建 zip 包
$zipPath = "$OutputDir-shine_helper.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath }
Compress-Archive -Path "$OutputDir\*" -DestinationPath $zipPath -Force

Write-Host "`n[5/5] 完成！" -ForegroundColor Green
Write-Host "输出目录: $OutputDir" -ForegroundColor Cyan
Write-Host "ZIP 包: $zipPath" -ForegroundColor Cyan
```

**Step 2: 提交**

```bash
git add scripts/package-offline.ps1
git commit -m "feat: add offline packaging script"
```

---

## Task 7: 验证完整流程

**Step 1: 模拟打包测试**

如果没有实际打包环境，进行以下验证：
1. 验证目录结构正确
2. 验证脚本语法
3. 验证 Rust 编译通过
4. 验证前端构建通过

**Step 2: 文档更新**

更新 README 说明便携版使用方式。

**Step 3: 最终提交**

```bash
git add .
git commit -m "feat: complete OpenClaw integration for portable offline use"
```

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-03-04-shine-helper-openclaw-integration-implementation-plan.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - In a new session using executing-plans, batch execution with checkpoints

**Which approach?**
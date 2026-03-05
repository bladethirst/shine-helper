# Shine Helper OpenClaw 集成设计文档

**项目名称**: Shine Helper  
**版本**: v1.0.0  
**日期**: 2026-03-04  
**状态**: 已批准

---

## 1. 设计目标

将 OpenClaw 完全集成到 Shine Helper 中，用户无需感知 OpenClaw 的存在，认为这是 Shine Helper 产品自带的功能。

---

## 2. 约束条件

| 约束 | 说明 |
|------|------|
| 交付形式 | 便携版（单一目录，解压即用） |
| 离线支持 | 必须支持内网/离线环境安装使用 |
| 技术栈 | OpenClaw (Node.js ≥22) + Tauri 2.x |

---

## 3. 方案选择

| 维度 | 选择 | 说明 |
|------|------|------|
| 打包策略 | 预装 Node.js + OpenClaw 目录 | 在有网络环境打包好所有依赖 |
| 集成方式 | 子进程启动 | Shine Helper 启动时自动启动 OpenClaw，通过 localhost:18789 通信 |
| 启动方式 | 一键启动 | 用户双击启动脚本即可使用 |

---

## 4. 目录结构

```
shine_helper_portable/
├── resources/
│   └── openclaw/
│       ├── node/                    # Node.js 运行时 (Windows x64)
│       │   ├── node.exe
│       │   └── ...
│       ├── openclaw/                # OpenClaw 代码 + node_modules
│       │   ├── package.json
│       │   ├── node_modules/
│       │   └── ...
│       └── data/                    # OpenClaw 工作目录
│           ├── config.json
│           └── ...
├── shine_helper.exe                 # Tauri 主程序
└── start.bat                        # 启动脚本 (Windows)
```

---

## 5. 启动流程

### 5.1 启动脚本 (start.bat)

```batch
@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "OPENCLAW_DIR=%SCRIPT_DIR%resources\openclaw"
set "NODE_DIR=%OPENCLAW_DIR%\node"
set "OPENCLAW_APP_DIR=%OPENCLAW_DIR%\openclaw"

echo [Shine Helper] 启动中...

REM 检查 OpenClaw 是否已运行
netstat -ano | findstr ":18789" > nul
if %errorlevel% equ 0 (
    echo [Shine Helper] OpenClaw 服务已运行
) else (
    echo [Shine Helper] 启动 OpenClaw 服务...
    cd /d "%OPENCLAW_APP_DIR%"
    start "OpenClaw" "%NODE_DIR%\node.exe" "gateway.js" --port 18789
    timeout /t 5 /nobreak > nul
)

echo [Shine Helper] 启动桌面应用...
start "" "%SCRIPT_DIR%shine_helper.exe"

exit
```

### 5.2 初始化流程

```
用户双击 start.bat
        │
        ▼
检查端口 18789 是否被占用
        │
        ├── 已占用 ──▶ 直接启动 shine_helper.exe
        │
        └── 未占用 ──▶ 启动 OpenClaw gateway 进程
                          │
                          ▼
                    等待服务就绪 (5秒)
                          │
                          ▼
                    启动 shine_helper.exe
```

---

## 6. Rust 后端集成

### 6.1 配置更新

在 `src-tauri/src/config.rs` 中添加 OpenClaw 集成配置：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub url: String,           // 默认: "http://localhost:18789"
    pub enabled: bool,         // 集成模式：始终启用
    pub auto_start: bool,      // 自动启动 OpenClaw
}
```

### 6.2 Tauri 启动时检查

在 Rust 主程序启动时：
1. 检查 OpenClaw 进程是否运行
2. 如果未运行，通过子进程启动
3. 等待服务就绪

---

## 7. 用户体验

| 场景 | 用户看到的行为 |
|------|----------------|
| 首次使用 | 解压 → 双击 start.bat → 等待启动 → 开始使用 |
| 后续使用 | 双击 start.bat → 直接使用 |
| 关闭应用 | 关闭 Shine Helper，OpenClaw 进程同时退出 |

用户完全不需要知道 OpenClaw 的存在，所有操作都在 Shine Helper 中完成。

---

## 8. 离线打包指南

### 8.1 准备工作

在有网络的环境下：

```bash
# 1. 下载 Node.js Windows x64 二进制
# https://nodejs.org/dist/v22.x.x/node-v22.x.x-win-x64.zip

# 2. 安装 OpenClaw
npm install -g openclaw@latest

# 3. 在项目目录创建 resources/openclaw
mkdir -p resources/openclaw
```

### 8.2 打包步骤

1. 复制 Node.js 运行时到 `resources/openclaw/node/`
2. 运行 `npm pack openclaw` 获取 tarball
3. 解压到 `resources/openclaw/openclaw/`
4. 在该目录运行 `npm install --production`
5. 复制整个目录结构
6. 打包为 zip

---

## 9. 验收标准

- [ ] 便携版解压后双击 start.bat 即可启动
- [ ] 离线环境下（无互联网）能正常运行
- [ ] 用户无需手动安装 OpenClaw
- [ ] Shine Helper 对话功能正常工作
- [ ] 关闭 Shine Helper 后 OpenClaw 进程正确退出

---

## 10. 后续工作

1. 创建实现计划
2. 打包脚本开发
3. 测试离线环境运行
4. 交付版本打包

---

*本文档由 Sisyphus AI 生成，日期: 2026-03-04*
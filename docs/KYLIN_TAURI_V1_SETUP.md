# 麒麟系统 Tauri v1 开发环境配置指南

## 一、问题背景

在银河麒麟 V10 SP1 操作系统上构建 Tauri 应用时，**只能使用 Tauri v1**。Tauri v2 由于 `wry` 版本与系统 `webkit2gtk-4.0` (版本 2.38.6) 不兼容，无法在麒麟系统上运行。

Tauri v1 的已知限制：
- `navigator.mediaDevices` 在某些平台（包括 Linux 麒麟系统）上可能是 undefined
- 无法直接通过浏览器 API 访问麦克风

因此需要使用 Rust 后端来处理麦克风音频捕获。

## 二、环境要求

### 系统环境

| 组件 | 版本要求 | 说明 |
|------|----------|------|
| OS | 银河麒麟 V10 SP1 | GLib 2.64.6, webkit2gtk-4.0 2.38.6 |
| Rust | 1.70+ | `rustc --version` |
| Node.js | 18+ | `node --version` |
| npm | 9+ | `npm --version` |

### 系统依赖安装

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup2.4-dev \
  libjavascriptcoregtk-4.0-dev \
  pkg-config \
  build-essential \
  curl
```

## 三、项目配置修改

### 1. Cargo.toml 修改

```toml
[package]
name = "shine-helper"
version = "1.0.0"
description = "Shine Helper - Desktop AI Assistant"
authors = ["Shine Team"]
edition = "2021"
rust-version = "1.70"

[lib]
name = "shine_helper_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "=1.5.0", features = [] }

[dependencies]
tauri = { version = "=1.5.0", features = [ "http-all", "shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
bytes = "1"
tokio-tungstenite = "0.21"
futures-util = "0.3"
rusqlite = { version = "0.32", features = ["bundled"] }
keyring = "3"
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
log = "0.4"
env_logger = "0.11"
thiserror = "1"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

**关键变更：**
- `tauri` 锁定到 `=1.5.0`（精确版本）
- `tauri-build` 锁定到 `=1.5.0`
- 添加 `[lib]` 配置
- 添加 `rust-version = "1.70"`

### 2. package.json 修改

```json
{
  "dependencies": {
    "@tauri-apps/api": "^1.6.0",
    "pinia": "^2.1.7",
    "vue": "^3.4.0",
    "vue-router": "^4.2.5"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.6.0"
  }
}
```

**关键变更：**
- `@tauri-apps/api`: `^1.6.0`
- `@tauri-apps/cli`: `^1.6.0`

### 3. 前端 API 导入修改

将所有 `@tauri-apps/api/core` 改为 `@tauri-apps/api/tauri`：
- `src/views/SkillsView.vue`
- `src/views/ChatView.vue`

### 4. tauri.conf.json 修改

从 Tauri v2 格式转换为 v1 格式，使用 `build.devPath` 而非 `build.devUrl`，`build.distDir` 等。

## 四、Rust 安装

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
```

### 配置环境变量

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

添加到 `~/.bashrc` 永久生效：

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

### 验证安装

```bash
rustc --version
cargo --version
```

## 五、wry 兼容性问题解决（关键）

### 问题描述

编译时报错：
```
error[E0599]: no method named `set_enable_webgl` found for struct `webkit2gtk::Settings`
```

这是因为 wry 0.24.11 使用的 `webkit2gtk v0.18.2` 与系统 webkit2gtk-4.0 2.38.6 存在 API 兼容性问题。`webkit2gtk v0.18` 绑定的 `SettingsExt` trait 方法不兼容系统库。

### 解决步骤

1. **克隆 wry 源码**

```bash
cd /tmp
git clone https://github.com/tauri-apps/wry.git wry-fix
cd wry-fix
git checkout wry-v0.24.11
```

2. **修复 webkitgtk/mod.rs**

在 `src/webview/webkitgtk/mod.rs` 文件中，添加 `SettingsExt` 到 use 语句：

```rust
// 修改前
use webkit2gtk::{
  traits::*, LoadEvent, NavigationPolicyDecision, PolicyDecisionType, URIRequest,
  UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime, WebView,
  WebViewBuilder,
};

// 修改后
use webkit2gtk::{
  traits::*, LoadEvent, NavigationPolicyDecision, PolicyDecisionType, SettingsExt, URIRequest,
  UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime, WebView,
  WebViewBuilder,
};
```

3. **在 Cargo.toml 中添加本地 patch**

```toml
[patch.crates-io]
wry = { path = "/tmp/wry-fix" }
```

4. **清理并重新编译**

```bash
rm -rf src-tauri/target
rm -f src-tauri/Cargo.lock
npm run tauri build
```

### 问题根因

`webkit2gtk` crate 0.18 版本在 Rust 层面使用了 `webkit2gtk_sys` 0.18，但系统安装的是 webkit2gtk-4.0 2.38.6。虽然版本号看起来兼容，但 Rust 绑定生成的部分 trait 方法（如 `SettingsExt`）未能正确导出到当前作用域。

通过修改 wry 源码显式导入 `SettingsExt` trait，解决了这个问题。

## 六、构建命令

### 开发模式

```bash
export PATH="$HOME/.cargo/bin:$PATH"
npm run tauri dev
```

### 生产构建

```bash
export PATH="$HOME/.cargo/bin:$PATH"
rm -rf src-tauri/target
npm run tauri build
```

### 构建产物位置

- **DEB**: `src-tauri/target/release/bundle/deb/shine-helper_1.0.0_amd64.deb`
- **RPM**: `src-tauri/target/release/bundle/rpm/shine-helper-1.0.0-1.x86_64.rpm`
- **AppImage**: `src-tauri/target/release/bundle/appimage/shine-helper_1.0.0_amd64.AppImage`

## 七、注意事项

1. **wry 补丁**: 必须使用本地路径 patch 方式，不能使用 git rev 方式
2. **前端 API**: 确保使用 Tauri v1 的 API 路径 (`@tauri-apps/api/tauri`)
3. **清理构建**: 遇到奇怪问题时，先执行 `rm -rf src-tauri/target` 和 `rm -f src-tauri/Cargo.lock`
4. **tauri 版本**: 必须锁定到 `=1.5.0`，使用 `^1.5` 可能导致版本不一致
5. **系统依赖**: 确保安装了所有 webkit2gtk 相关开发包

---

**创建日期**: 2026-03-08  
**适用系统**: 银河麒麟桌面操作系统 V10 (SP1)  
**Tauri 版本**: 1.5.x  
**WebKit2GTK 版本**: 2.38.6

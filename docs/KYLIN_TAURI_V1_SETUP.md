# 麒麟系统 Tauri v1 开发环境配置指南

## 一、问题背景

在银河麒麟 V10 SP1 操作系统上构建 Tauri v1 应用时，遇到以下问题：
1. `wry` 版本与系统 `webkit2gtk-4.0` (版本 2.38.6) 不兼容
2. 报错：`error[E0599]: no method named 'set_enable_webgl' found for struct 'webkit2gtk::Settings'`
3. Tauri v2 API 与 v1 不兼容

## 二、已完成的修改

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
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# ... 其他依赖保持不变

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

[patch.crates-io]
wry = { git = "https://github.com/tauri-apps/wry", rev = "wry-v0.22.6" }
```

**关键变更：**
- `tauri` 从 v2 降级到 `1.5`
- `tauri-build` 从 v2 降级到 `1.5`
- 添加 `[lib]` 配置
- 添加 `rust-version = "1.70"`
- 添加 `[patch.crates-io]` 强制降级 wry 到 0.22.6 解决兼容问题
- `thiserror` 从 v2 降级到 v1

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
    "@tauri-apps/cli": "^1.6.0",
    // ...
  }
}
```

**关键变更：**
- `@tauri-apps/api`: `^2.0.0` → `^1.6.0`
- `@tauri-apps/cli`: `^2.0.0` → `^1.6.0`

### 3. 前端 API 导入修改

将所有 `@tauri-apps/api/core` 改为 `@tauri-apps/api/tauri`：

- `src/views/SkillsView.vue`
- `src/views/ChatView.vue`

### 4. tauri.conf.json 修改

从 Tauri v2 格式转换为 v1 格式，使用 `build.devPath` 而非 `build.devUrl`，`build.distDir` 而非 `build.frontendDist` 等。

### 5. main.rs 修复

- 修复内部属性 `#![cfg_attr]` 位置错误
- 移除 `config` 模块的 Windows 条件编译（使其在所有平台可用）

## 三、环境要求

### 系统环境

| 组件 | 版本要求 | 说明 |
|------|----------|------|
| OS | 银河麒麟 V10 SP1 | GLib 2.64.6, webkit2gtk-4.0 2.38.6 |
| Rust | 1.70+ | `rustc --version` |
| Node.js | 18+ | `node --version` |
| npm | 9+ | `npm --version` |

### 系统依赖安装

```bash
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup2.4-dev \
  libjavascriptcoregtk-4.0-dev \
  libglib2.0-dev \
  libcairo2-dev \
  libgdk-pixbuf2.0-dev \
  libpango1.0-dev \
  libatk1.0-dev \
  pkg-config
```

## 四、Rust 环境（已安装）

### Cargo 路径配置

Rust 已安装在 `~/.cargo/` 目录。每次构建前需要加载环境：

```bash
source ~/.cargo/env
```

或添加到 `~/.bashrc` 永久生效：

```bash
echo 'source ~/.cargo/env' >> ~/.bashrc
```

### 验证安装

```bash
source ~/.cargo/env
cargo --version
# 输出: cargo 1.94.0

rustc --version
# 输出: rustc 1.94.0
```

## 五、构建命令

### 开发模式

```bash
source ~/.cargo/env
npm run tauri dev
```

### 生产构建

```bash
source ~/.cargo/env
rm -rf src-tauri/target  # 建议清理缓存
npm run tauri build
```

### 构建产物位置

- **DEB**: `src-tauri/target/release/bundle/deb/shine-helper_1.0.0_amd64.deb`
- **RPM**: `src-tauri/target/release/bundle/rpm/shine-helper-1.0.0-1.x86_64.rpm`
- **AppImage**: `src-tauri/target/release/bundle/appimage/shine-helper_1.0.0_amd64.AppImage`

## 六、注意事项

1. **wry 补丁**: 不要移除 `[patch.crates-io]` 中的 wry 降级配置，否则编译会失败
2. **前端 API**: 确保使用 Tauri v1 的 API 路径 (`@tauri-apps/api/tauri`)
3. **清理构建**: 遇到奇怪问题时，先执行 `rm -rf src-tauri/target`
4. **Windows 子系统**: 如需 Windows 支持，保持 main.rs 中的 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`

---

**创建日期**: 2026-03-07  
**适用系统**: 银河麒麟桌面操作系统 V10 (SP1)  
**Tauri 版本**: 1.5.x  
**WebKit2GTK 版本**: 2.38.6
# OpenClaw 离线打包说明

本目录用于存放 OpenClaw 便携版运行时。

## 当前目录结构

```
resources/openclaw/
├── openclaw.mjs       # OpenClaw 入口文件
├── package.json       # OpenClaw 包配置
├── dist/              # OpenClaw 构建输出
├── node_modules/      # OpenClaw 依赖
├── skills/           # OpenClaw 技能
├── extensions/        # OpenClaw 扩展
├── assets/           # OpenClaw 资源
├── data/             # OpenClaw 配置和数据
│   ├── openclaw.json # 主配置文件
│   ├── workspace/    # 工作区
│   ├── agents/       # Agent 配置
│   └── ...
├── openclaw.log      # OpenClaw 日志（运行时创建）
└── README.txt        # 本文件
```

## 依赖环境

- Node.js 22.12+ (当前系统已有 Node.js 25.8.0)
- 无需额外依赖

## 配置文件说明

`data/` 目录包含用户的 OpenClaw 配置：

- **openclaw.json** - 主配置文件，包含：
  - API 配置（阿里云通义千问等）
  - Telegram 频道配置
  - Gateway 配置（端口 18789）
  - 认证信息
- **workspace/** - 工作区文件
- **agents/** - Agent 配置

## 启动方式

### 方式一：使用启动脚本

```bash
cd /data/workspace/shine-helper
./start.sh
```

启动脚本会：
1. 检查 OpenClaw 是否已运行（端口 18789）
2. 如未运行，自动启动 OpenClaw（使用 bundled 配置）
3. 启动 Shine Helper 桌面应用

### 方式二：手动启动

```bash
cd resources/openclaw
export OPENCLAW_CONFIG_PATH="$(pwd)/data/openclaw.json"
node openclaw.mjs gateway run --port 18789
```

## 注意事项

1. **离线使用**：整个 `resources/openclaw` 目录可打包分发
2. **首次运行**：配置文件已预置，无需再次配置
3. **日志查看**：日志保存在 `openclaw.log`
4. **API Key**：配置文件中包含用户的 API Key，请妥善保管

## 重新打包步骤

如需重新打包（在有网络环境下）：

1. 安装 Node.js 22.12+（或使用系统现有版本）
2. 安装 cmake（用于编译 node-llama-cpp）：
   ```bash
   wget -q https://github.com/Kitware/CMake/releases/download/v3.28.1/cmake-3.28.1-linux-x86_64.tar.gz
   sudo tar -xzf cmake-3.28.1-linux-x86_64.tar.gz -C /opt
   sudo ln -sf /opt/cmake-3.28.1-linux-x86_64/bin/cmake /usr/local/bin/cmake
   ```
3. 全局安装 OpenClaw：
   ```bash
   npm install -g openclaw
   ```
4. 复制到 resources 目录：
   ```bash
   OPENCLAW_NPM_DIR="$(npm root -g)/openclaw"
   cp -r "$OPENCLAW_NPM_DIR/openclaw.mjs" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/dist" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/package.json" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/skills" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/extensions" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/assets" resources/openclaw/
   cp -r "$OPENCLAW_NPM_DIR/node_modules" resources/openclaw/
   ```
5. 复制配置文件：
   ```bash
   cp -r ~/.openclaw/* resources/openclaw/data/
   ```

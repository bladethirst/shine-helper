#!/bin/bash

# Shine Helper 离线便携版打包脚本 (Linux/macOS)
# 使用方法：chmod +x package-offline.sh && ./package-offline.sh

OUTPUT_DIR="./dist/portable"

echo "======================================"
echo "Shine Helper 离线便携版打包工具"
echo "======================================"
echo ""

# 检查 Node.js
echo "[1/5] 检查环境..."
NODE_VERSION=$(node --version 2>/dev/null)
if [ -z "$NODE_VERSION" ]; then
    echo "错误：需要安装 Node.js 22.x"
    echo "请访问 https://nodejs.org/"
    exit 1
fi
echo "Node.js: $NODE_VERSION"

NPM_VERSION=$(npm --version 2>/dev/null)
if [ -z "$NPM_VERSION" ]; then
    echo "错误：npm 未找到"
    exit 1
fi
echo "npm: $NPM_VERSION"

# 创建输出目录
echo ""
echo "[2/5] 准备输出目录..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/resources/openclaw"
mkdir -p "$OUTPUT_DIR/resources/.openclaw"

# 复制 Tauri 构建产物
echo ""
echo "[3/5] 复制应用文件..."

# 查找 Linux 构建产物
TARGET_BIN=""
if [ -f "src-tauri/target/release/shine-helper" ]; then
    TARGET_BIN="src-tauri/target/release/shine-helper"
fi

if [ -n "$TARGET_BIN" ]; then
    cp "$TARGET_BIN" "$OUTPUT_DIR/shine-helper"
    chmod +x "$OUTPUT_DIR/shine-helper"
    echo "  已复制：shine-helper"
else
    echo "  警告：未找到 Tauri Linux 构建产物"
    echo "  请先运行：npm run tauri build"
    echo "  或者手动复制 shine-helper 到输出目录"
fi

# 复制 resources 文件
# 复制 Node.js 运行时
if [ -d "resources/node" ]; then
    cp -r resources/node "$OUTPUT_DIR/resources/"
    echo "  已复制：resources/node/"
fi

# 复制 OpenClaw 应用
if [ -d "resources/openclaw" ]; then
    cp -r resources/openclaw/* "$OUTPUT_DIR/resources/openclaw/"
    echo "  已复制：resources/openclaw/"
fi

# 复制 OpenClaw 配置文件
if [ -f "resources/openclaw.json" ]; then
    cp "resources/openclaw.json" "$OUTPUT_DIR/resources/"
    echo "  已复制：resources/openclaw.json"
fi

# 复制启动脚本（可选，用户现在可以直接运行 shine_helper）
echo ""
echo "[4/5] 复制启动脚本..."
if [ -f "start.sh" ]; then
    cp "start.sh" "$OUTPUT_DIR/"
    chmod +x "$OUTPUT_DIR/start.sh"
    echo "  已复制：start.sh (可选)"
else
    echo "  提示：start.sh 不是必需的，用户可以直接运行 shine_helper"
fi

# 创建 README
cat > "$OUTPUT_DIR/README.txt" << 'EOF'
# Shine Helper 便携版 (Linux)

## 使用说明

### 方式一：直接运行主程序（推荐）

1. 解压到任意目录
2. 双击运行 shine_helper 可执行文件
3. 程序会自动启动 OpenClaw 服务并加载应用

### 方式二：使用启动脚本

1. 解压到任意目录
2. 给 start.sh 添加执行权限：chmod +x start.sh
3. 执行：./start.sh

## 文件说明

- shine-helper - 主程序（双击直接运行）
- start.sh - 启动脚本（可选，与直接运行 shine-helper 效果相同）
- resources/
  - node/ - Node.js 运行时 (v25.x)
  - openclaw/ - OpenClaw 应用目录
    - openclaw.mjs - OpenClaw 主程序
    - node_modules/ - OpenClaw 依赖
  - .openclaw/ - OpenClaw 状态目录 (由程序自动创建)
  - openclaw.json - OpenClaw 配置文件（包含 gateway token）

## 离线打包步骤

如需在离线环境使用，需要预打包 OpenClaw：

1. 下载 Node.js 22.x Linux x64:
   https://nodejs.org/dist/v22.x.x/node-v22.x.x-linux-x64.tar.xz

2. 解压到 resources/openclaw/node/

3. 安装 OpenClaw:
   在联网环境运行：npm install -g openclaw@latest
   导出：npm pack openclaw
   解压 tarball 到 resources/openclaw/openclaw/

4. 在 resources/openclaw/openclaw/ 运行:
   npm install --production

## 功能说明

- 自动启动 OpenClaw 服务（端口 18789）
- 自动创建必要的配置目录（~/Desktop/workspace/.openclaw）
- 日志文件位于：/tmp/openclaw.log
- 支持语音唤醒功能（需在配置中启用）

## 故障排除

如果启动失败，请检查：
1. resources/node/bin/node 是否存在且可执行
2. resources/openclaw/openclaw.mjs 是否存在
3. resources/openclaw.json 配置文件是否正确
4. resources/.openclaw 目录是否存在
5. 端口 18789 是否被占用
6. 查看 /tmp/openclaw.log 日志文件
EOF

echo "  已创建：README.txt"

# 打包为 tar.gz
echo ""
echo "[5/5] 生成压缩包..."
TAR_PATH="./dist/shine_helper_portable.tar.gz"
rm -f "$TAR_PATH"
mkdir -p ./dist
tar -czf "$TAR_PATH" -C "$OUTPUT_DIR" .

echo ""
echo "======================================"
echo "打包完成！"
echo "======================================"
echo ""
echo "输出目录：$OUTPUT_DIR"
echo "压缩包：$TAR_PATH"
echo ""
echo "提示：压缩包可直接分发，无需安装"
echo "用户只需运行 shine_helper 可执行文件即可，无需执行 start.sh"

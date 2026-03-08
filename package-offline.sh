#!/bin/bash

# Shine Helper 离线便携版打包脚本 (Linux/macOS)
# 使用方法: chmod +x package-offline.sh && ./package-offline.sh

OUTPUT_DIR="./dist/portable"

echo "======================================"
echo "Shine Helper 离线便携版打包工具"
echo "======================================"
echo ""

# 检查 Node.js
echo "[1/5] 检查环境..."
NODE_VERSION=$(node --version 2>/dev/null)
if [ -z "$NODE_VERSION" ]; then
    echo "错误: 需要安装 Node.js 22.x"
    echo "请访问 https://nodejs.org/"
    exit 1
fi
echo "Node.js: $NODE_VERSION"

NPM_VERSION=$(npm --version 2>/dev/null)
if [ -z "$NPM_VERSION" ]; then
    echo "错误: npm 未找到"
    exit 1
fi
echo "npm: $NPM_VERSION"

# 创建输出目录
echo ""
echo "[2/5] 准备输出目录..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/resources/openclaw/data"

# 复制 Tauri 构建产物
echo ""
echo "[3/5] 复制应用文件..."

# 查找 Linux 构建产物 (无扩展名)
TARGET_BIN=""
if [ -f "src-tauri/target/release/shine_helper" ]; then
    TARGET_BIN="src-tauri/target/release/shine_helper"
fi

if [ -n "$TARGET_BIN" ]; then
    cp "$TARGET_BIN" "$OUTPUT_DIR/shine_helper"
    chmod +x "$OUTPUT_DIR/shine_helper"
    echo "  已复制: shine_helper"
else
    echo "  警告: 未找到 Tauri Linux 构建产物"
    echo "  请先运行: npm run tauri build"
    echo "  或者手动复制 shine_helper 到输出目录"
fi

# 复制 resources 文件
if [ -d "resources/openclaw" ]; then
    cp -r resources/openclaw/* "$OUTPUT_DIR/resources/openclaw/"
    echo "  已复制: resources/openclaw/"
fi

# 复制启动脚本
echo ""
echo "[4/5] 复制启动脚本..."
if [ -f "start.sh" ]; then
    cp "start.sh" "$OUTPUT_DIR/"
    chmod +x "$OUTPUT_DIR/start.sh"
    echo "  已复制: start.sh"
else
    echo "  警告: 未找到 start.sh"
fi

# 创建 README
cat > "$OUTPUT_DIR/README.txt" << 'EOF'
# Shine Helper 便携版 (Linux)

## 使用说明

1. 解压到任意目录
2. 给 start.sh 添加执行权限: chmod +x start.sh
3. 执行: ./start.sh
4. 首次使用需要配置 OpenClaw（如果未预打包）

## 文件说明

- start.sh - 启动脚本（执行 chmod +x 后运行）
- shine_helper - 主程序
- resources/openclaw/ - OpenClaw 运行时（需自行配置）

## 离线打包步骤

如需在离线环境使用，需要预打包 OpenClaw：

1. 下载 Node.js 22.x Linux x64:
   https://nodejs.org/dist/v22.x.x/node-v22.x.x-linux-x64.tar.xz

2. 解压到 resources/openclaw/node/

3. 安装 OpenClaw:
   在联网环境运行: npm install -g openclaw@latest
   导出: npm pack openclaw
   解压 tarball 到 resources/openclaw/openclaw/

4. 在 resources/openclaw/openclaw/ 运行:
   npm install --production

## 故障排除

如果启动失败，请检查：
1. resources/openclaw/node/bin/node 是否存在且可执行
2. resources/openclaw/openclaw/gateway.js 是否存在
3. 端口 18789 是否被占用
EOF

echo "  已创建: README.txt"

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
echo "输出目录: $OUTPUT_DIR"
echo "压缩包: $TAR_PATH"
echo ""
echo "提示: 压缩包可直接分发，无需安装"
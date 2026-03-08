#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCLAW_DIR="$SCRIPT_DIR/resources/openclaw"
SHINE_BIN="$SCRIPT_DIR/src-tauri/target/release/shine-helper"

echo "[Shine Helper] 检查运行环境..."

# 使用系统 Node.js
NODE_CMD="node"
if ! command -v $NODE_CMD &> /dev/null; then
    echo "[错误] 未找到 Node.js 运行时"
    read -p "按回车键退出..."
    exit 1
fi

# 检查必要文件
if [ ! -f "$OPENCLAW_DIR/openclaw.mjs" ]; then
    echo "[错误] 未找到 OpenClaw 应用"
    echo "请确保 resources/openclaw 目录存在"
    read -p "按回车键退出..."
    exit 1
fi

if [ ! -d "$OPENCLAW_DIR/node_modules" ]; then
    echo "[错误] 未找到 OpenClaw 依赖"
    echo "请确保 resources/openclaw/node_modules 目录存在"
    read -p "按回车键退出..."
    exit 1
fi

if [ ! -x "$SHINE_BIN" ]; then
    echo "[错误] 未找到 shine_helper 可执行文件"
    echo "请先编译项目: npm run tauri build"
    read -p "按回车键退出..."
    exit 1
fi

echo "[Shine Helper] 检查 OpenClaw 服务状态..."

# 设置 OpenClaw 配置目录
export OPENCLAW_CONFIG_PATH="$OPENCLAW_DIR/data/openclaw.json"

# 检查端口 18789 是否已被占用
if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
    echo "[Shine Helper] OpenClaw 服务已在运行"
else
    echo "[Shine Helper] 启动 OpenClaw 服务..."
    
    # 创建 data 目录
    mkdir -p "$OPENCLAW_DIR/data"
    
    # 启动 OpenClaw（后台运行）
    cd "$OPENCLAW_DIR"
    export OPENCLAW_CONFIG_PATH="$OPENCLAW_DIR/data/openclaw.json"
    nohup $NODE_CMD openclaw.mjs gateway run --port 18789 > "$OPENCLAW_DIR/openclaw.log" 2>&1 &
    OPENCLAW_PID=$!
    
    echo "[Shine Helper] 等待服务启动..."
    sleep 10
    
    # 再次检查
    if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
        echo "[Shine Helper] OpenClaw 服务已就绪"
    else
        echo "[警告] OpenClaw 可能启动失败，请查看日志: $OPENCLAW_DIR/openclaw.log"
    fi
fi

echo "[Shine Helper] 启动桌面应用..."
"$SHINE_BIN"

echo "[Shine Helper] 启动完成"
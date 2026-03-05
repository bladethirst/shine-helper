#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCLAW_DIR="$SCRIPT_DIR/resources/openclaw"
NODE_DIR="$OPENCLAW_DIR/node"
OPENCLAW_APP_DIR="$OPENCLAW_DIR/openclaw"
SHINE_BIN="$SCRIPT_DIR/shine_helper"

echo "[Shine Helper] 检查运行环境..."

# 检查必要文件
if [ ! -x "$NODE_DIR/bin/node" ]; then
    echo "[错误] 未找到 Node.js 运行时"
    echo "请确保 resources/openclaw/node 目录存在 Node.js"
    read -p "按回车键退出..."
    exit 1
fi

if [ ! -f "$OPENCLAW_APP_DIR/package.json" ]; then
    echo "[错误] 未找到 OpenClaw 应用"
    echo "请确保 resources/openclaw/openclaw 目录存在"
    read -p "按回车键退出..."
    exit 1
fi

if [ ! -x "$SHINE_BIN" ]; then
    echo "[错误] 未找到 shine_helper 可执行文件"
    read -p "按回车键退出..."
    exit 1
fi

echo "[Shine Helper] 检查 OpenClaw 服务状态..."

# 检查端口 18789 是否已被占用
if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
    echo "[Shine Helper] OpenClaw 服务已在运行"
else
    echo "[Shine Helper] 启动 OpenClaw 服务..."
    
    # 创建 data 目录
    mkdir -p "$OPENCLAW_DIR/data"
    
    # 启动 OpenClaw（后台运行）
    cd "$OPENCLAW_APP_DIR"
    nohup "$NODE_DIR/bin/node" "$OPENCLAW_APP_DIR/gateway.js" --port 18789 --data "$OPENCLAW_DIR/data" > /dev/null 2>&1 &
    OPENCLAW_PID=$!
    
    echo "[Shine Helper] 等待服务启动..."
    sleep 8
    
    # 再次检查
    if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
        echo "[Shine Helper] OpenClaw 服务已就绪"
    else
        echo "[警告] OpenClaw 可能启动失败，继续尝试启动应用..."
    fi
fi

echo "[Shine Helper] 启动桌面应用..."
"$SHINE_BIN"

echo "[Shine Helper] 启动完成"
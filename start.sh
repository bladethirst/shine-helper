#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCLAW_DIR="$SCRIPT_DIR/resources/openclaw"
NODE_DIR="$OPENCLAW_DIR/node"
OPENCLAW_APP_DIR="$OPENCLAW_DIR/openclaw"

SHINE_BIN="$SCRIPT_DIR/shine_helper"
if [ ! -x "$SHINE_BIN" ]; then
    SHINE_BIN="$SCRIPT_DIR/src-tauri/target/release/shine-helper"
fi
if [ ! -x "$SHINE_BIN" ]; then
    SHINE_BIN="$SCRIPT_DIR/src-tauri/target/debug/shine-helper"
fi

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
    echo "尝试的位置:"
    echo "  - $SCRIPT_DIR/shine_helper"
    echo "  - $SCRIPT_DIR/src-tauri/target/release/shine-helper"
    echo "  - $SCRIPT_DIR/src-tauri/target/debug/shine-helper"
    echo ""
    echo "请先运行: cd src-tauri && cargo build --release"
    read -p "按回车键退出..."
    exit 1
fi

echo "[Shine Helper] 检查 OpenClaw 服务状态..."

# 检查端口 18789 是否已被占用
if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
    echo "[Shine Helper] OpenClaw 服务已在运行"
else
    echo "[Shine Helper] 启动 OpenClaw 服务..."

    # 检查是否需要初始化配置
    if [ ! -f "$HOME/.openclaw/openclaw.json" ]; then
        echo "[Shine Helper] 初始化 OpenClaw 配置..."
        cd "$OPENCLAW_APP_DIR"
        "$NODE_DIR/bin/node" "$OPENCLAW_APP_DIR/openclaw.mjs" onboard --non-interactive --accept-risk > /tmp/openclaw-setup.log 2>&1
    fi

    cd "$OPENCLAW_APP_DIR"
    nohup "$NODE_DIR/bin/node" "$OPENCLAW_APP_DIR/openclaw.mjs" gateway run --port 18789 > /tmp/openclaw.log 2>&1 &
    OPENCLAW_PID=$!
    
    echo "[Shine Helper] 等待服务启动..."
    sleep 8
    
    # 再次检查
    if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
        echo "[Shine Helper] OpenClaw 服务已就绪"
    else
        echo "[警告] OpenClaw 可能启动失败，检查日志: /tmp/openclaw.log"
        echo "日志内容:"
        tail -20 /tmp/openclaw.log
    fi
fi

echo "[Shine Helper] 启动桌面应用..."
"$SHINE_BIN"

echo "[Shine Helper] 启动完成"
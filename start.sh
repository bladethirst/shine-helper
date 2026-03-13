#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCLAW_DIR="$SCRIPT_DIR/resources/openclaw"
NODE_DIR="$OPENCLAW_DIR/node"
OPENCLAW_APP_DIR="$OPENCLAW_DIR/openclaw"

# OpenClaw 配置文件路径
OPENCLAW_CONFIG_PATH="$OPENCLAW_DIR/data/openclaw.json"

# OpenClaw 状态目录（用户桌面 workspace）
# 使用实际的用户家目录，避免权限问题
OPENCLAW_STATE_DIR="$HOME/Desktop/workspace/.openclaw"

# OpenClaw 主目录
OPENCLAW_HOME="$OPENCLAW_APP_DIR"

# 确保使用当前用户的实际家目录
REAL_HOME="$HOME"

SHINE_BIN="$SCRIPT_DIR/shine_helper"
if [ ! -x "$SHINE_BIN" ]; then
    SHINE_BIN="$SCRIPT_DIR/src-tauri/target/release/shine-helper"
fi
if [ ! -x "$SHINE_BIN" ]; then
    SHINE_BIN="$SCRIPT_DIR/src-tauri/target/debug/shine-helper"
fi

echo "[Shine Helper] 检查运行环境..."
echo "[Shine Helper] 当前用户：$(whoami)"
echo "[Shine Helper] HOME 目录：$HOME"
echo "[Shine Helper] OpenClaw 工作目录：$OPENCLAW_STATE_DIR"

# 创建 Desktop 目录（如果不存在）
if [ ! -d "$REAL_HOME/Desktop" ]; then
    echo "[Shine Helper] 创建 Desktop 目录：$REAL_HOME/Desktop"
    mkdir -p "$REAL_HOME/Desktop"
fi

# 创建 OpenClaw 状态目录（桌面 workspace）
if [ ! -d "$OPENCLAW_STATE_DIR" ]; then
    echo "[Shine Helper] 创建 OpenClaw 工作目录：$OPENCLAW_STATE_DIR"
    mkdir -p "$OPENCLAW_STATE_DIR"
fi

# 检查必要文件
if [ ! -x "$NODE_DIR/bin/node" ]; then
    echo "[错误] 未找到 Node.js 运行时"
    echo "请确保 resources/openclaw/node 目录存在 Node.js"
    read -p "按回车键退出..."
    exit 1
fi

if [ ! -f "$OPENCLAW_CONFIG_PATH" ]; then
    echo "[错误] 未找到 OpenClaw 配置文件"
    echo "配置文件路径：$OPENCLAW_CONFIG_PATH"
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
    echo "请先运行：cd src-tauri && cargo build --release"
    read -p "按回车键退出..."
    exit 1
fi

echo "[Shine Helper] 检查 OpenClaw 服务状态..."

# 检查端口 18789 是否已被占用
if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
    echo "[Shine Helper] OpenClaw 服务已在运行"
else
    echo "[Shine Helper] 启动 OpenClaw 服务..."

    # 设置 OpenClaw 环境变量
    export OPENCLAW_HOME="$OPENCLAW_HOME"
    export OPENCLAW_STATE_DIR="$OPENCLAW_STATE_DIR"
    export OPENCLAW_CONFIG_PATH="$OPENCLAW_CONFIG_PATH"

    echo "[Shine Helper] OpenClaw 环境变量:"
    echo "  OPENCLAW_HOME=$OPENCLAW_HOME"
    echo "  OPENCLAW_STATE_DIR=$OPENCLAW_STATE_DIR"
    echo "  OPENCLAW_CONFIG_PATH=$OPENCLAW_CONFIG_PATH"

    cd "$OPENCLAW_APP_DIR"
    OPENCLAW_HOME="$OPENCLAW_HOME" OPENCLAW_STATE_DIR="$OPENCLAW_STATE_DIR" OPENCLAW_CONFIG_PATH="$OPENCLAW_CONFIG_PATH" \
        nohup "$NODE_DIR/bin/node" "$OPENCLAW_APP_DIR/openclaw.mjs" gateway run --port 18789 > /tmp/openclaw.log 2>&1 &
    OPENCLAW_PID=$!

    echo "[Shine Helper] 等待服务启动..."
    sleep 8

    # 再次检查
    if netstat -ano 2>/dev/null | grep -q ":18789 " || ss -tuln 2>/dev/null | grep -q ":18789 "; then
        echo "[Shine Helper] OpenClaw 服务已就绪"
    else
        echo "[错误] OpenClaw 服务启动失败"
        echo "日志内容:"
        tail -20 /tmp/openclaw.log
        echo ""
        echo "[提示] 可能原因:"
        echo "  1. Node.js 未正确安装到 resources/openclaw/node 目录"
        echo "  2. OpenClaw 配置文件错误"
        echo "  3. 端口 18789 被其他程序占用"
        echo ""
        read -p "按回车键退出..."
        exit 1
    fi
fi

echo "[Shine Helper] 启动桌面应用..."
"$SHINE_BIN"

echo "[Shine Helper] 启动完成"

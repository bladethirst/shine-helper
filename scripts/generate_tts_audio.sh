#!/bin/bash
# TTS 音频生成脚本 - 使用 edge-tts 生成中文语音回复
# 使用方法：./generate_tts_audio.sh

set -e

OUTPUT_DIR="src-tauri/resources/tts"
mkdir -p "$OUTPUT_DIR"

echo "检查 edge-tts 是否已安装..."
if ! command -v edge-tts &> /dev/null; then
    echo "edge-tts 未安装，正在安装..."
    pip3 install edge-tts
fi

echo "生成 TTS 音频文件..."

# 使用 Azure 中文女声 zh-CN-XiaoxiaoNeural
VOICE="zh-CN-XiaoxiaoNeural"

# 生成唤醒回复语音
phrases=("我在" "请说" "在的" "在呢")

for phrase in "${phrases[@]}"; do
    echo "生成：$phrase"
    edge-tts --voice "$VOICE" --text "$phrase" --write-media "$OUTPUT_DIR/${phrase}.mp3"
    echo "  ✓ $OUTPUT_DIR/${phrase}.mp3"
done

echo ""
echo "所有音频文件生成完成！"
echo "位置：$OUTPUT_DIR"
ls -la "$OUTPUT_DIR"

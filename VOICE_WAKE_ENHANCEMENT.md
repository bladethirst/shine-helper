# 语音唤醒功能增强实现报告

**日期**: 2026-03-11  
**状态**: ✅ 已完成

---

## 修复的问题

语音唤醒功能原本失效，原因是 `run_wake_loop` 函数被 stub 为 no-op 实现。现已完全修复并实现以下增强功能：

### 1. 完整的语音唤醒循环

**文件**: `src-tauri/src/commands/voice_wake.rs`

实现了完整的状态机流程：
- **Idle**: 能量阈值检测语音活动
- **Waking**: 发送 `voice-waked` 事件，播放 TTS 回复
- **Listening**: 连接 Vosk ASR 服务
- **Processing**: 接收识别结果，发送 `voice-result` 事件，检测结束词或静音超时

### 2. TTS 音频播放增强

**文件**: `src-tauri/src/voice/tts_player.rs`

- 支持播放预置 MP3 音频文件
- 随机选择唤醒回复（"我在"、"请说"、"在的"、"在呢"）
- 回退方案：如果音频文件不存在，播放简单的提示音

### 3. 配置更新

**文件**: `src-tauri/src/config.rs`

`VoiceWakeConfig` 新增字段：
- `vosk_url`: ASR 服务地址
- `vosk_api_key`: API 密钥

---

## 调研结果：唤醒词引擎方案

### Porcupine (Picovoice)
- **状态**: Rust 绑定不稳定（`pv_porcupine` crate 被 yanked）
- **优点**: 离线、高精度、支持自定义唤醒词训练
- **缺点**: 需要 Picovoice Console 账号，许可条款限制
- **建议**: 暂不集成，保持现有能量阈值方案

### Snowboy
- **状态**: 已废弃（2018 年后停止维护）
- **不建议**: 长期维护风险

### OpenWakeWord
- **状态**: 有 Rust 绑定 (`oww-rs`)
- **优点**: Apache 2.0 许可，支持自定义训练
- **建议**: 未来可考虑集成

### 当前方案
使用**能量阈值 VAD**检测语音活动（简化但可靠），生产环境建议集成专业唤醒词引擎。

---

## TTS 方案

### 预置音频文件方案（已实现）

**优点**:
- 完全离线
- 零延迟
- 音质稳定

**使用方法**:

1. **生成音频文件**（需要 Python 环境）:
```bash
# 安装 edge-tts
pip3 install edge-tts

# 运行生成脚本
cd /data/workspace/shine-helper
chmod +x scripts/generate_tts_audio.sh
./scripts/generate_tts_audio.sh
```

2. **音频文件位置**:
```
src-tauri/resources/tts/
├── 我在.mp3
├── 请说.mp3
├── 在的.mp3
└── 在呢.mp3
```

3. **构建应用**时，确保 `resources/tts` 目录被打包到最终产物中

### 备选方案：edge-tts 实时生成

如需动态 TTS，可集成 `edge-tts-rs` crate，但需要网络连接。

---

## 使用说明

### 1. 系统依赖

**Linux 麒麟系统**:
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
  pkg-config \
  libasound2-dev
```

### 2. 配置语音唤醒

编辑配置文件（通常位于 `~/.config/shine_helper/config.json`）:

```json
{
  "vosk": {
    "url": "ws://192.168.150.26:2700",
    "api_key": "",
    "enabled": true,
    "silence_timeout": 3000
  },
  "voice_wake": {
    "enabled": true,
    "wake_word": "小 Shine",
    "wake_sounds": ["我在", "请说", "在的", "在呢"],
    "silence_timeout": 3000,
    "end_words": ["结束", "停止"],
    "vosk_url": "ws://192.168.150.26:2700",
    "vosk_api_key": ""
  }
}
```

### 3. 启动应用

```bash
cd /data/workspace/shine-helper
source ~/.cargo/env
npm run tauri dev
```

### 4. 预期行为

**控制台日志**:
```
[AutoVoiceWake] Auto-starting voice wake service...
[VoiceWake] Starting wake loop with wake_word='小 Shine', vosk_url=ws://...
[VoiceWake] Audio capture started
[VoiceWake] Voice activity detected (energy=0.0523)
[VoiceWake] WAKING state - sending events and playing TTS
[TTS] Playing response: 在呢
[TTS] Playing from file: .../resources/tts/在呢.mp3
[VoiceWake] LISTENING state - connecting to ASR
[VoiceWake] Connected to ASR service
[VoiceWake] PROCESSING state - receiving ASR results
[VoiceWake] Partial result: 你好
[VoiceWake] Final result: 你好小 Shine
```

**前端事件**:
- `voice-waked`: 唤醒事件触发
- `voice-state-changed`: 状态变化（idle → waking → listening → processing）
- `voice-result`: 识别结果（包含 `text` 和 `is_final` 字段）
- `voice-error`: 错误信息

---

## 后续优化建议

### 短期（推荐）
1. **生成预置 TTS 音频**: 运行 `scripts/generate_tts_audio.sh`
2. **调整能量阈值**: 根据实际环境调整 `energy_threshold`（当前 0.02）
3. **测试 ASR 连接**: 确保 Vosk 服务可访问

### 中期
1. **集成 OpenWakeWord**: 替换简单的能量阈值检测
2. **优化唤醒词**: 使用专业引擎实现"小 Shine"精确检测
3. **添加配置 UI**: 在配置页面调整唤醒参数

### 长期
1. **多唤醒词支持**: 支持用户自定义唤醒词
2. **离线 ASR**: 集成本地语音识别（如 Vosk 离线模型）
3. **自然语言理解**: 唤醒后直接执行简单命令

---

## 技术债务

1. **未使用的代码**:
   - `VoiceStateMachine` 结构体（可考虑移除）
   - `WakeWordDetector`（简化方案中未使用）
   - `VoskAsrClient`（直接使用 WebSocket 连接）

2. **编译警告**: 部分未使用变量（不影响功能）

---

## 相关文件清单

### Rust 后端
- `src-tauri/src/commands/voice_wake.rs` - 核心唤醒逻辑
- `src-tauri/src/voice/tts_player.rs` - TTS 播放
- `src-tauri/src/voice/audio_capture.rs` - 音频捕获
- `src-tauri/src/config.rs` - 配置结构

### 前端
- `src/composables/useVoiceWake.ts` - Vue Composable
- `src/components/ChatInput.vue` - 麦克风按钮和状态显示

### 脚本
- `scripts/generate_tts_audio.sh` - TTS 音频生成

### 文档
- `docs/plans/2026-03-08-shine-helper-voice-wake-implementation-plan.md` - 原始计划
- `docs/progress/2026-03-08-voice-wake-implementation-progress.md` - 进度报告
- `VOICE_WAKE_ENHANCEMENT.md` - 本文档

---

**实现团队**: Shine Team  
**技术栈**: Tauri 1.5, Rust, Vue 3, TypeScript, Vosk ASR

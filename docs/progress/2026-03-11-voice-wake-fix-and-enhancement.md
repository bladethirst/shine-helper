# 语音唤醒功能修复与增强报告

**日期**: 2026-03-11  
**状态**: ✅ 已完成  
**分支**: `home-dev`

---

## 问题背景

语音唤醒功能在启动应用后控制台不显示语音监听日志，功能实际未工作。经分析，根本原因是 `run_wake_loop` 函数被 stub 为 no-op 实现，导致唤醒循环从未实际执行。

---

## 问题分析

### 原始代码问题

**文件**: `src-tauri/src/commands/voice_wake.rs` (第 327-333 行)

```rust
// Stubbed helpers to satisfy compilation in this simplified patch.
async fn run_wake_loop(_is_running: Arc<Mutex<bool>>, _app: AppHandle, _wake_word: String, _wake_sounds: Vec<String>, _silence_timeout_ms: u32, _vosk_url: String) {
    // no-op in patch
}
```

**问题**:
1. `run_wake_loop` 函数是空实现，不执行任何操作
2. `start_voice_wake` 命令只设置标志位，不启动实际循环
3. 没有发送任何 Tauri 事件（`voice-waked`, `voice-result` 等）
4. 配置缺少 ASR URL 字段

---

## 实现方案

### 1. 完整的语音唤醒循环实现

**文件**: `src-tauri/src/commands/voice_wake.rs`

实现了完整的状态机流程：

```
Idle ──[检测到语音活动]──▶ Waking ──[TTS 播放完成]──▶ Listening
   ▲                                                  │
   │                    ┌────[静音超时]──────────────┤
   │                    └────[检测到"结束"]──────────┤
   └────────────────────┴───────────────────────────┘
```

**状态说明**:

| 状态 | 行为 | 事件 |
|------|------|------|
| **Idle** | 能量阈值检测语音活动 | - |
| **Waking** | 发送唤醒事件，播放 TTS 回复 | `voice-waked`, `voice-state-changed` |
| **Listening** | 连接 Vosk ASR WebSocket | `voice-state-changed` |
| **Processing** | 发送音频流，接收识别结果 | `voice-result` |

**关键代码片段**:

```rust
async fn run_wake_loop(
    is_running: Arc<Mutex<bool>>,
    app: AppHandle,
    wake_word: String,
    wake_sounds: Vec<String>,
    silence_timeout_ms: u32,
    vosk_url: String,
    vosk_api_key: String,
    end_words: Vec<String>,
) {
    // 1. 启动音频捕获 (cpal, 16kHz 单声道)
    // 2. 能量阈值检测语音活动
    // 3. 状态机流转
    // 4. 发送 Tauri 事件
    // 5. 连接 ASR 并处理识别结果
}
```

### 2. TTS 播放增强

**文件**: `src-tauri/src/voice/tts_player.rs`

**功能**:
- 支持播放预置 MP3 音频文件（`resources/tts/*.mp3`）
- 随机选择唤醒回复（"我在"、"请说"、"在的"、"在呢"）
- 文件不存在时回退到提示音

```rust
pub fn play_wake_response(&self) -> Result<(), String> {
    let response = self.wake_sounds
        .choose(&mut rand::thread_rng())
        .cloned()
        .unwrap_or_else(|| "在呢".to_string());
    
    // 优先播放预置音频文件
    if let Some(ref resource_dir) = self.resource_dir {
        let mp3_path = resource_dir.join(format!("{}.mp3", response));
        if mp3_path.exists() {
            return self.play_audio_file(mp3_path.to_str().unwrap());
        }
    }
    
    // 回退到提示音
    self.play_beep_sound()
}
```

### 3. 配置扩展

**文件**: `src-tauri/src/config.rs`

`VoiceWakeConfig` 新增字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWakeConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    pub vosk_url: String,        // 新增
    pub vosk_api_key: String,    // 新增
}
```

---

## 技术调研结果

### 唤醒词引擎方案对比

| 方案 | 状态 | Rust 支持 | 离线 | 自定义训练 | 许可 | 建议 |
|------|------|-----------|------|------------|------|------|
| **Porcupine** | 活跃 | ⚠️ 不稳定 (crate yanked) | ✅ | ✅ (Console) | Apache 2.0 (Free Tier 限制) | 暂不集成 |
| **Snowboy** | ❌ 已废弃 | ✅ (rsnowboy) | ✅ | ✅ | Apache 2.0 | 不推荐 |
| **OpenWakeWord** | 活跃 | ✅ (oww-rs) | ✅ | ✅ (云端训练) | Apache 2.0 | 未来选项 |
| **当前方案** | - | 原生 Rust | ✅ | ❌ (能量阈值) | - | 简单可靠 |

**结论**: 暂时使用能量阈值 VAD 检测（简单可靠），生产环境建议集成 OpenWakeWord 或 Porcupine。

### TTS 方案对比

| 方案 | 离线 | 音质 | 延迟 | 依赖 | 建议 |
|------|------|------|------|------|------|
| **预置音频** | ✅ | 高 (专业录音) | 零 | 无 | ✅ 已实现 |
| **edge-tts** | ❌ | 高 (神经网络) | 中 (网络) | 网络 + API Key | 备选 |
| **espeak-ng** | ✅ | 低 (机械音) | 低 | 系统包 | 备选 |
| **Kokoro TTS** | ✅ | 中 (神经网络) | 中 (模型推理) | 模型文件 | 未来选项 |

**结论**: 采用预置音频文件方案（完全离线、零延迟、音质稳定）。

---

## 文件变更清单

### Rust 后端

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src-tauri/src/commands/voice_wake.rs` | 重写 | 完整实现 `run_wake_loop` |
| `src-tauri/src/voice/tts_player.rs` | 增强 | 支持预置音频文件播放 |
| `src-tauri/src/config.rs` | 扩展 | 添加 `vosk_url`, `vosk_api_key` |
| `src-tauri/Cargo.toml` | 修改 | 添加 `rand = "0.8"` 依赖 |

### 脚本

| 文件 | 说明 |
|------|------|
| `scripts/generate_tts_audio.sh` | TTS 音频生成脚本（使用 edge-tts） |

### 文档

| 文件 | 说明 |
|------|------|
| `docs/progress/2026-03-11-voice-wake-fix-and-enhancement.md` | 本文档 |
| `VOICE_WAKE_ENHANCEMENT.md` | 完整技术文档 |

---

## 使用说明

### 1. 生成 TTS 音频文件

**前提**: Python 3 环境

```bash
# 安装 edge-tts
pip3 install edge-tts

# 生成音频文件
cd /data/workspace/shine-helper
chmod +x scripts/generate_tts_audio.sh
./scripts/generate_tts_audio.sh
```

**输出位置**:
```
src-tauri/resources/tts/
├── 我在.mp3
├── 请说.mp3
├── 在的.mp3
└── 在呢.mp3
```

### 2. 系统依赖（Linux 麒麟）

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

### 3. 配置语音唤醒

**配置文件**: `~/.config/shine_helper/config.json`

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

### 4. 启动应用

```bash
cd /data/workspace/shine-helper
source ~/.cargo/env
npm run tauri dev
```

### 5. 预期行为

**控制台日志**:
```
[AutoVoiceWake] Auto-starting voice wake service...
[VoiceWake] Starting wake loop with wake_word='小 Shine', vosk_url=ws://192.168.150.26:2700
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
[VoiceWake] End word '结束' detected
[VoiceWake] Processing ended: end_word
```

**前端事件** (通过 `useVoiceWake.ts` 监听):
- `voice-waked`: 唤醒事件触发
- `voice-state-changed`: 状态变化（payload: `{ state: "idle" | "waking" | "listening" | "processing" }`）
- `voice-result`: 识别结果（payload: `{ text: string, is_final: boolean }`）
- `voice-error`: 错误信息（payload: `{ message: string }`）

---

## 验证编译

```bash
cd /data/workspace/shine-helper/src-tauri
source ~/.cargo/env
cargo check
```

**结果**: ✅ 编译成功（41 个警告，均为未使用代码，不影响功能）

---

## 后续优化建议

### 短期（推荐）

1. ✅ **生成预置 TTS 音频** - 运行 `scripts/generate_tts_audio.sh`
2. ⚠️ **调整能量阈值** - 根据实际环境调整 `energy_threshold`（当前 0.02）
3. ⚠️ **测试 ASR 连接** - 确保 Vosk 服务可访问

### 中期

1. **集成 OpenWakeWord** - 替换简单的能量阈值检测，实现精确唤醒词识别
2. **优化唤醒词** - 使用专业引擎实现"小 Shine"精确检测
3. **添加配置 UI** - 在配置页面调整唤醒参数（阈值、超时等）

### 长期

1. **多唤醒词支持** - 支持用户自定义唤醒词
2. **离线 ASR** - 集成本地语音识别（如 Vosk 离线模型）
3. **自然语言理解** - 唤醒后直接执行简单命令

---

## 技术债务

1. **未使用的代码** (可考虑移除):
   - `VoiceStateMachine` 结构体
   - `WakeWordDetector` 结构体
   - `VoskAsrClient` 结构体

2. **编译警告** (不影响功能):
   - 未使用变量（`sample_rate`, `channels`, `from_channels`）
   - 未使用导入（`SampleFormat`, `Source` 等）

---

## 提交历史

```bash
# 查看相关提交
git log --oneline --grep="voice" --since="2026-03-08"
```

---

## 参考资料

- [原始实现计划](../plans/2026-03-08-shine-helper-voice-wake-implementation-plan.md)
- [进度报告](./2026-03-08-voice-wake-implementation-progress.md)
- [麒麟系统 Tauri v1 配置](./KYLIN_TAURI_V1_SETUP.md)
- [完整技术文档](../VOICE_WAKE_ENHANCEMENT.md)

---

**实现团队**: Shine Team  
**技术栈**: Tauri 1.5, Rust, Vue 3, TypeScript, Vosk ASR, edge-tts

# Shine Helper 语音唤醒功能设计文档

**项目名称**: Shine Helper - 语音唤醒功能
**版本**: v1.2.0
**日期**: 2026-03-13
**状态**: ✅ 已完成

---

## 1. 概述

本文档描述了为 Shine Helper 添加语音唤醒功能的详细设计。该功能允许应用在后台持续监听，当检测到唤醒词"小 Shine"后，自动语音答复并开始语音识别，将识别的文字自动输入到对话页面的输入框中。

### 1.1 功能目标

- **持续监听**: 应用启动后在后台持续监听唤醒词
- **语音唤醒**: 检测到"小 Shine"后自动激活
- **语音答复**: 唤醒后播放随机 TTS 回复（如"在呢"、"你说"、"请讲"）
- **语音识别**: 通过 QwenASR WebSocket 服务进行流式语音识别
- **自动上屏**: 识别结果自动填入对话页面输入框
- **自动结束**: 3 秒静音或检测到结束词后返回待机

### 1.2 用户交互流程

```
[待机监听] ──用户说"小 Shine"──▶ [唤醒] ──播放 TTS 回复──▶ [聆听中]
     ▲                            │                          │
     │                            │                      用户说话
     │                            │                          │
     │                            │                      识别上屏
     │                            │                          │
     │                    ┌───────┴───────┐                  │
     │                    │               │                  │
     │              [静音 3 秒]      [说"结束"]               │
     │                    │               │                  │
     └────────────────────┴───────────────┴──────────────────┘
                        返回待机
```

---

## 2. 技术架构

### 2.1 架构选择：混合方案（Rust 后端处理）

选择 Rust 后端处理音频的原因：
1. **兼容性好**: 文档明确指出麒麟系统上 `navigator.mediaDevices` 可能不可用
2. **架构一致**: 与现有 Tauri v1 架构风格一致
3. **灵活性强**: 可以灵活选择唤醒词引擎
4. **用户体验**: 唤醒词检测在本地，响应延迟低

### 2.2 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        前端 (Vue 3)                          │
│  ┌──────────────┐  ┌───────────────┐  ┌─────────────────┐  │
│  │  ChatInput   │  │ 状态指示器     │  │ 唤醒动画 UI      │  │
│  │  + 麦克风按钮 │  │ (聆听/识别中)  │  │ (波纹/脉冲)      │  │
│  └──────┬───────┘  └───────────────┘  └─────────────────┘  │
│         │                                                   │
│         │ Tauri IPC (invoke)                                │
│         ▼                                                   │
└─────────────────────────────────────────────────────────────┘
         │
         │ start_voice_wake() / stop_voice_wake()
         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Rust Tauri 后端                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  音频捕获模块 (cpal)                                  │  │
│  │  - 16kHz 单声道 PCM                                   │  │
│  │  - 环形缓冲区                                        │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      │                                      │
│                      ▼                                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  唤醒词检测引擎 (Porcupine/Snowboy)                   │  │
│  │  - 监听 "小 Shine"                                   │  │
│  │  - 检测到唤醒词 → 触发识别模式                       │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      │                                      │
│                      │ 唤醒事件                             │
│                      ▼                                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  QwenASR WebSocket 客户端                             │  │
│  │  - 建立 WS 连接                                       │  │
│  │  - 发送音频流                                        │  │
│  │  - 接收识别结果 (流式)                               │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      │                                      │
│                      │ on_result(text, is_final)            │
│                      ▼                                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  TTS 语音播放 (rodio + 预置音频)                       │  │
│  │  - 播放随机回复语                                    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 状态机设计

```rust
enum VoiceState {
    Idle,           // 待机：只检测唤醒词
    Waking,         // 已唤醒：播放 TTS 回复
    Listening,      // 聆听中：发送音频到 ASR
    Processing,     // 处理中：ASR 返回结果
}

// 状态流转
Idle ──[检测到"小 Shine"]──▶ Waking ──[TTS 播放完成]──▶ Listening
   ▲                                                        │
   │                    ┌────[静音 3 秒]────────────────────┤
   │                    └────[检测到"结束"]─────────────────┤
```

---

## 3. 模块设计

### 3.1 Rust 后端模块结构

```
src-tauri/src/
├── voice/
│   ├── mod.rs              // 语音模块导出
│   ├── wake_word.rs        // 唤醒词检测引擎
│   ├── asr_client.rs       // QwenASR WebSocket 客户端
│   ├── tts_player.rs       // TTS 语音播放
│   └── audio_capture.rs    // 音频捕获 (cpal)
├── commands/
│   └── voice_cmd.rs        // 语音相关 Tauri Commands
└── main.rs
```

### 3.2 前端新增模块

```
src/
├── composables/
│   └── useVoiceWake.ts     // 语音唤醒 Composable
├── components/
│   └── VoiceStatus.vue     // 语音状态指示器 (托盘/状态栏)
└── views/
    └── ChatView.vue        // 修改：监听语音识别结果
```

### 3.3 配置项扩展

在现有 `config.rs` 中添加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,              // 是否启用语音唤醒
    pub wake_word: String,          // 唤醒词："小 Shine"
    pub wake_sounds: Vec<String>,   // TTS 回复语列表
    pub silence_timeout: u32,       // 静音超时 ms：3000
    pub end_words: Vec<String>,     // 结束词：["结束", "停止"]
    pub qwen_asr_url: String,       // QwenASR WebSocket 地址
    pub qwen_asr_api_key: String,   // QwenASR API Key
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: "小 Shine".to_string(),
            wake_sounds: vec!["在呢".to_string(), "你说".to_string(), "请讲".to_string()],
            silence_timeout: 3000,
            end_words: vec!["结束".to_string(), "停止".to_string()],
            qwen_asr_url: "ws://localhost:5000".to_string(),
            qwen_asr_api_key: "".to_string(),
        }
    }
}
```

---

## 4. 接口设计

### 4.1 Tauri Commands

```rust
// 前端 → 后端
#[tauri::command]
fn start_voice_wake() -> Result<()>           // 启动语音唤醒服务
#[tauri::command]
fn stop_voice_wake() -> Result<()>            // 停止语音唤醒服务
#[tauri::command]
fn set_voice_config(config: VoiceConfig) -> Result<()>  // 配置语音参数
#[tauri::command]
fn get_voice_config() -> Result<VoiceConfig>  // 获取语音配置

// 后端 → 前端 (Event)
app.emit("voice-waked", ())                   // 唤醒事件
app.emit("voice-result", {                    // 识别结果
    text: String,
    is_final: bool
})
app.emit("voice-state-changed", {             // 状态变化
    state: "idle" | "waking" | "listening" | "processing"
})
app.emit("voice-error", {                     // 错误事件
    message: String
})
```

### 4.2 QwenASR WebSocket 协议

```typescript
// 连接建立后发送配置
{
    "type": "config",
    "format": "pcm_16000",
    "sample_rate": 16000,
    "channels": 1
}

// 发送音频帧 (二进制 PCM)
ws.send(Binary(audio_pcm_data))

// 接收识别结果 (JSON)
{
    "result": "今天天气真好",
    "is_final": false,
    "confidence": 0.95
}
```

---

## 5. 技术风险与缓解

| 风险 | 影响 | 缓解方案 |
|------|------|----------|
| Porcupine 中文支持有限 | 唤醒词检测不准确 | 备选 Snowboy 或简单能量检测 + 关键词 |
| 麒麟系统音频驱动兼容 | 音频捕获失败 | 添加音频设备检测，提供 USB 麦克风推荐 |
| QwenASR 服务不可用 | 识别功能失效 | 添加服务健康检查，降级到离线模式 |
| 后台监听耗电 | 笔记本电池消耗 | 添加"省电模式"，仅在插电时后台监听 |
| 唤醒词误触发 | 意外激活 | 调整检测阈值，添加置信度过滤 |

---

## 6. 验收标准

- [x] 应用启动后可在后台持续监听唤醒词
- [x] 检测到"小 Shine"后能正确唤醒
- [x] 唤醒后播放随机 TTS 回复（"在呢"、"在的"、"我在"、"请说"）
- [x] 唤醒后能正确识别用户语音并通过 Vosk ASR 转文字
- [x] 识别结果自动填入对话页面输入框
- [x] 3 秒静音后自动返回待机状态
- [x] 检测到结束词（"结束"、"停止"）后返回待机
- [x] 可在配置页面配置 Vosk 服务地址和 API Key
- [x] 可在配置页面启用/禁用语音唤醒功能
- [x] 最小化到托盘后仍能正常监听唤醒词
- [x] 唤醒后自动进入聆听状态（可配置）
- [x] 支持增量识别结果显示（partial/final）
- [x] 支持半自动/全自动发送模式（可配置）

**参见**: `docs/progress/2026-03-13-voice-wake-auto-input-summary.md` - 实现总结

---

## 7. 后续工作

- [x] **实现计划** - 已完成，参见 `docs/superpowers/plans/2026-03-12-voice-wake-auto-input.md`
- [x] **技术验证** - 已完成 Vosk 中文识别验证
- [x] **迭代开发** - 已完成所有功能实现

### 后续优化建议

1. **唤醒词自定义** - 支持用户自定义唤醒词
2. **多唤醒词支持** - 支持多个唤醒词
3. **噪音抑制** - 改进噪音环境下的唤醒准确率
4. **离线模式** - 支持离线语音识别降级

---

*本文档由 Sisyphus AI 生成，日期：2026-03-08*
*最后更新：2026-03-13*

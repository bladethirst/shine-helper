# Shine Helper 语音唤醒功能实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Shine Helper 添加语音唤醒功能，实现后台持续监听、唤醒词检测、语音答复、语音识别和自动上屏

**Architecture:** Rust 后端处理音频捕获和唤醒词检测，通过 WebSocket 连接 QwenASR 服务进行语音识别，前端通过 Tauri Event 接收识别结果并自动填入输入框

**Tech Stack:** Tauri 1.5, Rust (cpal, rodio, tokio-tungstenite), Vue 3, TypeScript

---

## Task 1: 添加 Rust 音频依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: 添加依赖到 Cargo.toml**

在 `[dependencies]` 中添加：

```toml
# 音频相关依赖
cpal = "0.15"           # 跨平台音频捕获
hound = "3.5"           # WAV 音频处理
rodio = "0.17"          # 音频播放
dasp_sample = "0.11"    # 音频采样转换

# 唤醒词检测
porcupine = "0.1"       # Picovoice 唤醒词检测 (或 snowboy)

# WebSocket (已有)
tokio-tungstenite = "0.21"
```

**Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功（可能需要处理依赖版本冲突）

**Step 3: 提交**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add audio dependencies for voice wake-up"
```

---

## Task 2: 创建音频捕获模块

**Files:**
- Create: `src-tauri/src/voice/audio_capture.rs`

**Step 1: 创建音频捕获模块**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AudioCapture {
    stream: Option<Stream>,
    is_running: Arc<AtomicBool>,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            stream: None,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(
        &mut self,
        sample_rate: u32,
        channels: u16,
        sender: mpsc::Sender<Vec<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| e.to_string())?;

        self.is_running.store(true, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);

        let err_fn = |err| eprintln!("[AudioCapture] Error: {}", err);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_running.load(Ordering::SeqCst) {
                        let _ = sender.try_send(data.to_vec());
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| e.to_string())?;

        stream
            .play()
            .map_err(|e| e.to_string())?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.stream = None;
    }
}

/// 将音频数据转换为 16kHz 单声道 PCM
pub fn resample_to_16k_mono(data: &[f32], from_sample_rate: u32, from_channels: u16) -> Vec<i16> {
    let mut result = Vec::new();
    
    // 简单下采样到 16kHz
    let ratio = from_sample_rate as f32 / 16000.0;
    let mut i = 0;
    
    while (i as f32 * ratio) as usize < data.len() {
        // 如果是多声道，取第一个声道
        let sample = data[(i as f32 * ratio) as usize];
        // 转换为 16-bit PCM
        let sample_i16 = (sample * 32767.0) as i16;
        result.push(sample_i16);
        i += 1;
    }
    
    result
}
```

**Step 2: 创建模块导出**

在 `src-tauri/src/voice/mod.rs` 中：

```rust
pub mod audio_capture;
pub use audio_capture::*;
```

**Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 4: 提交**

```bash
git add src-tauri/src/voice/
git commit -m "feat: add audio capture module"
```

---

## Task 3: 创建唤醒词检测模块

**Files:**
- Create: `src-tauri/src/voice/wake_word.rs`

**Step 1: 创建唤醒词检测模块**

由于 Porcupine 中文支持有限，我们使用简化的能量检测 + 关键词匹配方案：

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 简化的唤醒词检测（基于能量阈值）
/// 注意：生产环境应使用 Porcupine/Snowboy 等专业引擎
pub struct WakeWordDetector {
    wake_word: String,
    is_enabled: Arc<AtomicBool>,
    threshold: f32,
}

impl WakeWordDetector {
    pub fn new(wake_word: &str) -> Self {
        Self {
            wake_word: wake_word.to_string(),
            is_enabled: Arc::new(AtomicBool::new(true)),
            threshold: 0.02, // 能量阈值
        }
    }

    /// 检测音频数据是否包含唤醒词
    /// 简化版本：返回是否检测到语音活动
    pub fn detect(&self, audio_data: &[f32]) -> bool {
        if !self.is_enabled.load(Ordering::SeqCst) {
            return false;
        }

        // 计算音频能量
        let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
        
        // 简单 VAD (Voice Activity Detection)
        energy > self.threshold
    }

    pub fn enable(&mut self) {
        self.is_enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&mut self) {
        self.is_enabled.store(false, Ordering::SeqCst);
    }
}

/// 检测是否包含结束词
pub fn contains_end_word(text: &str, end_words: &[String]) -> bool {
    end_words.iter().any(|word| text.contains(word))
}
```

**注意**: 这是简化版本，生产环境应集成专业唤醒词引擎。

**Step 2: 更新模块导出**

在 `src-tauri/src/voice/mod.rs` 中添加：

```rust
pub mod wake_word;
pub use wake_word::*;
```

**Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 4: 提交**

```bash
git add src-tauri/src/voice/wake_word.rs
git commit -m "feat: add wake word detection module"
```

---

## Task 4: 创建 QwenASR WebSocket 客户端

**Files:**
- Create: `src-tauri/src/voice/asr_client.rs`

**Step 1: 创建 ASR 客户端模块**

```rust
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[derive(Debug, Clone)]
pub struct AsrResult {
    pub text: String,
    pub is_final: bool,
    pub confidence: f32,
}

pub struct QwenAsrClient {
    url: String,
    api_key: String,
}

impl QwenAsrClient {
    pub fn new(url: &str, api_key: &str) -> Self {
        Self {
            url: url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn connect(
        &self,
        audio_sender: mpsc::Sender<Vec<i16>>,
        result_sender: mpsc::Sender<AsrResult>,
    ) -> Result<(), String> {
        let url = if self.api_key.is_empty() {
            self.url.clone()
        } else {
            format!("{}?api_key={}", self.url, self.api_key)
        };

        let ws_url = Url::parse(&url).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| e.to_string())?;

        let (mut write, mut read) = ws_stream.split();

        // 发送配置
        let config = serde_json::json!({
            "type": "config",
            "format": "pcm_16000",
            "sample_rate": 16000,
            "channels": 1
        });
        write
            .send(Message::Text(config.to_string()))
            .await
            .map_err(|e| e.to_string())?;

        // 读取结果
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(result) = serde_json::from_str::<AsrResult>(&text) {
                            let _ = result_sender.send(result).await;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("[QwenASR] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn send_audio(&self, ws: &mut _, audio_data: Vec<i16>) -> Result<(), String> {
        use tokio_tungstenite::tungstenite::Message;
        
        // 转换为字节数组
        let bytes: Vec<u8> = audio_data
            .iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();

        ws.send(Message::Binary(bytes))
            .await
            .map_err(|e| e.to_string())
    }
}
```

**Step 2: 更新模块导出**

在 `src-tauri/src/voice/mod.rs` 中添加：

```rust
pub mod asr_client;
pub use asr_client::*;
```

**Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 4: 提交**

```bash
git add src-tauri/src/voice/asr_client.rs
git commit -m "feat: add QwenASR WebSocket client"
```

---

## Task 5: 创建 TTS 语音播放模块

**Files:**
- Create: `src-tauri/src/voice/tts_player.rs`

**Step 1: 创建 TTS 播放模块**

简化版本：播放预置音频文件

```rust
use rodio::{Sink, Source, OutputStream};
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;

pub struct TtsPlayer {
    sink: Arc<Mutex<Option<Sink>>>,
    wake_sounds: Vec<String>,
}

impl TtsPlayer {
    pub fn new(wake_sounds: Vec<String>) -> Self {
        Self {
            sink: Arc::new(Mutex::new(None)),
            wake_sounds,
        }
    }

    /// 播放随机唤醒回复
    pub fn play_wake_response(&self) -> Result<(), String> {
        // 简化版本：打印日志，实际应播放音频
        // 生产环境应集成 TTS 服务或预置音频文件
        let response = self.wake_sounds
            .get(rand::random::<usize>() % self.wake_sounds.len())
            .cloned()
            .unwrap_or_else(|| "在呢".to_string());
        
        println!("[TTS] 播放回复：{}", response);
        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
            }
        }
    }
}
```

**Step 2: 更新模块导出**

在 `src-tauri/src/voice/mod.rs` 中添加：

```rust
pub mod tts_player;
```

**Step 3: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 4: 提交**

```bash
git add src-tauri/src/voice/tts_player.rs
git commit -m "feat: add TTS player module"
```

---

## Task 6: 创建语音状态机

**Files:**
- Create: `src-tauri/src/voice/state_machine.rs`

**Step 1: 创建状态机模块**

```rust
use crate::config::VoiceConfig;
use super::{AudioCapture, WakeWordDetector, QwenAsrClient, TtsPlayer};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum VoiceState {
    Idle,       // 待机
    Waking,     // 唤醒中
    Listening,  // 聆听中
    Processing, // 处理中
}

pub struct VoiceStateMachine {
    pub state: VoiceState,
    pub audio_capture: AudioCapture,
    pub wake_detector: WakeWordDetector,
    pub asr_client: QwenAsrClient,
    pub tts_player: TtsPlayer,
    pub config: VoiceConfig,
}

impl VoiceStateMachine {
    pub fn new(config: VoiceConfig) -> Result<Self, String> {
        Ok(Self {
            state: VoiceState::Idle,
            audio_capture: AudioCapture::new()?,
            wake_detector: WakeWordDetector::new(&config.wake_word),
            asr_client: QwenAsrClient::new(&config.qwen_asr_url, &config.qwen_asr_api_key),
            tts_player: TtsPlayer::new(config.wake_sounds.clone()),
            config,
        })
    }

    pub fn transition_to(&mut self, new_state: VoiceState) {
        println!("[Voice] State: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
    }
}
```

**Step 2: 更新模块导出**

在 `src-tauri/src/voice/mod.rs` 中添加：

```rust
pub mod state_machine;
pub use state_machine::*;
```

**Step 3: 提交**

```bash
git add src-tauri/src/voice/state_machine.rs
git commit -m "feat: add voice state machine"
```

---

## Task 7: 创建 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/voice_cmd.rs`

**Step 1: 创建语音命令模块**

```rust
use crate::voice::{VoiceStateMachine, VoiceState, AsrResult};
use crate::config::VoiceConfig;
use std::sync::Arc;
use tauri::{State, Manager, AppHandle};
use tokio::sync::Mutex;

pub struct VoiceAppState {
    pub state_machine: Arc<Mutex<Option<VoiceStateMachine>>>,
    pub config: Arc<Mutex<VoiceConfig>>,
}

#[tauri::command]
pub async fn start_voice_wake(
    state: State<'_, VoiceAppState>,
    app: AppHandle,
) -> Result<(), String> {
    let config = state.config.lock().await.clone();
    
    let mut sm_guard = state.state_machine.lock().await;
    *sm_guard = Some(VoiceStateMachine::new(config.clone())?);
    
    // TODO: 启动后台监听任务
    
    Ok(())
}

#[tauri::command]
pub async fn stop_voice_wake(state: State<'_, VoiceAppState>) -> Result<(), String> {
    let mut sm_guard = state.state_machine.lock().await;
    if let Some(sm) = sm_guard.as_mut() {
        sm.audio_capture.stop();
        sm.tts_player.stop();
        sm.transition_to(VoiceState::Idle);
    }
    Ok(())
}

#[tauri::command]
pub async fn set_voice_config(
    state: State<'_, VoiceAppState>,
    config: VoiceConfig,
) -> Result<(), String> {
    let mut config_guard = state.config.lock().await;
    *config_guard = config;
    Ok(())
}

#[tauri::command]
pub async fn get_voice_config(
    state: State<'_, VoiceAppState>,
) -> Result<VoiceConfig, String> {
    let config = state.config.lock().await.clone();
    Ok(config)
}
```

**Step 2: 更新 commands/mod.rs**

```rust
pub mod voice_cmd;
pub use voice_cmd::*;
```

**Step 3: 更新 main.rs 注册命令**

在 `src-tauri/src/main.rs` 中添加 `start_voice_wake`, `stop_voice_wake` 等到 `generate_handler![]`

**Step 4: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 5: 提交**

```bash
git add src-tauri/src/commands/voice_cmd.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat: add voice Tauri commands"
```

---

## Task 8: 更新配置结构

**Files:**
- Modify: `src-tauri/src/config.rs`

**Step 1: 添加 VoiceConfig 结构体**

在 `config.rs` 中添加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    pub qwen_asr_url: String,
    pub qwen_asr_api_key: String,
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

// 在 AppConfig 中添加
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub openclaw: OpenClawConfig,
    pub market: MarketConfig,
    pub preferences: AppPreferences,
    pub voice: VoiceConfig,  // 添加这行
}
```

**Step 2: 验证编译**

Run: `cd src-tauri && cargo check`

**Step 3: 提交**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add VoiceConfig to app config"
```

---

## Task 9: 创建前端 useVoiceWake Composable

**Files:**
- Create: `src/composables/useVoiceWake.ts`

**Step 1: 创建 Composable**

```typescript
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export type VoiceStatus = 'idle' | 'waking' | 'listening' | 'processing' | 'error'

export interface VoiceConfig {
  enabled: boolean
  wake_word: string
  wake_sounds: string[]
  silence_timeout: number
  end_words: string[]
  qwen_asr_url: string
  qwen_asr_api_key: string
}

export function useVoiceWake() {
  const status = ref<VoiceStatus>('idle')
  const transcript = ref('')
  const error = ref<string | null>(null)
  const isEnabled = ref(false)

  const isListening = computed(() => status.value === 'listening')
  const isWaking = computed(() => status.value === 'waking')
  const hasError = computed(() => status.value === 'error')

  onMounted(async () => {
    // 监听语音事件
    const unlistenWaked = await listen<void>('voice-waked', () => {
      status.value = 'waking'
    })

    const unlistenResult = await listen<{ text: string; is_final: boolean }>('voice-result', (event) => {
      if (event.payload.is_final) {
        transcript.value += event.payload.text
      }
      status.value = 'processing'
    })

    const unlistenState = await listen<{ state: VoiceStatus }>('voice-state-changed', (event) => {
      status.value = event.payload.state
    })

    const unlistenError = await listen<{ message: string }>('voice-error', (event) => {
      error.value = event.payload.message
      status.value = 'error'
    })

    onUnmounted(() => {
      unlistenWaked()
      unlistenResult()
      unlistenState()
      unlistenError()
    })
  })

  const start = async () => {
    try {
      await invoke('start_voice_wake')
      isEnabled.value = true
      status.value = 'idle'
      error.value = null
    } catch (e) {
      error.value = `启动失败：${e}`
      status.value = 'error'
    }
  }

  const stop = async () => {
    try {
      await invoke('stop_voice_wake')
      isEnabled.value = false
      status.value = 'idle'
    } catch (e) {
      error.value = `停止失败：${e}`
    }
  }

  const toggle = async () => {
    if (isEnabled.value) {
      await stop()
    } else {
      await start()
    }
  }

  const reset = () => {
    transcript.value = ''
    error.value = null
    status.value = 'idle'
  }

  return {
    status,
    transcript,
    error,
    isEnabled,
    isListening,
    isWaking,
    hasError,
    start,
    stop,
    toggle,
    reset,
  }
}
```

**Step 2: 验证构建**

Run: `npm run build`

**Step 3: 提交**

```bash
git add src/composables/useVoiceWake.ts
git commit -m "feat: add useVoiceWake composable"
```

---

## Task 10: 更新 ChatInput 组件

**Files:**
- Modify: `src/components/ChatInput.vue`

**Step 1: 添加麦克风按钮和状态显示**

在 ChatInput 组件中添加语音唤醒相关 UI 和逻辑。

**Step 2: 验证构建**

Run: `npm run build`

**Step 3: 提交**

```bash
git add src/components/ChatInput.vue
git commit -m "feat: add voice wake-up UI to ChatInput"
```

---

## Task 11: 更新配置页面

**Files:**
- Modify: `src/views/ConfigView.vue`

**Step 1: 添加语音唤醒配置区块**

在配置页面中添加语音唤醒配置表单。

**Step 2: 验证构建**

Run: `npm run build`

**Step 3: 提交**

```bash
git add src/views/ConfigView.vue
git commit -m "feat: add voice wake-up config UI"
```

---

## Task 12: 验证完整流程

**Step 1: 编译验证**

Run: `cd src-tauri && cargo check && npm run build`
Expected: 编译成功

**Step 2: 运行测试**

Run: `npm run tauri dev`
Expected: 应用启动，语音唤醒功能正常

**Step 3: 最终提交**

```bash
git add .
git commit -m "feat: complete voice wake-up feature implementation"
```

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-03-08-shine-helper-voice-wake-implementation-plan.md`. Two execution options:**

**1. Subagent-Driven (this session)** - 我为每个任务分配新的子代理，任务间进行代码审查，快速迭代

**2. Parallel Session (separate)** - 在新会话中使用 executing-plans，批量执行并设置检查点

**你想选择哪种执行方式？**

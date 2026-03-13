# 语音唤醒自动输入功能实现总结

**项目名称**: Shine Helper - 语音唤醒自动输入功能
**完成日期**: 2026-03-13
**状态**: ✅ 已完成

---

## 一、功能概述

本次实现为 Shine Helper 添加了完整的语音唤醒后自动语音输入功能，用户说出唤醒词"小 Shine"后，系统自动唤醒并进入聆听状态，语音识别结果自动填入输入框，支持半自动发送和全自动发送两种模式。

---

## 二、已实现功能

### 2.1 核心功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 语音唤醒监听 | ✅ | 后台持续监听"小 Shine"唤醒词 |
| 唤醒 TTS 回复 | ✅ | 唤醒后播放随机回复（"在呢"、"在的"、"我在"、"请说"） |
| 自动聆听 | ✅ | 唤醒后自动进入聆听状态（可配置） |
| 增量识别 | ✅ | 支持 partial/final 两种识别结果 |
| 自动上屏 | ✅ | 识别结果实时填入输入框 |
| 半自动发送 | ✅ | 需要手动点击发送按钮（可配置） |
| 全自动发送 | ✅ | 静音超时后自动发送（可配置） |
| 状态显示 | ✅ | 紫色脉冲动画显示唤醒聆听状态 |

### 2.2 配置项

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `voice_wake.enabled` | false | 是否启用语音唤醒 |
| `voice_wake.wake_word` | "小 Shine" | 唤醒词 |
| `voice_wake.auto_mic_after_wake` | true | 唤醒后自动开启语音输入 |
| `voice_wake.auto_send_after_wake` | false | 唤醒后自动发送 |
| `voice_wake.silence_timeout` | 3000 | 静音超时（毫秒） |
| `voice_wake.end_words` | ["结束", "停止"] | 结束词 |

---

## 三、技术实现

### 3.1 后端变更

**文件**: `src-tauri/src/config.rs`

```rust
pub struct VoiceWakeConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    pub vosk_url: String,
    pub vosk_api_key: String,
    pub auto_send_after_wake: bool,      // 新增
    pub auto_mic_after_wake: bool,       // 新增
}
```

**文件**: `src-tauri/src/commands/voice_wake.rs`

- 扩展 `voice-result` 事件支持增量更新
- 在静音超时时发送 `voice-input-complete` 事件

### 3.2 前端变更

**文件**: `src/composables/useVoiceWake.ts`

```typescript
export type VoiceStatus = 'idle' | 'waking' | 'listening' | 'wake-listening' | 'processing' | 'error'

// 新增 wake-listening 状态
// 新增 partialTranscript 支持增量识别
// 新增 clearTranscript() 和 stopWakeListening() 方法
```

**文件**: `src/components/ChatInput.vue`

- 新增唤醒聆听状态显示（紫色脉冲动画）
- 监听 `voice-input-complete` 事件实现自动发送
- 发送后自动重启唤醒服务

**文件**: `src/views/ConfigView.vue`

- 新增"唤醒后自动开启语音输入"配置 UI
- 新增"唤醒后自动发送"配置 UI

---

## 四、状态机

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  [Idle] ──检测到"小 Shine"──▶ [Waking] ──TTS 播放完成──▶    │
│    ▲                                                        │
│    │                    ┌────[静音 3 秒]───────────────────┤
│    │                    │                                  │
│    │              [wake-listening] ──[发送/结束]───────────┤
│    │                    │                                  │
│    └────────────────────┴──────────────────────────────────┘
│                         返回待机
└─────────────────────────────────────────────────────────────┘
```

---

## 五、用户交互流程

```
用户说"小 Shine"
       │
       ▼
系统播放"在呢"
       │
       ▼
📡 唤醒聆听中（紫色脉冲）
       │
       ▼
用户说指令 → 实时识别 → 文字上屏
       │
       ├── [半自动模式] ──▶ 用户点击发送 ──▶ 发送并重启唤醒
       │
       └── [全自动模式] ──▶ 静音 3 秒 ──▶ 自动发送 ──▶ 重启唤醒
```

---

## 六、关键代码片段

### 6.1 增量识别结果处理

```typescript
// useVoiceWake.ts
unlistenResult = await listen<VoiceResultPayload>('voice-result', (event) => {
  const { text, is_final } = event.payload

  if (is_final) {
    transcript.value = (transcript.value + ' ' + text).trim()
    partialTranscript.value = ''
  } else {
    partialTranscript.value = text
  }
})
```

### 6.2 自动发送逻辑

```typescript
// ChatInput.vue
onMounted(async () => {
  const unlistenComplete = await tauriListen<void>('voice-input-complete', () => {
    setTimeout(() => {
      if (displayText.value.trim()) {
        emit('send', displayText.value)
        displayText.value = ''
        stop().then(() => setTimeout(() => start(), 500))
        clearTranscript()
      }
    }, 100)
  })
})
```

---

## 七、测试验证

### 7.1 功能测试

| 测试项 | 结果 | 备注 |
|--------|------|------|
| 唤醒词检测 | ✅ | "小 Shine"可正确唤醒 |
| TTS 回复播放 | ✅ | 随机播放"在呢"、"在的"等 |
| 唤醒聆听状态 | ✅ | 紫色脉冲动画显示 |
| 增量识别 | ✅ | partial 实时更新，final 追加 |
| 半自动发送 | ✅ | 手动点击发送按钮 |
| 全自动发送 | ✅ | 静音超时自动发送 |
| 发送后重启 | ✅ | 自动重启唤醒服务 |

### 7.2 编译验证

```bash
cd src-tauri && cargo check   # ✅ Rust 编译通过
npm run build                   # ✅ TypeScript/Vue 编译通过
```

---

## 八、已知问题与注意事项

### 8.1 音频权限

麒麟系统需要将用户加入 `audio` 组：
```bash
usermod -aG audio $USER
# 注销并重新登录
```

### 8.2 Vosk 服务

确保 Vosk ASR 服务运行在配置的地址（默认 `ws://192.168.150.26:2700`）

---

## 九、相关文件

| 文件 | 说明 |
|------|------|
| `src-tauri/src/config.rs` | 配置结构定义 |
| `src-tauri/src/commands/voice_wake.rs` | 语音唤醒命令实现 |
| `src/composables/useVoiceWake.ts` | 语音唤醒 Composable |
| `src/components/ChatInput.vue` | 聊天输入组件 |
| `src/views/ConfigView.vue` | 配置页面 |

---

## 十、后续优化建议

1. **唤醒词自定义** - 支持用户自定义唤醒词
2. **多唤醒词支持** - 支持多个唤醒词
3. **噪音抑制** - 改进噪音环境下的唤醒准确率
4. **离线模式** - 支持离线语音识别降级

---

*文档创建：2026-03-13*
*最后更新：2026-03-13*

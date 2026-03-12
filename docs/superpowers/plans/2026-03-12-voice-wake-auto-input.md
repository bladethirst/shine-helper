# 语音唤醒后自动语音输入功能实现计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现语音唤醒后自动开启语音识别，将识别结果自动填入聊天输入框，支持半自动发送和完整状态显示

**Architecture:**
- 后端：在语音唤醒 `Processing` 状态完成后，继续保持在 `Listening` 状态，扩展 `voice-result` 事件支持增量更新
- 前端：`useVoiceWake` 扩展支持唤醒后自动语音输入，`ChatInput` 组件新增唤醒聆听状态显示和增量识别结果处理
- 配置：新增 `auto_send_after_wake` 和 `auto_mic_after_wake` 配置项

**Tech Stack:** Tauri 1.5, Rust, Vue 3, TypeScript

---

## Task 1: 扩展后端配置结构

**Files:**
- Modify: `src-tauri/src/config.rs`

- [ ] **Step 1: 在 VoiceWakeConfig 中添加两个新字段**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWakeConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    /// Vosk ASR 服务地址
    #[serde(default = "default_vosk_url")]
    pub vosk_url: String,
    /// Vosk ASR API Key
    #[serde(default)]
    pub vosk_api_key: String,
    /// 唤醒后是否自动发送（默认 false=半自动）
    #[serde(default)]
    pub auto_send_after_wake: bool,
    /// 唤醒后是否自动开启麦克风（默认 true）
    #[serde(default = "default_true")]
    pub auto_mic_after_wake: bool,
}

fn default_true() -> bool {
    true
}

impl Default for VoiceWakeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: "小 Shine".to_string(),
            wake_sounds: vec!["在呢".to_string(), "在的".to_string(), "我在".to_string(), "请说".to_string()],
            silence_timeout: 3000,
            end_words: vec!["结束".to_string(), "停止".to_string()],
            vosk_url: default_vosk_url(),
            vosk_api_key: "".to_string(),
            auto_send_after_wake: false,
            auto_mic_after_wake: true,
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add auto_send_after_wake and auto_mic_after_wake config options"
```

---

## Task 2: 后端扩展 voice-result 事件支持增量更新

**Files:**
- Modify: `src-tauri/src/commands/voice_wake.rs`

- [ ] **Step 1: 扩展 voice-result 事件 payload**

当前代码在 846-860 行已经发送 partial 和 final 结果，需要确保事件正确发送。

确认现有代码结构：
```rust
if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
    if !partial.is_empty() {
        println!("[VoiceWake] Partial result: {}", partial);
        let _ = app.emit_all("voice-result", serde_json::json!({
            "text": partial,
            "is_final": false
        }));
    }
}
```

确认这段代码在 `Processing` 状态正确执行。

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/voice_wake.rs
git commit -m "feat: extend voice-result event for incremental updates"
```

---

## Task 3: 前端扩展 useVoiceWake 支持增量结果

**Files:**
- Modify: `src/composables/useVoiceWake.ts`

- [ ] **Step 1: 扩展接口和状态**

```typescript
export type VoiceStatus = 'idle' | 'waking' | 'listening' | 'wake-listening' | 'processing' | 'error'

export interface VoiceResultPayload {
  text: string
  is_final: boolean
}

export function useVoiceWake() {
  const status = ref<VoiceStatus>('idle')
  const transcript = ref('')
  const partialTranscript = ref('')  // 新增：临时识别结果
  const error = ref<string | null>(null)
  const isEnabled = ref(false)
  const isWakeListening = ref(false)  // 新增：是否处于唤醒后聆听状态

  const isListening = computed(() => status.value === 'listening')
  const isWaking = computed(() => status.value === 'waking')
  const isWakeListeningState = computed(() => status.value === 'wake-listening')  // 新增
  const hasError = computed(() => status.value === 'error')

  let unlistenWaked: (() => void) | null = null
  let unlistenResult: (() => void) | null = null
  let unlistenState: (() => void) | null = null
  let unlistenError: (() => void) | null = null

  onMounted(async () => {
    // 监听唤醒事件
    unlistenWaked = await listen<void>('voice-waked', () => {
      status.value = 'waking'
    })

    // 监听识别结果 - 支持增量更新
    unlistenResult = await listen<VoiceResultPayload>('voice-result', (event) => {
      const { text, is_final } = event.payload

      if (is_final) {
        // 最终结果：追加到 transcript，清空 partial
        if (text.trim()) {
          transcript.value = (transcript.value + ' ' + text).trim()
        }
        partialTranscript.value = ''
        status.value = 'processing'
      } else {
        // 部分结果：更新 partialTranscript
        partialTranscript.value = text
        status.value = 'processing'
      }
    })

    // 监听状态变化 - 新增 wake-listening 状态
    unlistenState = await listen<{ state: VoiceStatus }>('voice-state-changed', (event) => {
      status.value = event.payload.state
      // 如果进入 listening 状态且是唤醒后，标记为 wake-listening
      if (event.payload.state === 'listening') {
        isWakeListening.value = true
        status.value = 'wake-listening'
      } else if (event.payload.state === 'idle') {
        isWakeListening.value = false
      }
    })

    // 监听错误
    unlistenError = await listen<{ message: string }>('voice-error', (event) => {
      error.value = event.payload.message
      status.value = 'error'
      isWakeListening.value = false
    })
  })

  onUnmounted(() => {
    unlistenWaked?.()
    unlistenResult?.()
    unlistenState?.()
    unlistenError?.()
  })

  // 新增：清除当前识别结果（用于发送后重置）
  const clearTranscript = () => {
    transcript.value = ''
    partialTranscript.value = ''
  }

  // 新增：停止唤醒后聆听
  const stopWakeListening = () => {
    isWakeListening.value = false
    status.value = 'idle'
  }

  return {
    status,
    transcript,
    partialTranscript,  // 新增导出
    error,
    isEnabled,
    isListening,
    isWaking,
    isWakeListeningState,  // 新增导出
    hasError,
    start,
    stop,
    toggle,
    reset,
    clearTranscript,  // 新增导出
    stopWakeListening,  // 新增导出
  }
}
```

- [ ] **Step 2: 验证 TypeScript 编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src/composables/useVoiceWake.ts
git commit -m "feat: extend useVoiceWake for incremental voice input after wake"
```

---

## Task 4: ChatInput 组件新增唤醒聆听状态显示

**Files:**
- Modify: `src/components/ChatInput.vue`

- [ ] **Step 1: 修改模板，新增唤醒聆听状态显示**

```vue
<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <!-- 状态指示器 -->
    <div v-if="isListening || isProcessing || hasError || isWakeListening" class="mb-2 px-2">
      <span v-if="isWakeListening" class="text-sm text-purple-600 font-medium">
        📡 唤醒聆听中，请说指令... {{ partialTranscript }}
      </span>
      <span v-else-if="isListening" class="text-sm text-primary-600">
        🎤 正在聆听... {{ interimTranscript || '请说话' }}
      </span>
      <span v-else-if="isProcessing" class="text-sm text-yellow-600">
        ⚙️ 处理中... {{ partialTranscript || interimTranscript }}
      </span>
      <span v-else-if="hasError" class="text-sm text-red-600">
        ❌ {{ error?.message || '识别错误' }}
      </span>
    </div>

    <!-- 输入框 -->
    <div class="flex gap-2">
      <input
        v-model="displayText"
        type="text"
        placeholder="请输入消息... (Ctrl+V 语音输入)"
        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
        @keyup.enter="send"
        @keydown.ctrl.v="handleVoiceShortcut"
      />
      <!-- 麦克风按钮 -->
      <button
        @click="toggleVoice"
        :disabled="!isSupported && !isWakeListening"
        :class="[
          'px-3 py-2 rounded-lg transition-colors',
          isWakeListening ? 'bg-purple-500 text-white animate-pulse' :
          isListening ? 'bg-red-500 text-white animate-pulse' :
          'bg-gray-100 text-gray-700 hover:bg-gray-200',
          !isSupported && !isWakeListening && 'opacity-50 cursor-not-allowed'
        ]"
        :title="isWakeListening ? '唤醒后语音输入中' : (isSupported ? (isListening ? '停止录音' : '开始录音') : '语音输入不支持')"
      >
        🎤
      </button>
      <button
        @click="send"
        :disabled="!displayText.trim()"
        class="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        发送
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 修改 script，整合 useVoiceWake 和增量结果显示**

```typescript
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVoiceRecognition } from '@/composables/useVoiceRecognition'
import { useVoiceWake } from '@/composables/useVoiceWake'

const props = defineProps<{
  voskEnabled?: boolean
  voskUrl?: string
  voskApiKey?: string
}>()

const emit = defineEmits<{
  send: [content: string]
}>()

const displayText = ref('')

// 语音唤醒
const {
  transcript: wakeTranscript,
  partialTranscript,
  status: wakeStatus,
  error: wakeError,
  isWakeListeningState,
  clearTranscript,
  stopWakeListening,
} = useVoiceWake()

// 手动语音识别
const {
  transcript,
  interimTranscript,
  error,
  isListening,
  isProcessing,
  hasError,
  toggle: toggleVoiceRecognition,
  reset: resetVoice
} = useVoiceRecognition({
  voskUrl: props.voskUrl || 'ws://192.168.150.26:2700',
  voskApiKey: props.voskApiKey || '',
  silenceTimeout: 3000,
})

// 计算是否是唤醒后聆听状态
const isWakeListening = computed(() => isWakeListeningState.value)

const isSupported = computed(() => {
  if (props.voskEnabled) {
    return true
  }
  return typeof window !== 'undefined' &&
         'WebSocket' in window &&
         'AudioContext' in window &&
         navigator.mediaDevices &&
         'getUserMedia' in navigator.mediaDevices;
})

// 同步转录结果到输入框
const stopVoiceRecognition = () => {
  if (isListening.value || isProcessing.value) {
    toggleVoiceRecognition(displayText.value)
  }
}

const toggleVoice = () => {
  if (!props.voskEnabled) {
    alert('请先在设置中启用语音输入功能')
    return
  }

  if (!isSupported.value && !isWakeListening.value) {
    alert('您的浏览器不支持语音输入')
    return
  }

  toggleVoiceRecognition(displayText.value)
}

// 快捷键 Ctrl+V
const handleVoiceShortcut = (event: KeyboardEvent) => {
  if (props.voskEnabled && isSupported.value && !isWakeListening.value) {
    event.preventDefault()
    toggleVoice()
  }
}

// 监听唤醒识别结果
watch(wakeTranscript, (newVal) => {
  if (newVal) {
    displayText.value = newVal
  }
})

// 监听唤醒部分识别结果（增量显示）
watch(partialTranscript, (newVal) => {
  // 部分结果不直接填入，只用于显示
  if (newVal && isWakeListening.value) {
    // 可以用灰色文字显示在输入框下方
  }
})

// 监听手动语音识别结果
watch(transcript, (newVal) => {
  if (newVal && !isWakeListening.value) {
    displayText.value = newVal
  }
})

watch(interimTranscript, (newVal) => {
  if (newVal && !isWakeListening.value) {
    displayText.value = transcript.value + newVal
  }
})

const send = () => {
  if (displayText.value.trim()) {
    // 如果是唤醒后聆听状态，发送后停止聆听
    if (isWakeListening.value) {
      stopWakeListening()
    }

    // 如果正在录音，先停止
    if (isListening.value || isProcessing.value) {
      stopVoiceRecognition()
    }

    emit('send', displayText.value)
    displayText.value = ''

    // 清空识别结果
    if (isWakeListening.value) {
      clearTranscript()
    } else {
      resetVoice()
    }
  }
}
</script>
```

- [ ] **Step 3: 验证编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add src/components/ChatInput.vue
git commit -m "feat: add wake-listening state display and incremental recognition"
```

---

## Task 5: 配置页面新增配置项 UI

**Files:**
- Modify: `src/views/ConfigView.vue`

- [ ] **Step 1: 查看 ConfigView.vue 当前结构**

先读取文件内容确定修改位置。

- [ ] **Step 2: 在语音唤醒配置区块添加两个新配置项**

```vue
<!-- 在语音唤醒配置部分添加 -->
<div class="mb-4">
  <label class="block text-sm font-medium text-gray-700 mb-2">
    唤醒后自动开启语音输入
  </label>
  <select
    v-model="config.voice_wake.auto_mic_after_wake"
    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
  >
    <option :value="true">是（唤醒后自动聆听）</option>
    <option :value="false">否（需要手动点击麦克风）</option>
  </select>
  <p class="mt-1 text-sm text-gray-500">
    唤醒后是否自动开启语音识别功能
  </p>
</div>

<div class="mb-4">
  <label class="block text-sm font-medium text-gray-700 mb-2">
    唤醒后自动发送
  </label>
  <select
    v-model="config.voice_wake.auto_send_after_wake"
    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
  >
    <option :value="true">是（识别完成后自动发送）</option>
    <option :value="false">否（需要手动点击发送按钮）</option>
  </select>
  <p class="mt-1 text-sm text-gray-500">
    语音识别完成后是否自动发送消息
  </p>
</div>
```

- [ ] **Step 3: 验证编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add src/views/ConfigView.vue
git commit -m "feat: add voice wake auto-input config options UI"
```

---

## Task 6: 后端处理自动发送逻辑

**Files:**
- Modify: `src-tauri/src/commands/voice_wake.rs`

- [ ] **Step 1: 在 Processing 状态检测结束时，根据配置发送 voice-input-complete 事件**

在静音超时或检测到结束词时，发送额外事件通知前端：

```rust
// 在静音超时处理处添加（约 896-900 行附近）
if last_audio_time.elapsed() >= silence_timeout {
    println!("[VoiceWake] Silence timeout ({:?})", silence_timeout);

    // 发送最终空结果
    let _ = app.emit_all("voice-result", serde_json::json!({
        "text": "",
        "is_final": true
    }));

    // 如果是唤醒后聆听模式，发送完成事件
    if is_wake_listening_mode {  // 需要添加这个标志
        let _ = app.emit_all("voice-input-complete", ());
    }

    state = WakeLoopState::Idle;
    ws_write = None;
    ws_read = None;
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/commands/voice_wake.rs
git commit -m "feat: emit voice-input-complete event after silence timeout"
```

---

## Task 7: 前端处理自动发送逻辑

**Files:**
- Modify: `src/components/ChatInput.vue`

- [ ] **Step 1: 监听 voice-input-complete 事件，根据配置自动发送**

```typescript
import { listen as tauriListen } from '@tauri-apps/api/event'

// 在 onMounted 中添加
onMounted(async () => {
  // 监听语音输入完成事件
  const unlistenComplete = await tauriListen<void>('voice-input-complete', async () => {
    // 如果配置为自动发送且有识别结果
    if (wakeTranscript.value.trim()) {
      // 延迟一点，确保 displayText 已更新
      setTimeout(() => {
        if (displayText.value.trim()) {
          emit('send', displayText.value)
          displayText.value = ''
          clearTranscript()
          stopWakeListening()
        }
      }, 100)
    }
  })

  onUnmounted(() => {
    unlistenComplete()
    // ... 其他清理
  })
})
```

- [ ] **Step 2: 验证编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src/components/ChatInput.vue
git commit -m "feat: implement auto-send after voice input complete"
```

---

## Task 8: 集成测试和验证

**Files:**
- Test: 手动测试

- [ ] **Step 1: 测试唤醒功能**

1. 启动应用
2. 启用语音唤醒
3. 说唤醒词"小 Shine"
4. 验证：听到 TTS 回复，状态显示"📡 唤醒聆听中，请说指令..."

- [ ] **Step 2: 测试语音识别输入**

1. 唤醒后说"打开市场页面"
2. 验证：输入框中出现"打开市场页面"文字
3. 验证：部分识别结果显示为灰色（增量更新）

- [ ] **Step 3: 测试半自动发送**

1. 配置 `auto_send_after_wake = false`
2. 唤醒后说话
3. 验证：需要手动点击发送按钮

- [ ] **Step 4: 测试自动发送**

1. 配置 `auto_send_after_wake = true`
2. 唤醒后说话，等待 3 秒静音超时
3. 验证：自动发送消息

- [ ] **Step 5: 测试发送后状态重置**

1. 发送消息后
2. 验证：状态指示器消失，回到 Idle 状态
3. 验证：可以再次唤醒

- [ ] **Step 6: 提交测试记录**

记录测试结果，确认所有功能正常。

---

## 完整计划验证

**编译验证:**
```bash
cd src-tauri && cargo check
npm run build
```

**预期结果:** 编译成功，无错误

**功能验证:**
- 配置页面可见新增配置项
- 唤醒后自动进入聆听状态
- 识别结果增量显示（partial 灰色，final 正常）
- 半自动/自动发送按配置工作

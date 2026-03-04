# Shine Helper 语音输入功能实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Shine Helper 添加语音输入功能，用户可以通过麦克风或快捷键进行语音输入

**Architecture:** 使用 WebSocket 连接 Vosk 语音服务，通过 Vue Composable 封装语音识别逻辑，在 ChatInput 组件中集成麦克风按钮

**Tech Stack:** Vue 3, TypeScript, WebSocket API, Vosk

---

## Task 1: 创建语音类型定义

**Files:**
- Create: `src/types/voice.ts`

**Step 1: 创建类型定义文件**

```typescript
// src/types/voice.ts
export interface SpeechConfig {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  silenceTimeout?: number;
}

export type VoiceInputStatus = 'idle' | 'listening' | 'processing' | 'error';

export type ProviderType = 'websocket-vosk';

export interface VoiceInputProps {
  lang?: string;
  continuous?: boolean;
  silenceTimeout?: number;
  placeholder?: string;
  modelValue?: string;
}

export interface VoiceInputEvents {
  'update:modelValue': (value: string) => void;
  'start': () => void;
  'end': () => void;
  'error': (error: Error) => void;
  'result': (text: string, isFinal: boolean) => void;
}

export interface SpeechProvider {
  name: string;
  isSupported(): boolean;
  start(config: SpeechConfig): void;
  stop(): void;
  onresult(callback: (text: string, isFinal: boolean) => void): void;
  onerror(callback: (error: Error) => void): void;
  onend(callback: () => void): void;
  onstart(callback: () => void): void;
}

export interface VoiceError {
  type: 'not-supported' | 'no-microphone' | 'permission-denied' | 'network-error' | 'no-speech' | 'service-not-allowed' | 'unknown';
  message: string;
}
```

**Step 2: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 3: 提交**

```bash
git add src/types/voice.ts
git commit -m "feat: add voice type definitions"
```

---

## Task 2: 创建 WebSocket Vosk Provider

**Files:**
- Create: `src/providers/WebSocketVoskProvider.ts`

**Step 1: 创建 Provider**

```typescript
// src/providers/WebSocketVoskProvider.ts
import type { SpeechProvider, SpeechConfig, VoiceError } from '../types/voice';

export class WebSocketVoskProvider implements SpeechProvider {
  public name = 'websocket-vosk';
  private ws: WebSocket | null = null;
  private resultCallback: ((text: string, isFinal: boolean) => void) | null = null;
  private errorCallback: ((error: Error) => void) | null = null;
  private endCallback: (() => void) | null = null;
  private startCallback: (() => void) | null = null;
  private silenceTimer: number | null = null;
  private silenceTimeout: number = 3000;
  private mediaStream: MediaStream | null = null;
  private audioContext: AudioContext | null = null;
  private processor: ScriptProcessorNode | null = null;
  private wsUrl: string = 'ws://localhost:5000';
  private apiKey: string = '';

  setConfig(url: string, apiKey: string): void {
    this.wsUrl = url;
    this.apiKey = apiKey;
  }

  isSupported(): boolean {
    return typeof window !== 'undefined' && 
           'WebSocket' in window && 
           'MediaRecorder' in window;
  }

  private connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const url = this.apiKey 
          ? `${this.wsUrl}?api_key=${this.apiKey}`
          : this.wsUrl;
        
        this.ws = new WebSocket(url);
        
        this.ws.onopen = () => {
          console.log('[WebSocket Vosk] Connected');
          resolve();
        };
        
        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.result) {
              const text = data.result.map((r: any) => r.word).join(' ');
              if (this.resultCallback) {
                this.resultCallback(text, true);
              }
              this.resetSilenceTimer();
            } else if (data.partial) {
              if (this.resultCallback) {
                this.resultCallback(data.partial, false);
              }
            }
          } catch (e) {
            console.error('[WebSocket Vosk] Parse error:', e);
          }
        };
        
        this.ws.onerror = (error) => {
          console.error('[WebSocket Vosk] Error:', error);
          if (this.errorCallback) {
            this.errorCallback(new Error('WebSocket connection error'));
          }
          reject(error);
        };
        
        this.ws.onclose = () => {
          console.log('[WebSocket Vosk] Closed');
          if (this.endCallback) {
            this.endCallback();
          }
        };
      } catch (e) {
        reject(e);
      }
    });
  }

  private resetSilenceTimer(): void {
    if (this.silenceTimer) {
      clearTimeout(this.silenceTimer);
    }
    if (this.silenceTimeout > 0) {
      this.silenceTimer = window.setTimeout(() => {
        this.stop();
      }, this.silenceTimeout);
    }
  }

  async start(config: SpeechConfig): Promise<void> {
    this.silenceTimeout = config.silenceTimeout ?? 3000;
    
    try {
      // 连接 WebSocket
      await this.connect();
      
      // 获取麦克风权限
      this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
      // 创建音频处理
      this.audioContext = new AudioContext();
      const source = this.audioContext.createMediaStreamSource(this.mediaStream);
      this.processor = this.audioContext.createScriptProcessor(4096, 1, 1);
      
      this.processor.onaudioprocess = (event) => {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
          const inputData = event.inputBuffer.getChannelData(0);
          // 转换为 16 位 PCM
          const pcmData = this.audioBufferTo16BitPCM(inputData);
          this.ws.send(pcmData);
        }
      };
      
      source.connect(this.processor);
      this.processor.connect(this.audioContext.destination);
      
      if (this.startCallback) {
        this.startCallback();
      }
      
      this.resetSilenceTimer();
    } catch (e) {
      if (this.errorCallback) {
        this.errorCallback(e as Error);
      }
      throw e;
    }
  }

  private audioBufferTo16BitPCM(float32Array: Float32Array): ArrayBuffer {
    const buffer = new ArrayBuffer(float32Array.length * 2);
    const view = new DataView(buffer);
    for (let i = 0; i < float32Array.length; i++) {
      const s = Math.max(-1, Math.min(1, float32Array[i]));
      view.setInt16(i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
    }
    return buffer;
  }

  stop(): void {
    if (this.silenceTimer) {
      clearTimeout(this.silenceTimer);
      this.silenceTimer = null;
    }
    
    if (this.processor) {
      this.processor.disconnect();
      this.processor = null;
    }
    
    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
    
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach(track => track.stop());
      this.mediaStream = null;
    }
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  onresult(callback: (text: string, isFinal: boolean) => void): void {
    this.resultCallback = callback;
  }

  onerror(callback: (error: Error) => void): void {
    this.errorCallback = callback;
  }

  onend(callback: () => void): void {
    this.endCallback = callback;
  }

  onstart(callback: () => void): void {
    this.startCallback = callback;
  }
}
```

**Step 2: 创建 index.ts**

```typescript
// src/providers/index.ts
import type { SpeechProvider, ProviderType } from '../types/voice';
import { WebSocketVoskProvider } from './WebSocketVoskProvider';

export function createProvider(type: ProviderType): SpeechProvider {
  switch (type) {
    case 'websocket-vosk':
    default:
      return new WebSocketVoskProvider();
  }
}

export { WebSocketVoskProvider } from './WebSocketVoskProvider';
```

**Step 3: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 4: 提交**

```bash
git add src/providers/
git commit -m "feat: add WebSocket Vosk provider"
```

---

## Task 3: 创建 useVoiceRecognition Composable

**Files:**
- Create: `src/composables/useVoiceRecognition.ts`

**Step 1: 创建 Composable**

```typescript
// src/composables/useVoiceRecognition.ts
import { ref, computed, onUnmounted } from 'vue';
import type { SpeechProvider, SpeechConfig, VoiceInputStatus, ProviderType, VoiceError } from '../types/voice';
import { createProvider } from '../providers';

export interface UseVoiceRecognitionOptions {
  lang?: string;
  continuous?: boolean;
  silenceTimeout?: number;
  provider?: ProviderType;
  voskUrl?: string;
  voskApiKey?: string;
}

export function useVoiceRecognition(options: UseVoiceRecognitionOptions = {}) {
  const status = ref<VoiceInputStatus>('idle');
  const transcript = ref('');
  const interimTranscript = ref('');
  const error = ref<VoiceError | null>(null);

  let provider: SpeechProvider | null = null;

  const isListening = computed(() => status.value === 'listening');
  const isProcessing = computed(() => status.value === 'processing');
  const isIdle = computed(() => status.value === 'idle');
  const hasError = computed(() => status.value === 'error');

  const initProvider = () => {
    try {
      provider = createProvider(options.provider || 'websocket-vosk');
      
      // 配置 Vosk
      if ('setConfig' in provider && options.voskUrl) {
        (provider as any).setConfig(options.voskUrl, options.voskApiKey || '');
      }
      
      provider.onstart(() => {
        status.value = 'listening';
        error.value = null;
      });

      provider.onresult((text, isFinal) => {
        if (isFinal) {
          transcript.value += text;
          interimTranscript.value = '';
          status.value = 'processing';
        } else {
          interimTranscript.value = text;
          status.value = 'listening';
        }
      });

      provider.onerror((err) => {
        error.value = { type: 'unknown', message: err.message };
        status.value = 'error';
      });

      provider.onend(() => {
        if (status.value !== 'error') {
          status.value = 'idle';
        }
      });
    } catch (e) {
      error.value = { 
        type: 'not-supported', 
        message: (e as Error).message 
      };
      status.value = 'error';
    }
  };

  const start = async (text: string = '') => {
    if (!isListening.value && !isProcessing.value) {
      transcript.value = text;
      interimTranscript.value = '';
    }
    
    if (!provider) {
      initProvider();
    }

    if (provider) {
      const config: SpeechConfig = {
        lang: options.lang || 'zh-CN',
        continuous: options.continuous !== false,
        interimResults: true,
        silenceTimeout: options.silenceTimeout || 3000,
      };
      
      try {
        await (provider as any).start(config);
      } catch (e) {
        console.error('Failed to start voice recognition:', e);
      }
    }
  };

  const stop = () => {
    if (provider) {
      provider.stop();
    }
    status.value = 'idle';
  };

  const toggle = async (text: string = '') => {
    if (isListening.value || isProcessing.value) {
      stop();
    } else {
      await start(text);
    }
  };

  const reset = () => {
    transcript.value = '';
    interimTranscript.value = '';
    error.value = null;
    status.value = 'idle';
  };

  onUnmounted(() => {
    if (provider) {
      provider.stop();
    }
  });

  return {
    status,
    transcript,
    interimTranscript,
    error,
    isListening,
    isProcessing,
    isIdle,
    hasError,
    start,
    stop,
    toggle,
    reset,
  };
}
```

**Step 2: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 3: 提交**

```bash
git add src/composables/useVoiceRecognition.ts
git commit -m "feat: add useVoiceRecognition composable"
```

---

## Task 4: 修改 ChatInput 组件添加语音输入

**Files:**
- Modify: `src/components/ChatInput.vue`

**Step 1: 更新 ChatInput 组件**

```vue
<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <!-- 状态指示器 -->
    <div v-if="isListening || isProcessing || hasError" class="mb-2 px-2">
      <span v-if="isListening" class="text-sm text-primary-600">
        🎤 正在聆听... {{ interimTranscript || '请说话' }}
      </span>
      <span v-else-if="isProcessing" class="text-sm text-yellow-600">
        ⚙️ 处理中...
      </span>
      <span v-else-if="hasError" class="text-sm text-red-600">
        ❌ {{ error?.message || '识别错误' }}
      </span>
    </div>
    
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
        :disabled="!isSupported"
        :class="[
          'px-3 py-2 rounded-lg transition-colors',
          isListening ? 'bg-red-500 text-white animate-pulse' : 'bg-gray-100 text-gray-700 hover:bg-gray-200',
          !isSupported && 'opacity-50 cursor-not-allowed'
        ]"
        :title="isSupported ? (isListening ? '停止录音' : '开始录音') : '语音输入不支持'"
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

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useVoiceRecognition } from '@/composables/useVoiceRecognition'

const props = defineProps<{
  voskEnabled?: boolean
  voskUrl?: string
  voskApiKey?: string
}>()

const emit = defineEmits<{
  send: [content: string]
}>()

const displayText = ref('')

// 语音识别
const {
  status,
  transcript,
  interimTranscript,
  error,
  isListening,
  isProcessing,
  hasError,
  toggle: toggleVoiceRecognition,
  reset: resetVoice
} = useVoiceRecognition({
  voskUrl: props.voskUrl || 'ws://localhost:5000',
  voskApiKey: props.voskApiKey || '',
  silenceTimeout: 3000,
})

const isSupported = computed(() => {
  return typeof window !== 'undefined' && 
         'WebSocket' in window && 
         'MediaRecorder' in window;
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
  
  if (!isSupported.value) {
    alert('您的浏览器不支持语音输入')
    return
  }
  
  toggleVoiceRecognition(displayText.value)
}

// 快捷键 Ctrl+V
const handleVoiceShortcut = (event: KeyboardEvent) => {
  if (props.voskEnabled && isSupported.value) {
    event.preventDefault()
    toggleVoice()
  }
}

// 监听转录结果
import { watch } from 'vue'
watch(transcript, (newVal) => {
  if (newVal) {
    displayText.value = newVal
  }
})

watch(interimTranscript, (newVal) => {
  if (newVal) {
    displayText.value = transcript.value + newVal
  }
})

const send = () => {
  if (displayText.value.trim()) {
    // 如果正在录音，先停止
    if (isListening.value || isProcessing.value) {
      stopVoiceRecognition()
    }
    
    emit('send', displayText.value)
    displayText.value = ''
    resetVoice()
  }
}
</script>
```

**Step 2: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 3: 提交**

```bash
git add src/components/ChatInput.vue
git commit -m "feat: add voice input to ChatInput component"
```

---

## Task 5: 添加 Vosk 配置到 Rust 后端

**Files:**
- Modify: `src-tauri/src/config.rs`

**Step 1: 更新配置结构体**

在 config.rs 中添加 VoskConfig:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoskConfig {
    pub url: String,
    pub api_key: String,
    pub enabled: bool,
    pub silence_timeout: u32,
}

impl Default for VoskConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:5000".to_string(),
            api_key: "".to_string(),
            enabled: false,
            silence_timeout: 3000,
        }
    }
}

// 在 AppConfig 中添加
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub openclaw: OpenClawConfig,
    pub market: MarketConfig,
    pub preferences: AppPreferences,
    pub vosk: VoskConfig,  // 添加这行
}
```

**Step 2: 验证编译 (如果 Rust 已安装)**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

**Step 3: 提交**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add Vosk config to Rust backend"
```

---

## Task 6: 更新 ConfigView 添加 Vosk 配置界面

**Files:**
- Modify: `src/views/ConfigView.vue`

**Step 1: 添加 Vosk 配置界面**

在配置页面中添加语音输入配置区块：

```vue
<!-- 在 Skills 市场配置后面添加 -->
<div class="mb-8">
  <h2 class="text-lg font-semibold mb-4">语音输入 (Vosk)</h2>
  <div class="space-y-4">
    <div class="flex items-center gap-2">
      <input
        v-model="config.vosk.enabled"
        type="checkbox"
        id="vosk_enabled"
        class="w-4 h-4 text-primary-600"
      />
      <label for="vosk_enabled" class="text-sm">启用语音输入</label>
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Vosk 服务地址</label>
      <input
        v-model="config.vosk.url"
        type="text"
        placeholder="ws://localhost:5000"
        :disabled="!config.vosk.enabled"
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
      />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">API Key</label>
      <input
        v-model="config.vosk.api_key"
        type="password"
        placeholder="请输入 API Key"
        :disabled="!config.vosk.enabled"
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
      />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">静音超时 (毫秒)</label>
      <input
        v-model.number="config.vosk.silence_timeout"
        type="number"
        placeholder="3000"
        :disabled="!config.vosk.enabled"
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
      />
    </div>
  </div>
</div>
```

**Step 2: 更新 interface**

```typescript
interface Config {
  openclaw: {
    url: string
    use_local: boolean
  }
  market: {
    url: string
    enabled: boolean
  }
  preferences: {
    theme: string
    language: string
  }
  vosk: {  // 添加
    url: string
    api_key: string
    enabled: boolean
    silence_timeout: number
  }
}

const config = ref<Config>({
  openclaw: { url: 'http://localhost:8000', use_local: true },
  market: { url: 'http://localhost:3001', enabled: true },
  preferences: { theme: 'system', language: 'zh-CN' },
  vosk: { url: 'ws://localhost:5000', api_key: '', enabled: false, silence_timeout: 3000 }
})
```

**Step 3: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 4: 提交**

```bash
git add src/views/ConfigView.vue
git commit -m "feat: add Vosk config UI"
```

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-03-04-shine-helper-voice-input-implementation-plan.md`. Two execution options:**

**1. Subagent-Driven (this session)** - 我为每个任务分配新的子代理，任务间进行代码审查，快速迭代

**2. Parallel Session (separate)** - 在新会话中使用 executing-plans，批量执行并设置检查点

**你想选择哪种执行方式？**

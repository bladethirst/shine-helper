import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

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

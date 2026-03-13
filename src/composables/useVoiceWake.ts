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

    // 监听识别结果 - 支持增量更新（仅在 wake-listening 状态下处理）
    unlistenResult = await listen<VoiceResultPayload>('voice-result', (event) => {
      console.log('[useVoiceWake] Received voice-result:', event.payload, 'isWakeListening:', isWakeListening.value)
      // 使用 isWakeListening 判断，避免 status 被修改后导致后续事件被忽略
      if (!isWakeListening.value) {
        return
      }

      const { text, is_final } = event.payload

      if (is_final) {
        // 最终结果：追加到 transcript，清空 partial
        console.log('[useVoiceWake] Final result, appending:', text)
        if (text.trim()) {
          transcript.value = (transcript.value + ' ' + text).trim()
        }
        partialTranscript.value = ''
        // 不修改 status，保持 wake-listening 状态以继续接收后续事件
      } else {
        // 部分结果：更新 partialTranscript
        partialTranscript.value = text
        // 不修改 status，保持 wake-listening 状态以继续接收后续事件
      }
    })

    // 监听状态变化 - 新增 wake-listening 状态
    unlistenState = await listen<{ state: VoiceStatus }>('voice-state-changed', (event) => {
      console.log('[useVoiceWake] Received voice-state-changed:', event.payload.state)
      const newState = event.payload.state
      // 如果进入 listening 状态且是唤醒后，标记为 wake-listening
      if (newState === 'listening') {
        isWakeListening.value = true
        status.value = 'wake-listening'
      } else if (newState === 'idle') {
        console.log('[useVoiceWake] State changed to idle, clearing transcript')
        isWakeListening.value = false
        transcript.value = ''
        partialTranscript.value = ''
        status.value = 'idle'
      } else {
        // 其他状态（waking、processing、error），保持 isWakeListening 不变
        status.value = newState
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

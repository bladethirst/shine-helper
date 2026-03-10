import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export type VoiceStatus = 'idle' | 'waking' | 'listening' | 'processing' | 'error'

export interface VoiceWakeConfig {
  enabled: boolean
  wake_word: string
  wake_sounds: string[]
  silence_timeout: number
  end_words: string[]
}

export function useVoiceWake() {
  const status = ref<VoiceStatus>('idle')
  const transcript = ref('')
  const error = ref<string | null>(null)
  const isEnabled = ref(false)

  const isListening = computed(() => status.value === 'listening')
  const isWaking = computed(() => status.value === 'waking')
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

    // 监听识别结果
    unlistenResult = await listen<{ text: string; is_final: boolean }>('voice-result', (event) => {
      if (event.payload.is_final) {
        transcript.value += event.payload.text
      }
      status.value = 'processing'
    })

    // 监听状态变化
    unlistenState = await listen<{ state: VoiceStatus }>('voice-state-changed', (event) => {
      status.value = event.payload.state
    })

    // 监听错误
    unlistenError = await listen<{ message: string }>('voice-error', (event) => {
      error.value = event.payload.message
      status.value = 'error'
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

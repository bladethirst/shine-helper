<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <!-- 状态指示器 -->
    <div v-if="isListening || isProcessing || hasError || isWakeListening" class="mb-2 px-2">
      <span v-if="isWakeListening" class="text-sm text-purple-600 font-medium">
        📡 唤醒聆听中，请说指令... {{ wakePartialTranscript }}
      </span>
      <span v-else-if="isListening" class="text-sm text-primary-600">
        🎤 正在聆听... {{ interimTranscript || '请说话' }}
      </span>
      <span v-else-if="isProcessing" class="text-sm text-yellow-600">
        ⚙️ 处理中... {{ wakePartialTranscript || interimTranscript }}
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

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useVoiceRecognition } from '@/composables/useVoiceRecognition'
import { useVoiceWake } from '@/composables/useVoiceWake'
import { listen as tauriListen } from '@tauri-apps/api/event'

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
  partialTranscript: wakePartialTranscript,
  isWakeListeningState,
  status: _wakeStatus,
  clearTranscript,
  stopWakeListening: _stopWakeListening,
  start,
  stop,
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

// 监听唤醒识别结果（仅在唤醒聆听状态下生效）
watch(wakeTranscript, (newVal) => {
  console.log('[ChatInput] wakeTranscript changed:', newVal, 'isWakeListening:', isWakeListening.value)
  if (newVal && isWakeListening.value) {
    displayText.value = newVal
  }
})

// 监听手动语音识别结果（仅在非唤醒聆听状态下生效）
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
    // 如果是唤醒后聆听状态，发送后重启唤醒服务
    if (isWakeListening.value) {
      // 重启唤醒服务：先停止，等待一下，再启动
      stop().then(() => {
        // 等待 500ms 确保旧线程退出
        setTimeout(() => {
          start()
        }, 500)
      })
      clearTranscript()
    } else {
      // 如果正在录音，先停止
      if (isListening.value || isProcessing.value) {
        stopVoiceRecognition()
      }
      resetVoice()
    }

    emit('send', displayText.value)
    displayText.value = ''
  }
}

// 监听语音输入完成事件（用于自动发送）
onMounted(async () => {
  const unlistenComplete = await tauriListen<void>('voice-input-complete', () => {
    console.log('[ChatInput] Received voice-input-complete, isWakeListening:', isWakeListening.value, 'displayText:', displayText.value)
    // 延迟一点，确保 displayText 已更新
    setTimeout(() => {
      if (displayText.value.trim()) {
        // 有文字，自动发送并重启唤醒服务
        emit('send', displayText.value)
        displayText.value = ''
        // 重启唤醒服务：先停止，等待一下，再启动
        stop().then(() => {
          setTimeout(() => {
            start()
          }, 500)
        })
        clearTranscript()
      } else {
        // 没有文字（可能是说了"结束"但没有其他内容），直接重启唤醒服务
        console.log('[ChatInput] No displayText, just restarting wake service')
        stop().then(() => {
          setTimeout(() => {
            start()
          }, 500)
        })
        clearTranscript()
      }
    }, 100)
  })

  onUnmounted(() => {
    unlistenComplete()
  })
})
</script>

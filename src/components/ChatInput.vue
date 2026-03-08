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
        :disabled="!voskEnabled || !isSupported"
        :class="[
          'px-3 py-2 rounded-lg transition-colors',
          isListening ? 'bg-red-500 text-white animate-pulse' : 'bg-gray-100 text-gray-700 hover:bg-gray-200',
          (!voskEnabled || !isSupported) && 'opacity-50 cursor-not-allowed'
        ]"
        :title="!voskEnabled ? '请先在设置中启用语音输入' : (isSupported ? (isListening ? '停止录音' : '开始录音') : '您的浏览器不支持语音输入')"
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
import { ref, computed, watch } from 'vue'
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
    alert('语音输入已禁用\n\n请在设置页面：\n1. 进入配置页面\n2. 勾选"启用语音输入"\n3. 保存配置\n4. 返回聊天页面即可使用')
    return
  }
  
  if (!isSupported.value) {
    alert('您的浏览器不支持语音输入\n\n请使用现代浏览器：\n- Chrome 90+\n- Edge 90+\n- Firefox 88+')
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

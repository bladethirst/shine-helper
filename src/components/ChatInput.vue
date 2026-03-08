<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <!-- Voice status indicator -->
    <div v-if="isListening || isWaking || hasVoiceWakeError" class="mb-2 px-2">
      <span v-if="isWaking" class="text-sm text-yellow-600">
        🎤 唤醒中...
      </span>
      <span v-else-if="isListening" class="text-sm text-primary-600 animate-pulse">
        🎤 聆听中... {{ voiceWakeTranscript || '请说话' }}
      </span>
      <span v-else-if="hasVoiceWakeError" class="text-sm text-red-600">
        ❌ {{ voiceWakeError || '识别错误' }}
      </span>
    </div>
    
    <div class="flex gap-2">
      <input
        v-model="displayText"
        type="text"
        placeholder="请输入消息..."
        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
        @keyup.enter="send"
      />
      <!-- Microphone button -->
      <button
        @click="toggleVoice"
        :class="[
          'px-3 py-2 rounded-lg transition-colors',
          isListening ? 'bg-red-500 text-white animate-pulse' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
        ]"
        :title="isListening ? '停止录音' : '开始语音唤醒'"
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
import { ref, watch } from 'vue'
import { useVoiceWake } from '@/composables/useVoiceWake'

const props = defineProps<{
  voskEnabled?: boolean
}>()

const emit = defineEmits<{
  send: [content: string]
}>()

const displayText = ref('')

const {
  transcript: voiceWakeTranscript,
  error: voiceWakeError,
  isListening,
  isWaking,
  hasError: hasVoiceWakeError,
  toggle,
  reset
} = useVoiceWake()

const toggleVoice = async () => {
  if (!props.voskEnabled) {
    alert('请先在设置中启用语音唤醒功能')
    return
  }
  await toggle()
}

// Sync transcript to input field
watch(voiceWakeTranscript, (newVal) => {
  if (newVal) {
    displayText.value = newVal
  }
})

const send = () => {
  if (displayText.value.trim()) {
    if (isListening.value) {
      toggle() // Stop listening
    }
    emit('send', displayText.value)
    displayText.value = ''
    reset()
  }
}
</script>
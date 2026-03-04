<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <div class="flex gap-2">
      <input
        v-model="message"
        type="text"
        placeholder="请输入消息..."
        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
        @keyup.enter="send"
      />
      <button
        @click="send"
        :disabled="!message.trim()"
        class="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        发送
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const message = ref('')
const emit = defineEmits<{
  send: [content: string]
}>()

const send = () => {
  if (message.value.trim()) {
    emit('send', message.value)
    message.value = ''
  }
}
</script>

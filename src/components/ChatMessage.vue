<template>
  <div :class="['flex gap-3 mb-4', role === 'user' ? 'flex-row-reverse' : '']">
    <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm"
         :class="role === 'user' ? 'bg-primary-500 text-white' : 'bg-gray-200'">
      {{ role === 'user' ? '👤' : '🤖' }}
    </div>
    <div :class="['max-w-[70%] rounded-lg p-3', role === 'user' ? 'bg-primary-50' : 'bg-gray-100']">
      <p class="whitespace-pre-wrap">{{ content }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  role: 'user' | 'assistant'
  content: string
}>()
</script>

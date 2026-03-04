<template>
  <div class="h-full flex flex-col">
    <!-- 会话列表 -->
    <div class="border-b border-gray-200 p-4 flex items-center justify-between">
      <h2 class="text-lg font-semibold">会话列表</h2>
      <button
        @click="createNewSession"
        class="px-3 py-1 text-sm bg-primary-500 text-white rounded hover:bg-primary-600"
      >
        新建会话
      </button>
    </div>

    <!-- 消息列表 -->
    <div class="flex-1 overflow-auto p-4">
      <div v-if="messages.length === 0" class="text-center text-gray-400 mt-20">
        <p class="text-4xl mb-4">💬</p>
        <p>开始一段新对话吧</p>
      </div>
      <ChatMessage
        v-for="msg in messages"
        :key="msg.id"
        :role="msg.role as 'user' | 'assistant'"
        :content="msg.content"
      />
      <div v-if="isLoading" class="flex gap-3 mb-4">
        <div class="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center text-sm">🤖</div>
        <div class="bg-gray-100 rounded-lg p-3">
          <span class="animate-pulse">正在输入...</span>
        </div>
      </div>
    </div>

    <!-- 输入框 -->
    <ChatInput @send="handleSend" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ChatMessage from '@/components/ChatMessage.vue'
import ChatInput from '@/components/ChatInput.vue'

interface Message {
  id: string
  session_id: string
  role: string
  content: string
  timestamp: string
}

interface Session {
  id: string
  title: string
  created_at: string
  updated_at: string
}

const sessions = ref<Session[]>([])
const currentSession = ref<Session | null>(null)
const messages = ref<Message[]>([])
const isLoading = ref(false)

onMounted(async () => {
  try {
    sessions.value = await invoke<Session[]>('list_sessions')
    if (sessions.value.length > 0) {
      currentSession.value = sessions.value[0]
      messages.value = await invoke<Message[]>('get_messages', { sessionId: currentSession.value.id })
    }
  } catch (e) {
    console.error('Failed to load sessions:', e)
  }
})

const createNewSession = async () => {
  try {
    const session = await invoke<Session>('create_session', { title: '新会话' })
    sessions.value.unshift(session)
    currentSession.value = session
    messages.value = []
  } catch (e) {
    console.error('Failed to create session:', e)
  }
}

const handleSend = async (content: string) => {
  if (!currentSession.value) {
    await createNewSession()
  }
  
  // 添加用户消息
  try {
    const userMsg = await invoke<Message>('add_message', {
      sessionId: currentSession.value!.id,
      role: 'user',
      content
    })
    messages.value.push(userMsg)
    
    isLoading.value = true
    
    // TODO: 调用 OpenClaw API (Task 3.2)
    // 暂时显示模拟响应
    setTimeout(async () => {
      const assistantMsg = await invoke<Message>('add_message', {
        sessionId: currentSession.value!.id,
        role: 'assistant',
        content: '这是一条模拟回复。请先配置 OpenClaw 连接。'
      })
      messages.value.push(assistantMsg)
      isLoading.value = false
    }, 1000)
  } catch (e) {
    console.error('Failed to send message:', e)
    isLoading.value = false
  }
}
</script>

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
    </div>

    <!-- 输入框 -->
    <ChatInput @send="handleSend" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
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

interface StreamChunk {
  content: string
  done: boolean
}

const sessions = ref<Session[]>([])
const currentSession = ref<Session | null>(null)
const messages = ref<Message[]>([])
const isLoading = ref(false)
let unlistenChunk: UnlistenFn | null = null
let unlistenError: UnlistenFn | null = null

function scrollToBottom() {
  nextTick(() => {
    const container = document.querySelector('.flex-1.overflow-auto')
    if (container) {
      container.scrollTop = container.scrollHeight
    }
  })
}

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
  
  unlistenChunk = await listen<StreamChunk>('chat_chunk', (event) => {
    const chunk = event.payload
    const lastMsg = messages.value[messages.value.length - 1]
    
    if (chunk.done) {
      isLoading.value = false
    } else if (lastMsg && lastMsg.role === 'assistant') {
      lastMsg.content += chunk.content
      scrollToBottom()
    }
  })
  
  unlistenError = await listen<string>('chat_error', (event) => {
    console.error('Chat error:', event.payload)
    isLoading.value = false
  })
})

onUnmounted(() => {
  unlistenChunk?.()
  unlistenError?.()
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
  
  try {
    const userMsg = await invoke<Message>('add_message', {
      sessionId: currentSession.value!.id,
      role: 'user',
      content
    })
    messages.value.push(userMsg)
    scrollToBottom()
    
    isLoading.value = true
    
    const placeholderMsg: Message = {
      id: `msg_ai_${Date.now()}`,
      session_id: currentSession.value!.id,
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString()
    }
    messages.value.push(placeholderMsg)
    
    await invoke('send_message_stream', {
      sessionId: currentSession.value!.id,
      message: content
    })
  } catch (e: any) {
    console.error('Failed to send message:', e)
    isLoading.value = false
    
    const errorMessage = e?.message || e?.toString() || String(e)
    const errorMsg = await invoke<Message>('add_message', {
      sessionId: currentSession.value!.id,
      role: 'assistant',
      content: `发送失败: ${errorMessage}`
    })
    messages.value.push(errorMsg)
  }
}
</script>
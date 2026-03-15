<template>
  <div class="h-full flex">
    <!-- 左侧会话列表边栏 -->
    <div class="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
      <!-- 新建会话按钮 -->
      <div class="p-3 border-b border-gray-200">
        <button
          @click="createNewSession"
          class="w-full px-3 py-2 text-sm bg-primary-500 text-white rounded hover:bg-primary-600 flex items-center justify-center gap-2"
        >
          <span>+</span>
          <span>新建会话</span>
        </button>
      </div>

      <!-- 会话列表 -->
      <div class="flex-1 overflow-auto">
        <div
          v-for="session in sessions"
          :key="session.id"
          @click="selectSession(session)"
          class="px-3 py-2.5 mx-2 my-1 rounded cursor-pointer flex items-center justify-between group"
          :class="currentSession?.id === session.id ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-200'"
        >
          <div class="flex-1 truncate text-sm">
            {{ session.title || '新会话' }}
          </div>
          <button
            @click.stop="confirmDeleteSession(session)"
            class="opacity-0 group-hover:opacity-100 p-1 text-gray-400 hover:text-red-500 transition-opacity"
            title="删除会话"
          >
            ×
          </button>
        </div>

        <!-- 空状态 -->
        <div v-if="sessions.length === 0" class="text-center text-gray-400 mt-8 px-4">
          <p class="text-xs">暂无会话</p>
          <p class="text-xs mt-1">点击"新建会话"开始</p>
        </div>
      </div>
    </div>

    <!-- 右侧聊天区域 -->
    <div class="flex-1 flex flex-col">
      <!-- 消息列表 -->
      <div class="flex-1 overflow-auto p-4">
        <div v-if="messages.length === 0" class="text-center text-gray-400 mt-20">
          <p class="text-4xl mb-4">💬</p>
          <p>{{ currentSession ? '开始对话吧' : '选择一个会话或创建新会话' }}</p>
        </div>
        <ChatMessage
          v-for="msg in messages"
          :key="msg.id"
          :role="msg.role as 'user' | 'assistant'"
          :content="msg.content"
        />
      </div>

      <!-- 输入框 -->
      <ChatInput
        @send="handleSend"
        :vosk-enabled="voskEnabled"
        :vosk-url="voskUrl"
        :vosk-api-key="voskApiKey"
      />
    </div>
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

// Vosk 配置
const voskEnabled = ref(false)
const voskUrl = ref('ws://192.168.150.26:2700')
const voskApiKey = ref('')
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

interface AppConfig {
  openclaw: {
    url: string
    token: string
    use_local: boolean
    auto_start: boolean
  }
  vosk: {
    url: string
    api_key: string
    enabled: boolean
    silence_timeout: number
  }
}

const loadSessions = async () => {
  try {
    sessions.value = await invoke<Session[]>('list_sessions')
  } catch (e) {
    console.error('Failed to load sessions:', e)
  }
}

const selectSession = async (session: Session) => {
  if (currentSession.value?.id === session.id) return

  currentSession.value = session
  isLoading.value = true

  try {
    messages.value = await invoke<Message[]>('get_messages', { sessionId: session.id })
  } catch (e) {
    console.error('Failed to load messages:', e)
    messages.value = []
  } finally {
    isLoading.value = false
  }
}

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

const confirmDeleteSession = async (session: Session) => {
  if (!confirm(`确定要删除会话 "${session.title || '新会话'}" 吗？删除后无法恢复。`)) {
    return
  }

  try {
    await invoke('delete_session', { sessionId: session.id })
    sessions.value = sessions.value.filter(s => s.id !== session.id)

    if (currentSession.value?.id === session.id) {
      currentSession.value = sessions.value.length > 0 ? sessions.value[0] : null
      messages.value = currentSession.value
        ? await invoke<Message[]>('get_messages', { sessionId: currentSession.value.id })
        : []
    }
  } catch (e) {
    console.error('Failed to delete session:', e)
    alert('删除会话失败，请重试')
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
      content: `发送失败：${errorMessage}`
    })
    messages.value.push(errorMsg)
  }
}

onMounted(async () => {
  try {
    const config = await invoke<AppConfig>('get_app_config')
    voskEnabled.value = config.vosk.enabled
    voskUrl.value = config.vosk.url
    voskApiKey.value = config.vosk.api_key
  } catch (e) {
    console.error('Failed to load config:', e)
  }

  await loadSessions()

  if (sessions.value.length > 0) {
    await selectSession(sessions.value[0])
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
</script>

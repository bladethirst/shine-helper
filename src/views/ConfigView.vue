<template>
  <div class="h-full overflow-auto p-6">
    <h1 class="text-2xl font-bold mb-6">配置</h1>

    <!-- AI 服务配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">AI 服务 (OpenClaw)</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">服务地址</label>
          <input
            v-model="config.openclaw.url"
            type="text"
            placeholder="http://localhost:18789"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">Token</label>
          <input
            v-model="config.openclaw.token"
            type="password"
            placeholder="请输入 OpenClaw Token"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
      </div>
    </div>

    <!-- Skills 市场配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">Skills 市场</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">市场地址</label>
          <input
            v-model="config.market.url"
            type="text"
            placeholder="http://localhost:3001"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div class="flex items-center gap-2">
          <input
            v-model="config.market.enabled"
            type="checkbox"
            id="market_enabled"
            class="w-4 h-4 text-primary-600"
          />
          <label for="market_enabled" class="text-sm">启用市场</label>
        </div>
      </div>
    </div>

    <!-- 语音输入 (Vosk) 配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">语音输入 (Vosk)</h2>
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <input
            v-model="config.vosk.enabled"
            type="checkbox"
            id="vosk_enabled"
            class="w-4 h-4 text-primary-600"
          />
          <label for="vosk_enabled" class="text-sm">启用语音输入</label>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">Vosk 服务地址</label>
          <input
            v-model="config.vosk.url"
            type="text"
            placeholder="ws://localhost:5000"
            :disabled="!config.vosk.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">API Key</label>
          <input
            v-model="config.vosk.api_key"
            type="password"
            placeholder="请输入 API Key"
            :disabled="!config.vosk.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">静音超时 (毫秒)</label>
          <input
            v-model.number="config.vosk.silence_timeout"
            type="number"
            placeholder="3000"
            :disabled="!config.vosk.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
      </div>
    </div>

    <!-- 语音唤醒配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">语音唤醒</h2>
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <input
            v-model="config.voice.enabled"
            type="checkbox"
            id="voice_enabled"
            class="w-4 h-4 text-primary-600"
          />
          <label for="voice_enabled" class="text-sm">启用语音唤醒</label>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">唤醒词</label>
          <input
            v-model="config.voice.wake_word"
            type="text"
            placeholder="小 Shine"
            :disabled="!config.voice.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">QwenASR 服务地址</label>
          <input
            v-model="config.voice.qwen_asr_url"
            type="text"
            placeholder="ws://localhost:5000"
            :disabled="!config.voice.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">QwenASR API Key</label>
          <input
            v-model="config.voice.qwen_asr_api_key"
            type="password"
            placeholder="请输入 API Key"
            :disabled="!config.voice.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">静音超时 (毫秒)</label>
          <input
            v-model.number="config.voice.silence_timeout"
            type="number"
            placeholder="3000"
            :disabled="!config.voice.enabled"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
          />
        </div>
      </div>
    </div>

    <!-- 应用偏好 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">应用偏好</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">主题</label>
          <select
            v-model="config.preferences.theme"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
            <option value="system">跟随系统</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">语言</label>
          <select
            v-model="config.preferences.language"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <button
      @click="saveConfig"
      class="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
    >
      保存配置
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Config {
  openclaw: {
    url: string
    token: string
  }
  market: {
    url: string
    enabled: boolean
  }
  preferences: {
    theme: string
    language: string
  }
  vosk: {
    url: string
    api_key: string
    enabled: boolean
    silence_timeout: number
  }
  voice: {
    enabled: boolean
    wake_word: string
    qwen_asr_url: string
    qwen_asr_api_key: string
    silence_timeout: number
  }
}

const DEFAULT_TOKEN = 'bca6dd74789ed4ebd4eb8761215de98d11f62c85eb16239a'

const config = ref<Config>({
  openclaw: { url: 'http://localhost:18789', token: DEFAULT_TOKEN },
  market: { url: 'http://localhost:3001', enabled: true },
  preferences: { theme: 'system', language: 'zh-CN' },
  vosk: { url: 'ws://192.168.150.26:5000', api_key: '', enabled: false, silence_timeout: 3000 },
  voice: { enabled: false, wake_word: '小 Shine', qwen_asr_url: 'ws://localhost:5000', qwen_asr_api_key: '', silence_timeout: 3000 }
})

// 加载保存的配置
onMounted(() => {
  const saved = localStorage.getItem('shine_helper_config')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      if (parsed.openclaw) {
        config.value.openclaw.url = parsed.openclaw.serverUrl || config.value.openclaw.url
        config.value.openclaw.token = parsed.openclaw.token || config.value.openclaw.token
      }
      if (parsed.market) {
        config.value.market.url = parsed.market.url || config.value.market.url
        config.value.market.enabled = parsed.market.enabled ?? config.value.market.enabled
      }
      if (parsed.preferences) {
        config.value.preferences.theme = parsed.preferences.theme || config.value.preferences.theme
        config.value.preferences.language = parsed.preferences.language || config.value.preferences.language
      }
      if (parsed.vosk) {
        config.value.vosk.url = parsed.vosk.url || config.value.vosk.url
        config.value.vosk.api_key = parsed.vosk.api_key || config.value.vosk.api_key
        config.value.vosk.enabled = parsed.vosk.enabled ?? config.value.vosk.enabled
        config.value.vosk.silence_timeout = parsed.vosk.silence_timeout || config.value.vosk.silence_timeout
      }
      if (parsed.voice) {
        config.value.voice.enabled = parsed.voice.enabled ?? config.value.voice.enabled
        config.value.voice.wake_word = parsed.voice.wake_word || config.value.voice.wake_word
        config.value.voice.qwen_asr_url = parsed.voice.qwen_asr_url || config.value.voice.qwen_asr_url
        config.value.voice.qwen_asr_api_key = parsed.voice.qwen_asr_api_key || config.value.voice.qwen_asr_api_key
        config.value.voice.silence_timeout = parsed.voice.silence_timeout || config.value.voice.silence_timeout
      }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }
})

const saveConfig = () => {
  const fullConfig = {
    openclaw: {
      serverUrl: config.value.openclaw.url,
      token: config.value.openclaw.token
    },
    market: {
      url: config.value.market.url,
      enabled: config.value.market.enabled
    },
    preferences: {
      theme: config.value.preferences.theme,
      language: config.value.preferences.language
    },
    vosk: {
      url: config.value.vosk.url,
      api_key: config.value.vosk.api_key,
      enabled: config.value.vosk.enabled,
      silence_timeout: config.value.vosk.silence_timeout
    },
    voice: {
      enabled: config.value.voice.enabled,
      wake_word: config.value.voice.wake_word,
      qwen_asr_url: config.value.voice.qwen_asr_url,
      qwen_asr_api_key: config.value.voice.qwen_asr_api_key,
      silence_timeout: config.value.voice.silence_timeout
    }
  }
  localStorage.setItem('shine_helper_config', JSON.stringify(fullConfig))
  console.log('Saving config:', fullConfig)
  
  window.dispatchEvent(new CustomEvent('config-updated', { detail: fullConfig }))
  
  alert('配置已保存')
}
</script>
<template>
  <div class="h-full overflow-auto p-6">
    <h1 class="text-2xl font-bold mb-6">配置</h1>

    <!-- AI 服务配置 - 简化显示 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">AI 服务</h2>
      <div class="text-sm text-gray-500 mb-4">
        已集成 OpenClaw AI 助手
      </div>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">API Key (可选)</label>
          <input
            v-model="apiKey"
            type="password"
            placeholder="用于 OpenAI 等模型认证"
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
import { ref } from 'vue'

interface Config {
  openclaw: {
    url: string
    use_local: boolean
    auto_start: boolean
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
}

const config = ref<Config>({
  openclaw: { url: 'http://localhost:18789', use_local: true, auto_start: true },
  market: { url: 'http://localhost:3001', enabled: true },
  preferences: { theme: 'system', language: 'zh-CN' },
  vosk: { url: 'ws://localhost:5000', api_key: '', enabled: false, silence_timeout: 3000 }
})

const apiKey = ref('')

const saveConfig = () => {
  // 保存配置到后端
  console.log('Saving config:', config.value)
  alert('配置已保存')
}
</script>
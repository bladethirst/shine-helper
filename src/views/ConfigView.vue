<template>
  <div class="h-full overflow-auto p-6">
    <h1 class="text-2xl font-bold mb-6">配置</h1>

    <!-- OpenClaw 配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">OpenClaw 连接</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">服务地址</label>
          <input
            v-model="config.openclaw.url"
            type="text"
            placeholder="http://localhost:8000"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div class="flex items-center gap-2">
          <input
            v-model="config.openclaw.use_local"
            type="checkbox"
            id="use_local"
            class="w-4 h-4 text-primary-600"
          />
          <label for="use_local" class="text-sm">使用本地服务</label>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">API Key</label>
          <input
            v-model="apiKey"
            type="password"
            placeholder="请输入 API Key"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <button
          @click="testConnection"
          class="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
        >
          测试连接
        </button>
        <span v-if="connectionStatus" :class="connectionStatus === 'success' ? 'text-green-600' : 'text-red-600'">
          {{ connectionStatus === 'success' ? '✓ 连接成功' : '✗ 连接失败' }}
        </span>
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
  }
  market: {
    url: string
    enabled: boolean
  }
  preferences: {
    theme: string
    language: string
  }
}

const config = ref<Config>({
  openclaw: { url: 'http://localhost:8000', use_local: true },
  market: { url: 'http://localhost:3001', enabled: true },
  preferences: { theme: 'system', language: 'zh-CN' }
})

const apiKey = ref('')
const connectionStatus = ref<'success' | 'error' | null>(null)

const saveConfig = () => {
  // TODO: 保存配置到后端
  console.log('Saving config:', config.value)
  alert('配置已保存')
}

const testConnection = () => {
  // TODO: 测试连接
  console.log('Testing connection to:', config.value.openclaw.url)
  connectionStatus.value = 'success'
}
</script>

<template>
  <div class="h-full flex flex-col p-6">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">Skills 市场</h1>
      <button
        @click="refreshSkills"
        class="px-4 py-2 text-primary-600 border border-primary-600 rounded hover:bg-primary-50"
      >
        🔄 刷新
      </button>
    </div>

    <!-- 搜索框 -->
    <div class="mb-6">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索 Skills..."
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
      />
    </div>

    <!-- 已安装的 Skills -->
    <div v-if="installedSkills.length > 0" class="mb-8">
      <h2 class="text-lg font-semibold mb-4">已安装</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="skill in installedSkills"
          :key="skill.id"
          class="border border-gray-200 rounded-lg p-4"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-2xl">{{ skill.icon || '📦' }}</span>
            <h3 class="font-semibold">{{ skill.name }}</h3>
          </div>
          <p class="text-sm text-gray-600 mb-3">{{ skill.description }}</p>
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-400">v{{ skill.version }}</span>
            <button
              @click="uninstallSkill(skill.id)"
              class="px-3 py-1 text-sm text-red-600 border border-red-600 rounded hover:bg-red-50"
            >
              卸载
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 市场 Skills -->
    <div>
      <h2 class="text-lg font-semibold mb-4">可用 Skills</h2>
      <div v-if="filteredMarketSkills.length === 0" class="text-center text-gray-400 py-10">
        暂无可用的 Skills
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="skill in filteredMarketSkills"
          :key="skill.id"
          class="border border-gray-200 rounded-lg p-4"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-2xl">{{ skill.icon || '📦' }}</span>
            <h3 class="font-semibold">{{ skill.name }}</h3>
          </div>
          <p class="text-sm text-gray-600 mb-3">{{ skill.description }}</p>
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-400">{{ skill.author }}</span>
            <button
              @click="installSkill(skill)"
              class="px-3 py-1 text-sm bg-primary-500 text-white rounded hover:bg-primary-600"
            >
              安装
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

interface Skill {
  id: string
  name: string
  description: string
  version: string
  author: string
  icon?: string
  installed: boolean
  enabled: boolean
}

const searchQuery = ref('')
const installedSkills = ref<Skill[]>([])
const marketSkills = ref<Skill[]>([])

// 模拟市场数据（实际应从 API 获取）
onMounted(async () => {
  try {
    installedSkills.value = await invoke<Skill[]>('get_local_skills')
    marketSkills.value = [
      { id: 'report', name: '报表整理', description: '帮助整理各类报表数据', version: '1.0.0', author: '官方', installed: false, enabled: false },
      { id: 'email', name: '邮件发送', description: '自动化发送邮件功能', version: '1.0.0', author: '官方', installed: false, enabled: false },
      { id: 'data-fetch', name: '数据获取', description: '从系统获取数据', version: '1.0.0', author: '官方', installed: false, enabled: false },
    ]
  } catch (e) {
    console.error('Failed to load skills:', e)
  }
})

const filteredMarketSkills = computed(() => {
  const installed = new Set(installedSkills.value.map(s => s.id))
  return marketSkills.value
    .filter(s => !installed.has(s.id))
    .filter(s => s.name.includes(searchQuery.value) || s.description.includes(searchQuery.value))
})

const refreshSkills = async () => {
  try {
    installedSkills.value = await invoke<Skill[]>('get_local_skills')
  } catch (e) {
    console.error('Failed to refresh:', e)
  }
}

const installSkill = async (skill: Skill) => {
  try {
    await invoke('install_skill', { skillId: skill.id })
    await refreshSkills()
  } catch (e) {
    console.error('Failed to install:', e)
  }
}

const uninstallSkill = async (skillId: string) => {
  try {
    await invoke('uninstall_skill', { skillId })
    await refreshSkills()
  } catch (e) {
    console.error('Failed to uninstall:', e)
  }
}
</script>

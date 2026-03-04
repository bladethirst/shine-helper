<template>
  <div class="h-screen flex">
    <aside class="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
      <div class="p-4 border-b border-gray-200">
        <h1 class="text-xl font-bold text-primary-600">Shine Helper</h1>
      </div>
      <nav class="flex-1 p-4">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors"
          :class="$route.path === item.path ? 'bg-primary-100 text-primary-700' : 'text-gray-600 hover:bg-gray-100'"
        >
          <span class="text-xl">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </RouterLink>
      </nav>
    </aside>
    <main class="flex-1 overflow-auto bg-white">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'

const navItems = [
  { path: '/', label: '对话', icon: '💬' },
  { path: '/skills', label: 'Skills', icon: '🛒' },
  { path: '/config', label: '配置', icon: '⚙️' }
]
</script>

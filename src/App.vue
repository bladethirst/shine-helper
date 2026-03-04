<template>
  <div class="h-screen flex">
    <aside class="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
      <div class="p-4 border-b border-gray-200">
        <h1 class="text-xl font-bold text-primary-600">Shine Helper</h1>
      </div>
      <nav class="flex-1 p-4">
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg mb-2 bg-primary-100 text-primary-700">
          <span class="text-xl">💬</span>
          <span>对话</span>
        </div>
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg mb-2 text-gray-600 hover:bg-gray-100 cursor-pointer">
          <span class="text-xl">🛒</span>
          <span>Skills</span>
        </div>
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-600 hover:bg-gray-100 cursor-pointer">
          <span class="text-xl">⚙️</span>
          <span>配置</span>
        </div>
      </nav>
    </aside>
    <main class="flex-1 overflow-auto bg-white">
      <div class="h-full flex items-center justify-center text-gray-400">
        <p>选择左侧导航开始使用</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
</script>

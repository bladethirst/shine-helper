import { createRouter, createWebHistory } from 'vue-router'
import ChatView from '@/views/ChatView.vue'
import SkillsView from '@/views/SkillsView.vue'
import ConfigView from '@/views/ConfigView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'chat',
      component: ChatView
    },
    {
      path: '/skills',
      name: 'skills',
      component: SkillsView
    },
    {
      path: '/config',
      name: 'config',
      component: ConfigView
    }
  ]
})

export default router

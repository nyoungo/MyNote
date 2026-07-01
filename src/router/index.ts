import { createRouter, createWebHistory } from 'vue-router'
import MainView from '@/views/MainView.vue'
import ClipboardPanel from '@/components/clipboard/ClipboardPanel.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'main',
      component: MainView,
    },
    {
      path: '/clipboard',
      name: 'clipboard',
      component: ClipboardPanel,
    },
  ],
})

export default router

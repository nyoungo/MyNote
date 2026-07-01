<script setup lang="ts">
/**
 * 根组件
 * - 路由视图（<router-view>）
 * - 启动时加载用户资料（若有 Token）
 * - 初始化剪贴板监听（仅 Tauri 环境）
 * - 从设置恢复主题色
 */
import { onMounted } from 'vue'
import { useUserStore } from '@/stores/userStore'
import { useClipboardStore } from '@/stores/clipboardStore'

const userStore = useUserStore()

onMounted(async () => {
  // 如果有 Token，尝试恢复登录态
  if (userStore.token) {
    try {
      await userStore.loadProfile()
    } catch (e) {
      console.error('加载用户资料失败，Token 可能已过期:', e)
    }
  }

  // 初始化剪贴板监听（仅 Tauri 桌面端）
  try {
    const clipboardStore = useClipboardStore()
    clipboardStore.setupListeners()
  } catch (e) {
    console.warn('剪贴板面板初始化失败（非 Tauri 环境可忽略）:', e)
  }

  // 从设置恢复主题色
  try {
    const saved = JSON.parse(localStorage.getItem('mynote-settings') || '{}')
    const themeColor = saved.themeColor || 'blue'
    const validThemes = ['blue', 'orange', 'geekblue', 'green', 'red', 'purple']
    const theme = validThemes.includes(themeColor) ? themeColor : 'blue'
    document.documentElement.classList.add('theme-' + theme)
  } catch { /* */ }
})
</script>

<template>
  <router-view />
</template>

<style>
#app {
  height: 100%;
  width: 100%;
}
</style>

<template>
  <!-- 使用 ant-design-vue 的 a-avatar 展示用户头像 -->
  <div class="avatar-container">
    <a-avatar
      :size="40"
      :src="avatarUrl"
      :style="avatarStyle"
      class="user-avatar"
      :class="{ 'avatar-default': !avatarUrl }"
    >
      {{ initials }}
    </a-avatar>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useUserStore } from '@/stores/userStore'

const userStore = useUserStore()

const avatarUrl = computed(() => userStore.fullAvatarUrl || undefined)
const initials = computed(() => (userStore.username || 'U').charAt(0).toUpperCase())

const avatarStyle = computed(() => {
  if (userStore.fullAvatarUrl) return {}
  return {
    verticalAlign: 'middle',
    fontSize: '16px',
    fontWeight: 600,
    color: '#fff',
    cursor: 'pointer',
  }
})
</script>

<style scoped>
.avatar-container {
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none; /* 不拦截父容器的点击事件 */
}
.user-avatar {
  border: 2px solid var(--border-color);
  transition: border-color 0.2s;
  flex-shrink: 0;
}
.user-avatar:hover {
  border-color: var(--primary-color);
}
.avatar-default {
  background-color: var(--primary-color) !important;
}
</style>

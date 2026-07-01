<template>
  <!--
    侧边栏组件 — 基于 ant-design-vue
    - 顶部：用户头像与登录态（a-avatar）
    - 中部：主要导航按钮（a-tooltip）
    - 底部：设置入口
  -->
  <aside class="sidebar">
    <div class="sidebar-top" @click="handleAvatarClick" :title="loginHint">
      <a-avatar
        :size="36"
        :src="userStore.fullAvatarUrl || undefined"
        :style="!userStore.fullAvatarUrl ? { verticalAlign: 'middle', fontSize: '14px', fontWeight: 600, cursor: 'pointer', color: '#fff' } : { cursor: 'pointer', border: '2px solid transparent' }"
        class="sidebar-avatar"
        :class="{ 'avatar-default': !userStore.fullAvatarUrl }"
      >
        {{ initials }}
      </a-avatar>
      <span class="login-status" :class="{ logged: userStore.isLoggedIn }">
        {{ userStore.isLoggedIn ? userStore.username : '未登录' }}
      </span>
    </div>

    <nav class="sidebar-nav">
      <a-tooltip title="笔记" placement="right">
        <button
          class="nav-btn"
          :class="{ active: activeTab === 'notes' }"
          @click="switchTab('notes')"
        >
          <ReadOutlined style="font-size:20px" />
          <span class="nav-label">笔记</span>
        </button>
      </a-tooltip>
      <a-tooltip title="工具" placement="right">
        <button
          class="nav-btn"
          :class="{ active: activeTab === 'tools' }"
          @click="switchTab('tools')"
        >
          <ToolOutlined style="font-size:20px" />
          <span class="nav-label">工具</span>
        </button>
      </a-tooltip>
    </nav>

    <div class="sidebar-bottom">
      <a-tooltip title="设置" placement="right">
        <button class="nav-btn" @click="emit('openSettings')">
          <SettingOutlined style="font-size:20px" />
          <span class="nav-label">设置</span>
        </button>
      </a-tooltip>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  ReadOutlined,
  ToolOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import { useUserStore } from '@/stores/userStore'

const userStore = useUserStore()
const activeTab = ref<'notes' | 'tools'>('notes')

const emit = defineEmits<{
  openLogin: []
  openUserProfile: []
  openSettings: []
  tabChange: [tab: string]
}>()

const initials = computed(() => (userStore.username || 'U').charAt(0).toUpperCase())

const loginHint = computed(() =>
  userStore.isLoggedIn ? '点击查看用户信息' : '点击登录以使用云同步'
)

function handleAvatarClick() {
  if (userStore.isLoggedIn) {
    emit('openUserProfile')
  } else {
    emit('openLogin')
  }
}

function switchTab(tab: 'notes' | 'tools') {
  activeTab.value = tab
  emit('tabChange', tab)
}
</script>

<style scoped>
/*
 * 侧边栏布局：固定宽度，垂直弹性布局
 * 有道云笔记风格 — 简洁、干净、精致
 */
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  flex-shrink: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  padding: var(--spacing-xl) 0 var(--spacing-md);
  user-select: none;
  position: relative;
  z-index: var(--z-sidebar);
}

/* ---- 顶部头像区域 ---- */
.sidebar-top {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
  margin-bottom: var(--spacing-xxl);
  cursor: pointer;
  padding: 0 var(--spacing-xs);
}

.sidebar-avatar {
  border-radius: var(--radius-round);
  transition: transform var(--duration-fast), box-shadow var(--duration-fast);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
}

.avatar-default {
  background-color: var(--primary-color) !important;
}

.sidebar-top:hover .sidebar-avatar {
  transform: scale(1.05);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.18);
}

.login-status {
  font-size: 10px;
  color: var(--text-tertiary);
  white-space: nowrap;
  max-width: 48px;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: center;
  line-height: 1.2;
  transition: color var(--duration-fast);
}

.login-status.logged {
  color: var(--primary-color);
  font-weight: 500;
}

.sidebar-top:hover .login-status {
  color: var(--text-primary);
}

/* ---- 导航按钮区域 ---- */
.sidebar-nav {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  width: 100%;
  padding: 0 var(--spacing-sm);
}

.sidebar-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 0 var(--spacing-sm);
}

/*
 * 导航按钮：垂直图标+文字布局
 * 有道云风格：激活态带淡色背景+主色图标
 */
.nav-btn {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
  background: transparent;
  transition: all var(--duration-fast) var(--ease-out);
  gap: 2px;
  overflow: hidden;
  border: none;
  outline: none;
  cursor: pointer;
}

.nav-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.nav-btn.active {
  background-color: var(--primary-bg);
  color: var(--primary-color);
}

.nav-btn.active:hover {
  background-color: var(--primary-light);
}

.nav-label {
  font-size: 10px;
  line-height: 1;
  white-space: nowrap;
  font-weight: 500;
}
</style>

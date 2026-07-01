<script setup lang="ts">
/**
 * MainView — 应用主布局
 * 四栏式结构：Sidebar | ListView | Editor | ToolsView
 * 使用 flex 布局，侧边栏固定宽度，编辑器自适应
 */
import { ref } from 'vue'
import { useUserStore } from '@/stores/userStore'
import Sidebar from '@/components/common/Sidebar.vue'
import ListView from '@/components/common/ListView.vue'
import Editor from '@/components/common/Editor.vue'
import ToolsView from '@/views/ToolsView.vue'
import LoginDialog from '@/components/dialog/LoginDialog.vue'
import UserProfileModal from '@/components/user/UserProfileModal.vue'
import SettingsDialog from '@/components/dialog/SettingsDialog.vue'

const userStore = useUserStore()
const activeView = ref<'notes' | 'tools'>('notes')
const showLogin = ref(false)
const showUserProfile = ref(false)
const showSettings = ref(false)

function switchTab(tab: string) {
  if (tab === 'notes' || tab === 'tools') activeView.value = tab
}
</script>

<template>
  <div class="main-layout">
    <!-- 左侧固定导航栏 -->
    <Sidebar
      @open-login="showLogin = true"
      @open-user-profile="userStore.isLoggedIn ? showUserProfile = true : showLogin = true"
      @open-settings="showSettings = true"
      @tab-change="switchTab"
    />

    <!-- 笔记浏览 + 编辑主区域 -->
    <template v-if="activeView === 'notes'">
      <ListView />
      <Editor />
    </template>

    <!-- 工具页 -->
    <template v-else-if="activeView === 'tools'">
      <ToolsView />
    </template>

    <!-- 全局对话框 -->
    <LoginDialog v-if="showLogin" @close="showLogin = false" />
    <UserProfileModal v-if="showUserProfile && userStore.isLoggedIn" @close="showUserProfile = false" />
    <SettingsDialog v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<style>
/*
 * 主布局 — 横向三栏/四栏结构
 * 100vh 高度，flex 水平排列子组件
 */
.main-layout {
  display: flex;
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-primary);
}
</style>

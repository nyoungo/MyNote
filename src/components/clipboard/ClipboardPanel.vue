<template>
  <!--
    剪贴板面板 — 浮动窗口，支持键盘导航、搜索、置顶
    紧凑布局适应小窗口
  -->
  <div class="clipboard-panel" @keydown="handleKeydown">
    <!-- 标题栏（可拖动，Tauri 桌面端使用 data-tauri-drag-region） -->
    <div class="panel-header" data-tauri-drag-region>
      <span class="panel-title">
        <SnippetsOutlined style="font-size:14px" />
        剪贴板
      </span>
      <a-button type="text" size="small" @click="store.hidePanel()" title="关闭 (Esc)">
        <template #icon><CloseOutlined /></template>
      </a-button>
    </div>

    <!-- 搜索栏 -->
    <div class="panel-search">
      <a-input
        ref="searchInput"
        v-model:value="store.searchQuery"
        placeholder="搜索..."
        size="small"
        allow-clear
        @keydown.stop="handleSearchKeydown"
      >
        <template #prefix><SearchOutlined style="color:var(--text-tertiary);font-size:13px" /></template>
      </a-input>
    </div>

    <!-- 列表 -->
    <div class="items-list" ref="listRef">
      <!-- 空状态 -->
      <div v-if="store.filteredItems.length === 0" class="empty-state">
        <a-empty :image="AEmpty.PRESENTED_IMAGE_SIMPLE" description="暂无剪贴板历史">
          <template #description>
            <span style="color:var(--text-tertiary);">复制文本或图片后将自动记录在此</span>
          </template>
        </a-empty>
      </div>

      <!-- 条目列表 -->
      <div
        v-for="(item, index) in store.filteredItems"
        :key="item.id"
        class="item-card"
        :class="{ selected: selectedIndex === index }"
        @click="handlePaste(item)"
        @mouseenter="selectedIndex = index"
        @contextmenu.prevent="openContextMenu($event, item, index)"
      >
        <!-- 置顶按钮 -->
        <a-button
          type="text"
          size="small"
          class="pin-btn"
          :class="{ pinned: item.is_favorite }"
          @click.stop="handleToggleFavorite(item)"
          :title="item.is_favorite ? '取消置顶' : '置顶'"
        >
          <template #icon>
            <PushpinFilled v-if="item.is_favorite" style="color:var(--primary-color);font-size:12px" />
            <PushpinOutlined v-else style="font-size:12px" />
          </template>
        </a-button>

        <!-- 文本内容 -->
        <div v-if="item.content_type === 'text'" class="item-body">
          <div class="item-content">{{ item.content }}</div>
        </div>

        <!-- 图片内容 -->
        <div v-else-if="item.content_type.startsWith('image/')" class="item-body">
          <img
            :src="'data:' + item.content_type + ';base64,' + item.content"
            class="item-image"
            alt="剪贴板图片"
          />
        </div>

        <!-- 底部信息 -->
        <div class="item-meta">
          <span v-if="item.is_favorite" class="meta-pin">已置顶</span>
          <span class="meta-time">{{ formatTime(item.last_used_at) }}</span>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <a-dropdown
      :open="contextMenu.visible"
      trigger="contextMenu"
      @open-change="onDropdownVisibleChange"
    >
      <span style="position:fixed;left:0;top:0;width:0;height:0;"></span>
      <template #overlay>
        <a-menu @click="onContextAction">
          <a-menu-item key="favorite">
            <PushpinFilled v-if="contextMenu.item?.is_favorite" style="margin-right:8px;color:var(--primary-color)" />
            <PushpinOutlined v-else style="margin-right:8px" />
            {{ contextMenu.item?.is_favorite ? '取消置顶' : '置顶' }}
          </a-menu-item>
          <a-menu-divider />
          <a-menu-item key="delete" danger>
            <DeleteOutlined style="margin-right:8px" />
            删除
          </a-menu-item>
        </a-menu>
      </template>
    </a-dropdown>

    <!-- 底部栏 -->
    <div class="panel-footer">
      <span class="footer-count">{{ store.items.length }} 条记录</span>
      <a-button
        type="text"
        size="small"
        danger
        :disabled="store.items.length === 0"
        @click="handleClear"
      >
        <template #icon><DeleteOutlined /></template>
        清空
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, nextTick, onBeforeUnmount } from 'vue'
import { Empty as AEmpty } from 'ant-design-vue'
import {
  SearchOutlined,
  CloseOutlined,
  SnippetsOutlined,
  PushpinOutlined,
  PushpinFilled,
  DeleteOutlined,
} from '@ant-design/icons-vue'
import { useClipboardStore } from '@/stores/clipboardStore'
import type { ClipboardItem } from '@/stores/clipboardStore'

const store = useClipboardStore()
const searchInput = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLDivElement | null>(null)
const selectedIndex = ref(-1)

/** 右键菜单状态 */
const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  item: ClipboardItem | null
}>({
  visible: false,
  x: 0,
  y: 0,
  item: null,
})

function onDropdownVisibleChange(open: boolean) {
  if (!open) contextMenu.value.visible = false
}

/** 打开右键菜单 */
function openContextMenu(e: MouseEvent, item: ClipboardItem, index: number) {
  selectedIndex.value = index
  contextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    item,
  }
}

function onContextAction({ key }: { key: string }) {
  switch (key) {
    case 'favorite':
      const item = contextMenu.value.item
      if (item) store.toggleFavorite(item)
      break
    case 'delete':
      const delItem = contextMenu.value.item
      if (!delItem) return
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('delete_clipboard_item', { id: delItem.id })
        store.loadHistory()
      }).catch(console.error)
      break
  }
  contextMenu.value.visible = false
}

/** 面板显示时自动聚焦搜索框 */
onMounted(async () => {
  store.isPanelVisible = true
  setTimeout(() => (searchInput.value as any)?.focus?.(), 100)

  import('@tauri-apps/api/event').then(({ listen }) => {
    listen<null>('focus-search', async () => {
      await store.loadHistory()
      nextTick(() => { listRef.value?.scrollTo(0, 0) })
      setTimeout(() => (searchInput.value as any)?.focus?.(), 30)
    })
  }).catch(() => {})
})

async function handlePaste(item: ClipboardItem) {
  await store.pasteItem(item)
}

async function handleToggleFavorite(item: ClipboardItem) {
  await store.toggleFavorite(item)
}

function handleClear() {
  store.clearHistory()
}

/** 键盘导航 */
function handleKeydown(e: KeyboardEvent) {
  const items = store.filteredItems
  if (items.length === 0) return
  switch (e.key) {
    case 'Escape':
      e.preventDefault()
      store.hidePanel()
      break
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, items.length - 1)
      scrollToSelected()
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      scrollToSelected()
      break
    case 'Enter':
      e.preventDefault()
      if (selectedIndex.value >= 0 && selectedIndex.value < items.length) {
        handlePaste(items[selectedIndex.value])
      }
      break
  }
}

function scrollToSelected() {
  nextTick(() => {
    const container = listRef.value
    if (!container) return
    const selected = container.querySelector('.item-card.selected') as HTMLElement
    if (selected) {
      selected.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
    }
  })
}

function handleSearchKeydown(e: KeyboardEvent) {
  if (['ArrowDown', 'ArrowUp', 'Enter', 'Escape'].includes(e.key)) {
    handleKeydown(e)
  }
}

/** 相对时间格式化 */
function formatTime(iso: string): string {
  if (!iso) return ''
  try {
    const d = new Date(iso)
    const now = new Date()
    const min = Math.floor((now.getTime() - d.getTime()) / 60000)
    if (min < 1) return '刚刚'
    if (min < 60) return `${min} 分钟前`
    const h = Math.floor(min / 60)
    if (h < 24) return `${h} 小时前`
    return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
  } catch {
    return iso
  }
}
</script>

<style scoped>
.clipboard-panel {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
  overflow: hidden;
  user-select: none;
  outline: none;
  font-size: var(--font-size-md);
}

/* 标题栏 */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  height: 36px;
  padding: 0 var(--spacing-md);
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.panel-title {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  user-select: none;
}

.panel-title .anticon {
  color: var(--primary-color);
  font-size: 15px;
}

/* 搜索栏 */
.panel-search {
  flex-shrink: 0;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

/* 列表 */
.items-list {
  flex: 1;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-xl);
}

/* 条目卡片 */
.item-card {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 72px;
  padding: var(--spacing-xs) var(--spacing-md);
  cursor: pointer;
  transition: background var(--duration-fast);
  border-bottom: 1px solid var(--border-secondary);
  overflow: hidden;
}

.item-card:last-child {
  border-bottom: none;
}

.item-card:hover {
  background: var(--bg-hover);
}

.item-card.selected {
  background: var(--bg-active);
}

.item-card.selected:not(:hover) {
  border-left: 3px solid var(--primary-color);
  padding-left: calc(var(--spacing-md) - 3px);
}

/* 置顶按钮 */
.pin-btn {
  position: absolute;
  top: 2px;
  right: 2px;
  opacity: 0;
  z-index: 2;
  height: 20px;
  min-width: 20px;
}

.item-card:hover .pin-btn {
  opacity: 1;
}

.pin-btn.pinned {
  opacity: 1;
}

/* 文本内容 */
.item-body {
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.item-content {
  font-size: var(--font-size-sm);
  line-height: 1.5;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 图片内容 */
.item-image {
  height: 44px;
  max-width: 100%;
  object-fit: contain;
  display: block;
  border-radius: var(--radius-xs);
  background: var(--bg-secondary);
}

/* 底部元信息 */
.item-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--spacing-xs);
  margin-top: auto;
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
  padding-top: 2px;
}

.meta-pin {
  color: var(--primary-color);
  margin-right: auto;
  font-size: 10px;
  font-weight: 500;
}

/* 底部栏 */
.panel-footer {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-xs) var(--spacing-md);
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  font-size: 10px;
  height: 30px;
}

.footer-count {
  color: var(--text-tertiary);
}
</style>

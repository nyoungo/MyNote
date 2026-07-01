import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export interface ClipboardItem {
  id: string
  content_type: string
  content: string
  source_app?: string
  is_favorite: boolean
  created_at: string
  last_used_at: string
}

export const useClipboardStore = defineStore('clipboard', () => {
  const items = ref<ClipboardItem[]>([])
  const isPanelVisible = ref(false)
  const searchQuery = ref('')

  const filteredItems = computed(() => {
    if (!searchQuery.value) return items.value
    const q = searchQuery.value.toLowerCase()
    return items.value.filter(item =>
      item.content_type === 'text' && item.content.toLowerCase().includes(q)
    )
  })

  async function togglePanel() {
    if (!isTauri()) return
    const { invoke } = await import('@tauri-apps/api/core')

    if (isPanelVisible.value) {
      await invoke('hide_panel')
      isPanelVisible.value = false
    } else {
      await invoke('show_panel')
      isPanelVisible.value = true
      await loadHistory()
    }
  }

  /** 从 localStorage 读取设置的最大保存条数 */
  function getMaxItems(): number {
    try {
      const saved = JSON.parse(localStorage.getItem('mynote-settings') || '{}')
      return saved.clipboardMax || 100
    } catch { return 100 }
  }

  async function loadHistory() {
    if (!isTauri()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      items.value = await invoke<ClipboardItem[]>('get_clipboard_history', {
        maxItems: getMaxItems(),
      })
    } catch { /* */ }
  }

  async function pasteItem(item: ClipboardItem) {
    if (!isTauri()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('paste_item', {
        content: item.content,
        contentType: item.content_type,
      })
    } catch (e) {
      console.error('粘贴失败:', e)
    }
    isPanelVisible.value = false
  }

  async function hidePanel() {
    if (!isTauri()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('hide_panel')
    } catch { /* */ }
    isPanelVisible.value = false
  }

  async function clearHistory() {
    if (!isTauri()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('clear_clipboard_history')
    } catch { /* */ }
    items.value = []
  }

  /** 切换收藏（置顶） */
  async function toggleFavorite(item: ClipboardItem) {
    if (!isTauri()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const newVal = await invoke<boolean>('toggle_favorite', { id: item.id })
      item.is_favorite = newVal
      // 重新排序（收藏在前）
      await loadHistory()
    } catch (e) {
      console.error('切换收藏失败:', e)
    }
  }

  function setupListeners() {
    if (!isTauri()) return

    Promise.all([
      import('@tauri-apps/api/event'),
      import('@tauri-apps/api/webviewWindow'),
    ]).then(async ([{ listen }, { getCurrentWebviewWindow }]) => {
      const label = getCurrentWebviewWindow().label

      if (label === 'main') {
        listen<null>('toggle-clipboard', () => { togglePanel() })
      }

      listen<null>('clipboard-hide', () => {
        isPanelVisible.value = false
        if (label === 'main') {
          import('@tauri-apps/api/core').then(({ invoke }) => {
            invoke('hide_panel')
          })
        }
      })
    }).catch(() => { /* */ })
  }

  return {
    items, isPanelVisible, searchQuery, filteredItems,
    togglePanel, loadHistory, pasteItem, hidePanel, clearHistory,
    toggleFavorite, setupListeners,
  }
})

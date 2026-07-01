import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getNoteApi, updateNoteApi } from '@/utils/request'

/** 本地笔记 ID 前缀（离线创建） */
const LOCAL_ID_PREFIX = 'local_'
/** 本地内容缓存 key */
const CACHE_KEY = 'note-content-cache'

/**
 * 笔记编辑器状态管理
 *
 * 设计原则：
 * 1. 所有笔记内容优先存本地缓存（localStorage）
 * 2. selectNote 从缓存读取，服务器作为补充
 * 3. saveCurrentNote 先写缓存，再尝试同步服务器
 * 4. 切换笔记时无条件保存当前内容到缓存，不依赖 isDirty
 */
export const useEditorStore = defineStore('editor', () => {
  const selectedNodeId = ref<string | null>(null)
  const content = ref('')
  const isDirty = ref(false)
  const isSaving = ref(false)
  const mode = ref<'wysiwyg' | 'source'>('wysiwyg')

  const hasSelected = computed(() => selectedNodeId.value !== null)
  const isEmpty = computed(() => content.value === '')

  function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (!isTauri()) throw new Error('Not in Tauri environment')
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<T>(cmd, args)
  }

  // ===== 本地内容缓存（直接读写 localStorage）=====

  function getCachedContent(nodeId: string): string {
    try {
      const cache = JSON.parse(localStorage.getItem(CACHE_KEY) || '{}')
      return cache[nodeId] || ''
    } catch { return '' }
  }

  function setCachedContent(nodeId: string, text: string) {
    try {
      const cache = JSON.parse(localStorage.getItem(CACHE_KEY) || '{}')
      cache[nodeId] = text
      localStorage.setItem(CACHE_KEY, JSON.stringify(cache))
    } catch { /* ignore */ }
  }

  function removeCachedContent(nodeId: string) {
    try {
      const cache = JSON.parse(localStorage.getItem(CACHE_KEY) || '{}')
      delete cache[nodeId]
      localStorage.setItem(CACHE_KEY, JSON.stringify(cache))
    } catch { /* ignore */ }
  }

  function clearSelection() {
    selectedNodeId.value = null
    content.value = ''
    isDirty.value = false
  }

  /** 选中并加载一篇笔记 */
  async function selectNote(nodeId: string) {
    if (!nodeId) {
      clearSelection()
      return
    }

    // 切笔记：无条件保存当前内容到缓存（无论 isDirty / isSaving）
    if (selectedNodeId.value && selectedNodeId.value !== nodeId) {
      setCachedContent(selectedNodeId.value, content.value)
      isDirty.value = false
      // 后台尝试同步到服务器（不阻塞）
      syncToServer(selectedNodeId.value, content.value)
    }

    // 从本地缓存加载目标笔记
    const cached = getCachedContent(nodeId)
    content.value = cached || ''
    selectedNodeId.value = nodeId
    isDirty.value = false

    // 后台从服务器拉取最新内容（覆盖缓存）
    if (!nodeId.startsWith(LOCAL_ID_PREFIX)) {
      try {
        let noteContent = ''
        try {
          noteContent = await tauriInvoke<string>('load_note_content', { nodeId })
        } catch {
          const note = await getNoteApi(nodeId)
          noteContent = note.content || ''
        }
        if (noteContent) {
          content.value = noteContent
          setCachedContent(nodeId, noteContent)
        }
      } catch {
        console.warn('加载笔记失败，使用本地缓存:', nodeId)
      }
    }
  }

  /** 后台同步到服务器（不抛错、不阻塞 UI） */
  async function syncToServer(nodeId: string, text: string) {
    if (nodeId.startsWith(LOCAL_ID_PREFIX)) return
    try {
      await tauriInvoke('save_note_content', { nodeId, content: text })
    } catch {
      try {
        await updateNoteApi(nodeId, { content: text })
      } catch {
        // 离线，下次重试
      }
    }
  }

  /** 保存当前笔记 */
  async function saveCurrentNote() {
    if (!selectedNodeId.value) return
    // 先写本地缓存（永远成功）
    setCachedContent(selectedNodeId.value, content.value)
    isDirty.value = false
    // 后台同步服务器
    await syncToServer(selectedNodeId.value, content.value)
  }

  function updateContent(newContent: string) {
    content.value = newContent
    isDirty.value = true
  }

  function toggleMode() {
    mode.value = mode.value === 'wysiwyg' ? 'source' : 'wysiwyg'
  }

  return {
    selectedNodeId, content, isDirty, isSaving, mode,
    hasSelected, isEmpty,
    selectNote, clearSelection, saveCurrentNote, updateContent, toggleMode,
    getCachedContent, setCachedContent, removeCachedContent,
  }
}, {
  persist: {
    key: 'editor-store',
    storage: localStorage,
    paths: ['content', 'selectedNodeId', 'mode'],
  },
})

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getNotesApi, createNoteApi, updateNoteApi, deleteNoteApi } from '@/utils/request'
import { useEditorStore } from '@/stores/editorStore'

/** 本地笔记 ID 前缀（离线创建） */
const LOCAL_ID_PREFIX = 'local_'
/** 自动同步间隔：30 分钟 */
const SYNC_INTERVAL = 30 * 60 * 1000

/** 笔记/文件夹节点类型 */
export interface NoteNode {
  id: string
  user_id: string
  parent_id: string | null
  title: string
  content: string | null
  is_folder: number  // SQLite boolean: 0=笔记, 1=文件夹（API 返回整数）
  sort_order: number
  version: number
  created_at: string
  updated_at: string
  /** 前端使用的 children 缓存，服务端不返回 */
  children?: NoteNode[]
}

/**
 * 笔记与文件夹状态管理（树形结构）
 *
 * 设计原则：
 * 1. 本地优先 — 所有写操作立即更新本地状态
 * 2. 后台同步 — 每 30 分钟自动同步到服务器，或用户手动点击同步按钮
 * 3. 永不阻塞 — 网络失败不影响编辑使用
 */
export const useNoteStore = defineStore('notes', () => {
  const notes = ref<NoteNode[]>([])
  const selectedId = ref<string | null>(null)
  const offline = ref(false)
  const syncing = ref(false)
  const lastSyncTime = ref<number>(0)
  /** 有待同步变化的笔记 ID 集合 */
  const dirtyIds = ref<Set<string>>(new Set())
  let syncTimer: ReturnType<typeof setInterval> | null = null

  /** 按 parent_id 获取子节点 */
  function getChildren(parentId: string | null): NoteNode[] {
    return notes.value
      .filter(n => n.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order)
  }

  /** 获取完整的树形结构 */
  function getTree(parentId: string | null = null): NoteNode[] {
    return getChildren(parentId).map(node => ({
      ...node,
      children: node.is_folder ? getTree(node.id) : undefined,
    }))
  }

  /** 获取选中的笔记对象 */
  const selectedNote = computed(() =>
    notes.value.find(n => n.id === selectedId.value) ?? null
  )

  // ===== 同步机制 =====

  /** 初始化同步定时器（应用启动时调用一次） */
  function initSync() {
    if (syncTimer) clearInterval(syncTimer)
    syncTimer = setInterval(() => {
      if (dirtyIds.value.size > 0) syncNotes()
    }, SYNC_INTERVAL)
  }

  /** 停止同步定时器 */
  function stopSync() {
    if (syncTimer) {
      clearInterval(syncTimer)
      syncTimer = null
    }
  }

  /** 全量从服务器拉取列表（启动时 / 手动刷新） */
  async function fetchNotes() {
    try {
      const serverNotes = await getNotesApi()
      // 保留本地创建的笔记（local_ 前缀），合并到服务器数据
      const localNotes = notes.value.filter(n => n.id.startsWith(LOCAL_ID_PREFIX))
      notes.value = [...serverNotes, ...localNotes]
      offline.value = false
      lastSyncTime.value = Date.now()
    } catch (e) {
      console.warn('获取笔记列表失败，使用本地数据:', e)
      offline.value = true
      // 保留本地数据
    }
  }

  /** 后台同步：将本地脏笔记推送到服务器 */
  async function syncNotes() {
    if (syncing.value || dirtyIds.value.size === 0) return
    syncing.value = true
    const ids = [...dirtyIds.value]
    let hasError = false

    for (const id of ids) {
      const note = notes.value.find(n => n.id === id)
      if (!note) { dirtyIds.value.delete(id); continue }

      // 跳过本地笔记
      if (id.startsWith(LOCAL_ID_PREFIX)) {
        dirtyIds.value.delete(id)
        continue
      }

      // 尝试推送到服务器
      try {
        await updateNoteApi(id, {
          title: note.title,
          parent_id: note.parent_id,
          sort_order: note.sort_order,
        })
        dirtyIds.value.delete(id)
      } catch (e) {
        hasError = true
        console.warn('同步笔记失败，稍后重试:', id, e)
      }
    }

    if (!hasError) {
      offline.value = false
      lastSyncTime.value = Date.now()
    }
    syncing.value = false
  }

  /** 手动云同步（由按钮触发）：先推送本地变更，再拉取最新数据 */
  async function manualSync() {
    if (dirtyIds.value.size > 0) await syncNotes()
    await fetchNotes()
  }

  // ===== 标记脏笔记 =====

  function markDirty(id: string) {
    if (!id.startsWith(LOCAL_ID_PREFIX)) {
      dirtyIds.value = new Set([...dirtyIds.value, id])
    }
  }

  // ===== CRUD（本地优先）=====

  /** 创建笔记或文件夹 */
  async function createNote(data: {
    title: string
    parent_id?: string | null
    is_folder?: boolean
    content?: string
  }) {
    try {
      const note = await createNoteApi({
        title: data.title,
        parent_id: data.parent_id ?? null,
        is_folder: !!data.is_folder,
        content: data.content,
      })
      notes.value.push(note)
      offline.value = false
      if (!data.is_folder) selectedId.value = note.id
      return note
    } catch (e) {
      console.warn('创建笔记失败，使用临时数据:', e)
      offline.value = true
      const tempNote: NoteNode = {
        id: `local_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
        user_id: '',
        parent_id: data.parent_id ?? null,
        title: data.title,
        content: data.content ?? '',
        is_folder: data.is_folder ? 1 : 0,
        sort_order: notes.value.filter(n => n.parent_id === data.parent_id).length,
        version: 1,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      notes.value.push(tempNote)
      if (!data.is_folder) selectedId.value = tempNote.id
      return tempNote
    }
  }

  /** 更新笔记（本地优先） */
  async function updateNote(id: string, data: Partial<NoteNode>) {
    // 先更新本地
    const idx = notes.value.findIndex(n => n.id === id)
    if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...data }

    // 标记脏笔记
    markDirty(id)

    // 后台尝试同步到服务器（不 await，不阻塞）
    try {
      await updateNoteApi(id, data)
      offline.value = false
    } catch (e) {
      console.warn('更新笔记失败，本地已保存稍后同步:', e)
      offline.value = true
    }
  }

  /** 删除笔记或文件夹 */
  async function deleteNote(id: string) {
    // 先本地删除
    notes.value = notes.value.filter(n => n.id !== id)
    dirtyIds.value.delete(id)
    // 同时清理内容缓存
    const editorStore = useEditorStore()
    editorStore.removeCachedContent(id)

    if (selectedId.value === id) {
      selectedId.value = null
      editorStore.clearSelection()
    }

    // 后台尝试同步到服务器
    try {
      await deleteNoteApi(id)
      offline.value = false
    } catch (e) {
      console.warn('删除笔记失败，本地已删除稍后同步:', e)
      offline.value = true
    }
  }

  /** 移动节点（拖拽排序） */
  async function moveNode(nodeId: string, newParentId: string | null, newSortOrder: number) {
    await updateNote(nodeId, { parent_id: newParentId, sort_order: newSortOrder } as any)
  }

  return {
    notes, selectedId, selectedNote, offline, syncing, lastSyncTime, dirtyIds,
    getChildren, getTree,
    fetchNotes, syncNotes, manualSync, initSync, stopSync,
    createNote, updateNote, deleteNote, moveNode,
  }
}, {
  persist: {
    key: 'note-store',
    storage: localStorage,
    paths: ['notes', 'selectedId'],
  },
})

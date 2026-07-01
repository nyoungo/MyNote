<template>
  <!--
    富文本编辑器组件 — 基于 wangEditor 5
    - 支持标题编辑、自动保存、手动保存
    - 粘贴图片自动转 base64
    - Ctrl+S 快捷保存
  -->
  <section class="editor-panel">
    <!-- 未选中笔记时的占位符 -->
    <div v-if="!editorStore.hasSelected && !isLoading" class="editor-placeholder">
      <div class="placeholder-icon">
        <FileEditIcon :size="56" />
      </div>
      <p class="placeholder-title">选择一篇笔记开始编辑</p>
      <p class="placeholder-hint">从左侧列表中选择笔记，或创建一篇新笔记</p>
    </div>

    <!-- 加载中状态 -->
    <div v-else-if="isLoading" class="editor-placeholder">
      <div class="status-spinner" />
      <p class="placeholder-title">加载中...</p>
    </div>

    <!-- 编辑器主体 -->
    <template v-else>
      <!-- 顶部标题栏 -->
      <div class="editor-topbar">
        <input
          class="title-input"
          :value="titleText"
          @input="handleTitleInput"
          @blur="handleTitleSave"
          @keydown.enter="handleTitleSave"
          placeholder="无标题"
        />
        <div class="topbar-right">
          <span
            class="save-status"
            :class="saveStatusClass"
          >
            <SaveIcon v-if="editorStore.isSaving" :size="12" />
            {{ saveStatusText }}
          </span>
          <button
            class="tb-btn save-btn"
            :disabled="editorStore.isSaving || !editorStore.isDirty"
            @click="handleSave"
          >
            <SaveIcon :size="14" />
            <span>保存</span>
          </button>
        </div>
      </div>

      <!-- wangEditor 工具栏容器 -->
      <div ref="toolbarRef" class="w-editor-toolbar"></div>
      <!-- wangEditor 编辑器内容区 -->
      <div ref="editorRef" class="w-editor-content"></div>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Editor 组件 — 基于 wangEditor 5
 *
 * 设计要点：
 * 1. 使用 watch + nextTick 统一初始化编辑器，避免 onMounted 和 watch 重复调用
 * 2. 切换笔记时自动先保存当前内容，然后清除定时器再加载新内容
 * 3. 加载状态 isLoading 防止界面闪烁
 * 4. 自动保存定时器在组件卸载和笔记切换时正确清理
 */
import { ref, computed, watch, onBeforeUnmount, onMounted, nextTick } from 'vue'
import { Save as SaveIcon, FileEdit as FileEditIcon } from 'lucide-vue-next'
import { Boot, createEditor, createToolbar, type IDomEditor, type IEditorConfig, type IToolbarConfig } from '@wangeditor/editor'
import '@wangeditor/editor/dist/css/style.css'
import pluginMd from '@wangeditor/plugin-md'
import { useEditorStore } from '@/stores/editorStore'
import { useNoteStore } from '@/stores/noteStore'

// 注册 Markdown 快捷键插件（全局注册，只需一次）
Boot.registerModule(pluginMd)

const editorStore = useEditorStore()
const noteStore = useNoteStore()

const editorRef = ref<HTMLDivElement | null>(null)
const toolbarRef = ref<HTMLDivElement | null>(null)
let editorInstance: IDomEditor | null = null
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null
const isLoading = ref(false)

// ===== 标题编辑 =====

const editingTitle = ref('')

const titleText = computed(() => editingTitle.value || noteStore.selectedNote?.title || '')

function handleTitleInput(e: Event) {
  editingTitle.value = (e.target as HTMLInputElement).value
}

/** 保存标题（失焦或回车触发） */
async function handleTitleSave() {
  const note = noteStore.selectedNote
  if (!note) { editingTitle.value = ''; return }
  const t = editingTitle.value.trim()
  if (t && t !== note.title) {
    await noteStore.updateNote(note.id, { title: t } as any)
  }
  editingTitle.value = ''
}

// ===== wangEditor 生命周期管理 =====

/** 销毁编辑器实例 */
function destroyEditor() {
  if (editorInstance) {
    try {
      editorInstance.destroy()
    } catch (e) {
      console.warn('销毁编辑器实例失败:', e)
    }
    editorInstance = null
  }
}

/** 初始化 wangEditor */
async function initEditor() {
  destroyEditor()
  await nextTick()

  if (!editorRef.value || !toolbarRef.value) return
  if (!editorStore.selectedNodeId) return

  // 等待编辑器内容加载完成
  isLoading.value = true
  try {
    // 如果还没有内容，触发加载
    if (!editorStore.content && editorStore.selectedNodeId) {
      await editorStore.selectNote(editorStore.selectedNodeId)
    }
  } catch (e) {
    console.warn('加载笔记内容失败:', e)
  } finally {
    isLoading.value = false
  }

  await nextTick()
  if (!editorRef.value) return

  const toolbarConfig: Partial<IToolbarConfig> = {
    excludeKeys: ['fullScreen', 'insertVideo'],
  }

  const editorConfig: Partial<IEditorConfig> = {
    placeholder: '请输入内容...',
    onChange: () => {
      if (editorInstance) {
        editorStore.updateContent(editorInstance.getHtml())
      }
    },
    // 粘贴图片 → 转 base64 插入（避免依赖外部图床）
    // async 等待 FileReader 完成后才返回，避免 wangEditor 在异步回调前处理粘贴
    customPaste: (async (editor: IDomEditor, e: ClipboardEvent) => {
      const items = e.clipboardData?.items
      if (!items) return true

      for (const item of items) {
        if (item.type.startsWith('image/')) {
          e.preventDefault()
          const file = item.getAsFile()
          if (!file) continue
          const dataUrl = await new Promise<string>((resolve, reject) => {
            const reader = new FileReader()
            reader.onload = () => resolve(reader.result as string)
            reader.onerror = () => reject(reader.error)
            reader.readAsDataURL(file!)
          })
          editor.restoreSelection()
          editor.dangerouslyInsertHtml(`<img src="${dataUrl}" />`)
          return false
        }
      }
      return true
    }) as unknown as (editor: IDomEditor, e: ClipboardEvent) => boolean,
    MENU_CONF: {
      uploadImage: {
        async customUpload(file: File, insertFn: (url: string) => void) {
          const dataUrl = await new Promise<string>((resolve, reject) => {
            const reader = new FileReader()
            reader.onload = () => resolve(reader.result as string)
            reader.onerror = () => reject(reader.error)
            reader.readAsDataURL(file)
          })
          insertFn(dataUrl)
        },
      },
    },
  }

  try {
    const html = editorStore.content || ''

    editorInstance = createEditor({
      selector: editorRef.value,
      config: editorConfig,
      html,
    })

    createToolbar({
      editor: editorInstance,
      selector: toolbarRef.value,
      config: toolbarConfig,
    })
  } catch (e) {
    console.warn('编辑器初始化失败:', e)
  }
}

// ===== 使用 watch 统一管理编辑器初始化 =====
// 核心逻辑：每当 selectedNodeId 变化时，重新初始化编辑器
// 使用 immediate: false（默认），避免在组件挂载时和 onMounted 冲突
watch(
  () => editorStore.selectedNodeId,
  async (newId, oldId) => {
    // 保存当前笔记的未保存修改
    if (oldId && editorStore.isDirty) {
      await editorStore.saveCurrentNote()
    }
    // 清理旧的自动保存定时器
    if (autoSaveTimer) {
      clearTimeout(autoSaveTimer)
      autoSaveTimer = null
    }
    // 初始化新笔记的编辑器
    await initEditor()
  },
)

// ===== 键盘快捷键：Ctrl+S / Cmd+S =====
function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    handleSave()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  // 如果已有选中的笔记，初始化编辑器
  if (editorStore.selectedNodeId) {
    initEditor()
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  destroyEditor()
  if (autoSaveTimer) {
    clearTimeout(autoSaveTimer)
    autoSaveTimer = null
  }
})

// ===== 自动保存 =====

/** 监听内容变更，调度自动保存 */
watch(() => editorStore.isDirty, (dirty) => {
  if (dirty) scheduleAutoSave()
})

/** 从设置中读取自动保存间隔（秒） */
function getAutoSaveInterval(): number {
  try {
    const saved = JSON.parse(localStorage.getItem('mynote-settings') || '{}')
    return (saved.autoSaveInterval || 30) * 1000
  } catch {
    return 30000
  }
}

/** 调度自动保存定时器（每次调用会重置计时） */
function scheduleAutoSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(async () => {
    await editorStore.saveCurrentNote()
  }, getAutoSaveInterval())
}

/** 手动保存 */
async function handleSave() {
  if (autoSaveTimer) {
    clearTimeout(autoSaveTimer)
    autoSaveTimer = null
  }
  await editorStore.saveCurrentNote()
}

// ===== 保存状态显示 =====

const saveStatusText = computed(() => {
  if (editorStore.isSaving) return '保存中...'
  if (editorStore.isDirty) return '未保存'
  return '已保存'
})

const saveStatusClass = computed(() => {
  if (editorStore.isSaving) return 'status-saving'
  if (editorStore.isDirty) return 'status-dirty'
  return 'status-saved'
})
</script>

<style scoped>
/*
 * 编辑器面板 — 右侧主编辑区域
 * 有道云笔记风格：干净、舒适、高效
 */
.editor-panel {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  min-width: 0;
}

/* ===== 占位符 / 空状态 / 加载中 ===== */

.editor-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  user-select: none;
  gap: var(--spacing-sm);
}

.placeholder-icon {
  margin-bottom: var(--spacing-sm);
  opacity: 0.2;
}

.placeholder-title {
  font-size: var(--font-size-lg);
  font-weight: 500;
  color: var(--text-secondary);
}

.placeholder-hint {
  font-size: var(--font-size-md);
  color: var(--text-tertiary);
}

.status-spinner {
  width: 28px;
  height: 28px;
  border: 2.5px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  margin-bottom: var(--spacing-md);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ===== 顶部标题栏 ===== */

.editor-topbar {
  display: flex;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-xl);
  border-bottom: 1px solid var(--border-secondary);
  background: var(--bg-primary);
  flex-shrink: 0;
  gap: var(--spacing-sm);
  min-height: 44px;
}

.title-input {
  flex: 1;
  font-size: var(--font-size-xl);
  font-weight: 600;
  color: var(--text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  transition: background var(--duration-fast);
  min-width: 0;
}

.title-input:hover {
  background: var(--bg-hover);
}

.title-input:focus {
  background: var(--bg-secondary);
}

.title-input::placeholder {
  color: var(--text-tertiary);
  font-weight: 400;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-shrink: 0;
}

/* ---- 保存状态标签 ---- */
.save-status {
  font-size: var(--font-size-xs);
  padding: 2px var(--spacing-sm);
  border-radius: var(--radius-sm);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.status-saving {
  color: var(--warning-color);
}

.status-dirty {
  color: var(--text-tertiary);
}

.status-saved {
  color: var(--success-color);
}

/* ---- 工具栏按钮 ---- */
.tb-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 14px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
  border: 1px solid transparent;
  font-size: var(--font-size-md);
  transition: all var(--duration-fast);
}

.tb-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tb-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.save-btn {
  color: var(--primary-color);
  font-weight: 500;
  border-color: var(--primary-border);
}

.save-btn:hover:not(:disabled) {
  background: var(--primary-bg);
  color: var(--primary-color);
  border-color: var(--primary-color);
}

/* ===== wangEditor 工具栏覆盖（有道云风格） ===== */

.w-editor-toolbar {
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-secondary);
  background: var(--bg-secondary);
  padding: 0;
}

.w-editor-toolbar :deep(.w-e-toolbar) {
  background: var(--bg-secondary) !important;
  border: none !important;
  flex-wrap: wrap;
  padding: 3px var(--spacing-sm);
  gap: 0;
}

.w-editor-toolbar :deep(.w-e-bar-item) {
  margin: 0;
}

.w-editor-toolbar :deep(.w-e-bar-item button) {
  color: var(--text-tertiary) !important;
  border-radius: var(--radius-xs);
  transition: all var(--duration-fast);
  height: 30px;
  min-width: 30px;
  padding: 0 5px;
}

.w-editor-toolbar :deep(.w-e-bar-item button:hover) {
  background: var(--bg-hover) !important;
  color: var(--text-primary) !important;
}

.w-editor-toolbar :deep(.w-e-bar-item button .w-e-icon) {
  font-size: 14px !important;
}

.w-editor-toolbar :deep(.w-e-bar-divider) {
  background: var(--border-color) !important;
  margin: 0 3px;
  height: 20px;
}

/* ===== wangEditor 内容区覆盖 ===== */

.w-editor-content {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-primary);
}

.w-editor-content :deep(.w-e-text-container) {
  background: var(--bg-primary) !important;
  border: none !important;
}

.w-editor-content :deep(.w-e-text) {
  min-height: 100% !important;
  padding: var(--spacing-xl) var(--spacing-xxl) var(--spacing-xxxl) !important;
  color: var(--text-primary) !important;
}

/* 确保编辑器中所有文本元素都使用正确的颜色 */
.w-editor-content :deep(.w-e-text p),
.w-editor-content :deep(.w-e-text h1),
.w-editor-content :deep(.w-e-text h2),
.w-editor-content :deep(.w-e-text h3),
.w-editor-content :deep(.w-e-text h4),
.w-editor-content :deep(.w-e-text h5),
.w-editor-content :deep(.w-e-text span),
.w-editor-content :deep(.w-e-text div),
.w-editor-content :deep(.w-e-text blockquote),
.w-editor-content :deep(.w-e-text pre),
.w-editor-content :deep(.w-e-text code),
.w-editor-content :deep(.w-e-text li) {
  color: var(--text-primary) !important;
}

.w-editor-content :deep(.w-e-text h1) {
  font-size: 1.8em !important;
  font-weight: 600 !important;
  margin: 0.6em 0 0.3em !important;
}

.w-editor-content :deep(.w-e-text h2) {
  font-size: 1.5em !important;
  font-weight: 600 !important;
  margin: 0.5em 0 0.25em !important;
}

.w-editor-content :deep(.w-e-text h3) {
  font-size: 1.25em !important;
  font-weight: 600 !important;
  margin: 0.4em 0 0.2em !important;
}

/* 暗黑模式下的 wangEditor 滚动条 */
.w-editor-content :deep(.w-e-text-container ::-webkit-scrollbar-thumb) {
  background: var(--text-quaternary);
}
</style>

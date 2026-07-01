<template>
  <!--
    笔记列表组件 — 基于 ant-design-vue a-tree
    树形笔记/文件夹浏览器，支持搜索、右键菜单、拖拽排序
  -->
  <section class="list-view">
    <!-- 顶部新建按钮组 + 云同步 -->
    <div class="list-header">
      <a-button type="primary" style="width: calc(100% - 30px)" @click="handleAddNote">
        <template #icon><PlusOutlined /></template>
        新建笔记
      </a-button>
      <a-button  @click="handleAddFolder" title="新建文件夹">
        <template #icon><FolderAddOutlined /></template>
      </a-button>
      <a-button
        @click="noteStore.manualSync()"
        :loading="noteStore.syncing"
        title="云同步"
      >
        <template #icon><SyncOutlined /></template>
      </a-button>
    </div>

    <!-- 搜索框 -->
    <div class="search-box">
      <a-input-search
        v-model:value="searchQuery"
        placeholder="搜索笔记..."
        allow-clear
        @change="onSearchChange"
      />
    </div>

    <!-- 树形列表区域 -->
    <div class="tree-wrapper">
      <!-- 空状态 -->
      <div v-if="!treeData.length" class="tree-status">
        <a-empty
          :image="AEmpty.PRESENTED_IMAGE_SIMPLE"
          :description="searchQuery ? '未找到匹配的笔记' : '暂无笔记，点击上方按钮创建'"
        />
      </div>

      <!-- ant-design-vue 树形控件（参照官方拖拽示例） -->
      <a-tree
        v-else
        v-model:expandedKeys="expandedKeys"
        v-model:selectedKeys="selectedKeys"
        :tree-data="treeData"
        draggable
        :block-node="true"
        :show-icon="true"
        :virtual="false"
        class="note-tree"
        @select="onSelect"
        @dragenter="onDragEnter"
        @drop="onDrop"
        @rightClick="onRightClick"
      />
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <Transition name="fade">
        <div
          v-if="contextMenu.visible"
          class="context-menu-overlay"
          @mousedown="closeContextMenu"
        >
          <div
            class="context-menu"
            :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
            @mousedown.stop
          >
            <a-menu :selectable="false" @click="onContextAction">
              <a-menu-item key="pin">
                <PushpinOutlined style="margin-right:8px" />
                置顶
              </a-menu-item>
              <a-menu-item key="rename">
                <EditOutlined style="margin-right:8px" />
                重命名
              </a-menu-item>
              <a-menu-divider />
              <a-menu-item key="delete" danger>
                <DeleteOutlined style="margin-right:8px" />
                删除
              </a-menu-item>
            </a-menu>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 重命名弹窗 -->
    <a-modal
      v-model:open="renameState.visible"
      title="重命名"
      cancel-text="取消"
      ok-text="确定"
      :confirm-loading="renameState.loading"
      @ok="handleRenameOk"
      @cancel="renameState.visible = false"
    >
      <a-input
        v-model:value="renameState.name"
        placeholder="输入新名称"
        @press-enter="handleRenameOk"
      />
    </a-modal>

    <!-- 新建笔记弹窗 -->
    <a-modal
      v-model:open="addNoteState.visible"
      title="新建笔记"
      :confirm-loading="addNoteState.loading"
      @ok="handleAddNoteOk"
      cancel-text="取消"
      ok-text="确定"
      @cancel="addNoteState.visible = false"
    >
      <a-input
        v-model:value="addNoteState.name"
        placeholder="输入笔记标题"
        @press-enter="handleAddNoteOk"
      />
    </a-modal>

    <!-- 新建文件夹弹窗 -->
    <a-modal
      v-model:open="addFolderState.visible"
      title="新建文件夹"
      :confirm-loading="addFolderState.loading"
      @ok="handleAddFolderOk"
      @cancel="addFolderState.visible = false"
    >
      <a-input
        v-model:value="addFolderState.name"
        placeholder="输入文件夹名称"
        @press-enter="handleAddFolderOk"
      />
    </a-modal>
  </section>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, h } from 'vue'
import { Empty as AEmpty } from 'ant-design-vue'
import { message, Modal } from 'ant-design-vue'
import type { AntTreeNodeDropEvent, AntTreeNodeDragEnterEvent } from 'ant-design-vue/es/tree'
import {
  PlusOutlined,
  FolderAddOutlined,
  FileTextOutlined,
  FolderOutlined,
  PushpinOutlined,
  EditOutlined,
  DownloadOutlined,
  DeleteOutlined,
  SyncOutlined,
} from '@ant-design/icons-vue'
import { useNoteStore } from '@/stores/noteStore'
import { useEditorStore } from '@/stores/editorStore'

const noteStore = useNoteStore()
const editorStore = useEditorStore()

const searchQuery = ref('')
const expandedKeys = ref<string[]>([])
const selectedKeys = ref<string[]>([])
/** 搜索前的展开状态快照，清空搜索后恢复 */
const savedExpandedKeys = ref<string[]>([])

/** 右键菜单状态 */
const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  nodeKey: string
  type: 'file' | 'folder'
  title: string
}>({
  visible: false,
  x: 0, y: 0,
  nodeKey: '',
  type: 'file',
  title: '',
})

/** 重命名弹窗状态 */
const renameState = reactive({
  visible: false,
  loading: false,
  name: '',
  nodeKey: '',
})

/** 新建笔记弹窗状态 */
const addNoteState = reactive({
  visible: false,
  loading: false,
  name: '',
})

/** 新建文件夹弹窗状态 */
const addFolderState = reactive({
  visible: false,
  loading: false,
  name: '',
})

onMounted(() => {
  // 后台尝试从服务器拉取最新数据（不阻塞 UI，失败保留本地数据）
  noteStore.fetchNotes()
  // 启动 30 分钟自动同步定时器
  noteStore.initSync()
})

onBeforeUnmount(() => {
  noteStore.stopSync()
})

function closeContextMenu() {
  contextMenu.value.visible = false
}

// ===== 搜索过滤 =====

/** 搜索过滤后的笔记列表（保持完整树结构） */
const filteredNotes = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return noteStore.notes

  const matched = new Set<string>()
  for (const n of noteStore.notes) {
    if (n.title.toLowerCase().includes(q)) {
      matched.add(n.id)
      let pid = n.parent_id
      while (pid) {
        matched.add(pid)
        const parent = noteStore.notes.find(p => p.id === pid)
        pid = parent?.parent_id ?? null
      }
    }
  }
  return noteStore.notes.filter(n => matched.has(n.id))
})

// ===== ant-design-vue a-tree 数据结构 =====

/** 构建 a-tree 兼容的树形数据 */
const treeData = computed(() => {
  const nodes = filteredNotes.value
  function buildChildren(parentId: string | null): any[] {
    return nodes
      .filter(n => n.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order)
      .map(n => ({
        key: n.id,
        title: n.title || '未命名',
        isFolder: !!n.is_folder,
        isLeaf: !n.is_folder,
        icon: n.is_folder
          ? () => h(FolderOutlined, { style: 'color:var(--warning-color);font-size:15px;line-height:1' })
          : () => h(FileTextOutlined, { style: 'color:var(--text-secondary);font-size:15px;line-height:1' }),
        children: n.is_folder ? buildChildren(n.id) : undefined,
      }))
  }
  return buildChildren(null)
})

// ===== 搜索对展开状态的影响 =====

function onSearchChange() {
  const val = searchQuery.value

  // 第一次搜索时保存现有展开状态
  if (val && savedExpandedKeys.value.length === 0) {
    savedExpandedKeys.value = [...expandedKeys.value]
  }

  // 清空搜索时恢复
  if (!val) {
    expandedKeys.value = [...savedExpandedKeys.value]
    savedExpandedKeys.value = []
    return
  }

  // 搜索时展开所有匹配项的祖先文件夹
  const newExpanded = new Set(expandedKeys.value)
  for (const n of noteStore.notes) {
    if (n.title.toLowerCase().includes(val.trim().toLowerCase())) {
      let pid = n.parent_id
      while (pid) {
        newExpanded.add(pid)
        const parent = noteStore.notes.find(p => p.id === pid)
        pid = parent?.parent_id ?? null
      }
    }
  }
  expandedKeys.value = [...newExpanded]
}

// ===== 树节点事件 =====

function onSelect(keys: string[]) {
  if (!keys || keys.length === 0) return
  const key = keys[0]

  noteStore.selectedId = key

  // 通过 treeData 查找节点类型（不依赖 info.node，直接从数据层找）
  const found = findNodeByKey(treeData.value, key)
  if (found && !found.isFolder) {
    editorStore.selectNote(key)
  }
}

/** 在树中递归查找节点 */
function findNodeByKey(nodes: any[], key: string): any | null {
  for (const n of nodes) {
    if (n.key === key) return n
    if (n.children) {
      const found = findNodeByKey(n.children, key)
      if (found) return found
    }
  }
  return null
}

/** 在树中递归查找节点的父 key */
function findParentKeyInTree(nodes: any[], targetKey: string): string | null {
  for (const n of nodes) {
    if (n.children) {
      if (n.children.some((c: any) => c.key === targetKey)) return n.key
      const found = findParentKeyInTree(n.children, targetKey)
      if (found) return found
    }
  }
  return null
}

// ===== 拖拽处理（参照官方示例）=====

/** 拖拽进入——自动展开文件夹 */
function onDragEnter(info: AntTreeNodeDragEnterEvent) {
  // 当拖拽悬停在文件夹上时自动展开，方便拖入
  const key = info.node.key as string
  if (key && findNodeByKey(treeData.value, key)?.isFolder) {
    if (!expandedKeys.value.includes(key)) {
      expandedKeys.value = [...expandedKeys.value, key]
    }
  }
}

async function onDrop(info: AntTreeNodeDropEvent) {
  const dropKey = info.node.key as string
  const dragKey = info.dragNode.key as string

  if (!dragKey || !dropKey || dragKey === dropKey) return

  // 阻止将文件夹拖入自身或其子节点
  if (isDescendant(treeData.value, dropKey, dragKey)) {
    message.warning('不能将文件夹移动到自身内部')
    return
  }

  // 计算相对位置: -1 之前, 0 上面, 1 之后
  const dropPos = String(info.node.pos).split('-')
  const nodeIndex = Number(dropPos[dropPos.length - 1])
  const relativePos = info.dropPosition - nodeIndex

  const targetNode = findNodeByKey(treeData.value, dropKey)

  // 确定新父级
  // 注意: rc-tree 的 calcDropPosition 对空文件夹/未展开文件夹
  // 的中部区域也返回 dropPosition=1 而非 0，导致 dropToGap 恒为 true。
  // 所以对文件夹用 relativePos !== -1 覆盖"非顶部间隙"=拖入文件夹。
  let newParentId: string | null
  if (targetNode?.isFolder && relativePos !== -1) {
    // 拖到文件夹中间或底部 → 作为子节点
    newParentId = dropKey
  } else {
    // 拖到顶部间隙 或 拖到笔记上 → 同级插入
    newParentId = findParentKeyInTree(treeData.value, dropKey)
  }

  try {
    // 获取同级所有兄弟节点（排除被拖动的节点）
    const siblings = noteStore.notes
      .filter(n => n.parent_id === newParentId && n.id !== dragKey)
      .sort((a, b) => a.sort_order - b.sort_order)

    // 计算插入位置
    let insertIdx: number
    if (targetNode?.isFolder && relativePos !== -1) {
      // 拖入文件夹 → 追加到最后
      insertIdx = siblings.length
    } else if (relativePos === -1) {
      // 插入到目标节点之前
      insertIdx = siblings.findIndex(n => n.id === dropKey)
    } else {
      // 插入到目标节点之后 (relativePos === 0 或 1)
      insertIdx = siblings.findIndex(n => n.id === dropKey) + 1
    }
    if (insertIdx < 0) insertIdx = siblings.length

    // 构建新的排序顺序
    const sortedKeys = siblings.map(n => n.id)
    sortedKeys.splice(insertIdx, 0, dragKey)

    // 重新分配每个节点的 sort_order（已自动本地更新+后台同步）
    for (let i = 0; i < sortedKeys.length; i++) {
      await noteStore.updateNote(sortedKeys[i], {
        sort_order: i,
        ...(sortedKeys[i] === dragKey ? { parent_id: newParentId } : {}),
      } as any)
    }
  } catch (err) {
    console.error('拖拽移动失败:', err)
    message.error('移动失败，请重试')
  }
}

/** 检查 target 是否是 dragNode 的后代（防止循环） */
function isDescendant(nodes: any[], targetId: string, dragId: string): boolean {
  const dragNode = findNodeByKey(nodes, dragId)
  if (!dragNode?.children) return false
  for (const child of dragNode.children) {
    if (child.key === targetId) return true
    if (isDescendant(child.children || [], targetId, dragId)) return true
  }
  return false
}

// ===== 右键菜单 =====

function onRightClick(info: { event: MouseEvent; node: any }) {
  const node = info.node
  const dataRef = node.dataRef || node
  const key = dataRef.key || node.key
  if (!key) return

  contextMenu.value = {
    visible: true,
    x: info.event.clientX,
    y: info.event.clientY,
    nodeKey: key,
    type: dataRef.isFolder ? 'folder' : 'file',
    title: dataRef.title || '',
  }
}

function onContextAction({ key }: { key: string }) {
  switch (key) {
    case 'pin': doPin(); break
    case 'rename': doRename(); break
    case 'export': doExport(); break
    case 'delete': doDelete(); break
  }
  closeContextMenu()
}

/** 置顶 */
async function doPin() {
  const id = contextMenu.value.nodeKey
  if (!id) return
  await noteStore.updateNote(id, { sort_order: -1 } as any)
}

/** 重命名 — 打开弹窗 */
function doRename() {
  const node = contextMenu.value
  if (!node.nodeKey) return
  renameState.name = node.title
  renameState.nodeKey = node.nodeKey
  renameState.visible = true
}

/** 重命名 — 确认提交 */
async function handleRenameOk() {
  const name = renameState.name?.trim()
  if (!name) {
    message.warning('名称不能为空')
    return
  }
  if (name === contextMenu.value.title) {
    renameState.visible = false
    return
  }
  renameState.loading = true
  try {
    await noteStore.updateNote(renameState.nodeKey, { title: name } as any)
    renameState.visible = false
  } catch {
    message.error('重命名失败')
  } finally {
    renameState.loading = false
  }
}

/** 删除 — 使用 Ant Design 确认弹窗 */
function doDelete() {
  const id = contextMenu.value.nodeKey
  const title = contextMenu.value.title
  if (!id) return

  Modal.confirm({
    title: `确定删除「${title}」？`,
    content: '删除后不可恢复，笔记内容将永久丢失。',
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      await noteStore.deleteNote(id)
      message.success('已删除')
    },
  })
}

/** 导出为 PDF 文件 */
async function doExport() {
  const node = contextMenu.value
  if (!node.nodeKey || node.type !== 'file') return
  try {
    // 获取笔记内容
    let content = ''
    if (editorStore.selectedNodeId === node.nodeKey) {
      content = editorStore.content
    }
    if (!content) {
      const { getNoteApi } = await import('@/utils/request')
      const note = await getNoteApi(node.nodeKey)
      content = note.content || ''
    }

    // Markdown → HTML
    const { marked } = await import('marked')
    const html = await marked.parse(content)

    // 在新窗口打开打印预览 → 用户选择"另存为 PDF"
    const win = window.open('', '_blank')
    if (!win) {
      message.error('请允许弹出窗口')
      return
    }
    win.document.write(`
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="utf-8">
        <title>${node.title || '未命名'}</title>
        <style>
          @page { margin: 20mm; }
          * { box-sizing: border-box; }
          body {
            padding: 0;
            font: 14px/1.8 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            color: #333;
          }
          img { max-width: 100%; }
          pre { background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; }
          code { font-size: 13px; }
          table { border-collapse: collapse; width: 100%; }
          th, td { border: 1px solid #ddd; padding: 8px; }
        </style>
      </head>
      <body>${html}</body>
      </html>
    `)
    win.document.close()
    // 等渲染完成后触发打印
    win.onload = () => { win.print(); win.close() }
    // 部分浏览器 onload 在 document.write 后可能不触发，兜底
    setTimeout(() => { win.print(); win.close() }, 500)
  } catch (err) {
    console.error('导出 PDF 失败:', err)
    message.error('导出 PDF 失败')
  }
}

// ===== 按钮操作 =====

/** 新建笔记 — 打开弹窗 */
function handleAddNote() {
  addNoteState.name = ''
  addNoteState.visible = true
}

/** 新建笔记 — 确认提交 */
async function handleAddNoteOk() {
  const title = addNoteState.name?.trim()
  if (!title) {
    message.warning('请输入笔记标题')
    return
  }
  addNoteState.loading = true
  try {
    const note = await noteStore.createNote({ title, is_folder: false, content: '' })
    addNoteState.visible = false
    // 立即打开新创建的笔记
    if (note) editorStore.selectNote(note.id)
  } catch {
    message.error('创建失败')
  } finally {
    addNoteState.loading = false
  }
}

/** 新建文件夹 — 打开弹窗 */
function handleAddFolder() {
  addFolderState.name = ''
  addFolderState.visible = true
}

/** 新建文件夹 — 确认提交 */
async function handleAddFolderOk() {
  const title = addFolderState.name?.trim()
  if (!title) {
    message.warning('请输入文件夹名称')
    return
  }
  addFolderState.loading = true
  try {
    await noteStore.createNote({ title, is_folder: true })
    addFolderState.visible = false
  } catch {
    message.error('创建失败')
  } finally {
    addFolderState.loading = false
  }
}
</script>

<style scoped>
/* ===== 布局 ===== */
.list-view {
  width: 270px;
  min-width: 270px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  overflow: hidden;
}

.list-header {
  display: flex;
  gap: var(--spacing-xs);
  padding: var(--spacing-md) var(--spacing-md) var(--spacing-sm);
  flex-shrink: 0;
}

.list-header .ant-btn {
  height: 32px;
  font-size: var(--font-size-md);
  border-radius: var(--radius-sm);
}

.search-box {
  padding: 0 var(--spacing-md) var(--spacing-sm);
  flex-shrink: 0;
}

.search-box .ant-input-search .ant-input {
  border-radius: var(--radius-sm);
  background: var(--bg-primary);
  height: 32px;
}

.tree-wrapper {
  flex: 1;
  overflow-y: auto;
  padding: 2px 0;
}

/* 加载/空状态 */
.tree-status {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xxxl) var(--spacing-lg);
  color: var(--text-tertiary);
  font-size: var(--font-size-md);
  min-height: 200px;
}

/* ===== ant-design-vue a-tree 样式覆盖 ===== */
.note-tree {
  font-size: var(--font-size-md);
  background: transparent;
  padding: 0;
}

/* 树节点行 — 确保完整高度可点击 */
.note-tree :deep(.ant-tree-treenode) {
  padding: 0;
  width: 100%;
}

/* 节点内容区域 — 有道云风格：紧凑、左侧指示条 */
.note-tree :deep(.ant-tree-node-content-wrapper) {
  display: flex;
  align-items: center;
  min-height: 28px;
  padding: 2px var(--spacing-md) 2px 6px;
  border-radius: 0;
  border-left: 3px solid transparent;
  transition: all var(--duration-fast);
  background: transparent !important;
  gap: 4px;
}

/* 悬停态 */
.note-tree :deep(.ant-tree-node-content-wrapper:hover) {
  background: var(--bg-hover) !important;
}

/* 选中态 — 左侧橙色指示条 + 浅橙色背景 */
.note-tree :deep(.ant-tree-node-content-wrapper.ant-tree-node-selected) {
  background: var(--bg-active) !important;
  border-left-color: var(--primary-color);
}

/* 节点标题 */
.note-tree :deep(.ant-tree-node-content-wrapper .ant-tree-title) {
  color: var(--text-primary);
  font-size: var(--font-size-md);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 图标 */
.note-tree :deep(.ant-tree-iconEle) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  line-height: 1;
  flex-shrink: 0;
}

.note-tree :deep(.ant-tree-switcher) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 28px;
  line-height: 28px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.note-tree :deep(.ant-tree-switcher:hover) {
  color: var(--text-primary);
}

.note-tree :deep(.ant-tree-switcher-icon) {
  font-size: 11px;
  transition: transform var(--duration-normal);
}

.note-tree :deep(.ant-tree-switcher_open .ant-tree-switcher-icon) {
  transform: rotate(90deg);
}

/* 缩进线 */
.note-tree :deep(.ant-tree-indent-unit) {
  width: 18px;
}

/* ===== 拖拽反馈 ===== */
.note-tree :deep(.ant-tree-treenode.drag-over > .ant-tree-node-content-wrapper) {
  background: var(--bg-active) !important;
}

.note-tree :deep(.ant-tree-treenode.drag-over-gap-top > .ant-tree-node-content-wrapper) {
  border-top: 2px solid var(--primary-color) !important;
}

.note-tree :deep(.ant-tree-treenode.drag-over-gap-bottom > .ant-tree-node-content-wrapper) {
  border-bottom: 2px solid var(--primary-color) !important;
}

/* ===== 右键菜单 ===== */
.context-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-context-menu);
}

.context-menu {
  position: fixed;
  z-index: calc(var(--z-context-menu) + 1);
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  min-width: 150px;
}

.context-menu :deep(.ant-menu) {
  border: none !important;
  border-radius: var(--radius-md);
  background: transparent;
  padding: 4px;
}

.context-menu :deep(.ant-menu-item) {
  height: 30px;
  line-height: 30px;
  margin: 0;
  padding: 0 var(--spacing-md);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: var(--font-size-md);
}

.context-menu :deep(.ant-menu-item:hover) {
  background: var(--bg-hover) !important;
}

.context-menu :deep(.ant-menu-item-danger) {
  color: var(--danger-color);
}

.context-menu :deep(.ant-menu-item-danger:hover) {
  background: var(--danger-bg) !important;
}

.context-menu :deep(.ant-menu-divider) {
  margin: 3px 0;
  border-color: var(--border-color);
}
</style>

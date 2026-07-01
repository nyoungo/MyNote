<template>
  <!--
    设置对话框 — 基于 ant-design-vue
    主题色切换（6色）+ 剪贴板/编辑器设置
  -->
  <a-modal
    :open="true"
    title="设置"
    :closable="true"
    :mask-closable="true"
    width="520px"
    destroy-on-close
    class="settings-modal"
    @cancel="emit('close')"
    @ok="handleSave"
    :ok-text="'保存'"
    :cancel-text="'关闭'"
    :confirm-loading="saving"
  >
    <!-- 主题色设置 -->
    <div class="setting-group">
      <div class="setting-label">
        <BulbOutlined style="font-size:16px" />
        <span>主题色</span>
      </div>
      <div class="theme-color-grid">
        <label
          v-for="c in themeColors"
          :key="c.id"
          class="theme-color-option"
          :class="{ active: themeColor === c.id }"
          @click="themeColor = c.id; applyTheme(c.id)"
        >
          <span class="color-swatch" :style="{ background: c.gradient }"></span>
          <span class="color-label">{{ c.name }}</span>
        </label>
      </div>
    </div>

    <a-divider />

    <!-- 剪贴板设置 -->
    <div class="setting-group">
      <div class="setting-label">
        <SnippetsOutlined style="font-size:16px" />
        <span>剪贴板(Ctrl+Shift+V)</span>
      </div>
      <div class="setting-row">
        <span class="setting-row-label">最大保存条数</span>
        <a-input-number
          v-model:value="clipboardMax"
          :min="10"
          :max="1000"
          :step="10"
        />
      </div>
    </div>

    <a-divider />

    <!-- 编辑器设置 -->
    <div class="setting-group">
      <div class="setting-label">
        <FileTextOutlined style="font-size:16px" />
        <span>编辑器</span>
      </div>
      <div class="setting-row">
        <span class="setting-row-label">自动保存间隔（秒）</span>
        <a-input-number
          v-model:value="autoSaveInterval"
          :min="5"
          :max="300"
          :step="5"
        />
      </div>
    </div>

    <!-- 消息提示 -->
    <div v-if="message" style="margin-top:16px;">
      <a-alert :message="message" :type="msgType === 'error' ? 'error' : 'success'" show-icon />
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  BulbOutlined,
  SnippetsOutlined,
  FileTextOutlined,
} from '@ant-design/icons-vue'

const emit = defineEmits<{ close: [] }>()

const themeColor = ref('blue')
const clipboardMax = ref(100)
const autoSaveInterval = ref(5)
const saving = ref(false)

const message = ref('')
const msgType = ref<'success' | 'error'>('success')

const themeColors = [
  { id: 'blue',   name: '天空蓝',   gradient: 'linear-gradient(135deg, #1677ff, #4096ff)' },
  { id: 'orange', name: '热力橙', gradient: 'linear-gradient(135deg, #FF6A00, #FF8A2B)' },
  { id: 'geekblue', name: '极客蓝',   gradient: 'linear-gradient(135deg, #2F54EB, #597EF7)' },
  { id: 'green',  name: '青葱绿',     gradient: 'linear-gradient(135deg, #07C160, #2DD47A)' },
  { id: 'red',    name: '火山红',     gradient: 'linear-gradient(135deg, #F5222D, #FF4D4F)' },
  { id: 'purple', name: '炫酷紫',     gradient: 'linear-gradient(135deg, #722ED1, #9254DE)' },
]

const themeClassMap: Record<string, string> = {
  blue: 'theme-blue',
  orange: 'theme-orange',
  geekblue: 'theme-geekblue',
  green: 'theme-green',
  red: 'theme-red',
  purple: 'theme-purple',
}

/** 从 localStorage 加载已有设置 */
onMounted(() => {
  try {
    const saved = JSON.parse(localStorage.getItem('mynote-settings') || '{}')
    if (saved.themeColor) themeColor.value = saved.themeColor
    if (saved.clipboardMax) clipboardMax.value = saved.clipboardMax
    if (saved.autoSaveInterval) autoSaveInterval.value = saved.autoSaveInterval
    // 应用保存的主题色
    applyTheme(themeColor.value)
  } catch { /* 忽略解析错误 */ }
})

function applyTheme(t: string) {
  // 移除所有现有的主题 class
  Object.values(themeClassMap).forEach(cls => {
    document.documentElement.classList.remove(cls)
  })
  const targetClass = themeClassMap[t]
  if (targetClass) {
    document.documentElement.classList.add(targetClass)
  } else {
    // 回退到蓝色
    document.documentElement.classList.add('theme-blue')
  }
}

async function handleSave() {
  saving.value = true
  const settings = {
    themeColor: themeColor.value,
    clipboardMax: clipboardMax.value,
    autoSaveInterval: autoSaveInterval.value,
  }
  localStorage.setItem('mynote-settings', JSON.stringify(settings))
  applyTheme(themeColor.value)
  saving.value = false
  emit('close')
}
</script>

<style scoped>
.settings-modal :deep(.ant-modal-body) {
  padding: 16px 24px 20px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}
.setting-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.setting-label .anticon {
  font-size: 15px;
  color: var(--primary-color);
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) 0 var(--spacing-xs) 28px;
}
.setting-row-label {
  font-size: var(--font-size-md);
  color: var(--text-secondary);
}

/* 主题色选择器 — 3列网格 */
.theme-color-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
  padding: 6px 0 2px 28px;
}

.theme-color-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
  background: var(--bg-primary);
}

.theme-color-option:hover {
  border-color: var(--primary-color);
  background: var(--primary-bg);
}

.theme-color-option.active {
  border-color: var(--primary-color);
  background: var(--primary-bg);
  box-shadow: 0 0 0 1px var(--primary-color);
}

.color-swatch {
  width: 22px;
  height: 22px;
  border-radius: var(--radius-round);
  flex-shrink: 0;
  border: 2px solid rgba(0, 0, 0, 0.08);
}

.color-label {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
}

/* Divider in settings */
:deep(.ant-divider) {
  margin: 14px 0;
  border-color: var(--border-secondary);
}
</style>

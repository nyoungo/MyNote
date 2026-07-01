<template>
  <!--
    工具视图 — 工具集入口
    待后期增加更多工具图标入口
  -->
  <div class="tools-page">
    <div class="tools-placeholder">
      <div class="placeholder-icon">
        <ToolOutlined style="font-size:48px" />
      </div>
      <p class="placeholder-title">工具集</p>
      <p class="placeholder-hint">更多工具正在开发中，敬请期待</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ToolOutlined } from '@ant-design/icons-vue'
</script>

<style scoped>
.tools-page {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
}

.tools-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  user-select: none;
  gap: var(--spacing-sm);
  padding: var(--spacing-xxxl);
  background: var(--bg-primary);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
}

.placeholder-icon {
  opacity: 0.35;
  margin-bottom: var(--spacing-sm);
  color: var(--primary-color);
}

.placeholder-title {
  font-size: var(--font-size-xl);
  font-weight: 600;
  color: var(--text-primary);
}

.placeholder-hint {
  font-size: var(--font-size-md);
  color: var(--text-tertiary);
}
</style>

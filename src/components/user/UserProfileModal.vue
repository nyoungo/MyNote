<template>
  <!--
    用户信息弹窗 — 基于 ant-design-vue
    支持查看/编辑用户信息、上传头像、退出登录
  -->
  <a-modal
    :open="true"
    title="用户信息"
    :closable="true"
    :mask-closable="true"
    width="420px"
    destroy-on-close
    class="user-profile-modal"
    @cancel="emit('close')"
    :footer="null"
  >
    <!-- 头像区域 -->
    <div class="avatar-section">
      <a-avatar
        :size="76"
        :src="previewUrl || undefined"
        :style="!previewUrl ? { verticalAlign: 'middle', fontSize: '28px', fontWeight: 600, cursor: 'pointer', color: '#fff' } : { cursor: 'pointer' }"
        class="avatar-upload"
        :class="{ 'avatar-default': !previewUrl }"
        @click="triggerFileSelect"
      >
        {{ initials }}
      </a-avatar>
      <div v-if="!previewUrl" class="avatar-hint">点击更换头像</div>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      style="display:none"
      @change="handleFileSelect"
    />

    <!-- 信息表单 -->
    <a-form layout="vertical" :model="formData">
      <a-form-item label="用户名">
        <a-input :value="formUsername" disabled placeholder="用户名" />
      </a-form-item>
      <a-form-item label="邮箱">
        <a-input v-model:value="formEmail" placeholder="邮箱地址" type="email" />
      </a-form-item>
    </a-form>

    <!-- 消息提示 -->
    <div v-if="message" style="margin-bottom:16px;">
      <a-alert :message="message" :type="msgType === 'error' ? 'error' : 'success'" show-icon />
    </div>

    <!-- 底部按钮 -->
    <div class="modal-footer">
      <a-button danger @click="handleLogout">退出登录</a-button>
      <div class="footer-right">
        <a-button @click="emit('close')">取消</a-button>
        <a-button type="primary" :loading="saving" @click="handleSave">
          {{ saving ? '保存中...' : '保存' }}
        </a-button>
      </div>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useUserStore } from '@/stores/userStore'
import { uploadAvatarApi } from '@/utils/request'

const emit = defineEmits<{ close: [] }>()
const userStore = useUserStore()

const fileInput = ref<HTMLInputElement | null>(null)
const formUsername = ref(userStore.username || '')
const formEmail = ref(userStore.email || '')
const formData = ref({})
const previewUrl = ref(userStore.fullAvatarUrl || '')
const selectedFile = ref<File | null>(null)
const saving = ref(false)
const message = ref('')
const msgType = ref<'success' | 'error'>('success')

const initials = computed(() => (userStore.username || 'U').charAt(0).toUpperCase())

function triggerFileSelect() {
  fileInput.value?.click()
}

function handleFileSelect(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  selectedFile.value = file
  const reader = new FileReader()
  reader.onload = () => previewUrl.value = reader.result as string
  reader.readAsDataURL(file)
}

function handleLogout() {
  userStore.logout()
  emit('close')
}

async function handleSave() {
  saving.value = true
  message.value = ''
  try {
    await userStore.updateProfile({
      username: formUsername.value,
      email: formEmail.value,
    })

    if (selectedFile.value) {
      const res = await uploadAvatarApi(selectedFile.value)
      userStore.setAvatar(res.avatar_url)
    }

    message.value = '保存成功'
    msgType.value = 'success'
    setTimeout(() => emit('close'), 1000)
  } catch (err: any) {
    message.value = err.message || '保存失败'
    msgType.value = 'error'
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.user-profile-modal :deep(.ant-modal-body) {
  padding: 20px 24px 16px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xl);
  padding-top: 8px;
}

.avatar-section .ant-avatar {
  border: 3px solid var(--primary-border);
  transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.10);
}

.avatar-section .ant-avatar:hover {
  border-color: var(--primary-color);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.avatar-section .avatar-default {
  background-color: var(--primary-color) !important;
}

.avatar-hint {
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
  margin-top: 4px;
}
.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 4px;
}
.footer-right {
  display: flex;
  gap: var(--spacing-sm);
}
</style>

<template>
  <!--
    登录/注册对话框 — 基于 ant-design-vue
    使用 a-modal + a-tabs + a-form 构建
  -->
  <a-modal
    :open="true"
    :footer="null"
    :closable="true"
    :mask-closable="true"
    width="420px"
    destroy-on-close
    class="login-modal"
    @cancel="emit('close')"
    @keydown.enter.prevent="handleSubmit"
  >
    <!-- 头部标题 -->
    <div class="modal-header">
      <h2 class="modal-title" style="margin-bottom:4px;">MyNote</h2>
      <p class="modal-subtitle" style="margin:0;">登录以使用云同步功能</p>
    </div>

    <!-- 标签页切换 -->
    <a-tabs
      v-model:activeKey="activeTab"
      centered
      size="large"
      @change="switchTab"
    >
      <a-tab-pane key="login" tab="登录" />
      <a-tab-pane key="register" tab="注册" />
    </a-tabs>

    <!-- 表单 -->
    <a-form
      layout="vertical"
      :model="form"
      autocomplete="off"
      @finish="handleSubmit"
    >
      <!-- 用户名 -->
      <a-form-item
        label="用户名"
        name="username"
        :rules="[{ required: true, message: '请输入用户名' },
                 { min: 2, message: '用户名至少 2 个字符' },
                 { max: 32, message: '用户名不能超过 32 个字符' }]"
        :validate-status="errors.username ? 'error' : undefined"
        :help="errors.username || undefined"
      >
        <a-input
          v-model:value="form.username"
          placeholder="请输入用户名"
          autocomplete="username"
          @input="clearError('username')"
        />
      </a-form-item>

      <!-- 密码 -->
      <a-form-item
        label="密码"
        name="password"
        :rules="[{ required: true, message: '请输入密码' },
                 { min: 6, message: '密码至少 6 个字符' },
                 { max: 128, message: '密码不能超过 128 个字符' }]"
        :validate-status="errors.password ? 'error' : undefined"
        :help="errors.password || undefined"
      >
        <a-input-password
          v-model:value="form.password"
          placeholder="请输入密码"
          autocomplete="current-password"
          @input="clearError('password')"
        />
      </a-form-item>

      <!-- 邮箱（仅注册时显示） -->
      <a-form-item
        v-if="activeTab === 'register'"
        label="邮箱"
        name="email"
        :rules="[{ required: true, message: '请输入邮箱' },
                 { type: 'email', message: '邮箱格式不正确' }]"
        :validate-status="errors.email ? 'error' : undefined"
        :help="errors.email || undefined"
      >
        <a-input
          v-model:value="form.email"
          placeholder="请输入邮箱地址"
          autocomplete="email"
          @input="clearError('email')"
        />
      </a-form-item>

      <!-- 全局提交错误 -->
      <a-form-item v-if="submitError">
        <a-alert :message="submitError" type="error" show-icon closable @close="submitError = ''" />
      </a-form-item>

      <!-- 提交按钮 -->
      <a-form-item>
        <a-button
          type="primary"
          html-type="submit"
          :loading="submitting"
          block
          size="large"
        >
          {{ activeTab === 'login' ? '登录' : '注册' }}
        </a-button>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useUserStore } from '@/stores/userStore'

const emit = defineEmits<{ close: [] }>()
const userStore = useUserStore()

const activeTab = ref<'login' | 'register'>('login')
const form = reactive({ username: '', password: '', email: '' })
const errors = reactive({ username: '', password: '', email: '' })
const submitError = ref('')
const submitting = ref(false)

function switchTab(tab: string | number) {
  const t = tab as 'login' | 'register'
  activeTab.value = t
  form.username = ''; form.password = ''; form.email = ''
  errors.username = ''; errors.password = ''; errors.email = ''
  submitError.value = ''
}

function clearError(field: 'username' | 'password' | 'email') {
  errors[field] = ''; submitError.value = ''
}

async function handleSubmit() {
  // 手动校验字段
  let valid = true
  if (!form.username.trim() || form.username.trim().length < 2 || form.username.trim().length > 32) {
    errors.username = '用户名长度 2-32 字符'
    valid = false
  }
  if (!form.password || form.password.length < 6 || form.password.length > 128) {
    errors.password = '密码长度 6-128 字符'
    valid = false
  }
  if (activeTab.value === 'register') {
    const email = form.email.trim()
    if (!email) { errors.email = '请输入邮箱'; valid = false }
    else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) { errors.email = '邮箱格式不正确'; valid = false }
  }
  if (!valid) return

  submitting.value = true; submitError.value = ''
  try {
    if (activeTab.value === 'login') {
      await userStore.login(form.username.trim(), form.password)
    } else {
      await userStore.register(form.username.trim(), form.password, form.email.trim() || undefined)
    }
    emit('close')
  } catch (e: any) {
    submitError.value = e?.message || '操作失败，请重试'
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.modal-header {
  text-align: center;
  padding-top: 4px;
  margin-bottom: 4px;
}
.modal-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--primary-color);
  letter-spacing: -0.02em;
  line-height: 1.3;
}
.modal-subtitle {
  font-size: var(--font-size-md);
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* 登录/注册标签页美化 */
.login-modal :deep(.ant-tabs-nav) {
  margin-bottom: 20px;
}

.login-modal :deep(.ant-tabs-tab) {
  font-size: 15px;
  padding: 8px 0;
  margin: 0 16px;
}

.login-modal :deep(.ant-tabs-tab + .ant-tabs-tab) {
  margin: 0 16px;
}

/* 表单美化 */
.login-modal :deep(.ant-form-item) {
  margin-bottom: 18px;
}

.login-modal :deep(.ant-input),
.login-modal :deep(.ant-input-password) {
  height: 40px;
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  border-color: var(--border-color);
}

.login-modal :deep(.ant-input:hover),
.login-modal :deep(.ant-input-password:hover) {
  border-color: var(--primary-color);
}

.login-modal :deep(.ant-input:focus),
.login-modal :deep(.ant-input-password:focus) {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px var(--primary-bg);
}

.login-modal :deep(.ant-btn-primary) {
  height: 40px;
  border-radius: var(--radius-sm);
  font-size: 15px;
  font-weight: 500;
}
</style>

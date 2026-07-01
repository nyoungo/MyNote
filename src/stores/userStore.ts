import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { loginApi, registerApi, getProfileApi, updateProfileApi, API_BASE } from '@/utils/request'

/**
 * 用户认证状态管理
 * 管理登录/注册、Token、用户资料
 */
export const useUserStore = defineStore('user', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const userId = ref<string | null>(null)
  const username = ref<string>('')
  const email = ref<string | null>(null)
  const avatarUrl = ref<string | null>(null)
  const isLoggedIn = computed(() => !!token.value)

  /** 完整头像 URL：服务端返回相对路径时拼接 API 基础地址 */
  const fullAvatarUrl = computed(() => {
    if (!avatarUrl.value) return null
    if (avatarUrl.value.startsWith('http')) return avatarUrl.value
    return `${API_BASE}${avatarUrl.value}`
  })

  /** 登录 */
  async function login(loginUsername: string, password: string) {
    const res = await loginApi(loginUsername, password)
    token.value = res.token
    userId.value = res.user_id
    username.value = res.username
    email.value = res.email
    localStorage.setItem('token', res.token)
    return res
  }

  /** 注册 */
  async function register(regUsername: string, password: string, regEmail?: string) {
    const res = await registerApi(regUsername, password, regEmail)
    token.value = res.token
    userId.value = res.user_id
    username.value = res.username
    email.value = res.email
    localStorage.setItem('token', res.token)
    return res
  }

  /** 退出登录 */
  function logout() {
    token.value = null
    userId.value = null
    username.value = ''
    email.value = null
    avatarUrl.value = null
    localStorage.removeItem('token')
  }

  /** 加载用户资料 */
  async function loadProfile() {
    if (!token.value) return
    try {
      const profile = await getProfileApi()
      userId.value = profile.id
      username.value = profile.username
      email.value = profile.email
      avatarUrl.value = profile.avatar_url
    } catch (e) {
      console.error('加载用户资料失败:', e)
    }
  }

  /** 更新用户资料 */
  async function updateProfile(data: { username?: string; email?: string }) {
    const res = await updateProfileApi(data)
    if (res.username) username.value = res.username
    if (res.email !== undefined) email.value = res.email
    return res
  }

  /** 更新头像 URL */
  function setAvatar(url: string) {
    avatarUrl.value = url
  }

  return {
    token, userId, username, email, avatarUrl, fullAvatarUrl, isLoggedIn,
    login, register, logout, loadProfile, updateProfile, setAvatar,
  }
}, {
  // 持久化 token 到 localStorage
  persist: {
    key: 'user-store',
    storage: localStorage,
    paths: ['token', 'userId', 'username', 'email', 'avatarUrl'],
  },
})

import axios, { type AxiosInstance } from 'axios'

/**
 * Axios HTTP 请求封装
 * 自动附加 JWT Token、统一错误处理
 */

// 服务端基础地址（开发环境）
export const API_BASE = 'http://localhost:8082'

const http: AxiosInstance = axios.create({
  baseURL: API_BASE,
  timeout: 15000,
  headers: { 'Content-Type': 'application/json' },
})

// 请求拦截器：自动注入 Authorization header
http.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// 响应拦截器：统一处理错误状态码
http.interceptors.response.use(
  (res) => res,
  (error) => {
    if (error.response?.status === 401) {
      // Token 无效或过期，清除登录态（不跳转，由各组件自行处理）
      localStorage.removeItem('token')
    }
    const msg = error.response?.data?.error || error.message || '请求失败'
    return Promise.reject(new Error(msg))
  }
)

// ======== 认证 API ========
export function loginApi(username: string, password: string) {
  return http.post('/api/auth/login', { username, password }).then(r => r.data)
}
export function registerApi(username: string, password: string, email?: string) {
  return http.post('/api/auth/register', { username, password, email }).then(r => r.data)
}

// ======== 用户 API ========
export function getProfileApi() {
  return http.get('/api/user/profile').then(r => r.data)
}
export function updateProfileApi(data: { username?: string; email?: string }) {
  return http.put('/api/user/profile', data).then(r => r.data)
}
export function uploadAvatarApi(file: File) {
  const form = new FormData()
  form.append('avatar', file)
  return http.post('/api/user/avatar', form, {
    headers: { 'Content-Type': 'multipart/form-data' },
  }).then(r => r.data)
}

// ======== 笔记 API ========
export function getNotesApi() {
  return http.get('/api/notes').then(r => r.data)
}
export function createNoteApi(data: {
  title: string; parent_id?: string | null; is_folder?: boolean; content?: string
}) {
  return http.post('/api/notes', data).then(r => r.data)
}
export function getNoteApi(id: string) {
  return http.get(`/api/notes/${id}`).then(r => r.data)
}
export function updateNoteApi(id: string, data: any) {
  return http.put(`/api/notes/${id}`, data).then(r => r.data)
}
export function deleteNoteApi(id: string) {
  return http.delete(`/api/notes/${id}`).then(r => r.data)
}

// ======== 同步 API ========
export function syncNotesApi(operations: any[]) {
  return http.post('/api/notes/sync', { operations }).then(r => r.data)
}

export default http

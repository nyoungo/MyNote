/** 用户 */
export interface User {
  id: string
  username: string
  email?: string
  password_hash: string
  avatar_url?: string
  created_at: string
  updated_at: string
}

/** 笔记/文件夹节点 */
export interface Note {
  id: string
  user_id: string
  parent_id: string | null
  title: string
  content: string | null
  is_folder: number       // 0=笔记, 1=文件夹
  sort_order: number
  version: number
  created_at: string
  updated_at: string
}

/** 同步操作记录 */
export interface SyncOperation {
  id: string
  user_id: string
  operation: 'create' | 'update' | 'delete' | 'conflict_skip'
  target_type: 'note' | 'folder'
  target_id: string
  payload?: string
  client_version?: number
  created_at: string
}

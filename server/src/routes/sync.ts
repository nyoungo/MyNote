import { Router, Response } from 'express'
import { v4 as uuidv4 } from 'uuid'
import { getDb } from '../services/db'
import { authMiddleware, AuthRequest } from '../middleware/auth'

const router = Router()

router.use(authMiddleware)

/**
 * POST /api/notes/sync
 * 同步操作（含版本冲突检测）
 *
 * 冲突策略：Last-Writer-Wins
 * - client_version < server.version → 返回 conflict 状态
 * - client_version >= server.version → 正常应用
 */
router.post('/', (req: AuthRequest, res: Response) => {
  try {
    const { operations } = req.body
    const db = getDb()

    if (!Array.isArray(operations)) {
      return res.status(400).json({ error: 'operations 必须是数组' })
    }

    const results: any[] = []

    // 事务保证原子性
    db.transaction(() => {
      for (const op of operations) {
        const { operation, target_type, target_id, payload, client_version } = op

        // ===== 冲突检测 =====
        if ((operation === 'create' || operation === 'update') && payload) {
          const rows = db.prepare(
            'SELECT version FROM notes WHERE id = ? AND user_id = ?'
          ).all([target_id, req.userId]) as any[]

          if (rows.length > 0 && client_version !== undefined && client_version < rows[0].version) {
            // 冲突：服务端版本更新，返回最新数据供客户端合并
            const serverNote = db.prepare('SELECT * FROM notes WHERE id = ?').get([target_id])
            const syncId = uuidv4()
            db.prepare(`
              INSERT INTO sync_operations (id, user_id, operation, target_type, target_id, payload)
              VALUES (?, ?, 'conflict_skip', ?, ?, ?)
            `).run([syncId, req.userId, target_type, target_id, JSON.stringify(payload)])

            results.push({
              target_id,
              status: 'conflict',
              server_version: rows[0].version,
              server_data: serverNote,
            })
            return // continue to next operation
          }
        }

        // 记录同步日志
        const syncId = uuidv4()
        db.prepare(`
          INSERT INTO sync_operations (id, user_id, operation, target_type, target_id, payload)
          VALUES (?, ?, ?, ?, ?, ?)
        `).run([syncId, req.userId, operation, target_type, target_id, JSON.stringify(payload)])

        // 应用到数据库
        switch (operation) {
          case 'create':
          case 'update': {
            if (payload) {
              const { id, title, content, parent_id, is_folder, sort_order, version } = payload
              db.prepare(`
                INSERT INTO notes (id, user_id, title, content, parent_id, is_folder, sort_order, version)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  title = COALESCE(?, title),
                  content = COALESCE(?, content),
                  parent_id = ?,
                  sort_order = COALESCE(?, sort_order),
                  version = version + 1,
                  updated_at = datetime('now')
              `).run([
                id, req.userId, title, content, parent_id ?? null,
                is_folder ?? 0, sort_order ?? 0, version ?? 1,
                // ON CONFLICT 部分参数
                title, content, parent_id ?? null, sort_order ?? null,
              ])
            }
            break
          }
          case 'delete': {
            db.prepare('DELETE FROM notes WHERE id = ? AND user_id = ?')
              .run([target_id, req.userId])
            break
          }
        }

        results.push({ syncId, target_id, status: 'synced' })
      }
    })

    db.save()

    // 返回服务端近 1 小时内的变更
    const serverOps = db.prepare(`
      SELECT * FROM sync_operations
      WHERE user_id = ? AND created_at > datetime('now', '-1 hour')
      ORDER BY created_at ASC
    `).all([req.userId])

    res.json({ results, serverOps })
  } catch (error) {
    console.error('同步失败:', error)
    res.status(500).json({ error: '同步失败' })
  }
})

export default router

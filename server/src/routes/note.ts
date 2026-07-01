import { Router, Response } from 'express'
import { v4 as uuidv4 } from 'uuid'
import { getDb } from '../services/db'
import { authMiddleware, AuthRequest } from '../middleware/auth'

const router = Router()

// 所有笔记路由都需要 JWT 认证
router.use(authMiddleware)

/**
 * GET /api/notes
 * 获取当前用户的所有笔记和文件夹（扁平列表，前端构建树）
 */
router.get('/', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb()
    const notes = db.prepare(
      'SELECT * FROM notes WHERE user_id = ? ORDER BY sort_order ASC, updated_at DESC'
    ).all([req.userId])
    res.json(notes)
  } catch (error) {
    console.error('获取笔记列表失败:', error)
    res.status(500).json({ error: '获取笔记列表失败' })
  }
})

/**
 * POST /api/notes
 * 创建笔记或文件夹
 */
router.post('/', (req: AuthRequest, res: Response) => {
  try {
    const { title, parent_id, is_folder, content } = req.body
    const db = getDb()

    // 计算排序序号：同层最大值 + 1
    const row = db.prepare(
      'SELECT MAX(sort_order) as max_order FROM notes WHERE user_id = ? AND parent_id IS ?'
    ).all([req.userId, parent_id || null]) as any[]

    const sortOrder = (row[0]?.max_order ?? -1) + 1
    const id = uuidv4()

    db.prepare(`
      INSERT INTO notes (id, user_id, title, parent_id, is_folder, sort_order, content)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `).run([id, req.userId, title, parent_id || null, is_folder ? 1 : 0, sortOrder, content || null])
    db.save()

    const note = db.prepare('SELECT * FROM notes WHERE id = ?').get([id])
    res.status(201).json(note)
  } catch (error) {
    console.error('创建笔记失败:', error)
    res.status(500).json({ error: '创建笔记失败' })
  }
})

/**
 * GET /api/notes/:id
 * 获取单篇笔记
 */
router.get('/:id', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb()
    const note = db.prepare(
      'SELECT * FROM notes WHERE id = ? AND user_id = ?'
    ).get([req.params.id, req.userId])

    if (!note) return res.status(404).json({ error: '笔记不存在' })
    res.json(note)
  } catch (error) {
    console.error('获取笔记失败:', error)
    res.status(500).json({ error: '获取笔记失败' })
  }
})

/**
 * PUT /api/notes/:id
 * 更新笔记（部分更新）
 */
router.put('/:id', (req: AuthRequest, res: Response) => {
  try {
    const { title, content, parent_id, sort_order, is_folder } = req.body
    const db = getDb()

    const existing = db.prepare(
      'SELECT * FROM notes WHERE id = ? AND user_id = ?'
    ).get([req.params.id, req.userId])

    if (!existing) return res.status(404).json({ error: '笔记不存在' })

    // 动态构建 SET 子句：只更新请求中显式传入的字段
    // 避免未传 parent_id 时误将节点移出文件夹
    const sets: string[] = []
    const params: any[] = []

    if (title !== undefined) { sets.push('title = ?'); params.push(title) }
    if (content !== undefined) { sets.push('content = ?'); params.push(content) }
    if ('parent_id' in req.body) { sets.push('parent_id = ?'); params.push(parent_id ?? null) }
    if (sort_order !== undefined) { sets.push('sort_order = ?'); params.push(sort_order) }
    if (is_folder !== undefined) { sets.push('is_folder = ?'); params.push(is_folder ? 1 : 0) }

    if (sets.length === 0) return res.status(400).json({ error: '没有需要更新的字段' })

    sets.push("version = version + 1")
    sets.push("updated_at = datetime('now')")
    params.push(req.params.id, req.userId)

    db.prepare(`UPDATE notes SET ${sets.join(', ')} WHERE id = ? AND user_id = ?`).run(params)
    db.save()

    const updated = db.prepare('SELECT * FROM notes WHERE id = ?').get([req.params.id])
    res.json(updated)
  } catch (error) {
    console.error('更新笔记失败:', error)
    res.status(500).json({ error: '更新笔记失败' })
  }
})

/**
 * DELETE /api/notes/:id
 * 删除笔记
 */
router.delete('/:id', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb()
    const result = db.prepare(
      'DELETE FROM notes WHERE id = ? AND user_id = ?'
    ).run([req.params.id, req.userId])

    if (result.changes === 0) return res.status(404).json({ error: '笔记不存在' })
    db.save()
    res.json({ message: '删除成功' })
  } catch (error) {
    console.error('删除笔记失败:', error)
    res.status(500).json({ error: '删除笔记失败' })
  }
})

export default router

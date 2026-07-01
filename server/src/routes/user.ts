import { Router, Response } from 'express'
import multer from 'multer'
import path from 'path'
import fs from 'fs'
import { v4 as uuidv4 } from 'uuid'
import { getDb } from '../services/db'
import { authMiddleware, AuthRequest } from '../middleware/auth'
import { config } from '../config'

const router = Router()

router.use(authMiddleware)

// ---------- 头像上传配置 ----------
const storage = multer.diskStorage({
  destination: (_req, _file, cb) => {
    const uploadDir = path.resolve(config.dataDir, '../uploads/avatars')
    fs.mkdirSync(uploadDir, { recursive: true })
    cb(null, uploadDir)
  },
  filename: (_req, file, cb) => {
    const ext = path.extname(file.originalname) || '.jpg'
    cb(null, `${uuidv4()}${ext}`)
  },
})

const upload = multer({
  storage,
  limits: { fileSize: config.maxUploadSize },
  fileFilter: (_req, file, cb) => {
    const allowed = ['image/jpeg', 'image/png', 'image/webp', 'image/gif']
    if (allowed.includes(file.mimetype)) {
      cb(null, true)
    } else {
      cb(new Error('不支持的文件类型，仅支持 JPEG、PNG、WebP、GIF'))
    }
  },
})

/**
 * GET /api/user/profile
 * 获取当前用户信息
 */
router.get('/profile', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb()
    const user = db.prepare(
      'SELECT id, username, email, avatar_url, created_at FROM users WHERE id = ?'
    ).get([req.userId])

    if (!user) return res.status(404).json({ error: '用户不存在' })
    res.json(user)
  } catch (error) {
    console.error('获取用户信息失败:', error)
    res.status(500).json({ error: '获取用户信息失败' })
  }
})

/**
 * PUT /api/user/profile
 * 更新用户资料
 */
router.put('/profile', (req: AuthRequest, res: Response) => {
  try {
    const { email, username } = req.body
    const db = getDb()

    db.prepare(`
      UPDATE users SET
        email = COALESCE(?, email),
        username = COALESCE(?, username),
        updated_at = datetime('now')
      WHERE id = ?
    `).run([email ?? null, username ?? null, req.userId])
    db.save()

    const user = db.prepare(
      'SELECT id, username, email, avatar_url, created_at FROM users WHERE id = ?'
    ).get([req.userId])

    res.json(user)
  } catch (error) {
    console.error('更新用户信息失败:', error)
    res.status(500).json({ error: '更新用户信息失败' })
  }
})

/**
 * POST /api/user/avatar
 * 上传头像
 */
router.post('/avatar', (req: AuthRequest, res: Response) => {
  upload.single('avatar')(req, res, (err) => {
    if (err) return res.status(400).json({ error: err.message })
    if (!req.file) return res.status(400).json({ error: '请选择要上传的文件' })

    try {
      const db = getDb()
      const avatarUrl = `/uploads/avatars/${req.file.filename}`

      db.prepare(
        "UPDATE users SET avatar_url = ?, updated_at = datetime('now') WHERE id = ?"
      ).run([avatarUrl, req.userId])
      db.save()

      res.json({ avatar_url: avatarUrl })
    } catch (error) {
      console.error('上传头像失败:', error)
      res.status(500).json({ error: '上传头像失败' })
    }
  })
})

export default router

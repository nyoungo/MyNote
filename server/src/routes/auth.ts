import { Router, Request, Response } from 'express'
import bcrypt from 'bcryptjs'
import { v4 as uuidv4 } from 'uuid'
import { getDb } from '../services/db'
import { generateToken } from '../utils/jwt'

const router = Router()

/**
 * POST /api/auth/register
 * 用户注册：创建新用户并返回 JWT Token
 */
router.post('/register', (req: Request, res: Response) => {
  try {
    const { username, password, email } = req.body

    // ---------- 参数校验 ----------
    if (!username || !password) {
      return res.status(400).json({ error: '用户名和密码不能为空' })
    }
    if (typeof username !== 'string' || username.length < 2 || username.length > 32) {
      return res.status(400).json({ error: '用户名长度需在 2-32 个字符之间' })
    }
    if (typeof password !== 'string' || password.length < 6) {
      return res.status(400).json({ error: '密码长度不能少于 6 个字符' })
    }

    const db = getDb()

    // 检查用户名是否已存在
    const existing = db.prepare('SELECT id FROM users WHERE username = ?').get([username])
    if (existing) {
      return res.status(409).json({ error: '用户名已存在' })
    }

    // bcryptjs 哈希密码（10 轮 salt）
    const salt = bcrypt.genSaltSync(10)
    const passwordHash = bcrypt.hashSync(password, salt)

    const userId = uuidv4()
    db.prepare(
      'INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)'
    ).run([userId, username, email || null, passwordHash])
    db.save() // sql.js 需手动持久化

    const token = generateToken(userId)

    res.status(201).json({ token, user_id: userId, username, email: email || null })
  } catch (error) {
    console.error('注册失败:', error)
    res.status(500).json({ error: '服务器内部错误' })
  }
})

/**
 * POST /api/auth/login
 * 用户登录：验证凭据并返回 JWT Token
 */
router.post('/login', (req: Request, res: Response) => {
  try {
    const { username, password } = req.body

    if (!username || !password) {
      return res.status(400).json({ error: '用户名和密码不能为空' })
    }

    const db = getDb()

    const user = db.prepare(
      'SELECT id, username, email, password_hash FROM users WHERE username = ?'
    ).get([username]) as any

    if (!user || !bcrypt.compareSync(password, user.password_hash)) {
      return res.status(401).json({ error: '用户名或密码错误' })
    }

    const token = generateToken(user.id)
    res.json({ token, user_id: user.id, username: user.username, email: user.email })
  } catch (error) {
    console.error('登录失败:', error)
    res.status(500).json({ error: '服务器内部错误' })
  }
})

export default router

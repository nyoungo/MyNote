import { Request, Response, NextFunction } from 'express'
import { verifyToken, type JwtPayload } from '../utils/jwt'

/**
 * 扩展 Express Request，注入 userId
 */
export interface AuthRequest extends Request {
  userId?: string
}

/**
 * JWT 认证中间件
 * 验证 Bearer Token，将 userId 注入到请求对象
 * 验证失败返回 401
 */
export function authMiddleware(req: AuthRequest, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: '未提供认证令牌' })
  }

  const token = authHeader.substring(7)

  try {
    const payload: JwtPayload = verifyToken(token)
    // 将用户 ID 注入请求，后续路由处理器可直接使用
    req.userId = payload.sub
    next()
  } catch (error) {
    return res.status(401).json({ error: '认证令牌无效或已过期' })
  }
}

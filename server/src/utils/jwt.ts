import jwt from 'jsonwebtoken'
import { config } from '../config'

/** JWT 载荷结构 */
export interface JwtPayload {
  sub: string   // 用户 ID
  exp: number   // 过期时间戳（秒）
}

/**
 * 生成 JWT Token
 * @param userId - 用户 ID
 * @returns JWT 字符串
 */
export function generateToken(userId: string): string {
  const payload: JwtPayload = {
    sub: userId,
    exp: Math.floor(Date.now() / 1000) + config.jwtExpireDays * 24 * 60 * 60,
  }
  return jwt.sign(payload, config.jwtSecret)
}

/**
 * 验证并解析 JWT Token
 * @param token - JWT 字符串
 * @returns 解析后的载荷
 * @throws 当 token 无效或过期时抛出
 */
export function verifyToken(token: string): JwtPayload {
  return jwt.verify(token, config.jwtSecret) as JwtPayload
}

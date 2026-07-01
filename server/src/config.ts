import dotenv from 'dotenv'
import path from 'path'
import fs from 'fs'

// 检测是否在 pkg 打包环境中运行
const isPkg = !!(process as any).pkg

/**
 * 解析命令行参数（支持 --port / -p）
 */
function parseArgs() {
  const args = process.argv.slice(2)
  const result: Record<string, string> = {}
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--port' || args[i] === '-p') {
      result.port = args[++i]
    }
  }
  return result
}

// 加载 .env 环境变量（pkg 环境下从 cwd 加载）
if (isPkg) {
  const envPath = path.resolve(process.cwd(), '.env')
  if (fs.existsSync(envPath)) {
    dotenv.config({ path: envPath })
  }
} else {
  dotenv.config()
}

const cliArgs = parseArgs()

/**
 * 服务端统一配置
 * 所有配置项集中管理，优先级：CLI 参数 > 环境变量 > 默认值
 */
export const config = {
  port: parseInt(cliArgs.port || process.env.PORT || '8081', 10),
  dataDir: path.resolve(process.env.DATA_DIR || './data'),
  jwtSecret: process.env.JWT_SECRET || 'your-secret-key-change-me-in-production',
  jwtExpireDays: parseInt(process.env.JWT_EXPIRE_DAYS || '7', 10),
  logLevel: process.env.LOG_LEVEL || 'dev',
  maxUploadSize: (parseInt(process.env.MAX_UPLOAD_SIZE_MB || '10', 10)) * 1024 * 1024,
  isPkg,
} as const

export type Config = typeof config

import path from 'path'
import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import morgan from 'morgan'
import { initDatabase } from './services/db'
import { config } from './config'
import authRoutes from './routes/auth'
import noteRoutes from './routes/note'
import userRoutes from './routes/user'
import syncRoutes from './routes/sync'

async function main() {
  // ===== 初始化数据库（sql.js 异步加载 WASM） =====
  await initDatabase()

  const app = express()

  // ===== 中间件 =====
  // CSP 由 Tauri 客户端控制，服务端 helmet 不覆盖
  app.use(helmet({
    contentSecurityPolicy: false,
    crossOriginEmbedderPolicy: false,
  }))
  app.use(cors({
    origin: '*',
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
    allowedHeaders: ['Content-Type', 'Authorization'],
  }))
  app.use(morgan(config.logLevel))
  app.use(express.json({ limit: '10mb' }))
  // 允许跨域访问上传资源（头像等）
  app.use('/uploads', (req, res, next) => {
    res.setHeader('Access-Control-Allow-Origin', '*')
    res.setHeader('Cross-Origin-Resource-Policy', 'cross-origin')
    next()
  }, express.static(path.resolve(config.dataDir, '../uploads')))

  // ===== 健康检查 =====
  app.get('/health', (_req, res) => {
    res.json({ status: 'ok', version: '1.0.0' })
  })

  // ===== 路由（sync 必须在 notes 之前） =====
  app.use('/api/auth', authRoutes)
  app.use('/api/notes/sync', syncRoutes)
  app.use('/api/notes', noteRoutes)
  app.use('/api/user', userRoutes)

  // ===== 错误处理 =====
  app.use((err: any, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    console.error('未捕获错误:', err)
    res.status(err.status || 500).json({ error: err.message || '服务器内部错误' })
  })

  // ===== 启动 =====
  app.listen(config.port, () => {
    console.log(`🚀 MyNote Server v1.0.0`)
    if (config.isPkg) {
      console.log(`📦 运行模式: 独立可执行文件`)
      console.log(`💡 自定义端口: ./mynote-server --port 9090`)
      console.log(`               或设置环境变量 PORT=9090 ./mynote-server`)
    }
    console.log(`📂 数据目录: ${config.dataDir}`)
    console.log(`🌐 监听: http://0.0.0.0:${config.port}`)
  })
}

main().catch(console.error)

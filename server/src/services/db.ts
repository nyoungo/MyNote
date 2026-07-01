import initSqlJs, { type Database as SqlJsDatabase } from 'sql.js'
import path from 'path'
import fs from 'fs'
import { config } from '../config'

let db: Database

/**
 * 数据库封装，提供与 better-sqlite3 类似的 API
 * .prepare(sql).get(params) — 查询单行
 * .prepare(sql).all(params) — 查询多行
 * .prepare(sql).run(params) — 执行写入
 * .transaction(fn) — 事务
 */
export class Database {
  private _db: SqlJsDatabase

  constructor(sqlDb: SqlJsDatabase) {
    this._db = sqlDb
  }

  exec(sql: string) {
    this._db.exec(sql)
  }

  prepare(sql: string) {
    const stmt = this._db.prepare(sql)
    return {
      get: (params?: any) => {
        if (params !== undefined) {
          stmt.bind(params instanceof Array ? params : [params])
        }
        if (stmt.step()) {
          const result = stmt.getAsObject()
          stmt.reset()
          return result
        }
        stmt.reset()
        return undefined
      },
      all: (params?: any) => {
        const results: any[] = []
        if (params !== undefined) {
          stmt.bind(params instanceof Array ? params : [params])
        }
        while (stmt.step()) {
          results.push(stmt.getAsObject())
        }
        stmt.reset()
        return results
      },
      run: (params?: any) => {
        if (params !== undefined) {
          stmt.bind(params instanceof Array ? params : [params])
        }
        stmt.step()
        const changes = this._db.getRowsModified()
        stmt.reset()
        return { changes }
      },
    }
  }

  transaction<T>(fn: () => T): T {
    this._db.exec('BEGIN')
    try {
      const result = fn()
      this._db.exec('COMMIT')
      return result
    } catch (e) {
      this._db.exec('ROLLBACK')
      throw e
    }
  }

  /** 保存到文件（sql.js 纯内存，需手动持久化） */
  save() {
    const data = this._db.export()
    const dbPath = path.join(config.dataDir, 'mynote.db')
    fs.writeFileSync(dbPath, Buffer.from(data))
  }
}

/**
 * 获取已初始化的数据库实例
 */
export function getDb(): Database {
  if (!db) {
    throw new Error('数据库未初始化，请先调用 initDatabase()')
  }
  return db
}

/**
 * 初始化 SQLite 数据库（基于 sql.js 纯 WASM 实现）
 * sql.js 运行在内存中，通过文件 IO 实现持久化
 */
export async function initDatabase(): Promise<Database> {
  fs.mkdirSync(config.dataDir, { recursive: true })

  const dbPath = path.join(config.dataDir, 'mynote.db')
  console.log(`📂 数据库路径: ${dbPath}`)

  // 初始化 sql.js WASM 引擎
  // 确定 sql-wasm.wasm 路径（dev 和 pkg 模式路径不同）
  let wasmPath: string
  if (config.isPkg) {
    // pkg 模式：dist/ 下的 asset
    wasmPath = path.resolve(__dirname, '..', 'sql-wasm.wasm')
  } else {
    // 开发/生产模式：node_modules 中
    wasmPath = path.resolve(path.dirname(require.resolve('sql.js')), 'sql-wasm.wasm')
  }
  const SQL = await initSqlJs({
    locateFile: () => wasmPath,
  })

  let sqlDb: SqlJsDatabase
  if (fs.existsSync(dbPath)) {
    const buffer = fs.readFileSync(dbPath)
    sqlDb = new SQL.Database(buffer)
  } else {
    sqlDb = new SQL.Database()
  }

  // sql.js 不支持 PRAGMA 的返回值，但支持执行
  sqlDb.run('PRAGMA journal_mode = WAL')
  sqlDb.run('PRAGMA synchronous = NORMAL')
  sqlDb.run('PRAGMA busy_timeout = 10000')
  sqlDb.run('PRAGMA foreign_keys = ON')

  db = new Database(sqlDb)
  runMigrations()
  console.log('✅ 数据库已连接: mynote.db')
  return db
}

/**
 * 增量数据库迁移
 */
function runMigrations() {
  db.exec(`
    CREATE TABLE IF NOT EXISTS _migrations (
      version TEXT PRIMARY KEY,
      applied_at TEXT DEFAULT (datetime('now'))
    )
  `)

  const migrations: { version: string; sql: string[] }[] = [
    {
      version: '001',
      sql: [
        `CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          username TEXT UNIQUE NOT NULL,
          email TEXT UNIQUE,
          password_hash TEXT NOT NULL,
          avatar_url TEXT,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now'))
        )`,
        `CREATE TABLE IF NOT EXISTS notes (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          parent_id TEXT,
          title TEXT NOT NULL,
          content TEXT,
          is_folder INTEGER DEFAULT 0,
          sort_order INTEGER DEFAULT 0,
          version INTEGER DEFAULT 1,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now')),
          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
          FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
        )`,
        `CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes(user_id)`,
        `CREATE INDEX IF NOT EXISTS idx_notes_parent_id ON notes(parent_id)`,
        `CREATE TABLE IF NOT EXISTS sync_operations (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          operation TEXT NOT NULL,
          target_type TEXT NOT NULL,
          target_id TEXT NOT NULL,
          payload TEXT,
          client_version INTEGER,
          created_at TEXT DEFAULT (datetime('now')),
          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )`,
        `CREATE INDEX IF NOT EXISTS idx_sync_user_created ON sync_operations(user_id, created_at)`,
      ],
    },
  ]

  const applied = new Set(
    db.prepare('SELECT version FROM _migrations').all().map((row: any) => row.version)
  )

  for (const m of migrations) {
    if (!applied.has(m.version)) {
      console.log(`📦 应用迁移: ${m.version}`)
      db.transaction(() => {
        for (const sql of m.sql) {
          db.exec(sql)
        }
        db.prepare('INSERT INTO _migrations (version) VALUES (?)').run([m.version])
      })
      db.save()
    }
  }
}

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqlitePool};
use sqlx::ConnectOptions;
use std::path::Path;

/// 客户端本地 SQLite 数据库连接池
pub type DbPool = SqlitePool;

/// 初始化客户端本地数据库
/// 使用 WAL 模式提升并发读性能
pub async fn init_database(db_path: &str) -> Result<DbPool, sqlx::Error> {
    // 确保父目录存在
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let _db_url = format!("sqlite:{}?mode=rwc", db_path);

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(1)  // 桌面客户端单连接即可
        .connect_with(options)
        .await?;

    // 创建核心表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            node_type TEXT NOT NULL CHECK (node_type IN ('note', 'folder')),
            parent_id TEXT REFERENCES nodes(id) ON DELETE CASCADE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            content TEXT,
            is_favorite BOOLEAN DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clipboard_history (
            id TEXT PRIMARY KEY,
            content_type TEXT NOT NULL,
            content TEXT,
            source_app TEXT,
            is_favorite BOOLEAN DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_queue (
            id TEXT PRIMARY KEY,
            operation TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            payload TEXT,
            status TEXT DEFAULT 'pending',
            retry_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_parent_id ON nodes(parent_id)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_sort_order ON nodes(sort_order)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_created ON clipboard_history(created_at DESC)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

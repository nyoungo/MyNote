-- MyNote 客户端本地数据库初始化（Tauri Rust 端）
-- 由 infrastructure/database.rs 中的 init_database() 自动执行

CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    node_type TEXT NOT NULL CHECK (node_type IN ('note', 'folder')),
    parent_id TEXT REFERENCES nodes(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    content TEXT,
    is_favorite BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_nodes_parent_id ON nodes(parent_id);
CREATE INDEX IF NOT EXISTS idx_nodes_sort_order ON nodes(sort_order);

-- 剪贴板历史表（纯本地，不涉及云端同步）
CREATE TABLE IF NOT EXISTS clipboard_history (
    id TEXT PRIMARY KEY,
    content_type TEXT NOT NULL,
    content TEXT,
    source_app TEXT,
    is_favorite BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_clipboard_created ON clipboard_history(created_at DESC);

-- 离线同步队列
CREATE TABLE IF NOT EXISTS sync_queue (
    id TEXT PRIMARY KEY,
    operation TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    payload TEXT,
    status TEXT DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

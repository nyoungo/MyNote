# MyNote 专业笔记程序 - 完整开发方案

> **版本**: v1.0.0
> **客户端**: Windows 10/11 (x86_64)
> **服务端**: Linux x86_64 (SQLite, 单文件部署)

---

## 目录

- [一、项目概述](#一项目概述)
- [二、技术选型](#二技术选型)
- [三、项目结构](#三项目结构)
- [四、数据模型设计](#四数据模型设计)
- [五、核心功能实现](#五核心功能实现)
- [六、全局快捷键与剪贴板（纯本地）](#六全局快捷键与剪贴板纯本地)
- [七、前端布局实现](#七前端布局实现)
- [八、服务端实现](#八服务端实现)
- [九、CI/CD 配置](#九cicd-配置)
- [十、部署指南](#十部署指南)
- [十一、开发命令](#十一开发命令)
- [十二、开发检查清单](#十二开发检查清单)
- [十三、常见问题](#十三常见问题)
- [十四、总结](#十四总结)

---

## 一、项目概述

**MyNote** 是一款使用 Tauri 2.0 + Node.js 构建的轻量级专业笔记程序，提供安全、高效、体验一致的笔记管理体验。

| 项目信息 | 详情 |
| -------- | ---- |
| 项目名称 | MyNote |
| 版本 | v1.0.0 |
| **客户端** | **Windows 10/11 (x86_64)** |
| **服务端** | **Node.js + Express + SQLite (跨平台)** |
| 协议 | MIT License |

### 核心特性

- 📝 **笔记管理**：树形文件夹结构，支持拖拽排序
- ✨ **WYSIWYG 编辑器**：基于 TipTap，所见即所得
- 📋 **智能剪贴板**：全局快捷键 `Ctrl+Shitf+V` 呼出，纯本地管理，不涉及云端同步
- 🔄 **云同步**：与服务端双向同步（仅笔记和文件夹）
- 👤 **用户系统**：注册/登录/JWT 认证
- 🖼️ **头像管理**：剪裁 + 压缩上传

---

## 二、技术选型

### 2.1 核心技术栈

| 层级 | 技术 | 说明                                        |
| ---- | ---- |-------------------------------------------|
| 桌面框架 | Tauri 2.0 | 轻量、高性能、安全                                 |
| 前端框架 | Vue 3 + TypeScript | 组合式 API，类型安全                              |
| 状态管理 | Pinia | Vue 3 官方推荐                                |
| 样式方案 | TailwindCSS | 现代、实用优先                                   |
| 编辑器内核 | TipTap (ProseMirror) | WYSIWYG 编辑                                |
| 本地数据库 | SQLite (rusqlite) | 客户端本地存储                                   |
| **服务端框架** | **Express** | Node.js 经典 Web 框架                         |
| **服务端数据库** | **SQLite (better-sqlite3 / sqlite3)** | 嵌入式，单文件部署                                 |
| **服务端语言** | **Node.js 18+ / TypeScript** | 高效、生态丰富                                   |
| 认证方案 | JWT + bcrypt | 无状态认证                                     |
| 剪贴板 | arboard | 模仿 Windows 剪贴板访问，增加搜索功能，扩展存储条目数量（纯本地，不同步） |
| 全局快捷键 | tauri-plugin-global-shortcut | 注册系统级热键                                   |

### 2.2 开发工具

```bash
# 必需工具
Node.js 18+         # 服务端运行时 + 前端构建
pnpm 8+             # 包管理器
Rust 1.70+          # 客户端后端 (Tauri)
Tauri CLI 2.0       # 客户端构建工具
Git 2.40+           # 版本控制
SQLite 3            # 本地数据库调试


三、项目结构
3.1 完整目录结构
text
MyNote/
├── .github/workflows/
│   ├── build-client.yml          # Windows 客户端构建
│   └── build-server.yml          # Linux 服务端构建
├── src-tauri/                    # Rust 客户端后端
│   ├── src/
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── note.rs           # 笔记 CRUD
│   │   │   ├── folder.rs         # 文件夹管理
│   │   │   ├── user.rs           # 用户认证
│   │   │   ├── clipboard.rs      # 剪贴板管理（纯本地）
│   │   │   └── sync.rs           # 云同步
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── node.rs
│   │   │   └── user.rs
│   │   ├── infrastructure/
│   │   │   ├── mod.rs
│   │   │   ├── database.rs
│   │   │   └── file_system.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── sync_service.rs
│   │   │   └── clipboard_service.rs   # 剪贴板服务（纯本地）
│   │   └── main.rs
│   ├── migrations/
│   │   └── 20240101000000_init.sql
│   ├── Cargo.toml
│   └── tauri.conf.json
├── server/                       # Node.js Express 服务端
│   ├── src/
│   │   ├── routes/
│   │   │   ├── auth.ts            # 认证路由 (注册/登录)
│   │   │   ├── note.ts            # 笔记 CRUD 路由
│   │   │   ├── user.ts            # 用户信息路由
│   │   │   └── sync.ts            # 同步路由
│   │   ├── middleware/
│   │   │   ├── auth.ts            # JWT 认证中间件
│   │   │   └── rateLimit.ts       # 速率限制
│   │   ├── services/
│   │   │   ├── db.ts              # SQLite 数据库连接
│   │   │   ├── userService.ts     # 用户业务逻辑
│   │   │   └── noteService.ts     # 笔记业务逻辑
│   │   ├── utils/
│   │   │   ├── jwt.ts             # JWT 工具函数
│   │   │   └── errors.ts          # 错误处理
│   │   ├── types/
│   │   │   ├── index.ts           # 类型定义
│   │   │   ├── user.ts
│   │   │   └── note.ts
│   │   ├── config.ts              # 配置管理
│   │   └── app.ts                 # Express 入口
│   ├── migrations/
│   │   ├── 001_init.sql
│   │   └── 002_add_avatar.sql
│   ├── uploads/                   # 上传文件目录
│   ├── .env.example
│   ├── package.json
│   ├── tsconfig.json
│   └── nodemon.json               # 开发热重载配置
├── src/                          # 前端 Vue 3 应用
│   ├── components/
│   │   ├── common/
│   │   │   ├── Sidebar.vue
│   │   │   ├── ListView.vue
│   │   │   └── Editor.vue
│   │   ├── editor/
│   │   │   ├── EditorCore.vue
│   │   │   └── EditorToolbar.vue
│   │   ├── clipboard/
│   │   │   └── ClipboardPanel.vue    # 剪贴板面板（纯本地）
│   │   ├── user/
│   │   │   ├── AvatarCropper.vue
│   │   │   └── UserProfile.vue
│   │   └── dialog/
│   │       ├── ConfirmDialog.vue
│   │       └── Toast.vue
│   ├── views/
│   │   ├── MainView.vue
│   │   └── LoginView.vue
│   ├── stores/
│   │   ├── noteStore.ts
│   │   ├── editorStore.ts
│   │   ├── clipboardStore.ts          # 剪贴板 Store（纯本地）
│   │   └── userStore.ts
│   ├── composables/
│   │   ├── useKeyboard.ts
│   │   ├── useDragDrop.ts
│   │   └── useAutoSave.ts
│   ├── utils/
│   │   ├── request.ts
│   │   └── image.ts
│   ├── styles/
│   │   └── main.css
│   ├── App.vue
│   └── main.ts
├── scripts/
│   ├── setup-dev.ps1              # Windows 开发环境配置
│   └── deploy-server.sh           # Linux 服务端部署
├── docs/
│   ├── API.md
│   ├── ARCHITECTURE.md
│   └── DEPLOY.md
├── Cargo.toml                     # 工作区配置
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── tailwind.config.js
├── vite.config.ts
└── README.md
四、数据模型设计
4.1 客户端数据库 (SQLite)
sql
-- nodes 表：统一管理笔记和文件夹
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    node_type TEXT NOT NULL CHECK (node_type IN ('note', 'folder')),
    parent_id TEXT REFERENCES nodes(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    content TEXT,                       -- 笔记内容 (Markdown)
    is_favorite BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_nodes_parent_id ON nodes(parent_id);
CREATE INDEX idx_nodes_sort_order ON nodes(sort_order);

-- users 表
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    email TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- clipboard_history 表（纯本地，不同步到服务端）
CREATE TABLE clipboard_history (
    id TEXT PRIMARY KEY,
    content_type TEXT NOT NULL,         -- text, html, image
    content TEXT,                       -- 文本内容或 Base64
    source_app TEXT,
    is_favorite BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_clipboard_created ON clipboard_history(created_at DESC);

-- sync_queue 表（离线同步队列）
CREATE TABLE sync_queue (
    id TEXT PRIMARY KEY,
    operation TEXT NOT NULL,            -- create, update, delete
    target_type TEXT NOT NULL,          -- note, folder
    target_id TEXT NOT NULL,
    payload TEXT,                       -- JSON 数据
    status TEXT DEFAULT 'pending',      -- pending, synced, failed
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
4.2 服务端数据库 (SQLite)
sql
-- server/migrations/20240101000000_init.sql
-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- 笔记表（树形结构）
CREATE TABLE IF NOT EXISTS notes (
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
);

CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes(user_id);
CREATE INDEX IF NOT EXISTS idx_notes_parent_id ON notes(parent_id);

-- 同步操作记录表
CREATE TABLE IF NOT EXISTS sync_operations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    operation TEXT NOT NULL,            -- create, update, delete
    target_type TEXT NOT NULL,          -- note, folder
    target_id TEXT NOT NULL,
    payload TEXT,                       -- JSON 数据
    client_version INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sync_user_created ON sync_operations(user_id, created_at);

-- server/migrations/20240115000000_add_avatar.sql
ALTER TABLE users ADD COLUMN avatar_url TEXT;
五、核心功能实现
5.1 客户端 Cargo.toml
toml
# src-tauri/Cargo.toml
[package]
name = "mynote"
version = "1.0.0"
edition = "2021"

[lib]
name = "mynote_lib"
crate-type = ["cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["api-all"] }
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"

# 数据库 - SQLite
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono"] }

# 剪贴板
arboard = "3"

# Windows 特定依赖（统一使用 windows crate）
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Input",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
] }
5.2 服务端 package.json
json
{
  "name": "mynote-server",
  "version": "1.0.0",
  "description": "MyNote 笔记服务端 - Express + SQLite",
  "main": "dist/app.js",
  "scripts": {
    "dev": "tsx watch src/app.ts",
    "build": "tsc",
    "start": "node dist/app.js",
    "migrate": "tsx src/services/db.ts",
    "test": "jest --passWithNoTests",
    "lint": "eslint src/"
  },
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5",
    "helmet": "^7.1.0",
    "morgan": "^1.10.0",
    "better-sqlite3": "^9.4.3",
    "bcryptjs": "^2.4.3",
    "jsonwebtoken": "^9.0.2",
    "multer": "^1.4.5-lts.1",
    "sharp": "^0.33.2",
    "uuid": "^9.0.0",
    "dotenv": "^16.3.1"
  },
  "devDependencies": {
    "@types/express": "^4.17.21",
    "@types/better-sqlite3": "^7.6.8",
    "@types/bcryptjs": "^2.4.6",
    "@types/jsonwebtoken": "^9.0.5",
    "@types/multer": "^1.4.11",
    "@types/morgan": "^1.9.9",
    "@types/uuid": "^9.0.7",
    "@types/cors": "^2.8.17",
    "@types/node": "^20.10.0",
    "typescript": "^5.3.2",
    "tsx": "^4.6.2",
    "eslint": "^8.55.0",
    "@typescript-eslint/eslint-plugin": "^6.13.1",
    "@typescript-eslint/parser": "^6.13.1",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.1"
  }
}
5.3 服务端 tsconfig.json
json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
5.4 客户端 - 统一节点模型 (Rust)
rust
// src-tauri/src/domain/node.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub content: Option<String>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Note,
    Folder,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeDto {
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeDto {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
    pub content: Option<String>,
    pub is_favorite: Option<bool>,
}
5.4 客户端 - 核心 Tauri 命令
rust
// src-tauri/src/commands/note.rs
use tauri::State;
use crate::domain::node::{Node, CreateNodeDto, UpdateNodeDto};
use crate::infrastructure::database::AppState;

// 获取子节点列表
#[tauri::command]
pub async fn get_children(
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Node>, String> {
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT * FROM nodes WHERE parent_id = ? ORDER BY sort_order ASC"
    )
    .bind(parent_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(nodes)
}

// 创建节点
#[tauri::command]
pub async fn create_node(
    dto: CreateNodeDto,
    state: State<'_, AppState>,
) -> Result<Node, String> {
    let id = Uuid::new_v4().to_string();
  
    let max_order: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM nodes WHERE parent_id = ?"
    )
    .bind(&dto.parent_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
  
    let sort_order = max_order.unwrap_or(0) + 1;
  
    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, name, node_type, parent_id, sort_order, content)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&id)
    .bind(&dto.name)
    .bind(&dto.node_type)
    .bind(&dto.parent_id)
    .bind(sort_order)
    .bind(&dto.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;
  
    Ok(node)
}

// 移动节点（拖拽排序）
#[tauri::command]
pub async fn move_node(
    node_id: String,
    new_parent_id: Option<String>,
    new_sort_order: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
  
    sqlx::query(
        "UPDATE nodes SET parent_id = ?, sort_order = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(new_parent_id)
    .bind(new_sort_order)
    .bind(node_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
  
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// 保存笔记内容
#[tauri::command]
pub async fn save_note_content(
    node_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE nodes SET content = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(content)
    .bind(node_id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
5.5 客户端 - 编辑器状态管理 (Vue 3)
typescript
// src/stores/editorStore.ts
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useNoteStore } from './noteStore'

export const useEditorStore = defineStore('editor', {
  state: () => ({
    selectedNodeId: null as string | null,
    content: '',
    isDirty: false,
    isSaving: false,
    mode: 'wysiwyg' as 'wysiwyg' | 'source',
  }),
  
  getters: {
    hasSelected: (state) => state.selectedNodeId !== null,
    isEmpty: (state) => state.content === '',
  },
  
  actions: {
    async selectNote(nodeId: string) {
      if (this.isDirty && this.selectedNodeId) {
        await this.saveCurrentNote()
      }
    
      try {
        const content = await invoke<string>('load_note_content', { nodeId })
        this.selectedNodeId = nodeId
        this.content = content
        this.isDirty = false
      } catch (error) {
        console.error('加载笔记失败:', error)
      }
    },
  
    async saveCurrentNote() {
      if (!this.selectedNodeId || this.isSaving) return
    
      this.isSaving = true
      try {
        await invoke('save_note_content', {
          nodeId: this.selectedNodeId,
          content: this.content,
        })
        this.isDirty = false
        await this.checkAndSync()
      } catch (error) {
        console.error('保存失败:', error)
      } finally {
        this.isSaving = false
      }
    },
  
    updateContent(newContent: string) {
      this.content = newContent
      this.isDirty = true
    },
  
    async checkAndSync() {
      // 触发云同步检查
    },
  },
})
六、全局快捷键与剪贴板（纯本地）
6.1 核心原理
MyNote 的剪贴板面板为纯本地功能，数据不离开本地设备，通过以下机制实现"不抢夺当前窗口焦点"：

机制	说明
全局快捷键监听	通过 tauri-plugin-global-shortcut 注册 Ctrl+Shift+v，操作系统级监听
无焦点窗口	窗口配置 focus: false，不请求键盘焦点
模拟粘贴	使用 Win32 SendInput API 模拟 Ctrl+V，直接发送到当前活动窗口
失焦自动隐藏	窗口失去焦点时自动关闭，不干扰用户
6.2 Tauri 窗口配置
json
// src-tauri/tauri.conf.json（剪贴板窗口部分）
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "MyNote",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "decorations": true,
        "fullscreen": false,
        "resizable": true,
        "center": true
      },
      {
        "label": "clipboard",
        "title": "剪贴板",
        "width": 400,
        "height": 500,
        "decorations": false,
        "transparent": true,
        "always_on_top": true,
        "skip_taskbar": true,
        "focus": false,
        "resizable": false,
        "visible": false
      }
    ]
  },
  "plugins": {
    "global-shortcut": {
      "shortcuts": [
        { "key": "Ctrl+Shift+V", "command": "toggle_clipboard_panel" }
      ]
    }
  }
}
6.3 剪贴板服务
rust
// src-tauri/src/services/clipboard_service.rs
use arboard::Clipboard;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ClipboardService {
    clipboard: Arc<Mutex<Clipboard>>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
        }
    }

    pub async fn get_text(&self) -> Result<String, String> {
        let mut clipboard = self.clipboard.lock().await;
        clipboard.get_text().map_err(|e| e.to_string())
    }

    pub async fn set_text(&self, text: &str) -> Result<(), String> {
        let mut clipboard = self.clipboard.lock().await;
        clipboard.set_text(text).map_err(|e| e.to_string())
    }
}
6.4 Windows 模拟粘贴
rust
// src-tauri/src/commands/clipboard.rs
use tauri::State;
use crate::services::clipboard_service::ClipboardService;

/// Windows：模拟 Ctrl+V 粘贴到当前活动窗口（不切换焦点）
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn paste_to_active_window(
    content: Option<String>,
    state: State<'_, ClipboardService>,
) -> Result<(), String> {
    use windows::Win32::UI::Input::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        VK_CONTROL, VK_V,
    };
    use windows::Win32::UI::Input::INPUT_0;

    if let Some(text) = content {
        state.set_text(&text).await?;
    }

    unsafe {
        let down_control = INPUT {
            r#type: windows::Win32::UI::Input::INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL.0,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let down_v = INPUT {
            r#type: windows::Win32::UI::Input::INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V.0,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let up_v = INPUT {
            r#type: windows::Win32::UI::Input::INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V.0,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP.0,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let up_control = INPUT {
            r#type: windows::Win32::UI::Input::INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL.0,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP.0,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let inputs = [down_control, down_v, up_v, up_control];
        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
            return Err("SendInput failed".to_string());
        }
    }
    Ok(())
}

/// 获取当前光标位置（用于定位剪贴板弹出窗口，使用 windows crate）
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_cursor_position() -> Result<(i32, i32), String> {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::Foundation::POINT;
    
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point).is_ok() {
            Ok((point.x, point.y))
        } else {
            Err("获取光标位置失败".to_string())
        }
    }
}
6.5 前端剪贴板 Store
typescript
// src/stores/clipboardStore.ts
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'

interface ClipboardItem {
  id: string
  content_type: string
  content: string
  source_app?: string
  is_favorite: boolean
  created_at: string
  last_used_at: string
}

export const useClipboardStore = defineStore('clipboard', {
  state: () => ({
    items: [] as ClipboardItem[],
    isPanelVisible: false,
    searchQuery: '',
  }),

  getters: {
    filteredItems(state) {
      if (!state.searchQuery) return state.items
      const q = state.searchQuery.toLowerCase()
      return state.items.filter(item =>
        item.content.toLowerCase().includes(q)
      )
    },
  },

  actions: {
    async togglePanel() {
      const clipboardWindow = getCurrentWindow('clipboard')

      if (this.isPanelVisible) {
        await clipboardWindow.hide()
        this.isPanelVisible = false
      } else {
        try {
          const [x, y] = await invoke<[number, number]>('get_cursor_position')
          await clipboardWindow.setPosition(
            new LogicalPosition(x - 200, y - 150)
          )
        } catch {
          await clipboardWindow.center()
        }
        await clipboardWindow.show()
        this.isPanelVisible = true
        await this.loadHistory()
      }
    },

    async loadHistory() {
      try {
        const items = await invoke<ClipboardItem[]>('get_clipboard_history')
        this.items = items
      } catch (error) {
        console.error('加载剪贴板历史失败:', error)
      }
    },

    async pasteItem(item: ClipboardItem) {
      try {
        await invoke('paste_to_active_window', { content: item.content })
        item.last_used_at = new Date().toISOString()
        await this.hidePanel()
      } catch (error) {
        console.error('粘贴失败:', error)
      }
    },

    async hidePanel() {
      const window = getCurrentWindow('clipboard')
      await window.hide()
      this.isPanelVisible = false
    },

    async clearHistory() {
      await invoke('clear_clipboard_history')
      this.items = []
    },

    setupListeners() {
      listen('toggle-clipboard', () => {
        this.togglePanel()
      })
      const window = getCurrentWindow('clipboard')
      window.onFocusChanged(({ payload }) => {
        if (!payload && this.isPanelVisible) {
          this.hidePanel()
        }
      })
    },
  },
})
---
**注意**：必须在应用入口（如 `App.vue` 的 `onMounted` 或 `main.ts`）调用一次 `clipboardStore.setupListeners()`，否则全局快捷键 Alt+C 不会触发面板。
---
6.6 剪贴板面板组件
vue
<!-- src/components/clipboard/ClipboardPanel.vue -->
<template>
  <div class="clipboard-panel" ref="panelRef" @keydown.esc="hidePanel">
    <div class="panel-header">
      <input v-model="searchQuery" placeholder="搜索剪贴板..." class="search-input" ref="searchInput" @click.stop />
      <span class="item-count">{{ filteredItems.length }}</span>
    </div>
    <div class="panel-body">
      <div v-for="item in filteredItems" :key="item.id" class="clipboard-item" @click="pasteItem(item)" @mousedown.prevent>
        <div class="item-content">{{ item.content }}</div>
        <div class="item-meta">
          <span>{{ item.source_app || '未知来源' }}</span>
          <span>{{ formatTime(item.created_at) }}</span>
        </div>
      </div>
      <div v-if="filteredItems.length === 0" class="empty-state">暂无剪贴板历史</div>
    </div>
    <div class="panel-footer">
      <button @click.stop="clearHistory" class="clear-btn">清空全部</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useClipboardStore } from '@/stores/clipboardStore'
import { onClickOutside } from '@vueuse/core'

const store = useClipboardStore()
const { isPanelVisible, searchQuery, filteredItems } = storeToRefs(store)
const { pasteItem, hidePanel, clearHistory } = store

const searchInput = ref<HTMLInputElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)

onClickOutside(panelRef, () => {
  if (isPanelVisible.value) hidePanel()
})

watch(isPanelVisible, (visible) => {
  if (visible) nextTick(() => searchInput.value?.focus())
})

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return date.toLocaleDateString()
}
</script>

<style scoped>
.clipboard-panel {
  width: 400px; max-height: 500px; background: var(--bg-primary);
  border-radius: 12px; box-shadow: 0 12px 48px rgba(0,0,0,0.3);
  display: flex; flex-direction: column; overflow: hidden;
  border: 1px solid var(--border-color); backdrop-filter: blur(20px);
  background: rgba(255,255,255,0.95);
}
.dark .clipboard-panel { background: rgba(30,30,30,0.95); }
.panel-header { padding: 12px 16px; border-bottom: 1px solid var(--border-color); display: flex; align-items: center; gap: 8px; }
.search-input { flex: 1; border: none; outline: none; background: transparent; font-size: 14px; color: var(--text-primary); }
.search-input::placeholder { color: var(--text-secondary); }
.item-count { font-size: 12px; color: var(--text-secondary); }
.panel-body { flex: 1; overflow-y: auto; padding: 8px; }
.clipboard-item { padding: 10px 12px; border-radius: 8px; cursor: pointer; transition: background 0.15s; margin-bottom: 2px; }
.clipboard-item:hover { background: var(--bg-hover); }
.item-content { font-size: 14px; color: var(--text-primary); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; word-break: break-all; }
.item-meta { display: flex; justify-content: space-between; margin-top: 4px; font-size: 12px; color: var(--text-secondary); }
.empty-state { text-align: center; color: var(--text-secondary); padding: 40px 0; font-size: 14px; }
.panel-footer { padding: 8px 16px; border-top: 1px solid var(--border-color); display: flex; justify-content: flex-end; }
.clear-btn { background: none; border: none; color: var(--text-secondary); font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px; }
.clear-btn:hover { background: var(--bg-hover); color: var(--danger-color); }
.panel-body::-webkit-scrollbar { width: 4px; }
.panel-body::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 4px; }
</style>
七、前端布局实现
7.1 三栏布局结构
vue
<!-- src/views/MainView.vue -->
<template>
  <div class="app-layout">
    <Sidebar />
    <ListView />
    <Editor />
    <!-- 剪贴板面板为独立窗口，由全局快捷键 Ctrl+Shift+V 触发 -->
  </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/common/Sidebar.vue'
import ListView from '@/components/common/ListView.vue'
import Editor from '@/components/common/Editor.vue'
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--bg-primary);
}
</style>
7.2 侧边栏组件
vue
<!-- src/components/common/Sidebar.vue -->
<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <AvatarCropper 
        :user-id="userId" 
        @avatar-updated="handleAvatarUpdate"
      />
    </div>
  
    <nav class="sidebar-nav">
      <button 
        v-for="tab in tabs" 
        :key="tab.key"
        class="nav-item"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        <component :is="tab.icon" class="nav-icon" />
        <span class="nav-label">{{ tab.label }}</span>
      </button>
    </nav>
  
    <div class="sidebar-footer">
      <button class="nav-item" @click="openSettings">
        <SettingsIcon class="nav-icon" />
        <span class="nav-label">设置</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NotebookIcon, WrenchIcon, SettingsIcon } from 'lucide-vue-next'
import AvatarCropper from '@/components/user/AvatarCropper.vue'

const activeTab = ref('notes')
const userId = ref('current-user-id')

const tabs = [
  { key: 'notes', label: '笔记', icon: NotebookIcon },
  { key: 'tools', label: '工具', icon: WrenchIcon },
]

const handleAvatarUpdate = (url: string) => {
  console.log('头像已更新:', url)
}

const openSettings = () => {
  // 打开设置弹窗
}
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  width: 72px;
  height: 100%;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  padding: 16px 0;
  align-items: center;
  flex-shrink: 0;
}

.sidebar-header {
  margin-bottom: 24px;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s;
  width: 100%;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--bg-active);
  color: var(--primary-color);
}

.nav-icon {
  width: 24px;
  height: 24px;
}

.nav-label {
  font-size: 11px;
}

.sidebar-footer {
  margin-top: auto;
}
</style>
7.3 头像剪裁组件
vue
<!-- src/components/user/AvatarCropper.vue -->
<template>
  <div class="avatar-container">
    <div class="avatar-wrapper" @click="triggerFileSelect">
      <img 
        v-if="avatarUrl" 
        :src="avatarUrl" 
        alt="头像"
        class="avatar-image"
      />
      <div v-else class="avatar-placeholder">
        {{ userInitials }}
      </div>
      <div class="avatar-overlay">
        <CameraIcon class="camera-icon" />
      </div>
    </div>
  
    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      style="display: none"
      @change="handleFileSelect"
    />
  
    <!-- 剪裁弹窗 -->
    <Teleport to="body">
      <div v-if="showCropper" class="cropper-modal" @click.self="closeCropper">
        <div class="cropper-content">
          <div class="cropper-header">
            <h3>剪裁头像</h3>
            <button @click="closeCropper" class="close-btn">✕</button>
          </div>
        
          <div class="cropper-body">
            <img ref="imageRef" :src="imageUrl" style="max-width: 100%;" />
          </div>
        
          <div class="cropper-footer">
            <button @click="closeCropper" class="btn-cancel">取消</button>
            <button @click="confirmCrop" class="btn-confirm">确认剪裁</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Cropper from 'cropperjs'
import 'cropperjs/dist/cropper.css'
import { CameraIcon } from 'lucide-vue-next'

const props = defineProps<{
  userId: string
  initialAvatar?: string
}>()

const emit = defineEmits<{
  (e: 'avatar-updated', url: string): void
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const imageRef = ref<HTMLImageElement | null>(null)
const avatarUrl = ref(props.initialAvatar || '')
const showCropper = ref(false)
const imageUrl = ref('')
let cropper: Cropper | null = null

const userInitials = computed(() => 'MY')

const triggerFileSelect = () => {
  fileInput.value?.click()
}

const handleFileSelect = (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file && file.type.startsWith('image/')) {
    const reader = new FileReader()
    reader.onload = (e) => {
      imageUrl.value = e.target?.result as string
      showCropper.value = true
      nextTick(() => {
        if (imageRef.value) {
          cropper = new Cropper(imageRef.value, {
            aspectRatio: 1,
            viewMode: 1,
            autoCropArea: 0.8,
          })
        }
      })
    }
    reader.readAsDataURL(file)
  }
  input.value = ''
}

const confirmCrop = async () => {
  if (!cropper) return
  
  try {
    const canvas = cropper.getCroppedCanvas({ width: 256, height: 256 })
    const blob = await new Promise<Blob>((resolve) => {
      canvas.toBlob((b) => resolve(b!), 'image/webp', 0.8)
    })
  
    const base64 = await new Promise<string>((resolve) => {
      const reader = new FileReader()
      reader.readAsDataURL(blob)
      reader.onload = () => resolve(reader.result as string)
    })
  
    const url = await invoke<string>('upload_avatar', {
      userId: props.userId,
      imageData: base64,
    })
  
    avatarUrl.value = url
    emit('avatar-updated', url)
    closeCropper()
  } catch (error) {
    console.error('上传失败:', error)
  }
}

const closeCropper = () => {
  showCropper.value = false
  cropper?.destroy()
  cropper = null
}
</script>

<style scoped>
.avatar-wrapper {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  overflow: hidden;
  cursor: pointer;
  border: 2px solid var(--border-color);
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-color);
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.avatar-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.avatar-wrapper:hover .avatar-overlay {
  opacity: 1;
}

.camera-icon {
  width: 20px;
  height: 20px;
  color: white;
}

/* 剪裁弹窗样式 */
.cropper-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.cropper-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 600px;
  width: 90%;
  max-height: 90vh;
}

.cropper-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.cropper-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 16px;
}
</style>
八、服务端实现
8.1 服务端配置 (TypeScript)
typescript
// server/src/config.ts
import dotenv from 'dotenv';
import path from 'path';

dotenv.config();

export const config = {
  port: parseInt(process.env.PORT || '8080', 10),
  dataDir: path.resolve(process.env.DATA_DIR || './data'),
  jwtSecret: process.env.JWT_SECRET || 'your-secret-key-change-me-in-production',
  jwtExpireDays: parseInt(process.env.JWT_EXPIRE_DAYS || '7', 10),
  logLevel: process.env.LOG_LEVEL || 'dev',
  maxUploadSize: (parseInt(process.env.MAX_UPLOAD_SIZE_MB || '10', 10)) * 1024 * 1024,
};

export type Config = typeof config;
8.2 SQLite 数据库服务 (TypeScript)
typescript
// server/src/services/db.ts
import Database from 'better-sqlite3';
import path from 'path';
import fs from 'fs';
import { config } from '../config';

let db: Database.Database;

export function getDb(): Database.Database {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

export function initDatabase(): Database.Database {
  // 确保数据目录存在
  fs.mkdirSync(config.dataDir, { recursive: true });

  const dbPath = path.join(config.dataDir, 'mynote.db');
  console.log(`📂 数据库路径: ${dbPath}`);

  db = new Database(dbPath);

  // 启用 WAL 模式提升并发性能
  db.pragma('journal_mode = WAL');
  db.pragma('synchronous = NORMAL');
  db.pragma('busy_timeout = 10000');
  db.pragma('foreign_keys = ON');

  // 运行数据库迁移
  runMigrations();

  console.log('✅ 数据库已连接: mynote.db');
  return db;
}

function runMigrations() {
  // 创建迁移记录表
  db.exec(`
    CREATE TABLE IF NOT EXISTS _migrations (
      version TEXT PRIMARY KEY,
      applied_at TEXT DEFAULT (datetime('now'))
    )
  `);

  const migrations: { version: string; sql: string }[] = [
    {
      version: '001',
      sql: `
        -- 用户表
        CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          username TEXT UNIQUE NOT NULL,
          email TEXT UNIQUE,
          password_hash TEXT NOT NULL,
          avatar_url TEXT,
          created_at TEXT DEFAULT (datetime('now')),
          updated_at TEXT DEFAULT (datetime('now'))
        );

        -- 笔记表（树形结构）
        CREATE TABLE IF NOT EXISTS notes (
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
        );

        CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes(user_id);
        CREATE INDEX IF NOT EXISTS idx_notes_parent_id ON notes(parent_id);

        -- 同步操作记录表
        CREATE TABLE IF NOT EXISTS sync_operations (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          operation TEXT NOT NULL,
          target_type TEXT NOT NULL,
          target_id TEXT NOT NULL,
          payload TEXT,
          client_version INTEGER,
          created_at TEXT DEFAULT (datetime('now')),
          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_sync_user_created ON sync_operations(user_id, created_at);
      `,
    },
    {
      version: '002',
      sql: `
        ALTER TABLE users ADD COLUMN avatar_url TEXT;
      `,
    },
  ];

  const applied = new Set(
    db
      .prepare('SELECT version FROM _migrations')
      .all()
      .map((row: any) => row.version)
  );

  for (const m of migrations) {
    if (!applied.has(m.version)) {
      console.log(`📦 应用迁移: ${m.version}`);
      const runMigration = db.transaction(() => {
        // 对 DDL，better-sqlite3 需要逐个语句执行
        const statements = m.sql
          .split(';')
          .map((s) => s.trim())
          .filter((s) => s.length > 0);
        for (const stmt of statements) {
          db.exec(stmt + ';');
        }
        db.prepare('INSERT INTO _migrations (version) VALUES (?)').run(m.version);
      });
      runMigration();
    }
  }
}
8.3 服务端主入口 (TypeScript)
typescript
// server/src/app.ts
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import { initDatabase } from './services/db';
import { config } from './config';
import authRoutes from './routes/auth';
import noteRoutes from './routes/note';
import userRoutes from './routes/user';
import syncRoutes from './routes/sync';

const app = express();

// 初始化数据库
initDatabase();

// 中间件
// 注意：Tauri 客户端已通过 tauri.conf.json 配置 CSP，
// 服务端 helmet 需放宽内联样式限制以兼容 Tauri 前端
app.use(helmet({
  contentSecurityPolicy: false,  // 由 Tauri 客户端控制 CSP
  crossOriginEmbedderPolicy: false,
}));
app.use(cors({
  origin: '*',
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
  allowedHeaders: ['Content-Type', 'Authorization'],
}));
app.use(morgan(config.logLevel));
app.use(express.json({ limit: '10mb' }));
app.use('/uploads', express.static('uploads'));

// 健康检查
app.get('/health', (_req, res) => {
  res.json({ status: 'ok', version: '1.0.0' });
});

// 路由
// 注意：同步路由必须在笔记路由之前注册，避免 /api/notes/sync 被 /api/notes/:id 捕获
app.use('/api/auth', authRoutes);
app.use('/api/notes/sync', syncRoutes);
app.use('/api/notes', noteRoutes);
app.use('/api/user', userRoutes);

// 全局错误处理
app.use((err: any, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
  console.error('未捕获错误:', err);
  res.status(err.status || 500).json({
    error: err.message || '服务器内部错误',
  });
});

// 启动服务器
app.listen(config.port, () => {
  console.log(`🚀 MyNote Server v1.0.0`);
  console.log(`📂 数据目录: ${config.dataDir}`);
  console.log(`🌐 监听: http://0.0.0.0:${config.port}`);
});

export default app;
8.4 JWT 工具函数
typescript
// server/src/utils/jwt.ts
import jwt from 'jsonwebtoken';
import { config } from '../config';

export interface JwtPayload {
  sub: string;
  exp: number;
}

export function generateToken(userId: string): string {
  const payload: JwtPayload = {
    sub: userId,
    exp: Math.floor(Date.now() / 1000) + config.jwtExpireDays * 24 * 60 * 60,
  };
  return jwt.sign(payload, config.jwtSecret);
}

export function verifyToken(token: string): JwtPayload {
  return jwt.verify(token, config.jwtSecret) as JwtPayload;
}
8.5 类型定义
typescript
// server/src/types/index.ts
export interface User {
  id: string;
  username: string;
  email?: string;
  password_hash: string;
  avatar_url?: string;
  created_at: string;
  updated_at: string;
}

export interface Note {
  id: string;
  user_id: string;
  parent_id?: string;
  title: string;
  content?: string;
  is_folder: number;
  sort_order: number;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface SyncOperation {
  id: string;
  user_id: string;
  operation: 'create' | 'update' | 'delete';
  target_type: 'note' | 'folder';
  target_id: string;
  payload?: string;
  client_version?: number;
  created_at: string;
}
8.6 服务端认证路由
typescript
// server/src/routes/auth.ts
import { Router, Request, Response } from 'express';
import bcrypt from 'bcryptjs';
import { v4 as uuidv4 } from 'uuid';
import { getDb } from '../services/db';
import { generateToken } from '../utils/jwt';

const router = Router();

// POST /api/auth/register
router.post('/register', (req: Request, res: Response) => {
  try {
    const { username, password, email } = req.body;

    // 参数校验
    if (!username || !password) {
      return res.status(400).json({ error: '用户名和密码不能为空' });
    }
    if (username.length < 2 || username.length > 32) {
      return res.status(400).json({ error: '用户名长度需在 2-32 个字符之间' });
    }
    if (password.length < 6) {
      return res.status(400).json({ error: '密码长度不能少于 6 个字符' });
    }

    const db = getDb();

    // 检查用户名是否已存在
    const existing = db.prepare('SELECT id FROM users WHERE username = ?').get(username);
    if (existing) {
      return res.status(409).json({ error: '用户名已存在' });
    }

    // 哈希密码
    const salt = bcrypt.genSaltSync(10);
    const passwordHash = bcrypt.hashSync(password, salt);

    // 创建用户
    const userId = uuidv4();
    db.prepare(
      'INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)'
    ).run(userId, username, email || null, passwordHash);

    // 生成 JWT
    const token = generateToken(userId);

    res.status(201).json({
      token,
      user_id: userId,
      username,
      email: email || null,
    });
  } catch (error) {
    console.error('注册失败:', error);
    res.status(500).json({ error: '服务器内部错误' });
  }
});

// POST /api/auth/login
router.post('/login', (req: Request, res: Response) => {
  try {
    const { username, password } = req.body;

    if (!username || !password) {
      return res.status(400).json({ error: '用户名和密码不能为空' });
    }

    const db = getDb();

    // 查找用户
    const user = db.prepare(
      'SELECT id, username, email, password_hash FROM users WHERE username = ?'
    ).get(username) as any;

    if (!user) {
      return res.status(401).json({ error: '用户名或密码错误' });
    }

    // 验证密码
    if (!bcrypt.compareSync(password, user.password_hash)) {
      return res.status(401).json({ error: '用户名或密码错误' });
    }

    // 生成 JWT
    const token = generateToken(user.id);

    res.json({
      token,
      user_id: user.id,
      username: user.username,
      email: user.email,
    });
  } catch (error) {
    console.error('登录失败:', error);
    res.status(500).json({ error: '服务器内部错误' });
  }
});

export default router;
8.7 JWT 认证中间件
typescript
// server/src/middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import { verifyToken, JwtPayload } from '../utils/jwt';

export interface AuthRequest extends Request {
  userId?: string;
}

export function authMiddleware(req: AuthRequest, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: '未提供认证令牌' });
  }

  const token = authHeader.substring(7);

  try {
    const payload: JwtPayload = verifyToken(token);
    req.userId = payload.sub;
    next();
  } catch (error) {
    return res.status(401).json({ error: '认证令牌无效或已过期' });
  }
}
8.8 笔记 CRUD 路由
typescript
// server/src/routes/note.ts
import { Router, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import { getDb } from '../services/db';
import { authMiddleware, AuthRequest } from '../middleware/auth';

const router = Router();

// 所有笔记路由都需要认证
router.use(authMiddleware);

// GET /api/notes - 获取笔记列表
router.get('/', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb();
    const notes = db.prepare(
      'SELECT * FROM notes WHERE user_id = ? ORDER BY sort_order ASC, updated_at DESC'
    ).all(req.userId);
    res.json(notes);
  } catch (error) {
    console.error('获取笔记列表失败:', error);
    res.status(500).json({ error: '获取笔记列表失败' });
  }
});

// POST /api/notes - 创建笔记/文件夹
router.post('/', (req: AuthRequest, res: Response) => {
  try {
    const { title, parent_id, is_folder, content } = req.body;
    const db = getDb();

    // 计算排序序号
    const maxOrder = db.prepare(
      'SELECT MAX(sort_order) as max_order FROM notes WHERE user_id = ? AND parent_id IS ?'
    ).get(req.userId, parent_id || null) as any;

    const sortOrder = (maxOrder?.max_order || 0) + 1;
    const id = uuidv4();

    db.prepare(`
      INSERT INTO notes (id, user_id, title, parent_id, is_folder, sort_order, content)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `).run(id, req.userId, title, parent_id || null, is_folder ? 1 : 0, sortOrder, content || null);

    const note = db.prepare('SELECT * FROM notes WHERE id = ?').get(id);
    res.status(201).json(note);
  } catch (error) {
    console.error('创建笔记失败:', error);
    res.status(500).json({ error: '创建笔记失败' });
  }
});

// GET /api/notes/:id - 获取单个笔记
router.get('/:id', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb();
    const note = db.prepare(
      'SELECT * FROM notes WHERE id = ? AND user_id = ?'
    ).get(req.params.id, req.userId);

    if (!note) {
      return res.status(404).json({ error: '笔记不存在' });
    }
    res.json(note);
  } catch (error) {
    console.error('获取笔记失败:', error);
    res.status(500).json({ error: '获取笔记失败' });
  }
});

// PUT /api/notes/:id - 更新笔记
router.put('/:id', (req: AuthRequest, res: Response) => {
  try {
    const { title, content, parent_id, sort_order, is_folder } = req.body;
    const db = getDb();

    const existing = db.prepare(
      'SELECT * FROM notes WHERE id = ? AND user_id = ?'
    ).get(req.params.id, req.userId);

    if (!existing) {
      return res.status(404).json({ error: '笔记不存在' });
    }

    db.prepare(`
      UPDATE notes SET
        title = COALESCE(?, title),
        content = COALESCE(?, content),
        parent_id = COALESCE(?, parent_id),
        sort_order = COALESCE(?, sort_order),
        is_folder = COALESCE(?, is_folder),
        version = version + 1,
        updated_at = datetime('now')
      WHERE id = ? AND user_id = ?
    `).run(
      title || null,
      content !== undefined ? content : null,
      parent_id !== undefined ? parent_id : null,
      sort_order !== undefined ? sort_order : null,
      is_folder !== undefined ? (is_folder ? 1 : 0) : null,
      req.params.id,
      req.userId
    );

    const updated = db.prepare('SELECT * FROM notes WHERE id = ?').get(req.params.id);
    res.json(updated);
  } catch (error) {
    console.error('更新笔记失败:', error);
    res.status(500).json({ error: '更新笔记失败' });
  }
});

// DELETE /api/notes/:id - 删除笔记
router.delete('/:id', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb();
    const result = db.prepare(
      'DELETE FROM notes WHERE id = ? AND user_id = ?'
    ).run(req.params.id, req.userId);

    if (result.changes === 0) {
      return res.status(404).json({ error: '笔记不存在' });
    }
    res.json({ message: '删除成功' });
  } catch (error) {
    console.error('删除笔记失败:', error);
    res.status(500).json({ error: '删除笔记失败' });
  }
});

export default router;
8.9 同步路由
typescript
// server/src/routes/sync.ts
import { Router, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import { getDb } from '../services/db';
import { authMiddleware, AuthRequest } from '../middleware/auth';

const router = Router();

router.use(authMiddleware);

// POST /api/notes/sync - 同步操作（含版本冲突检测：Last-Writer-Wins）
router.post('/', (req: AuthRequest, res: Response) => {
  try {
    const { operations } = req.body;
    const db = getDb();

    if (!Array.isArray(operations)) {
      return res.status(400).json({ error: 'operations 必须是数组' });
    }

    const results: any[] = [];
    const sync = db.transaction(() => {
      for (const op of operations) {
        const { operation, target_type, target_id, payload, client_version } = op;

        // --- 冲突检测：仅当客户端版本 >= 服务端版本时才应用更新 ---
        if ((operation === 'create' || operation === 'update') && payload) {
          const existing = db.prepare(
            'SELECT version FROM notes WHERE id = ? AND user_id = ?'
          ).get(target_id, req.userId) as { version: number } | undefined;

          if (existing && client_version !== undefined && client_version < existing.version) {
            // 服务端版本更新，跳过此操作，返回服务端最新数据
            const serverNote = db.prepare('SELECT * FROM notes WHERE id = ?').get(target_id);
            results.push({
              target_id,
              status: 'conflict',
              server_version: existing.version,
              server_data: serverNote,
            });
            // 仍然记录同步操作但不应用
            const syncId = uuidv4();
            db.prepare(`
              INSERT INTO sync_operations (id, user_id, operation, target_type, target_id, payload)
              VALUES (?, ?, 'conflict_skip', ?, ?, ?)
            `).run(syncId, req.userId, target_type, target_id, JSON.stringify(payload));
            return; // 跳过此操作
          }
        }

        // 记录同步操作
        const syncId = uuidv4();
        db.prepare(`
          INSERT INTO sync_operations (id, user_id, operation, target_type, target_id, payload)
          VALUES (?, ?, ?, ?, ?, ?)
        `).run(syncId, req.userId, operation, target_type, target_id, JSON.stringify(payload));

        // 应用操作到本地数据库
        switch (operation) {
          case 'create':
          case 'update': {
            if (payload) {
              db.prepare(`
                INSERT INTO notes (id, user_id, title, content, parent_id, is_folder, sort_order, version)
                VALUES (@id, @user_id, @title, @content, @parent_id, @is_folder, @sort_order, @version)
                ON CONFLICT(id) DO UPDATE SET
                  title = COALESCE(@title, title),
                  content = COALESCE(@content, content),
                  parent_id = @parent_id,
                  sort_order = COALESCE(@sort_order, sort_order),
                  version = version + 1,
                  updated_at = datetime('now')
              `).run({
                ...payload,
                user_id: req.userId,
              });
            }
            break;
          }
          case 'delete': {
            db.prepare('DELETE FROM notes WHERE id = ? AND user_id = ?')
              .run(target_id, req.userId);
            break;
          }
        }

        results.push({ syncId, target_id, status: 'synced' });
      }
    });

    sync();

    // 获取服务端最新变更
    const serverOps = db.prepare(`
      SELECT * FROM sync_operations
      WHERE user_id = ? AND created_at > datetime('now', '-1 hour')
      ORDER BY created_at ASC
    `).all(req.userId);

    res.json({ results, serverOps });
  } catch (error) {
    console.error('同步失败:', error);
    res.status(500).json({ error: '同步失败' });
  }
});

export default router;
8.10 用户信息路由
typescript
// server/src/routes/user.ts
import { Router, Response } from 'express';
import multer from 'multer';
import path from 'path';
import fs from 'fs';
import { v4 as uuidv4 } from 'uuid';
import { getDb } from '../services/db';
import { authMiddleware, AuthRequest } from '../middleware/auth';
import { config } from '../config';

const router = Router();

// 所有用户路由都需要认证
router.use(authMiddleware);

// 配置文件上传
const storage = multer.diskStorage({
  destination: (_req, _file, cb) => {
    const uploadDir = path.join(config.dataDir, '../uploads/avatars');
    fs.mkdirSync(uploadDir, { recursive: true });
    cb(null, uploadDir);
  },
  filename: (_req, file, cb) => {
    const ext = path.extname(file.originalname) || '.jpg';
    cb(null, `${uuidv4()}${ext}`);
  },
});

const upload = multer({
  storage,
  limits: { fileSize: config.maxUploadSize },
  fileFilter: (_req, file, cb) => {
    const allowedTypes = ['image/jpeg', 'image/png', 'image/webp', 'image/gif'];
    if (allowedTypes.includes(file.mimetype)) {
      cb(null, true);
    } else {
      cb(new Error('不支持的文件类型，仅支持 JPEG、PNG、WebP、GIF'));
    }
  },
});

// GET /api/user/profile - 获取用户信息
router.get('/profile', (req: AuthRequest, res: Response) => {
  try {
    const db = getDb();
    const user = db.prepare(
      'SELECT id, username, email, avatar_url, created_at FROM users WHERE id = ?'
    ).get(req.userId);

    if (!user) {
      return res.status(404).json({ error: '用户不存在' });
    }
    res.json(user);
  } catch (error) {
    console.error('获取用户信息失败:', error);
    res.status(500).json({ error: '获取用户信息失败' });
  }
});

// PUT /api/user/profile - 更新用户信息
router.put('/profile', (req: AuthRequest, res: Response) => {
  try {
    const { email, username } = req.body;
    const db = getDb();

    db.prepare(`
      UPDATE users SET
        email = COALESCE(?, email),
        username = COALESCE(?, username),
        updated_at = datetime('now')
      WHERE id = ?
    `).run(email || null, username || null, req.userId);

    const user = db.prepare(
      'SELECT id, username, email, avatar_url, created_at FROM users WHERE id = ?'
    ).get(req.userId);

    res.json(user);
  } catch (error) {
    console.error('更新用户信息失败:', error);
    res.status(500).json({ error: '更新用户信息失败' });
  }
});

// POST /api/user/avatar - 上传头像
router.post('/avatar', (req: AuthRequest, res: Response) => {
  upload.single('avatar')(req, res, (err) => {
    if (err) {
      return res.status(400).json({ error: err.message });
    }

    if (!req.file) {
      return res.status(400).json({ error: '请选择要上传的文件' });
    }

    try {
      const db = getDb();
      const avatarUrl = `/uploads/avatars/${req.file.filename}`;

      db.prepare('UPDATE users SET avatar_url = ?, updated_at = datetime(\'now\') WHERE id = ?')
        .run(avatarUrl, req.userId);

      res.json({ avatar_url: avatarUrl });
    } catch (error) {
      console.error('上传头像失败:', error);
      res.status(500).json({ error: '上传头像失败' });
    }
  });
});

export default router;
8.11 服务端环境配置
bash
# server/.env.example
# 端口
PORT=8080

# 数据目录（存放 mynote.db 和上传文件）
DATA_DIR=./data

# JWT 密钥（生产环境请使用强随机字符串）
JWT_SECRET=your-secret-key-change-me-in-production

# JWT 过期天数
JWT_EXPIRE_DAYS=7

# 日志级别 (dev, combined, common, short, tiny)
LOG_LEVEL=dev

# 单文件上传大小限制 (MB)
MAX_UPLOAD_SIZE_MB=10
8.12 服务端构建与部署脚本
bash
# server/build.sh - Node.js 构建脚本
#!/bin/bash

set -e

echo "🔨 构建 MyNote Server (Node.js)"

# 安装依赖
npm install

# 构建 TypeScript
npm run build

# 打包
VERSION=$(node -e "console.log(require('./package.json').version)")
PKG_NAME="mynote-server-${VERSION}-node"

mkdir -p release

# 复制构建产物
cp -r dist release/
cp package.json release/
cp package-lock.json release/
cp .env.example release/
cp -r migrations release/
cp -r node_modules release/

cat > release/start.sh << 'EOF'
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f .env ]; then
    echo "⚠️  未找到 .env 文件，从 .env.example 复制"
    cp .env.example .env
    echo "📝 请编辑 .env 配置 JWT_SECRET 等参数"
fi

echo "🚀 启动 MyNote Server..."
exec node dist/app.js
EOF

chmod +x release/start.sh

cd release
tar -czf "../${PKG_NAME}.tar.gz" .
cd ..

echo "📦 打包完成: ${PKG_NAME}.tar.gz"
echo ""
echo "📋 使用方法:"
echo "  1. tar -xzf ${PKG_NAME}.tar.gz"
echo "  2. cd ${PKG_NAME}"
echo "  3. ./start.sh"
九、CI/CD 配置
9.1 Windows 客户端构建
yaml
# .github/workflows/build-client.yml
name: Build Client (Windows)

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    
    steps:
      - uses: actions/checkout@v3
    
      - name: Setup Rust
        uses: actions-rs/setup-rust@v1
        with:
          rust-version: stable
    
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
    
      - name: Install dependencies
        run: |
          npm install -g pnpm
          pnpm install
    
      - name: Build Tauri app
        run: cargo tauri build
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
    
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: MyNote-windows
          path: src-tauri/target/release/bundle/nsis/*.exe
9.2 Linux 服务端构建
yaml
# .github/workflows/build-server.yml
name: Build Server (Node.js)

on:
  push:
    tags:
      - 'server-v*'
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
    
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
    
      - name: Install dependencies
        run: |
          cd server
          npm ci
    
      - name: Build server
        run: |
          cd server
          npm run build
    
      - name: Run tests
        run: |
          cd server
          npm test
    
      - name: Package server
        run: |
          cd server
          chmod +x build.sh
          ./build.sh
    
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: mynote-server
          path: server/mynote-server-*.tar.gz
十、部署指南
10.1 客户端下载
用户可从以下地址下载最新版本：

text
https://github.com/[your-username]/MyNote/releases/latest
安装包格式：NSIS 安装包 (.exe)

10.2 服务端部署脚本
bash
#!/bin/bash
# scripts/deploy-server.sh - 部署脚本

set -e

VERSION="${1:-latest}"
INSTALL_DIR="/opt/mynote-server"

echo "📦 下载 MyNote Server ${VERSION}..."

wget -O /tmp/mynote-server.tar.gz \
  "https://github.com/your-username/MyNote/releases/download/server-${VERSION}/mynote-server-${VERSION}-node.tar.gz"

sudo mkdir -p "${INSTALL_DIR}"
sudo rm -rf "${INSTALL_DIR:?}"/*
sudo tar -xzf /tmp/mynote-server.tar.gz -C "${INSTALL_DIR}"

sudo chown -R "${USER}:${USER}" "${INSTALL_DIR}"
chmod +x "${INSTALL_DIR}/start.sh"

sudo tee /etc/systemd/system/mynote-server.service << EOF
[Unit]
Description=MyNote Server
After=network.target

[Service]
Type=simple
User=${USER}
WorkingDirectory=${INSTALL_DIR}
ExecStart=${INSTALL_DIR}/start.sh
Restart=on-failure
RestartSec=10
StandardOutput=append:${INSTALL_DIR}/logs/out.log
StandardError=append:${INSTALL_DIR}/logs/err.log

[Install]
WantedBy=multi-user.target
EOF

mkdir -p "${INSTALL_DIR}/logs"

sudo systemctl daemon-reload
sudo systemctl enable mynote-server
sudo systemctl restart mynote-server

echo ""
echo "✅ 部署完成！"
echo ""
echo "📋 服务信息:"
echo "  - 安装目录: ${INSTALL_DIR}"
echo "  - 数据目录: ${INSTALL_DIR}/data"
echo "  - 日志目录: ${INSTALL_DIR}/logs"
echo ""
echo "🔍 查看状态: sudo systemctl status mynote-server"
echo "📋 查看日志: tail -f ${INSTALL_DIR}/logs/out.log"

# 建议在生产环境前置 nginx 反向代理以支持 HTTPS
cat << 'NginxConfig'
---
📌 生产环境建议：在 Express 之前添加 nginx 反向代理
```
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /uploads/ {
        alias /opt/mynote-server/uploads/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}

# HTTP 重定向到 HTTPS
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```
NginxConfig
10.3 服务端目录结构（部署后）
text
/opt/mynote-server/
├── dist/                        # TypeScript 编译产物
│   ├── app.js
│   ├── config.js
│   ├── routes/
│   ├── services/
│   ├── middleware/
│   └── utils/
├── node_modules/                # 依赖包
├── .env                        # 配置文件
├── .env.example                # 配置示例
├── start.sh                    # 启动脚本
├── data/                       # 数据目录
│   └── mynote.db               # SQLite 数据库文件
├── logs/
│   ├── out.log                 # 标准输出日志
│   └── err.log                 # 错误日志
└── uploads/                    # 头像等上传文件
十一、开发命令
11.1 Windows 客户端开发
powershell
# Windows PowerShell
# 安装依赖
pnpm install

# 开发模式
pnpm run dev

# 构建 Windows 安装包
cargo tauri build

# 运行测试
cargo test
pnpm run test
11.2 Linux 服务端开发
bash
# 进入服务端目录
cd server

# 安装依赖
npm install

# 开发模式（热重载）
npm run dev

# 构建生产版本
npm run build

# 启动生产版本
npm start

# 运行测试
npm test
11.3 数据库迁移
bash
# 客户端 (SQLite)
# 迁移文件在 src-tauri/migrations/ 目录

# 服务端 (SQLite)
cd server
# 迁移文件在 server/migrations/ 目录
# 迁移会在服务启动时自动执行（通过 src/services/db.ts 中的 runMigrations()）
npm run dev
十二、开发检查清单
12.1 功能实现
三栏布局 (侧边栏 + 列表 + 编辑器)

用户登录/注册

头像上传 (剪裁 + 压缩)

修改密码

笔记/文件夹树形结构

拖拽排序

WYSIWYG Markdown 编辑器

源码/预览模式

剪贴板历史 (Ctrl+Shift+V 呼出)

剪贴板搜索

全局粘贴（不抢焦点）

云同步 (触发式)

12.2 部署准备
Windows 客户端 GitHub Actions 构建

Linux 服务端 GitHub Actions 构建

服务端 TypeScript 编译验证

Node.js 运行时版本验证 (>=18)

systemd 服务配置

部署脚本测试

12.3 代码规范
Rust: cargo fmt + cargo clippy (仅客户端)

TypeScript/Vue/Node.js: ESLint + Prettier

约定式提交 (Conventional Commits)

错误码规范

十三、常见问题
13.1 客户端
Q: 全局快捷键 Ctrl+Shift+V 不生效？

text
检查 src-tauri/tauri.conf.json 中 plugins.global-shortcut 配置
确保应用以管理员权限运行（Windows）
Q: 编辑器无法加载？

text
检查 src-tauri/tauri.conf.json 中的权限配置
确保 allowlist 中启用了必要的功能
Q: 拖拽排序不生效？

text
确保数据库中 sort_order 字段已正确更新
检查前端拖拽事件是否正确触发 move_node 命令
13.2 服务端
Q: 服务端无法启动？

text
检查 .env 文件是否存在
检查 DATA_DIR 目录是否有写入权限
检查端口是否被占用 (PORT)
确保 Node.js 版本 >= 18
检查是否已执行 npm install
Q: SQLite 数据库被锁定？

text
检查是否有多个进程访问同一数据库文件
检查文件权限是否允许读写
SQLite 使用 WAL 模式，通常支持并发读
Q: 如何备份数据？

bash
# 直接复制数据库文件即可
cp /opt/mynote-server/data/mynote.db /backup/mynote-$(date +%Y%m%d).db


十四、总结
本方案涵盖了 MyNote 从开发到部署的完整生命周期：

开发：清晰的项目结构和规范

构建：自动化 CI/CD 流程（Windows 客户端 + Linux 服务端）

分发：通过 GitHub Releases 发布

部署：单文件服务端，开箱即用

维护：SQLite 数据库迁移简单，备份方便

关键技术亮点
全局快捷键 + 不抢焦点：通过 Tauri 插件 + Win32 API 实现

SQLite + Express 轻量服务端：无需独立数据库服务，配置简单、部署方便

TypeScript 全栈：客户端（Vue 3）、服务端（Express）统一使用 TypeScript，类型安全

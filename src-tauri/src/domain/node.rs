use serde::{Deserialize, Serialize};

/// 笔记/文件夹统一节点模型（客户端本地数据库）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: String,        // "note" | "folder"
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub content: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建节点 DTO
#[derive(Debug, Deserialize)]
pub struct CreateNodeDto {
    pub name: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub content: Option<String>,
}

/// 更新节点 DTO
#[derive(Debug, Deserialize)]
pub struct UpdateNodeDto {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
    pub content: Option<String>,
    pub is_favorite: Option<bool>,
}

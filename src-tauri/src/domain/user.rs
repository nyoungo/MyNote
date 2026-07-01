use serde::{Deserialize, Serialize};

/// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

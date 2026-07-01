use tauri::State;
use uuid::Uuid;
use crate::domain::node::{Node, CreateNodeDto, UpdateNodeDto};
use crate::infrastructure::database::DbPool;

/// 获取指定父节点下的所有子节点（按排序序号排列）
#[tauri::command]
pub async fn get_children(
    parent_id: Option<String>,
    db: State<'_, DbPool>,
) -> Result<Vec<Node>, String> {
    sqlx::query_as::<_, Node>(
        "SELECT * FROM nodes WHERE parent_id IS ? ORDER BY sort_order ASC"
    )
    .bind(parent_id)
    .fetch_all(&*db)
    .await
    .map_err(|e| e.to_string())
}

/// 创建笔记或文件夹节点
#[tauri::command]
pub async fn create_node(
    dto: CreateNodeDto,
    db: State<'_, DbPool>,
) -> Result<Node, String> {
    let id = Uuid::new_v4().to_string();

    // 计算同级排序号
    let max_order: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM nodes WHERE parent_id IS ?"
    )
    .bind(&dto.parent_id)
    .fetch_one(&*db)
    .await
    .map_err(|e| e.to_string())?;

    let sort_order = max_order.unwrap_or(0) + 1;

    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, name, node_type, parent_id, sort_order, content)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(&id)
    .bind(&dto.name)
    .bind(&dto.node_type)
    .bind(&dto.parent_id)
    .bind(sort_order)
    .bind(&dto.content)
    .fetch_one(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(node)
}

/// 移动节点（拖拽排序）
#[tauri::command]
pub async fn move_node(
    node_id: String,
    new_parent_id: Option<String>,
    new_sort_order: i32,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE nodes SET parent_id = ?, sort_order = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(new_parent_id)
    .bind(new_sort_order)
    .bind(node_id)
    .execute(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 加载笔记内容
#[tauri::command]
pub async fn load_note_content(
    node_id: String,
    db: State<'_, DbPool>,
) -> Result<String, String> {
    let content: Option<String> = sqlx::query_scalar(
        "SELECT content FROM nodes WHERE id = ?"
    )
    .bind(node_id)
    .fetch_one(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(content.unwrap_or_default())
}

/// 保存笔记内容（UPSERT — 不存在则插入，存在则更新）
///
/// 因笔记列表来自 HTTP API（远程同步），本地 SQLite 中可能尚无该节点，
/// 故使用 INSERT OR REPLACE 确保内容始终被持久化到本地。
#[tauri::command]
pub async fn save_note_content(
    node_id: String,
    content: String,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO nodes (id, name, node_type, content, updated_at)
         VALUES (?1, 'Note', 'note', ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(id) DO UPDATE SET content = ?2, updated_at = CURRENT_TIMESTAMP"
    )
    .bind(&node_id)
    .bind(&content)
    .execute(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 更新节点属性
#[tauri::command]
pub async fn update_node(
    node_id: String,
    dto: UpdateNodeDto,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE nodes SET
            name = COALESCE(?, name),
            parent_id = COALESCE(?, parent_id),
            sort_order = COALESCE(?, sort_order),
            content = COALESCE(?, content),
            is_favorite = COALESCE(?, is_favorite),
            updated_at = CURRENT_TIMESTAMP
         WHERE id = ?"
    )
    .bind(&dto.name)
    .bind(&dto.parent_id)
    .bind(dto.sort_order)
    .bind(&dto.content)
    .bind(dto.is_favorite)
    .bind(node_id)
    .execute(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 删除节点
#[tauri::command]
pub async fn delete_node(
    node_id: String,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM nodes WHERE id = ?")
        .bind(node_id)
        .execute(&*db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

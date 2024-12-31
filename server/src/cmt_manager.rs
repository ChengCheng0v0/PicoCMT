use clogger::*;
use sqlx::{FromRow, MySqlPool};

// 评论的数据结构
#[derive(Debug, FromRow)]
pub struct Comment {
    pub id: String,
    pub parent_id: Option<String>,
    pub nickname: String,
    pub email: Option<String>,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub ip_address: Option<String>,
}

pub async fn get_top_comments(db_pool: &MySqlPool) -> Result<Vec<Comment>, sqlx::Error> {
    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE parent_id IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(db_pool)
    .await?;

    Ok(comments)
}

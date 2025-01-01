use clogger::*;
use sqlx::{FromRow, MySqlPool};
use uuid::Uuid;

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

// 新增评论信息的数据结构
#[derive(Debug, FromRow)]
pub struct NewCommentInfo {
    pub parent_id: Option<String>,
    pub nickname: String,
    pub email: Option<String>,
    pub content: String,
    pub ip_address: Option<String>,
}

// 获取所有顶级评论
pub async fn get_top_comments(db_pool: &MySqlPool) -> Result<Vec<Comment>, sqlx::Error> {
    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE parent_id IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(db_pool)
    .await?;

    Ok(comments)
}

// 新增一条评论
pub async fn add_comment(
    db_pool: &MySqlPool,
    new_comment_info: NewCommentInfo,
) -> Result<Comment, sqlx::Error> {
    // 生成动态信息
    let id = Uuid::new_v4();
    let created_at = chrono::Utc::now().naive_utc();

    // 填充数据结构
    let comment = Comment {
        id: id.to_string(),
        parent_id: new_comment_info.parent_id,
        nickname: new_comment_info.nickname,
        email: new_comment_info.email,
        content: new_comment_info.content,
        created_at,
        updated_at: None,
        ip_address: new_comment_info.ip_address,
    };

    // 写入数据库
    sqlx::query(
        "INSERT INTO comments (id, parent_id, nickname, email, content, created_at, ip_address) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&comment.id)
    .bind(&comment.parent_id)
    .bind(&comment.nickname)
    .bind(&comment.email)
    .bind(&comment.content)
    .bind(comment.created_at)
    .bind(&comment.ip_address)
    .execute(db_pool)
    .await?;

    Ok(comment)
}

use std::net::SocketAddr;

use axum::{extract::ConnectInfo, extract::State, response::IntoResponse, Json};
use clogger::*;
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::cmt_manager;

// 请求的结构体
#[derive(Deserialize)]
pub struct Request {
    parent_id: Option<String>,
    nickname: String,
    email: Option<String>,
    content: String,
}

// Axum Handler
pub async fn handler(
    State(db_pool): State<MySqlPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<Request>,
) -> impl IntoResponse {
    // 构建新评论的基本信息
    let new_comment_info = cmt_manager::NewCommentInfo {
        parent_id: payload.parent_id,
        nickname: payload.nickname,
        email: payload.email,
        content: payload.content,
        ip_address: Some(addr.ip().to_string()),
    };

    match cmt_manager::add_comment(&db_pool, new_comment_info).await {
        Ok(comment) => {
            c_log!(format!("(IP: {}) 新增了一条评论: {:?}", addr.ip(), comment));

            // 返回 JSON
            Json("success").into_response()
        }
        Err(e) => {
            c_error!(format!("新增评论时出现错误: {}", e));

            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "出现内部错误，无法新增评论",
            )
                .into_response()
        }
    }
}

use std::borrow::Cow;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use clogger::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;

use crate::cmt_manager;

// 请求参数的结构体
#[derive(Deserialize)]
pub struct Params {
    parent_id: String,
}

// 响应的结构体
#[derive(Serialize)]
struct Response {
    id: String,
    parent_id: Option<String>,
    nickname: String,
    email: Option<String>,
    content: String,
    created_at: String,
    updated_at: Option<String>,
    ip_address: Option<String>,
}

// 响应结构体的转换方法
impl From<cmt_manager::Comment> for Response {
    fn from(comment: cmt_manager::Comment) -> Self {
        Response {
            id: comment.id,
            parent_id: comment.parent_id,
            nickname: comment.nickname,
            email: comment.email,
            content: comment.content,
            created_at: comment.created_at.to_string(),
            updated_at: comment.updated_at.map(|dt| dt.to_string()),
            ip_address: comment.ip_address,
        }
    }
}

// Axum Handler
pub async fn handler(
    State(db_pool): State<MySqlPool>,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    if Some(&params.parent_id).is_some() {
        match cmt_manager::get_sub_comments(&db_pool, Cow::Owned(params.parent_id)).await {
            Ok(comments) => {
                // 将 Comment 转换为 Response
                let response: Vec<Response> = comments.into_iter().map(Response::from).collect();
                // 返回 JSON
                Json(response).into_response()
            }
            Err(e) => {
                c_error!(format!("获取子评论时出现错误: {}", e));

                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"result": "出现内部错误，无法获取子评论"})),
                )
                    .into_response()
            }
        }
    } else {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"result": "缺少 parent_id 参数，无法获取子评论"})),
        )
            .into_response()
    }
}

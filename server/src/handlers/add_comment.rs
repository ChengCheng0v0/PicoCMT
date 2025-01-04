use std::net::SocketAddr;

use axum::{extract::ConnectInfo, extract::State, response::IntoResponse, Json};
use clogger::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

// 响应的结构体
#[derive(Serialize)]
struct Response {
    result: String,
    comment: cmt_manager::Comment,
}

// 校验错误的枚举
enum ValidationError {
    EmptyField(String),
    TooLong(String, usize),
    InvalidFormat(String),
}

// 校验错误枚举的方法
impl ValidationError {
    fn into_response_with_addr(self, addr: &SocketAddr) -> axum::response::Response {
        let message = match self {
            ValidationError::EmptyField(field) => format!("字段 '{}' 不能为空", field),
            ValidationError::TooLong(field, limit) => {
                format!("字段 '{}' 的长度不能超过 {} 个字符", field, limit)
            }
            ValidationError::InvalidFormat(field) => format!("字段 '{}' 的格式不正确", field),
        };

        c_warn!(format!(
            "(IP: {}) 拒绝了此次非法的新增评论请求，原因: {:?}",
            addr.ip(),
            message
        ));

        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "result": format!("非法请求: {}", message) })),
        )
            .into_response()
    }
}

// 校验请求的函数
fn validate_request(payload: &Request) -> Result<(), ValidationError> {
    // 检查 content
    if payload.content.trim().is_empty() {
        return Err(ValidationError::EmptyField("content".into()));
    }
    if payload.content.chars().count() > 256 {
        return Err(ValidationError::TooLong("content".into(), 256));
    }

    // 检查 nickname
    if payload.nickname.trim().is_empty() {
        return Err(ValidationError::EmptyField("nickname".into()));
    }
    if payload.nickname.chars().count() > 16 {
        return Err(ValidationError::TooLong("nickname".into(), 16));
    }

    // 检查 email
    if let Some(email) = &payload.email {
        if email.chars().count() > 32 {
            return Err(ValidationError::TooLong("email".into(), 32));
        }

        // NOTE: 匹配电子邮箱的正则表达式 (e.g. chengcheng@miao.ms)
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("无效的正则表达式");
        if !email_regex.is_match(email) {
            return Err(ValidationError::InvalidFormat("email".into()));
        }
    }

    Ok(())
}

// Axum Handler
pub async fn handler(
    State(db_pool): State<MySqlPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<Request>,
) -> impl IntoResponse {
    // 校验请求是否合法
    if let Err(e) = validate_request(&payload) {
        return e.into_response_with_addr(&addr);
    }

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
            Json(Response {
                result: "success".into(),
                comment,
            })
            .into_response()
        }
        Err(e) => {
            c_error!(format!("新增评论时出现错误: {}", e));

            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"result": "出现内部错误，无法新增评论"})),
            )
                .into_response()
        }
    }
}

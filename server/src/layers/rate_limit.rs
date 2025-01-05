use clogger::*;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;

use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde_json::json;

// 使用固定时间窗口按 IP 限制的配置结构体
pub struct FixedTimeWindowByIpConfig {
    pub requests: Mutex<HashMap<IpAddr, (u64, u32)>>,
    pub limit: u32,
    pub window_size: u64,
}

// 使用固定时间窗口按 IP 限制
pub async fn fixed_time_window_by_ip(
    config: Arc<FixedTimeWindowByIpConfig>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let now = chrono::Utc::now().timestamp() as u64;

    let ip = match request.extensions().get::<ConnectInfo<SocketAddr>>() {
        Some(addr) => addr.0.ip(),
        None => {
            c_log!(format!(
                "(IP: None) 拒绝了此次请求，原因: 无法获取客户端 IP 地址",
            ));

            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"result": "无法获取客户端 IP 地址，保命要紧，这次请求我就先拒绝啦"})),
            )
                .into_response();
        }
    };

    // 计算当前时间窗口
    let current_window = now / config.window_size;

    // 从 HashMap 中获取请求信息
    let mut requests = config.requests.lock().await;
    let entry = requests.entry(ip).or_insert((0, 0));
    let (request_window, request_count) = entry;

    if *request_window == current_window {
        if *request_count < config.limit {
            c_debug!(format!("(IP: {ip}) 请求次数 +1"));

            // 如果在当前时间窗口内请求次数未超过限制则次数 +1 并继续
            *request_count += 1;
            next.run(request).await
        } else {
            // 否则拒绝访问并返回错误信息
            c_log!(format!("(IP: {ip}) 拒绝了此次请求，原因: 触发限制条件",));
            (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"result": "呜... 你太快啦，请温柔点！！"})),
            )
                .into_response()
        }
    } else {
        c_debug!(format!("(IP: {ip}) 的时间窗口和请求次数已重置"));

        // 如果不在时间窗口内则重置时间窗口和请求次数
        *request_window = current_window;
        *request_count = 1;
        next.run(request).await
    }
}

use axum::{routing::get, serve::Listener, Router};
use clogger::*;
use tokio::net::TcpListener;

mod handlers;

#[tokio::main]
async fn main() {
    // 初始化 CLogger
    init_clogger("/dev/null"); // 将输出写入黑洞

    // 定义路由
    let app = Router::new().route("/", get(handlers::root));

    // 监听地址
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    c_log!(format!(
        "正在监听地址: [{}]",
        listener.local_addr().unwrap()
    ));

    // 启动服务
    axum::serve(listener, app).await.unwrap();
}

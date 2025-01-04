use std::{net::SocketAddr, process};

use axum::{
    routing::{get, post},
    Router,
};
use clogger::*;
use sqlx::{MySql, MySqlPool, Pool};
use tokio::net::TcpListener;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;

mod cmt_manager;
mod handlers;

#[tokio::main]
async fn main() {
    // 初始化 CLogger
    init_clogger("/dev/null"); // 将输出写入黑洞

    // 路由的结构体
    struct Routes {
        general: Router<Pool<MySql>>,
        add_comment: Router<Pool<MySql>>,
    }

    // 定义路由
    let routes = Routes {
        general: Router::new()
            .route("/", get(handlers::root))
            .route(
                "/api/get_top_comments",
                get(handlers::get_top_comments::handler),
            )
            .route(
                "/api/get_sub_comments",
                get(handlers::get_sub_comments::handler),
            ),
        add_comment: Router::new().route("/api/add_comment", post(handlers::add_comment::handler)),
    };

    // 连接数据库
    let database_url = "mysql://test:thisisapasswd@127.0.0.1/picocmt";
    let db_pool = match MySqlPool::connect(database_url).await {
        Ok(pool) => {
            c_log!(format!("数据库连接成功: {}", database_url));
            pool
        }
        Err(e) => {
            c_error!(format!("数据库连接失败: {}", e));
            process::exit(1); // 直接终止程序
        }
    };

    // 合并路由
    let app = routes
        .general
        .merge(routes.add_comment)
        // 添加全局中间件
        .layer(TraceLayer::new_for_http()) // 日志跟踪
        .layer(ConcurrencyLimitLayer::new(10)) // 全局并发限制
        .with_state(db_pool); // 共享数据库池

    // 监听地址
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    c_log!(format!("正在监听地址: {}", listener.local_addr().unwrap()));

    // 启动服务
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

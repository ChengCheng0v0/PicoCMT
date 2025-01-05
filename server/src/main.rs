use std::{collections::HashMap, net::SocketAddr, process, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use clogger::*;
use layers::rate_limit::{self, FixedTimeWindowByIpConfig};
use sqlx::{MySql, MySqlPool, Pool};
use tokio::{net::TcpListener, sync::Mutex};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;

mod cmt_manager;
mod handlers;
mod layers;

#[tokio::main]
async fn main() {
    // 初始化 CLogger
    init_clogger("/dev/null"); // 将输出写入黑洞

    // 频率限制中间件的配置结构体
    struct RateLimitConfigs {
        add_comment: Arc<FixedTimeWindowByIpConfig>,
    }

    // 定义频率限制中间件配置
    let rate_limit_configs = RateLimitConfigs {
        add_comment: Arc::new(FixedTimeWindowByIpConfig {
            requests: Mutex::new(HashMap::new()),
            limit: 1,
            window_size: 120,
        }),
    };

    // 路由的结构体
    struct Routes {
        root: Router<Pool<MySql>>,
        general: Router<Pool<MySql>>,
    }

    // 定义路由
    let routes = Routes {
        root: Router::new().route("/", get(handlers::root)),
        general: Router::new()
            .route(
                "/api/get_top_comments",
                get(handlers::get_top_comments::handler),
            )
            .route(
                "/api/get_sub_comments",
                get(handlers::get_sub_comments::handler),
            )
            .route(
                "/api/add_comment",
                post(handlers::add_comment::handler).layer(middleware::from_fn({
                    let config = Arc::clone(&rate_limit_configs.add_comment);
                    move |req, next| rate_limit::fixed_time_window_by_ip(config.clone(), req, next)
                })),
            ),
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
        .root
        .merge(routes.general)
        // 添加全局中间件
        .layer(TraceLayer::new_for_http()) // 日志跟踪
        .layer(ConcurrencyLimitLayer::new(80)) // 全局并发限制
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

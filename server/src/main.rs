use std::{collections::HashMap, fs, net::SocketAddr, process, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use clap::Parser;
use clogger::*;
use layers::rate_limit::{self, FixedTimeWindowByIpConfig};
use serde::Deserialize;
use sqlx::{MySql, MySqlPool, Pool};
use tokio::{net::TcpListener, sync::Mutex};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;

mod cmt_manager;
mod handlers;
mod layers;

#[tokio::main]
async fn main() {
    // 运行参数的结构体
    #[derive(Parser)]
    struct Args {
        #[arg(long, default_value = "./config.toml")]
        config_path: String,
    }

    // 获取运行参数
    let args = Args::parse();

    // 配置的结构体
    #[derive(Deserialize)]
    struct Config {
        #[allow(dead_code)]
        version: String,
        server: ServerConfig,
        database: DatabaseConfig,
    }
    #[derive(Deserialize)]
    struct DatabaseConfig {
        r#type: String,
        host: String,
        port: u16,
        name: String,
        username: String,
        password: String,
    }
    #[derive(Deserialize)]
    struct ServerConfig {
        log_output_file: String,
        listener_addr: String,
        limit: ServerLimitConfig,
    }
    #[derive(Deserialize)]
    struct ServerLimitConfig {
        max_concurrency: usize,
        add_comment: AddCommentLimitConfig,
    }
    #[derive(Deserialize)]
    struct AddCommentLimitConfig {
        limit: u32,
        window_size: u64,
    }

    // 获取配置文件内容
    println!("[Info] 准备加载配置文件: {}", args.config_path);
    let config_origin = match fs::read_to_string(args.config_path) {
        Ok(content) => content,
        Err(e) => {
            println!("[Error] 无法读取配置文件: {}", e);
            process::exit(1); // 直接终止程序
        }
    };
    // 解析配置
    let config: Config = match toml::from_str(&config_origin) {
        Ok(config) => config,
        Err(e) => {
            println!("[Error] 无法解析 TOML 配置: {}", e);
            process::exit(1); // 直接终止程序
        }
    };

    // 初始化 CLogger
    init_clogger(&config.server.log_output_file);

    // 频率限制中间件的配置结构体
    struct RateLimitConfigs {
        add_comment: Arc<FixedTimeWindowByIpConfig>,
    }

    // 定义频率限制中间件配置
    let rate_limit_configs = RateLimitConfigs {
        add_comment: Arc::new(FixedTimeWindowByIpConfig {
            requests: Mutex::new(HashMap::new()),
            limit: config.server.limit.add_comment.limit,
            window_size: config.server.limit.add_comment.window_size,
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
    let database_url = format!(
        "{}://{}:{}@{}:{}/{}",
        config.database.r#type,
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name
    );
    let db_pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => {
            c_log!(format!("数据库连接成功: {}", database_url));
            pool
        }
        Err(e) => {
            c_error!(format!("数据库 '{database_url}' 连接失败: {e}"));
            process::exit(1); // 直接终止程序
        }
    };

    // 合并路由
    let app = routes
        .root
        .merge(routes.general)
        // 添加全局中间件
        .layer(TraceLayer::new_for_http()) // 日志跟踪
        .layer(ConcurrencyLimitLayer::new(
            config.server.limit.max_concurrency,
        )) // 全局并发限制
        .with_state(db_pool); // 共享数据库池

    // 监听地址
    let listener = TcpListener::bind(config.server.listener_addr)
        .await
        .unwrap();
    c_log!(format!("正在监听地址: {}", listener.local_addr().unwrap()));

    // 启动服务
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

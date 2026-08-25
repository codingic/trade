//! HTTP 层：只负责「接收请求 → 调用业务层 → 返回响应」
//!
//! 具体业务逻辑（拉数据、算指标）都在 `service/` 里，本文件不碰。
//! 这就是「关注点分离」：HTTP 层不关心数据从哪来、怎么算。

mod handlers;
mod routes;
pub(crate) mod types;

use std::net::SocketAddr;

pub async fn run() -> anyhow::Result<()> {
    let app = routes::build_router();
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;

    println!("HTTP 服务已启动：http://{addr}");
    println!("接口示例：http://{addr}/api/klines?symbol=BTCUSDT&interval=1m&limit=300");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
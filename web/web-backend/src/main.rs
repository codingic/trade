//! 币安合约量化 —— 第 4 步：HTTP 服务 + 前端可视化
//!
//! 目标：启动一个 HTTP 服务，向前端页面提供 K 线 + 指标数据。
//! 启动后，用浏览器打开 web/web-client/index.html 即可看到图表。

use web_backend::server;

/// `#[tokio::main]` 把 main 函数变成一个异步入口，内部跑一个 Tokio 运行时
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}

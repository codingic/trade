//! 数据采集器（Data Collector）
//!
//! 一个独立的后端程序，专门负责：从币安拉取 K 线 → 增量写入本地 SQLite。
//! 与 Web 服务（main.rs）解耦，可单独运行、常驻后台。
//!
//! 运行方式：
//! ```bash
//! # 常驻增量采集（默认）
//! cargo run --bin collector
//!
//! # 回填历史数据：拉取最近 N 天（默认 90 天）的历史 K 线后退出
//! cargo run --bin collector -- backfill 90
//! ```
//!
//! 设计要点：
//! - 回填：从「N 天前」正向分页拉到「当前」，`INSERT OR IGNORE` 去重，可反复执行
//! - 增量拉取：先查库里已存到哪个时间点，只拉「之后」的新数据，避免重复
//! - 多合约：一次拉多个交易对
//! - 常驻循环：拉完一轮 → 休眠 → 再拉，持续积累数据

mod app;
mod backfill;
mod collect;
mod config;
mod time;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    app::run().await
}

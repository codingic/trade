//! 策略引擎（Strategy Engine）
//!
//! 独立的后端程序，负责「读库 → 算指标 → 判断信号 → 下单」的自动交易闭环。
//! 与采集器（collector）和 Web 服务（main）解耦，通过 SQLite 交换数据。
//!
//! 运行方式：
//! ```bash
//! BINANCE_API_KEY=xxx BINANCE_SECRET_KEY=yyy cargo run -p strategy -- --fast-ma 9 --slow-ma 30 --quantity 0.5
//! ```
//! 默认也会读取 workspace 根目录下的 `trade.toml` 里的 `[strategy]` 配置。
//!
//! 当前策略：双均线金叉/死叉
//! - 快线 MA 上穿慢线 MA → 金叉 → 开多（BUY）
//! - 快线 MA 下穿慢线 MA → 死叉 → 开空（SELL）
//! - MA 周期和每次下单数量都可通过命令行参数调整
//!
//! ⚠️ 默认连测试网（假钱），确认策略无误后再改主网。

mod config;
mod runtime;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    runtime::run().await
}

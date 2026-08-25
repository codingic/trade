//! 离线回测器（Backtest）
//!
//! 专门负责：从本地 SQLite 读取历史 K 线 → 复用策略信号 → 本地模拟成交与盈亏。
//! 不调用币安私有接口，因此**不需要 API Key**。
//! 默认会读取 workspace 根目录下的 `trade.toml` 里的 `[backtest]` 配置。
//!
//! 运行示例：
//! ```bash
//! cargo run -p backtest
//! cargo run -p backtest -- --symbol ETHUSDT --interval 1m --limit 5000
//! cargo run -p backtest -- --symbol BTCUSDT --fast-ma 9 --slow-ma 30 --quantity 0.02 --capital 20000 --fee 0.0004
//! cargo run -p backtest -- --export-summary-csv tradedata/backtests/custom_summary.csv
//! ```

fn main() -> anyhow::Result<()> {
    backtest::app::run()
}
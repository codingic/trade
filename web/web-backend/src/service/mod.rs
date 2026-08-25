//! 业务服务层（service）
//!
//! 这是「数据 → 指标 → 结果」的核心业务逻辑，介于 HTTP 层和底层模块之间：
//!
//! ```text
//! server.rs (HTTP 层)  →  service/ (业务层)  →  binance/ + indicators/ + storage/
//!     只管收发请求            编排业务           数据获取 / 计算 / 持久化
//! ```
//!
//! 好处：数据来源（币安实时 / 本地数据库 / 缓存）怎么变，都只改这里，
//! HTTP 层和前端完全无感。

pub mod backtest;
pub mod db;
pub mod market;

pub use backtest::list_strategies;
pub use backtest::run_backtest_preview;
pub use backtest::run_compound_backtest_request;
pub use backtest::run_custom_backtest;
pub use backtest::run_strategy_catalog;
pub use backtest::run_sweep;
pub use db::get_db_overview;
pub use market::get_kline_chart;

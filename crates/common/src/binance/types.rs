//! 币安合约 API 的数据结构定义
//!
//! 所有「从币安接口反序列化出来的类型」都集中放在这里，方便统一查看和维护。
//! 注意：币安用**字符串**传输价格/数量，是为了避免浮点精度丢失——
//! 量化里一分钱都不能错，后续计算金额时要用 `rust_decimal` 或 `f64` 的字符串解析。

use serde::Deserialize;

/// 最新成交价，对应接口 `/fapi/v1/ticker/price`
#[derive(Debug, Deserialize)]
pub struct TickerPrice {
    pub symbol: String,
    pub price: String,
    pub time: u64,
}

/// 24 小时行情统计，对应接口 `/fapi/v1/ticker/24hr`
/// `#[serde(rename)]` 把 JSON 里的驼峰字段名映射到 Rust 的下划线命名
#[derive(Debug, Deserialize)]
pub struct Ticker24h {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "priceChangePercent")]
    pub change_percent: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
}

/// 单根 K 线（OHLCV），对应接口 `/fapi/v1/klines`
///
/// 注意：币安 K 线接口返回的是「数组套数组」，不是标准的 JSON 对象，
/// 所以这里**不用** serde 自动反序列化，而是在 `market.rs` 里用
/// `serde_json::Value` 手动逐项解析（见 `parse_kline` 函数）。
#[derive(Debug, Clone)]
pub struct Kline {
    /// 开盘时间（毫秒时间戳）
    pub open_time: u64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量（基础资产数量，如 BTC 的个数）
    pub volume: f64,
    /// 收盘时间（毫秒时间戳）
    pub close_time: u64,
    /// 成交额（计价资产，如 USDT）
    pub quote_volume: f64,
    /// 成交笔数
    pub trades: u64,
    /// 主动买入成交量
    pub taker_buy_base: f64,
    /// 主动买入成交额
    pub taker_buy_quote: f64,
}

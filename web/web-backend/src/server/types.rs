use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// /api/klines 的查询参数
#[derive(Deserialize)]
pub struct KlineQuery {
    /// 合约，如 BTCUSDT
    pub symbol: Option<String>,
    /// 周期，如 1m / 5m / 1h
    pub interval: Option<String>,
    /// 拉取根数（默认 300）
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct BacktestRequest {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    pub capital: Option<f64>,
    pub quantity: Option<f64>,
    pub leverage: Option<f64>,
    pub fee: Option<f64>,
    pub fast_ma: Option<usize>,
    pub slow_ma: Option<usize>,
    pub lookback: Option<usize>,
    pub limit: Option<usize>,
}

/// 自定义参数回测请求：指定策略 ID + 参数覆盖
#[derive(Deserialize)]
pub struct CustomBacktestRequest {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    pub capital: Option<f64>,
    pub quantity: Option<f64>,
    pub leverage: Option<f64>,
    pub fee: Option<f64>,
    pub lookback: Option<usize>,
    pub limit: Option<usize>,
    pub strategy_id: String,
    pub params: Option<HashMap<String, Value>>,
}

/// 参数扫描请求（query）：对全部策略做参数网格扫描，取收益前 N
#[derive(Deserialize)]
pub struct SweepQuery {
    /// 合约，默认 BTCUSDT
    pub symbol: Option<String>,
    /// 周期，默认 4h，支持 1h/2h/4h/6h/8h/12h/1d 等
    pub interval: Option<String>,
    /// 扫描天数（自然天），默认 120
    pub days: Option<usize>,
    /// 取收益率前 N 名，默认 20
    pub top: Option<usize>,
}

/// 复利回测请求：用与 sweep 完全相同的复利引擎重新跑一次指定组合。
#[derive(Deserialize)]
pub struct CompoundBacktestRequest {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    /// 回测最近多少天（按 interval 每天根数换算），默认 120
    pub days: Option<usize>,
    pub lookback: Option<usize>,
    pub kind: String,
    pub capital: Option<f64>,
    pub leverage: Option<f64>,
    pub fee: Option<f64>,
    pub params: Option<HashMap<String, Value>>,
}

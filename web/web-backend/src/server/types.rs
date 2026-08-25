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
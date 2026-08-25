//! 行情业务逻辑：从本地数据库读取 K 线 → 计算指标 → 组装 JSON
//!
//! 本模块是「数据管道」的编排层，职责是**读**，不再负责**写**：
//! 1. 从本地数据库读取最近 N 根 K 线
//! 2. 计算所有技术指标
//! 3. 组装成 ECharts 需要的结构
//!
//! 数据写入由独立的采集器程序（`bin/collector.rs`）负责。
//! Web 服务只管读库，二者通过 SQLite 解耦。

mod data;
mod indicators;
mod response;

use anyhow::Result;
use serde_json::Value;

pub async fn get_kline_chart(symbol: &str, interval: &str, limit: u32) -> Result<Value> {
    let series = data::load_kline_series(symbol, interval, limit)?;
    let indicators = indicators::compute_indicators(&series);
    Ok(response::build_kline_chart(symbol, interval, &series, indicators))
}
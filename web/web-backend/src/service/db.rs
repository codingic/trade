//! 数据库概览业务逻辑
//!
//! 读取本地 SQLite 里存了哪些「合约 + 周期」的数据，
//! 返回统计概览（数量、时间范围），供前端「数据库」按钮弹窗展示。

use anyhow::Result;
use serde_json::{json, Value};

use trade_common::storage;

/// 返回数据库里所有「合约 + 周期」组合的统计概览
pub async fn get_db_overview() -> Result<Value> {
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let stats = storage::kline_stats(&conn)?;

    let rows: Vec<Value> = stats
        .iter()
        .map(|s| {
            json!({
                "symbol": s.symbol,
                "interval": s.interval,
                "count": s.count,
                "earliest": s.earliest,
                "latest": s.latest,
            })
        })
        .collect();

    Ok(json!({ "total": rows.len(), "rows": rows }))
}

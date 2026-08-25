use crate::binance::types::Kline;
use anyhow::Result;
use rusqlite::{params, Connection};

/// 批量插入 K 线，返回实际写入的条数
///
/// `INSERT OR IGNORE`：遇到重复（同一 open_time）自动跳过，不报错。
pub fn insert_klines(
    conn: &Connection,
    symbol: &str,
    interval: &str,
    klines: &[Kline],
) -> Result<usize> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO klines
         (symbol, interval, open_time, close_time, open, high, low, close,
          volume, quote_volume, trades, taker_buy_base, taker_buy_quote)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )?;

    let mut inserted = 0;
    for kline in klines {
        inserted += stmt.execute(params![
            symbol,
            interval,
            kline.open_time as i64,
            kline.close_time as i64,
            kline.open,
            kline.high,
            kline.low,
            kline.close,
            kline.volume,
            kline.quote_volume,
            kline.trades as i64,
            kline.taker_buy_base,
            kline.taker_buy_quote,
        ])?;
    }
    Ok(inserted)
}
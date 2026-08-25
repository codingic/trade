use crate::binance::types::Kline;
use anyhow::Result;
use rusqlite::{params, Connection, Row};

/// 统计某个合约某个周期在库里存了多少根 K 线
pub fn count_klines(conn: &Connection, symbol: &str, interval: &str) -> Result<i64> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM klines WHERE symbol = ?1 AND interval = ?2",
        params![symbol, interval],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// 某个「合约 + 周期」组合在库里的统计信息
pub struct KlineStat {
    pub symbol: String,
    pub interval: String,
    pub count: i64,
    /// 最早一根的开盘时间（毫秒时间戳），无数据时为 None
    pub earliest: Option<i64>,
    /// 最新一根的开盘时间（毫秒时间戳）
    pub latest: Option<i64>,
}

/// 列出库里所有「合约 + 周期」组合的统计概览
pub fn kline_stats(conn: &Connection) -> Result<Vec<KlineStat>> {
    let mut stmt = conn.prepare(
        "SELECT symbol, interval, COUNT(*), MIN(open_time), MAX(open_time)
         FROM klines
         GROUP BY symbol, interval
         ORDER BY symbol, interval",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(KlineStat {
            symbol: row.get(0)?,
            interval: row.get(1)?,
            count: row.get(2)?,
            earliest: row.get(3)?,
            latest: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// 查询某个合约某个周期「已存储的最新一根 K 线的开盘时间」
pub fn latest_open_time(conn: &Connection, symbol: &str, interval: &str) -> Result<Option<u64>> {
    let result = conn.query_row(
        "SELECT MAX(open_time) FROM klines WHERE symbol = ?1 AND interval = ?2",
        params![symbol, interval],
        |row| row.get::<_, Option<i64>>(0),
    )?;
    Ok(result.map(|value| value as u64))
}

/// 读取某个合约某个周期的全部 K 线（按时间升序返回）
pub fn klines(conn: &Connection, symbol: &str, interval: &str) -> Result<Vec<Kline>> {
    if interval == "1m" {
        return klines_exact(conn, symbol, interval);
    }

    if let Some(interval_ms) = interval_to_ms(interval) {
        let base = klines_exact(conn, symbol, "1m")?;
        return Ok(aggregate_klines(&base, interval_ms));
    }

    klines_exact(conn, symbol, interval)
}

fn klines_exact(conn: &Connection, symbol: &str, interval: &str) -> Result<Vec<Kline>> {
    let mut stmt = conn.prepare(
        "SELECT open_time, close_time, open, high, low, close, volume,
                quote_volume, trades, taker_buy_base, taker_buy_quote
         FROM klines
         WHERE symbol = ?1 AND interval = ?2
         ORDER BY open_time ASC",
    )?;

    let rows = stmt.query_map(params![symbol, interval], map_kline_row)?;
    let result: Vec<Kline> = rows.collect::<std::result::Result<_, _>>()?;
    Ok(result)
}

/// 读取某个合约某个周期「最近 N 根」K 线（按时间升序返回）
pub fn latest_klines(
    conn: &Connection,
    symbol: &str,
    interval: &str,
    limit: u32,
) -> Result<Vec<Kline>> {
    if interval == "1m" {
        return latest_klines_exact(conn, symbol, interval, limit);
    }

    if let Some(interval_ms) = interval_to_ms(interval) {
        let factor = (interval_ms / 60_000) as u32;
        let base_limit = limit.saturating_mul(factor).saturating_add(factor);
        let base = latest_klines_exact(conn, symbol, "1m", base_limit)?;
        let mut aggregated = aggregate_klines(&base, interval_ms);
        if aggregated.len() > limit as usize {
            aggregated = aggregated.split_off(aggregated.len() - limit as usize);
        }
        return Ok(aggregated);
    }

    latest_klines_exact(conn, symbol, interval, limit)
}

fn latest_klines_exact(
    conn: &Connection,
    symbol: &str,
    interval: &str,
    limit: u32,
) -> Result<Vec<Kline>> {
    let mut stmt = conn.prepare(
        "SELECT open_time, close_time, open, high, low, close, volume,
                quote_volume, trades, taker_buy_base, taker_buy_quote
         FROM klines
         WHERE symbol = ?1 AND interval = ?2
         ORDER BY open_time DESC
         LIMIT ?3",
    )?;

    let rows = stmt.query_map(params![symbol, interval, limit], map_kline_row)?;

    let mut result: Vec<Kline> = rows.collect::<std::result::Result<_, _>>()?;
    result.reverse();
    Ok(result)
}

fn interval_to_ms(interval: &str) -> Option<u64> {
    match interval {
        "1m" => Some(60_000),
        "3m" => Some(3 * 60_000),
        "5m" => Some(5 * 60_000),
        "15m" => Some(15 * 60_000),
        "30m" => Some(30 * 60_000),
        "1h" => Some(60 * 60_000),
        "2h" => Some(2 * 60 * 60_000),
        "3h" => Some(3 * 60 * 60_000),
        "4h" => Some(4 * 60 * 60_000),
        "6h" => Some(6 * 60 * 60_000),
        "8h" => Some(8 * 60 * 60_000),
        "12h" => Some(12 * 60 * 60_000),
        "24h" => Some(24 * 60 * 60_000),
        "1d" => Some(24 * 60 * 60_000),
        _ => None,
    }
}

fn aggregate_klines(source: &[Kline], interval_ms: u64) -> Vec<Kline> {
    let mut result = Vec::new();
    let mut current: Option<Kline> = None;
    let mut current_bucket = 0u64;

    for kline in source {
        let bucket_start = kline.open_time / interval_ms * interval_ms;
        match &mut current {
            Some(active) if current_bucket == bucket_start => {
                active.high = active.high.max(kline.high);
                active.low = active.low.min(kline.low);
                active.close = kline.close;
                active.close_time = kline.close_time;
                active.volume += kline.volume;
                active.quote_volume += kline.quote_volume;
                active.trades += kline.trades;
                active.taker_buy_base += kline.taker_buy_base;
                active.taker_buy_quote += kline.taker_buy_quote;
            }
            Some(_) => {
                result.push(current.take().unwrap());
                current_bucket = bucket_start;
                current = Some(start_bucket(kline, bucket_start));
            }
            None => {
                current_bucket = bucket_start;
                current = Some(start_bucket(kline, bucket_start));
            }
        }
    }

    if let Some(active) = current {
        result.push(active);
    }

    result
}

fn start_bucket(kline: &Kline, bucket_start: u64) -> Kline {
    Kline {
        open_time: bucket_start,
        open: kline.open,
        high: kline.high,
        low: kline.low,
        close: kline.close,
        volume: kline.volume,
        close_time: kline.close_time,
        quote_volume: kline.quote_volume,
        trades: kline.trades,
        taker_buy_base: kline.taker_buy_base,
        taker_buy_quote: kline.taker_buy_quote,
    }
}

fn map_kline_row(row: &Row<'_>) -> rusqlite::Result<Kline> {
    Ok(Kline {
        open_time: row.get::<_, i64>(0)? as u64,
        close_time: row.get::<_, i64>(1)? as u64,
        open: row.get(2)?,
        high: row.get(3)?,
        low: row.get(4)?,
        close: row.get(5)?,
        volume: row.get(6)?,
        quote_volume: row.get(7)?,
        trades: row.get::<_, i64>(8)? as u64,
        taker_buy_base: row.get(9)?,
        taker_buy_quote: row.get(10)?,
    })
}
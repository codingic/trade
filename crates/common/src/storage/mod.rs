//! 数据存储：把 K 线持久化到本地 SQLite 数据库
//!
//! 为什么从 CSV 换成 SQLite：
//! - 支持 SQL 查询（按时间段/条件筛选），CSV 只能全量读
//! - 支持增量写入 + 去重（`INSERT OR IGNORE`），CSV 会重复
//! - 单文件、无需安装数据库服务，最适合个人量化起步
//!
//! 注意：`rusqlite` 是**同步**库。当前数据量小（每次几百行），
//! 直接同步调用即可；将来数据量大时再用 `tokio::task::spawn_blocking` 包一层。

mod query;
mod schema;
mod write;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::Connection;

pub use query::{count_klines, klines, kline_stats, latest_klines, latest_open_time, KlineStat};
pub use rusqlite::Connection as DbConnection;
pub use write::insert_klines;

/// 默认数据库文件路径
///
/// 默认会按以下顺序解析：
/// 1. 环境变量 `TRADE_DB_PATH`
/// 2. 当前工作目录下现有的 `tradedata/market.db`
/// 3. workspace 根目录下的 `tradedata/market.db`
pub const DEFAULT_DB_PATH: &str = "tradedata/market.db";

/// 打开（不存在则创建）数据库，并确保表结构存在
pub fn open(path: &str) -> Result<Connection> {
    let resolved = resolve_db_path(path);
    if let Some(parent) = resolved.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let conn = Connection::open(&resolved)?;
    schema::init_schema(&conn)?;
    Ok(conn)
}

pub fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("TRADE_DB_PATH") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }

    let cwd_relative = PathBuf::from(DEFAULT_DB_PATH);
    if cwd_relative.exists() {
        return cwd_relative;
    }

    workspace_db_path()
}

fn resolve_db_path(path: &str) -> PathBuf {
    if path == DEFAULT_DB_PATH {
        return default_db_path();
    }

    PathBuf::from(path)
}

fn workspace_db_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(DEFAULT_DB_PATH)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::binance::types::Kline;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::init_schema(&conn).unwrap();
        conn
    }

    fn sample_kline(open_time: u64, close: f64) -> Kline {
        Kline {
            open_time,
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 12.5,
            close_time: open_time + 59_999,
            quote_volume: 1000.0,
            trades: 42,
            taker_buy_base: 6.0,
            taker_buy_quote: 500.0,
        }
    }

    #[test]
    fn latest_klines_returns_recent_rows_in_ascending_order() {
        let conn = test_conn();
        let klines = vec![
            sample_kline(3_000, 103.0),
            sample_kline(1_000, 101.0),
            sample_kline(2_000, 102.0),
        ];

        insert_klines(&conn, "BTCUSDT", "1m", &klines).unwrap();

        let latest = latest_klines(&conn, "BTCUSDT", "1m", 2).unwrap();
        let open_times: Vec<u64> = latest.iter().map(|k| k.open_time).collect();

        assert_eq!(open_times, vec![2_000, 3_000]);
        assert_eq!(latest_open_time(&conn, "BTCUSDT", "1m").unwrap(), Some(3_000));
    }

    #[test]
    fn latest_klines_aggregates_higher_interval_from_1m() {
        let conn = test_conn();
        let mut rows = Vec::new();
        for minute in 0..10u64 {
            rows.push(Kline {
                open_time: minute * 60_000,
                open: 100.0 + minute as f64,
                high: 101.0 + minute as f64,
                low: 99.0 + minute as f64,
                close: 100.5 + minute as f64,
                volume: 1.0,
                close_time: minute * 60_000 + 59_999,
                quote_volume: 10.0,
                trades: 1,
                taker_buy_base: 0.5,
                taker_buy_quote: 5.0,
            });
        }

        insert_klines(&conn, "BTCUSDT", "1m", &rows).unwrap();

        let aggregated = latest_klines(&conn, "BTCUSDT", "5m", 2).unwrap();

        assert_eq!(aggregated.len(), 2);
        assert_eq!(aggregated[0].open_time, 0);
        assert_eq!(aggregated[0].open, 100.0);
        assert_eq!(aggregated[0].close, 104.5);
        assert_eq!(aggregated[0].high, 105.0);
        assert_eq!(aggregated[0].low, 99.0);
        assert_eq!(aggregated[0].volume, 5.0);
        assert_eq!(aggregated[1].open_time, 300_000);
        assert_eq!(aggregated[1].open, 105.0);
        assert_eq!(aggregated[1].close, 109.5);
        assert_eq!(aggregated[1].volume, 5.0);
    }

    #[test]
    fn klines_returns_full_series_in_ascending_order() {
        let conn = test_conn();
        let rows = vec![
            sample_kline(3_000, 103.0),
            sample_kline(1_000, 101.0),
            sample_kline(2_000, 102.0),
        ];

        insert_klines(&conn, "BTCUSDT", "1m", &rows).unwrap();

        let loaded = klines(&conn, "BTCUSDT", "1m").unwrap();
        let open_times: Vec<u64> = loaded.iter().map(|k| k.open_time).collect();
        assert_eq!(open_times, vec![1_000, 2_000, 3_000]);
    }

    #[test]
    fn insert_klines_ignores_duplicates_and_stats_track_range() {
        let conn = test_conn();
        let btc_klines = vec![sample_kline(1_000, 101.0), sample_kline(2_000, 102.0)];
        let eth_klines = vec![sample_kline(5_000, 205.0)];

        insert_klines(&conn, "BTCUSDT", "1m", &btc_klines).unwrap();
        insert_klines(&conn, "BTCUSDT", "1m", &btc_klines).unwrap();
        insert_klines(&conn, "ETHUSDT", "5m", &eth_klines).unwrap();

        assert_eq!(count_klines(&conn, "BTCUSDT", "1m").unwrap(), 2);

        let stats = kline_stats(&conn).unwrap();
        assert_eq!(stats.len(), 2);

        let btc = stats
            .iter()
            .find(|row| row.symbol == "BTCUSDT" && row.interval == "1m")
            .unwrap();
        assert_eq!(btc.count, 2);
        assert_eq!(btc.earliest, Some(1_000));
        assert_eq!(btc.latest, Some(2_000));
    }
}
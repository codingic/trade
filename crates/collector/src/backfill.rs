use std::time::Duration;

use anyhow::Result;
use trade_common::binance::BinanceClient;
use trade_common::storage;

use crate::config::{BACKFILL_PAGE, INTERVAL, INTERVAL_MS, SYMBOLS};
use crate::time::{backfill_start_ms, now_ms};

pub async fn run(days: u64) -> Result<()> {
    let start = backfill_start_ms(days);
    let end = now_ms();
    println!("=== 历史数据回填 ===");
    println!("合约: {:?}", SYMBOLS);
    println!("周期: {INTERVAL}，范围: 最近 {days} 天\n");

    let client = BinanceClient::mainnet();
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;

    for symbol in SYMBOLS {
        let total = backfill_symbol(&client, &conn, symbol, start, end).await?;
        println!("{symbol}: 回填完成，共入库 {total} 根");
    }

    println!("\n历史数据回填结束");
    Ok(())
}

async fn backfill_symbol(
    client: &BinanceClient,
    conn: &storage::DbConnection,
    symbol: &str,
    start: u64,
    end: u64,
) -> Result<usize> {
    let mut total = 0usize;
    let mut cursor_start = start;

    loop {
        let batch = client
            .klines_range(symbol, INTERVAL, Some(cursor_start), Some(end), BACKFILL_PAGE)
            .await?;

        if batch.is_empty() {
            break;
        }

        total += storage::insert_klines(conn, symbol, INTERVAL, &batch)?;

        let Some(next_start) = next_cursor_start(
            batch.len(),
            batch.last().map(|k| k.open_time),
            cursor_start,
            end,
        ) else {
            break;
        };

        cursor_start = next_start;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    Ok(total)
}

fn next_cursor_start(
    batch_len: usize,
    latest_open_time: Option<u64>,
    cursor_start: u64,
    end: u64,
) -> Option<u64> {
    if batch_len < BACKFILL_PAGE as usize {
        return None;
    }

    let latest = latest_open_time.unwrap_or(cursor_start);
    let next_start = latest.saturating_add(INTERVAL_MS);
    if next_start <= cursor_start || next_start > end {
        None
    } else {
        Some(next_start)
    }
}

#[cfg(test)]
mod tests {
    use super::next_cursor_start;

    #[test]
    fn next_cursor_start_advances_full_page_by_one_interval() {
        let next = next_cursor_start(1000, Some(1_000), 0, 100_000);
        assert_eq!(next, Some(61_000));
    }

    #[test]
    fn next_cursor_start_stops_on_partial_page() {
        let next = next_cursor_start(999, Some(1_000), 0, 10_000);
        assert_eq!(next, None);
    }

    #[test]
    fn next_cursor_start_stops_when_advancing_past_end() {
        let next = next_cursor_start(1000, Some(9_500), 9_000, 10_000);
        assert_eq!(next, None);
    }
}
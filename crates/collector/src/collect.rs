use std::time::Duration;

use anyhow::Result;
use trade_common::binance::BinanceClient;
use trade_common::storage;

use crate::config::{BATCH_SIZE, INTERVAL, POLL_INTERVAL_SECS, SYMBOLS};
use crate::time::chrono_now;

pub async fn run_forever(client: &BinanceClient) -> Result<()> {
    loop {
        match collect_once(client).await {
            Ok(summary) => {
                let now = chrono_now();
                println!("[{now}] {summary}");
            }
            Err(e) => {
                eprintln!("采集出错: {e}");
            }
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

pub async fn collect_once(client: &BinanceClient) -> Result<String> {
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let mut summary = String::new();

    for symbol in SYMBOLS {
        let start = storage::latest_open_time(&conn, symbol, INTERVAL)?;
        let fresh = client
            .klines_range(*symbol, INTERVAL, start, None, BATCH_SIZE)
            .await?;
        let inserted = storage::insert_klines(&conn, symbol, INTERVAL, &fresh)?;

        if !summary.is_empty() {
            summary.push_str("  |  ");
        }
        summary.push_str(&format!("{symbol}: +{inserted} 根"));
    }

    Ok(summary)
}
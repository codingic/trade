use anyhow::Result;
use trade_common::binance::BinanceClient;

use crate::backfill;
use crate::collect;
use crate::config::{INTERVAL, POLL_INTERVAL_SECS, SYMBOLS};

pub async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 && args[1] == "backfill" {
        let days: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(90);
        return backfill::run(days).await;
    }

    println!("=== 币安合约数据采集器启动 ===");
    println!("合约: {:?}", SYMBOLS);
    println!("周期: {INTERVAL}，每 {POLL_INTERVAL_SECS}s 拉取一轮");
    println!("提示：如需回填历史数据，请用 `cargo run --bin collector -- backfill 90`\n");

    let client = BinanceClient::mainnet();
    collect::run_forever(&client).await
}
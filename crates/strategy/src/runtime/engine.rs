use std::time::Duration;

use anyhow::Result;
use strategy::detect_signal_with_params;
use trade_common::binance::BinanceClient;
use trade_common::storage;

use crate::config::StrategyConfig;

pub async fn run_loop(client: &BinanceClient, config: &StrategyConfig) -> Result<()> {
    let mut last_signal = None;

    loop {
        match run_once(client, config, &mut last_signal).await {
            Ok(Some(msg)) => println!("[信号] {msg}"),
            Ok(None) => {}
            Err(e) => eprintln!("策略执行出错: {e}"),
        }

        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

async fn run_once(
    client: &BinanceClient,
    config: &StrategyConfig,
    last_signal: &mut Option<&'static str>,
) -> Result<Option<String>> {
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let klines = storage::latest_klines(
        &conn,
        &config.symbol,
        &config.interval,
        config.lookback as u32,
    )?;
    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();

    let Some(signal) = detect_signal_with_params(&closes, config.signal) else {
        return Ok(None);
    };

    if *last_signal == Some(signal.side) {
        return Ok(None);
    }
    *last_signal = Some(signal.side);

    println!(
        "  触发{}信号：MA{}={:.2} MA{}={:.2}，准备下单 {} 手",
        signal.side,
        config.signal.fast_ma_period,
        signal.fast_ma,
        config.signal.slow_ma_period,
        signal.slow_ma,
        config.quantity,
    );
    let order = client
        .place_market_order(&config.symbol, signal.side, config.quantity)
        .await?;
    println!("  下单结果: {}", serde_json::to_string(&order)?);

    Ok(Some(format!(
        "{} 信号已触发，下单 {} 手 {}",
        signal.side, config.quantity, config.symbol
    )))
}
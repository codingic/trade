use anyhow::Result;

mod client;
mod engine;

use crate::config::StrategyConfig;

pub async fn run() -> Result<()> {
    let config = StrategyConfig::from_args()?;
    let client = client::build_client();

    println!("=== 策略引擎启动（测试网） ===");
    println!(
        "合约: {}，周期: {}，策略: 双均线 MA{}/MA{}，数量: {}\n",
        config.symbol,
        config.interval,
        config.signal.fast_ma_period,
        config.signal.slow_ma_period,
        config.quantity,
    );

    client::verify_api_key(&client).await;
    engine::run_loop(&client, &config).await
}
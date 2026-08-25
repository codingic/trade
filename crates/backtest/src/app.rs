use anyhow::{anyhow, Context, Result};
use trade_common::storage;

use crate::catalog::common_strategy_presets;
use crate::config::BacktestConfig;
use crate::engine;
use crate::report;

pub fn run() -> Result<()> {
    let config = BacktestConfig::from_args()?;

    if config.list_strategies {
        print_strategy_catalog();
        return Ok(());
    }

    let conn = storage::open(storage::DEFAULT_DB_PATH)?;

    let mut klines = storage::klines(&conn, &config.symbol, &config.interval).with_context(|| {
        format!("读取历史数据失败: {} {}", config.symbol, config.interval)
    })?;

    if klines.is_empty() {
        return Err(anyhow!(
            "数据库里没有 {} {} 的历史数据，请先运行 `cargo run -p collector -- backfill 90`",
            config.symbol,
            config.interval,
        ));
    }

    if let Some(limit) = config.limit {
        if klines.len() > limit {
            klines = klines.split_off(klines.len() - limit);
        }
    }

    println!("=== 开始离线回测 ===");
    println!(
        "合约: {}  周期: {}  MA{}/MA{}  样本: {} 根  初始资金: {:.2}U  单次保证金: {:.2}U  倍数: {:.2}  单次名义仓位: {:.2}U  手续费率: {:.4}",
        config.symbol,
        config.interval,
        config.fast_ma_period,
        config.slow_ma_period,
        klines.len(),
        config.initial_capital,
        config.margin_per_trade(),
        config.leverage,
        config.notional_per_trade(),
        config.fee_rate,
    );

    let result = engine::run_backtest(&config, &klines);
    report::print_report(&config, &result);
    if let Some(path) = config.export_csv.as_deref() {
        report::export_trades_csv(path, &config, &result)?;
        println!("成交明细 CSV 已导出: {path}");
    }
    if let Some(path) = config.export_summary_csv.as_deref() {
        report::export_summary_csv(path, &config, &result)?;
        println!("回测汇总 CSV 已导出: {path}");
    }
    Ok(())
}

fn print_strategy_catalog() {
    let presets = common_strategy_presets();
    println!("=== 内置策略预设 (共 {} 个) ===\n", presets.len());

    let mut current_category = "";
    for (i, p) in presets.iter().enumerate() {
        if p.category != current_category {
            if i > 0 {
                println!();
            }
            println!("【{}】", p.category);
            current_category = p.category;
        }
        println!("  {:>3}. {:<25} lookback={:<3}  {}", i + 1, p.name, p.lookback, p.description);
    }
    println!("\n=== 共 {} 个策略 ===", presets.len());
}
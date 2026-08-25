use std::fmt::Write;
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config::BacktestConfig;
use crate::engine::{BacktestResult, TradeRecord};

pub fn print_report(config: &BacktestConfig, result: &BacktestResult) {
    println!("=== 离线回测报告 ===");
    println!("合约: {}  |  周期: {}", result.symbol, result.interval);
    println!(
        "参数: MA{}/MA{}  |  Lookback {}  |  单次保证金 {:.2}U  |  倍数 {:.2}  |  单次名义仓位 {:.2}U  |  手续费率 {:.4}  |  初始资金 {:.2}",
        config.fast_ma_period,
        config.slow_ma_period,
        config.lookback,
        config.margin_per_trade(),
        config.leverage,
        config.notional_per_trade(),
        config.fee_rate,
        config.initial_capital,
    );
    println!(
        "样本: {} 根  |  时间: {} -> {}",
        result.bars,
        format_ts(result.first_open_time),
        format_ts(result.last_close_time),
    );
    println!(
        "初始资金: {:.2}  |  最终权益: {:.2}  |  净收益: {:.2} ({:.2}%)",
        result.initial_capital,
        result.final_equity,
        result.net_profit,
        result.return_pct,
    );
    println!(
        "最大回撤: {:.2}%  |  总手续费: {:.4}  |  胜率: {:.2}%",
        result.max_drawdown_pct,
        result.total_fees,
        result.win_rate_pct,
    );
    println!(
        "交易笔数: {}  |  盈利: {}  |  亏损: {}",
        result.trades.len(),
        result.win_count,
        result.loss_count,
    );

    if result.trades.is_empty() {
        println!("本次回测没有触发任何成交。");
        return;
    }

    println!("最近 {} 笔成交:", result.trades.len().min(5));
    for trade in result
        .trades
        .iter()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        print_trade(trade);
    }
}

pub fn export_trades_csv(
    path: &str,
    config: &BacktestConfig,
    result: &BacktestResult,
) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, build_trades_csv(config, result))?;
    Ok(())
}

pub fn export_summary_csv(
    path: &str,
    config: &BacktestConfig,
    result: &BacktestResult,
) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, build_summary_csv(config, result))?;
    Ok(())
}

fn print_trade(trade: &TradeRecord) {
    println!(
        "  {}  入场 {:.2} @ {}  ->  出场 {:.2} @ {}  |  净收益 {:.2}  |  手续费 {:.4}  |  持有 {} 根",
        trade.side,
        trade.entry_price,
        trade.entry_time,
        trade.exit_price,
        trade.exit_time,
        trade.net_pnl,
        trade.fee,
        trade.bars_held,
    );
}

fn format_ts(value: Option<u64>) -> String {
    value.map(|ts| ts.to_string()).unwrap_or_else(|| "-".to_string())
}

fn ensure_parent_dir(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn build_trades_csv(config: &BacktestConfig, result: &BacktestResult) -> String {
    let mut output = String::new();
    output.push_str(
        "symbol,interval,fast_ma,slow_ma,lookback,margin_per_trade,leverage,notional_per_trade,fee_rate,initial_capital,trade_index,side,entry_time,entry_price,exit_time,exit_price,gross_pnl,net_pnl,fee,bars_held\n",
    );

    for (index, trade) in result.trades.iter().enumerate() {
        let _ = writeln!(
            output,
            "{},{},{},{},{},{:.8},{:.8},{:.8},{:.8},{:.2},{},{},{},{:.8},{},{:.8},{:.8},{:.8},{:.8},{}",
            result.symbol,
            result.interval,
            config.fast_ma_period,
            config.slow_ma_period,
            config.lookback,
            config.margin_per_trade(),
            config.leverage,
            config.notional_per_trade(),
            config.fee_rate,
            config.initial_capital,
            index + 1,
            trade.side,
            trade.entry_time,
            trade.entry_price,
            trade.exit_time,
            trade.exit_price,
            trade.gross_pnl,
            trade.net_pnl,
            trade.fee,
            trade.bars_held,
        );
    }

    output
}

fn build_summary_csv(config: &BacktestConfig, result: &BacktestResult) -> String {
    let mut output = String::new();
    output.push_str(
        "symbol,interval,fast_ma,slow_ma,lookback,margin_per_trade,leverage,notional_per_trade,fee_rate,initial_capital,first_open_time,last_close_time,bars,trade_count,win_count,loss_count,win_rate_pct,total_fees,net_profit,final_equity,return_pct,max_drawdown_pct\n",
    );
    let _ = writeln!(
        output,
        "{},{},{},{},{},{:.8},{:.8},{:.8},{:.8},{:.2},{},{},{},{},{},{},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8}",
        result.symbol,
        result.interval,
        config.fast_ma_period,
        config.slow_ma_period,
        config.lookback,
        config.margin_per_trade(),
        config.leverage,
        config.notional_per_trade(),
        config.fee_rate,
        config.initial_capital,
        format_ts(result.first_open_time),
        format_ts(result.last_close_time),
        result.bars,
        result.trades.len(),
        result.win_count,
        result.loss_count,
        result.win_rate_pct,
        result.total_fees,
        result.net_profit,
        result.final_equity,
        result.return_pct,
        result.max_drawdown_pct,
    );
    output
}

#[cfg(test)]
mod tests {
    use crate::config::BacktestConfig;
    use crate::engine::{BacktestResult, TradeRecord};

    use super::{build_summary_csv, build_trades_csv};

    #[test]
    fn build_trades_csv_includes_parameters_and_trade_rows() {
        let config = BacktestConfig {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            initial_capital: 10000.0,
            quantity: 10.0,
            leverage: 3.0,
            fee_rate: 0.0002,
            lookback: 60,
            fast_ma_period: 9,
            slow_ma_period: 30,
            limit: None,
            export_csv: Some("tradedata/backtests/test.csv".to_string()),
            export_summary_csv: Some("tradedata/backtests/test_summary.csv".to_string()),
            list_strategies: false,
        };
        let result = BacktestResult {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            bars: 100,
            first_open_time: Some(1),
            last_close_time: Some(2),
            initial_capital: 10000.0,
            final_equity: 10100.0,
            net_profit: 100.0,
            return_pct: 1.0,
            total_fees: 2.0,
            trades: vec![TradeRecord {
                side: "LONG",
                entry_time: 10,
                entry_price: 100.0,
                exit_time: 20,
                exit_price: 110.0,
                gross_pnl: 0.2,
                net_pnl: 0.18,
                fee: 0.02,
                bars_held: 3,
            }],
            win_count: 1,
            loss_count: 0,
            win_rate_pct: 100.0,
            max_drawdown_pct: 0.5,
        };

        let csv = build_trades_csv(&config, &result);

    assert!(csv.contains("symbol,interval,fast_ma,slow_ma,lookback,margin_per_trade,leverage,notional_per_trade"));
    assert!(csv.contains("BTCUSDT,1m,9,30,60,10.00000000,3.00000000,30.00000000,0.00020000,10000.00,1,LONG"));
    }

    #[test]
    fn build_summary_csv_includes_summary_metrics() {
        let config = BacktestConfig {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            initial_capital: 10000.0,
            quantity: 10.0,
            leverage: 2.0,
            fee_rate: 0.0002,
            lookback: 60,
            fast_ma_period: 9,
            slow_ma_period: 30,
            limit: Some(5000),
            export_csv: None,
            export_summary_csv: Some("tradedata/backtests/test_summary.csv".to_string()),
            list_strategies: false,
        };
        let result = BacktestResult {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            bars: 100,
            first_open_time: Some(1),
            last_close_time: Some(2),
            initial_capital: 10000.0,
            final_equity: 10100.0,
            net_profit: 100.0,
            return_pct: 1.0,
            total_fees: 2.0,
            trades: vec![TradeRecord {
                side: "LONG",
                entry_time: 10,
                entry_price: 100.0,
                exit_time: 20,
                exit_price: 110.0,
                gross_pnl: 0.2,
                net_pnl: 0.18,
                fee: 0.02,
                bars_held: 3,
            }],
            win_count: 1,
            loss_count: 0,
            win_rate_pct: 100.0,
            max_drawdown_pct: 0.5,
        };

        let csv = build_summary_csv(&config, &result);

        assert!(csv.contains("trade_count,win_count,loss_count"));
        assert!(csv.contains("BTCUSDT,1m,9,30,60,10.00000000,2.00000000,20.00000000,0.00020000,10000.00,1,2,100,1,1,0,100.00000000,2.00000000,100.00000000,10100.00000000,1.00000000,0.50000000"));
    }
}
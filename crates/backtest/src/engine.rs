use strategy::detect_signal_with_params;
use trade_common::binance::types::Kline;

use crate::catalog::{evaluate_preset, evaluate_with_params, StrategyKind, StrategyParams, StrategyPreset};
use crate::config::BacktestConfig;

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub side: &'static str,
    pub entry_time: u64,
    pub entry_price: f64,
    pub exit_time: u64,
    pub exit_price: f64,
    pub gross_pnl: f64,
    pub net_pnl: f64,
    pub fee: f64,
    pub bars_held: usize,
}

#[derive(Debug)]
pub struct BacktestResult {
    pub symbol: String,
    pub interval: String,
    pub bars: usize,
    pub first_open_time: Option<u64>,
    pub last_close_time: Option<u64>,
    pub initial_capital: f64,
    pub final_equity: f64,
    pub net_profit: f64,
    pub return_pct: f64,
    pub total_fees: f64,
    pub trades: Vec<TradeRecord>,
    pub win_count: usize,
    pub loss_count: usize,
    pub win_rate_pct: f64,
    pub max_drawdown_pct: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    fn from_signal(side: &'static str) -> Self {
        match side {
            "BUY" => Self::Long,
            "SELL" => Self::Short,
            _ => unreachable!("未知信号方向: {}", side),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SignalAction {
    OpenLong,
    OpenShort,
    Exit,
    None,
}

fn parse_signal(signal: &'static str) -> SignalAction {
    match signal {
        "BUY" => SignalAction::OpenLong,
        "SELL" => SignalAction::OpenShort,
        "EXIT" | "EXIT_LONG" | "EXIT_SHORT" => SignalAction::Exit,
        _ => SignalAction::None,
    }
}

#[derive(Clone, Copy, Debug)]
struct Position {
    side: PositionSide,
    entry_time: u64,
    entry_price: f64,
    entry_index: usize,
    quantity: f64,
    margin: f64,
    entry_fee: f64,
}

pub fn run_backtest(config: &BacktestConfig, klines: &[Kline]) -> BacktestResult {
    run_backtest_with_signal(config, klines, config.lookback, |window| {
        let closes: Vec<f64> = window.iter().map(|item| item.close).collect();
        detect_signal_with_params(&closes, config.signal_params()).map(|signal| signal.side)
    })
}

pub fn run_backtest_with_preset(
    config: &BacktestConfig,
    klines: &[Kline],
    preset: &StrategyPreset,
) -> BacktestResult {
    run_backtest_with_signal(config, klines, preset.lookback, |window| {
        evaluate_preset(preset, window)
    })
}

pub fn run_backtest_with_dynamic_params(
    config: &BacktestConfig,
    klines: &[Kline],
    lookback: usize,
    kind: StrategyKind,
    params: &StrategyParams,
) -> BacktestResult {
    run_backtest_with_signal(config, klines, lookback, |window| {
        evaluate_with_params(kind, params, window)
    })
}

fn run_backtest_with_signal<F>(
    config: &BacktestConfig,
    klines: &[Kline],
    lookback: usize,
    evaluate: F,
) -> BacktestResult
where
    F: Fn(&[Kline]) -> Option<&'static str>,
{
    let margin_per_trade = config.margin_per_trade();
    let notional_per_trade = config.notional_per_trade();
    let mut cash = config.initial_capital;
    let mut total_fees = 0.0;
    let mut peak_equity = config.initial_capital;
    let mut max_drawdown = 0.0;
    let mut position: Option<Position> = None;
    let mut trades = Vec::new();
    let mut win_count = 0usize;
    let mut loss_count = 0usize;

    for (index, bar) in klines.iter().enumerate() {
        let equity = mark_to_market(cash, position.as_ref(), bar.close);
        if equity > peak_equity {
            peak_equity = equity;
        }
        if peak_equity > 0.0 {
            let drawdown = (peak_equity - equity) / peak_equity;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        if index + 1 < lookback || index + 1 >= klines.len() {
            continue;
        }

        let window_start = index + 1 - lookback;
        let Some(signal) = evaluate(&klines[window_start..=index]) else {
            continue;
        };

        let action = parse_signal(signal);

        // Check if we need to close current position
        let need_close = match action {
            SignalAction::OpenLong => position.as_ref().map(|p| p.side) != Some(PositionSide::Long),
            SignalAction::OpenShort => position.as_ref().map(|p| p.side) != Some(PositionSide::Short),
            SignalAction::Exit => position.is_some(),
            SignalAction::None => false,
        };

        if !need_close {
            continue;
        }

        let execution_bar = &klines[index + 1];
        if let Some(active) = position.take() {
            let trade = close_position(
                active,
                execution_bar.open,
                execution_bar.open_time,
                index + 1,
                config.fee_rate,
            );
            cash += active.margin + trade.net_pnl;
            total_fees += trade.fee;
            if trade.net_pnl > 0.0 {
                win_count += 1;
            } else {
                loss_count += 1;
            }
            trades.push(trade);
        }

        // If signal is EXIT, don't open new position
        if action == SignalAction::Exit {
            continue;
        }

        let desired_side = match action {
            SignalAction::OpenLong => PositionSide::Long,
            SignalAction::OpenShort => PositionSide::Short,
            _ => unreachable!(),
        };

        let entry_quantity = notional_per_trade / execution_bar.open;
        let entry_fee = notional_per_trade * config.fee_rate;
        if cash < margin_per_trade {
            continue;
        }

        cash -= margin_per_trade;
        position = Some(Position {
            side: desired_side,
            entry_time: execution_bar.open_time,
            entry_price: execution_bar.open,
            entry_index: index + 1,
            quantity: entry_quantity,
            margin: margin_per_trade,
            entry_fee,
        });
    }

    if let Some(active) = position.take() {
        if let Some(last_bar) = klines.last() {
            let trade = close_position(
                active,
                last_bar.close,
                last_bar.close_time,
                klines.len() - 1,
                config.fee_rate,
            );
            cash += active.margin + trade.net_pnl;
            total_fees += trade.fee;
            if trade.net_pnl > 0.0 {
                win_count += 1;
            } else {
                loss_count += 1;
            }
            trades.push(trade);
        }
    }

    if cash > peak_equity {
        peak_equity = cash;
    }
    if peak_equity > 0.0 {
        let final_drawdown = (peak_equity - cash) / peak_equity;
        if final_drawdown > max_drawdown {
            max_drawdown = final_drawdown;
        }
    }

    let net_profit = cash - config.initial_capital;
    let trade_count = trades.len();
    let win_rate_pct = if trade_count == 0 {
        0.0
    } else {
        win_count as f64 / trade_count as f64 * 100.0
    };

    BacktestResult {
        symbol: config.symbol.clone(),
        interval: config.interval.clone(),
        bars: klines.len(),
        first_open_time: klines.first().map(|item| item.open_time),
        last_close_time: klines.last().map(|item| item.close_time),
        initial_capital: config.initial_capital,
        final_equity: cash,
        net_profit,
        return_pct: net_profit / config.initial_capital * 100.0,
        total_fees,
        trades,
        win_count,
        loss_count,
        win_rate_pct,
        max_drawdown_pct: max_drawdown * 100.0,
    }
}

fn mark_to_market(cash: f64, position: Option<&Position>, close_price: f64) -> f64 {
    match position {
        Some(active) => {
            cash + active.margin + gross_pnl(active.side, active.entry_price, close_price, active.quantity)
        }
        None => cash,
    }
}

fn close_position(
    position: Position,
    exit_price: f64,
    exit_time: u64,
    exit_index: usize,
    fee_rate: f64,
) -> TradeRecord {
    let gross = gross_pnl(position.side, position.entry_price, exit_price, position.quantity);
    let exit_fee = exit_price * position.quantity * fee_rate;
    let fee = position.entry_fee + exit_fee;

    TradeRecord {
        side: position.side.label(),
        entry_time: position.entry_time,
        entry_price: position.entry_price,
        exit_time,
        exit_price,
        gross_pnl: gross,
        net_pnl: gross - fee,
        fee,
        bars_held: exit_index.saturating_sub(position.entry_index),
    }
}

fn gross_pnl(side: PositionSide, entry_price: f64, exit_price: f64, quantity: f64) -> f64 {
    match side {
        PositionSide::Long => (exit_price - entry_price) * quantity,
        PositionSide::Short => (entry_price - exit_price) * quantity,
    }
}

#[cfg(test)]
mod tests {
    use trade_common::binance::types::Kline;

    use super::run_backtest;
    use crate::config::BacktestConfig;

    fn sample_bar(open_time: u64, open: f64, close: f64) -> Kline {
        Kline {
            open_time,
            open,
            high: open.max(close),
            low: open.min(close),
            close,
            volume: 1.0,
            close_time: open_time + 59_999,
            quote_volume: close,
            trades: 1,
            taker_buy_base: 0.5,
            taker_buy_quote: close / 2.0,
        }
    }

    #[test]
    fn run_backtest_opens_and_closes_profitable_long_trade() {
        let mut klines = Vec::new();
        for i in 0..24 {
            klines.push(sample_bar(i * 60_000, 100.0, 100.0));
        }
        klines.push(sample_bar(24 * 60_000, 90.0, 90.0));
        klines.push(sample_bar(25 * 60_000, 200.0, 200.0));
        klines.push(sample_bar(26 * 60_000, 210.0, 210.0));
        klines.push(sample_bar(27 * 60_000, 220.0, 220.0));

        let config = BacktestConfig {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            initial_capital: 1_000.0,
            quantity: 210.0,
            leverage: 1.0,
            fee_rate: 0.0,
            lookback: 26,
            fast_ma_period: 7,
            slow_ma_period: 25,
            limit: None,
            export_csv: None,
            export_summary_csv: None,
            list_strategies: false,
        };

        let result = run_backtest(&config, &klines);

        assert_eq!(result.trades.len(), 1);
        assert_eq!(result.trades[0].side, "LONG");
        assert_eq!(result.trades[0].entry_price, 210.0);
        assert_eq!(result.trades[0].exit_price, 220.0);
        assert_eq!(result.net_profit, 10.0);
        assert_eq!(result.final_equity, 1_010.0);
        assert_eq!(result.win_count, 1);
    }

    #[test]
    fn run_backtest_scales_pnl_with_leverage() {
        let mut klines = Vec::new();
        for i in 0..24 {
            klines.push(sample_bar(i * 60_000, 100.0, 100.0));
        }
        klines.push(sample_bar(24 * 60_000, 90.0, 90.0));
        klines.push(sample_bar(25 * 60_000, 200.0, 200.0));
        klines.push(sample_bar(26 * 60_000, 210.0, 210.0));
        klines.push(sample_bar(27 * 60_000, 220.0, 220.0));

        let base = BacktestConfig {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            initial_capital: 1_000.0,
            quantity: 210.0,
            leverage: 1.0,
            fee_rate: 0.0,
            lookback: 26,
            fast_ma_period: 7,
            slow_ma_period: 25,
            limit: None,
            export_csv: None,
            export_summary_csv: None,
            list_strategies: false,
        };
        let mut leveraged = base.clone();
        leveraged.leverage = 3.0;

        let base_result = run_backtest(&base, &klines);
        let leveraged_result = run_backtest(&leveraged, &klines);

        assert_eq!(base_result.net_profit * 3.0, leveraged_result.net_profit);
        assert_eq!(base_result.final_equity, 1_010.0);
        assert_eq!(leveraged_result.final_equity, 1_030.0);
    }
}
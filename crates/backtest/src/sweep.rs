//! 参数扫描（sweep）：对全部策略做参数网格遍历，找出收益率最高的组合。
//!
//! 该模块同时服务于命令行 `bin/sweep.rs`（打印报表）与 Web 后端 `/api/backtest/sweep`
//! （返回 JSON），避免两份逻辑重复维护。

use serde::Serialize;
use trade_common::binance::types::Kline;

use crate::catalog::{evaluate_with_params, StrategyKind, StrategyParams};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepRow {
    pub rank: usize,
    pub name: String,
    pub category: String,
    pub kind: StrategyKind,
    pub params: StrategyParams,
    pub params_desc: String,
    pub return_pct: f64,
    pub win_rate: f64,
    pub trades: usize,
    pub max_dd: f64,
    pub net_profit: f64,
    pub lookback: usize,
}

/// 复利回测单笔成交记录（与 engine::TradeRecord 字段对齐，方便前端统一渲染）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompoundTradeRecord {
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

/// 复利回测完整结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompoundBacktestResult {
    pub bars: usize,
    pub first_open_time: Option<u64>,
    pub last_close_time: Option<u64>,
    pub initial_capital: f64,
    pub final_equity: f64,
    pub net_profit: f64,
    pub return_pct: f64,
    pub total_fees: f64,
    pub trades: Vec<CompoundTradeRecord>,
    pub win_count: usize,
    pub loss_count: usize,
    pub win_rate_pct: f64,
    pub max_drawdown_pct: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Long,
    Short,
}

struct Position {
    side: Side,
    entry_time: u64,
    entry_price: f64,
    entry_index: usize,
    qty: f64,
    margin: f64,
    entry_fee: f64,
}

impl Position {
    fn gross_pnl(&self, exit_price: f64) -> f64 {
        match self.side {
            Side::Long => (exit_price - self.entry_price) * self.qty,
            Side::Short => (self.entry_price - exit_price) * self.qty,
        }
    }

    fn label(&self) -> &'static str {
        match self.side {
            Side::Long => "LONG",
            Side::Short => "SHORT",
        }
    }
}

/// 复利全仓回测（每次用 95% 可用资金开仓，更贴近「最高收益」语义）
pub fn run_compound_backtest(
    klines: &[Kline],
    lookback: usize,
    kind: StrategyKind,
    params: &StrategyParams,
    initial_capital: f64,
    leverage: f64,
    fee_rate: f64,
) -> CompoundBacktestResult {
    let mut cash = initial_capital;
    let mut position: Option<Position> = None;
    let mut win_count = 0usize;
    let mut loss_count = 0usize;
    let mut peak_equity = initial_capital;
    let mut max_drawdown = 0.0f64;
    let mut total_fees = 0.0f64;
    let mut trade_records = Vec::new();

    for index in lookback..klines.len().saturating_sub(1) {
        let window = &klines[index + 1 - lookback..=index];
        let bar = &klines[index];
        let next_bar = &klines[index + 1];

        let equity = match &position {
            Some(p) => cash + p.margin + p.gross_pnl(bar.close),
            None => cash,
        };
        if equity > peak_equity {
            peak_equity = equity;
        }
        if peak_equity > 0.0 {
            let dd = (peak_equity - equity) / peak_equity * 100.0;
            if dd > max_drawdown {
                max_drawdown = dd;
            }
        }

        let Some(signal) = evaluate_with_params(kind, params, window) else {
            continue;
        };

        let is_exit = signal == "EXIT" || signal == "EXIT_LONG" || signal == "EXIT_SHORT";
        let desired_side = if signal == "BUY" {
            Some(Side::Long)
        } else if signal == "SELL" {
            Some(Side::Short)
        } else {
            None
        };

        let need_close = if is_exit {
            position.is_some()
        } else if let Some(side) = desired_side {
            position.as_ref().map(|a| a.side) != Some(side)
        } else {
            false
        };

        if !need_close {
            continue;
        }

        // 平掉已有仓位
        if let Some(active) = position.take() {
            let exit_price = next_bar.open;
            let exit_time = next_bar.open_time;
            let gross = active.gross_pnl(exit_price);
            let exit_fee = exit_price * active.qty * fee_rate;
            let fee = active.entry_fee + exit_fee;
            let net = gross - fee;
            cash += active.margin + gross - exit_fee;
            total_fees += fee;
            if net > 0.0 {
                win_count += 1;
            } else {
                loss_count += 1;
            }
            trade_records.push(CompoundTradeRecord {
                side: active.label(),
                entry_time: active.entry_time,
                entry_price: active.entry_price,
                exit_time,
                exit_price,
                gross_pnl: gross,
                net_pnl: net,
                fee,
                bars_held: index + 1 - active.entry_index,
            });
        }

        if is_exit {
            continue;
        }

        let Some(desired_side) = desired_side else {
            continue;
        };

        // 开新仓：用 95% 可用资金，预留手续费空间
        let margin = cash * 0.95;
        if margin < 1.0 {
            continue;
        }
        let notional = margin * leverage;
        let entry_price = next_bar.open;
        let entry_fee = notional * fee_rate;
        if cash < margin + entry_fee {
            continue;
        }
        cash -= margin + entry_fee;

        position = Some(Position {
            side: desired_side,
            entry_time: next_bar.open_time,
            entry_price,
            entry_index: index + 1,
            qty: notional / entry_price,
            margin,
            entry_fee,
        });
    }

    // 结算剩余仓位
    if let Some(active) = position.take() {
        let last_bar = klines.last().unwrap();
        let exit_price = last_bar.close;
        let exit_time = last_bar.close_time;
        let gross = active.gross_pnl(exit_price);
        let exit_fee = exit_price * active.qty * fee_rate;
        let fee = active.entry_fee + exit_fee;
        let net = gross - fee;
        cash += active.margin + gross - exit_fee;
        total_fees += fee;
        if net > 0.0 {
            win_count += 1;
        } else {
            loss_count += 1;
        }
        trade_records.push(CompoundTradeRecord {
            side: active.label(),
            entry_time: active.entry_time,
            entry_price: active.entry_price,
            exit_time,
            exit_price,
            gross_pnl: gross,
            net_pnl: net,
            fee,
            bars_held: klines.len() - 1 - active.entry_index,
        });
    }

    let final_equity = cash;
    let net_profit = final_equity - initial_capital;
    let return_pct = net_profit / initial_capital * 100.0;
    let trade_count = trade_records.len();
    let win_rate_pct = if trade_count > 0 {
        win_count as f64 / trade_count as f64 * 100.0
    } else {
        0.0
    };

    CompoundBacktestResult {
        bars: klines.len(),
        first_open_time: klines.first().map(|k| k.open_time),
        last_close_time: klines.last().map(|k| k.close_time),
        initial_capital,
        final_equity,
        net_profit,
        return_pct,
        total_fees,
        trades: trade_records,
        win_count,
        loss_count,
        win_rate_pct,
        max_drawdown_pct: max_drawdown,
    }
}

/// 对全部策略做参数网格扫描，返回按收益率降序的前 `top_n` 组合。
///
/// `klines` 应为已经聚合到目标周期（如 4h）的 K 线；调用方负责按「最近 N 天」切片。
pub fn run_sweep(
    klines: &[Kline],
    initial_capital: f64,
    leverage: f64,
    fee_rate: f64,
    top_n: usize,
) -> Vec<SweepRow> {
    let mut results: Vec<SweepRow> = Vec::new();

    let grids: Vec<(StrategyKind, Vec<(String, StrategyParams, usize)>)> = vec![
        (StrategyKind::MaCross, grid_ma_cross()),
        (StrategyKind::RsiReversal, grid_rsi_reversal()),
        (StrategyKind::RsiMidline, grid_rsi_midline()),
        (StrategyKind::MacdCross, grid_macd()),
        (StrategyKind::BollReversion, grid_boll(false)),
        (StrategyKind::BollBreakout, grid_boll(true)),
        (StrategyKind::KdjCross, grid_kdj()),
        (StrategyKind::CciReversal, grid_cci_reversal()),
        (StrategyKind::CciMidline, grid_cci_midline()),
        (StrategyKind::PriceMaCross, grid_price_ma()),
        (StrategyKind::DonchianBreakout, grid_donchian()),
    ];

    for (kind, grid) in &grids {
        for (name, params, lookback) in grid {
            let stats = run_compound_backtest(klines, *lookback, *kind, params, initial_capital, leverage, fee_rate);
            if stats.trades.len() >= 3 {
                results.push(SweepRow {
                    rank: 0,
                    name: name.clone(),
                    category: kind_category(*kind).to_string(),
                    kind: *kind,
                    params: params.clone(),
                    params_desc: param_desc(*kind, params),
                    return_pct: stats.return_pct,
                    win_rate: stats.win_rate_pct,
                    trades: stats.trades.len(),
                    max_dd: stats.max_drawdown_pct,
                    net_profit: stats.net_profit,
                    lookback: *lookback,
                });
            }
        }
    }

    results.sort_by(|a, b| b.return_pct.partial_cmp(&a.return_pct).unwrap_or(std::cmp::Ordering::Equal));
    let mut top: Vec<SweepRow> = results.into_iter().take(top_n).collect();
    for (i, row) in top.iter_mut().enumerate() {
        row.rank = i + 1;
    }
    top
}

fn kind_category(kind: StrategyKind) -> &'static str {
    match kind {
        StrategyKind::MaCross => "均线交叉",
        StrategyKind::RsiReversal => "RSI反转",
        StrategyKind::RsiMidline => "RSI中线",
        StrategyKind::MacdCross => "MACD",
        StrategyKind::RsiLongOnly => "RSI做多",
        StrategyKind::BollReversion => "布林回归",
        StrategyKind::BollBreakout => "布林突破",
        StrategyKind::KdjCross => "KDJ",
        StrategyKind::CciReversal => "CCI反转",
        StrategyKind::CciMidline => "CCI中线",
        StrategyKind::PriceMaCross => "价格/MA",
        StrategyKind::DonchianBreakout => "唐奇安突破",
    }
}

fn grid_ma_cross() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &use_ema in &[false, true] {
        for &fast in &[3, 5, 7, 8, 10, 12, 15, 20] {
            for &slow in &[15, 20, 25, 30, 40, 50, 60, 80] {
                if slow <= fast {
                    continue;
                }
                let label = if use_ema { "EMA" } else { "SMA" };
                out.push((
                    format!("{label} {fast}/{slow} 交叉", fast = fast, slow = slow),
                    StrategyParams { fast, slow, use_ema, ..Default::default() },
                    slow + 10,
                ));
            }
        }
    }
    out
}

fn grid_rsi_reversal() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 6, 7, 8, 9, 10, 14, 21] {
        for &oversold in &[10.0, 15.0, 20.0, 25.0, 30.0, 35.0] {
            for &overbought in &[65.0, 70.0, 75.0, 80.0, 85.0, 90.0] {
                out.push((
                    format!("RSI{period} 反转 ({os}/{ob})", period = period, os = oversold as i32, ob = overbought as i32),
                    StrategyParams { period, oversold, overbought, ..Default::default() },
                    period + 10,
                ));
            }
        }
    }
    out
}

fn grid_rsi_midline() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 7, 10, 14, 21] {
        for &bull in &[35.0, 40.0, 45.0, 50.0, 55.0] {
            for &bear in &[45.0, 50.0, 55.0, 60.0, 65.0] {
                if bear <= bull {
                    continue;
                }
                out.push((
                    format!("RSI{period} 中线 ({bull}/{bear})", period = period, bull = bull as i32, bear = bear as i32),
                    StrategyParams { period, bull_level: bull, bear_level: bear, ..Default::default() },
                    period + 10,
                ));
            }
        }
    }
    out
}

fn grid_macd() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &fast in &[5, 8, 12] {
        for &slow in &[17, 21, 26, 35] {
            if slow <= fast {
                continue;
            }
            for &signal in &[3, 5, 7, 9] {
                out.push((
                    format!("MACD {fast}/{slow}/{signal}", fast = fast, slow = slow, signal = signal),
                    StrategyParams { fast, slow, signal, ..Default::default() },
                    slow + signal + 10,
                ));
            }
        }
    }
    out
}

fn grid_boll(is_breakout: bool) -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    let name = if is_breakout { "突破" } else { "回归" };
    for &period in &[10, 14, 20, 25, 30] {
        for &k in &[1.5_f64, 1.8, 2.0, 2.2, 2.5] {
            out.push((
                format!("BOLL{period} {name} k={k}", period = period, name = name, k = k),
                StrategyParams { period, k, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_kdj() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 7, 9, 14, 21] {
        out.push((
            format!("KDJ{period} 金叉", period = period),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn grid_cci_reversal() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 14, 20, 30] {
        for &threshold in &[80.0, 100.0, 150.0, 200.0] {
            out.push((
                format!("CCI{period} 反转 ±{t}", period = period, t = threshold as i32),
                StrategyParams { period, threshold, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_cci_midline() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 14, 20, 30] {
        out.push((
            format!("CCI{period} 穿零轴", period = period),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn grid_price_ma() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &use_ema in &[false, true] {
        for &period in &[5, 10, 15, 20, 30, 50] {
            let label = if use_ema { "EMA" } else { "SMA" };
            out.push((
                format!("Price/{label} 穿{period}", label = label, period = period),
                StrategyParams { period, use_ema, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_donchian() -> Vec<(String, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 15, 20, 30, 40, 55] {
        out.push((
            format!("Donchian{period} 突破", period = period),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn param_desc(kind: StrategyKind, p: &StrategyParams) -> String {
    match kind {
        StrategyKind::MaCross | StrategyKind::PriceMaCross => format!(
            "fast/period={fp},slow={sl},ema={em}",
            fp = if kind == StrategyKind::MaCross { p.fast } else { p.period },
            sl = if kind == StrategyKind::MaCross { p.slow } else { 0 },
            em = p.use_ema,
        ),
        StrategyKind::RsiReversal => format!("p={p},os={o},ob={b}", p = p.period, o = p.oversold, b = p.overbought),
        StrategyKind::RsiMidline => format!("p={p},bull={bl},bear={br}", p = p.period, bl = p.bull_level, br = p.bear_level),
        StrategyKind::MacdCross => format!("{},{},{}", p.fast, p.slow,  p.signal),
        StrategyKind::RsiLongOnly => format!("p={},os={},ob={}", p.period, p.oversold, p.overbought),
        StrategyKind::BollReversion | StrategyKind::BollBreakout => format!("p={},k={}", p.period, p.k),
        StrategyKind::KdjCross | StrategyKind::CciMidline => format!("p={}", p.period),
        StrategyKind::CciReversal => format!("p={},t={}", p.period, p.threshold),
        StrategyKind::DonchianBreakout => format!("p={}", p.period),
    }
}

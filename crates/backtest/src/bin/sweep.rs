use anyhow::Result;
use backtest::catalog::{self, StrategyKind, StrategyParams};
use trade_common::binance::types::Kline;
use trade_common::storage;

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str()) as &'static str
}

fn main() -> Result<()> {
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let klines_1m = storage::klines(&conn, "BTCUSDT", "1m")?;
    println!("原始1m数据: {} 根", klines_1m.len());

    let klines_4h = resample_to_4h(&klines_1m);
    println!("聚合4h数据: {} 根 (约{}天)", klines_4h.len(), klines_4h.len() * 4 / 24);

    let initial_capital = 100.0;
    let leverage = 1.0;
    let fee_rate = 0.0004;

    let mut results: Vec<SweepResult> = Vec::new();

    let param_grids: Vec<(&str, StrategyKind, Vec<(&str, StrategyParams, usize)>)> = vec![
        // MA Cross
        ("ma", StrategyKind::MaCross, grid_ma_cross()),
        // RSI Reversal
        ("rsi_rev", StrategyKind::RsiReversal, grid_rsi_reversal()),
        // RSI Midline
        ("rsi_mid", StrategyKind::RsiMidline, grid_rsi_midline()),
        // MACD
        ("macd", StrategyKind::MacdCross, grid_macd()),
        // Boll Reversion
        ("boll_rev", StrategyKind::BollReversion, grid_boll(false)),
        // Boll Breakout
        ("boll_brk", StrategyKind::BollBreakout, grid_boll(true)),
        // KDJ
        ("kdj", StrategyKind::KdjCross, grid_kdj()),
        // CCI Reversal
        ("cci_rev", StrategyKind::CciReversal, grid_cci_reversal()),
        // CCI Midline
        ("cci_mid", StrategyKind::CciMidline, grid_cci_midline()),
        // Price/MA
        ("price_ma", StrategyKind::PriceMaCross, grid_price_ma()),
        // Donchian
        ("donch", StrategyKind::DonchianBreakout, grid_donchian()),
    ];

    for (_tag, kind, grid) in &param_grids {
        for (name, params, lookback) in grid {
            let r = run_compound_backtest(&klines_4h, *lookback, *kind, params, initial_capital, leverage, fee_rate);
            if r.trades >= 3 {
                results.push(SweepResult {
                    name: name.to_string(),
                    params_desc: param_desc(*kind, params),
                    return_pct: r.return_pct,
                    win_rate: r.win_rate,
                    trades: r.trades,
                    max_dd: r.max_dd,
                    net_profit: r.net_profit,
                });
            }
        }
    }

    println!("\n共扫描 {} 个参数组合（交易≥3笔）\n", results.len());

    results.sort_by(|a, b| b.return_pct.partial_cmp(&a.return_pct).unwrap_or(std::cmp::Ordering::Equal));

    println!("{:<4} {:<30} {:<22} {:>9} {:>8} {:>6} {:>9} {:>10}",
        "排名", "策略名称", "参数", "收益率%", "胜率%", "笔数", "最大回撤%", "净收益U");
    println!("{}", "-".repeat(110));

    for (i, r) in results.iter().take(15).enumerate() {
        println!("{:<4} {:<30} {:<22} {:>9.2} {:>8.1} {:>6} {:>9.2} {:>10.4}",
            i+1,
            truncate(&r.name, 30),
            truncate(&r.params_desc, 22),
            r.return_pct,
            r.win_rate,
            r.trades,
            r.max_dd,
            r.net_profit,
        );
    }

    Ok(())
}

fn run_compound_backtest(
    klines: &[Kline],
    lookback: usize,
    kind: StrategyKind,
    params: &StrategyParams,
    initial_capital: f64,
    leverage: f64,
    fee_rate: f64,
) -> BacktestStats {
    let mut cash = initial_capital;
    let mut position: Option<Position> = None;
    let mut trades = 0usize;
    let mut wins = 0usize;
    let mut peak_equity = initial_capital;
    let mut max_drawdown = 0.0f64;
    let mut total_fees = 0.0f64;
    let mut gross_pnl_sum = 0.0f64;

    for index in lookback..klines.len() - 1 {
        let window = &klines[index + 1 - lookback..=index];
        let bar = &klines[index];
        let next_bar = &klines[index + 1];

        let equity = match &position {
            Some(p) => cash + p.margin + p.gross_pnl(bar.close),
            None => cash,
        };
        if equity > peak_equity { peak_equity = equity; }
        if peak_equity > 0.0 {
            let dd = (peak_equity - equity) / peak_equity * 100.0;
            if dd > max_drawdown { max_drawdown = dd; }
        }

        let Some(signal) = catalog::evaluate_with_params(kind, params, window) else {
            continue;
        };

        let is_exit = signal == "EXIT" || signal == "EXIT_LONG" || signal == "EXIT_SHORT";
        let desired_side = if signal == "BUY" { Some(Side::Long) } else if signal == "SELL" { Some(Side::Short) } else { None };

        // Check if we need to close
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

        // Close existing position
        if let Some(active) = position.take() {
            let exit_price = next_bar.open;
            let gross = active.gross_pnl(exit_price);
            let exit_fee = exit_price * active.qty * fee_rate;
            let fee = active.entry_fee + exit_fee;
            let net = gross - fee;
            cash += active.margin + gross - exit_fee;
            total_fees += fee;
            gross_pnl_sum += gross;
            trades += 1;
            if net > 0.0 { wins += 1; }
        }

        // If EXIT signal, don't open new position
        if is_exit {
            continue;
        }

        let Some(desired_side) = desired_side else { continue; };

        // Open new position
        let margin = cash * 0.95; // use 95% to leave room for fees
        if margin < 1.0 { continue; }

        let notional = margin * leverage;
        let entry_price = next_bar.open;
        let entry_fee = notional * fee_rate;
        if cash < margin + entry_fee { continue; }
        cash -= margin + entry_fee;
        total_fees += entry_fee;

        position = Some(Position {
            side: desired_side,
            entry_price,
            qty: notional / entry_price,
            margin,
            entry_fee,
        });
    }

    // Close any open position at last bar
    if let Some(active) = position.take() {
        let exit_price = klines.last().unwrap().close;
        let gross = active.gross_pnl(exit_price);
        let exit_fee = exit_price * active.qty * fee_rate;
        let fee = active.entry_fee + exit_fee;
        let net = gross - fee;
        cash += active.margin + gross - exit_fee;
        total_fees += fee;
        gross_pnl_sum += gross;
        trades += 1;
        if net > 0.0 { wins += 1; }
    }

    let final_equity = cash;
    let net_profit = final_equity - initial_capital;
    let return_pct = net_profit / initial_capital * 100.0;
    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };

    BacktestStats {
        return_pct,
        win_rate,
        trades,
        max_dd: max_drawdown,
        net_profit,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Side { Long, Short }

struct Position {
    side: Side,
    entry_price: f64,
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
}

struct BacktestStats {
    return_pct: f64,
    win_rate: f64,
    trades: usize,
    max_dd: f64,
    net_profit: f64,
}

#[derive(Clone)]
struct SweepResult {
    name: String,
    params_desc: String,
    return_pct: f64,
    win_rate: f64,
    trades: usize,
    max_dd: f64,
    net_profit: f64,
}

// ========== Parameter Grids ==========

fn grid_ma_cross() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &use_ema in &[false, true] {
        for &fast in &[3, 5, 7, 8, 10, 12, 15, 20] {
            for &slow in &[15, 20, 25, 30, 40, 50, 60, 80] {
                if slow <= fast { continue; }
                let label = if use_ema { "EMA" } else { "SMA" };
                out.push((
                    leak_str(format!("{} {}/{} 交叉", label, fast, slow)),
                    StrategyParams { fast, slow, use_ema, ..Default::default() },
                    slow + 10,
                ));
            }
        }
    }
    out
}

fn grid_rsi_reversal() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 6, 7, 8, 9, 10, 14, 21] {
        for &oversold in &[10.0, 15.0, 20.0, 25.0, 30.0, 35.0] {
            for &overbought in &[65.0, 70.0, 75.0, 80.0, 85.0, 90.0] {
                out.push((
                    leak_str(format!("RSI{} 反转 ({}/{})", period, oversold as i32, overbought as i32)),
                    StrategyParams { period, oversold, overbought, ..Default::default() },
                    period + 10,
                ));
            }
        }
    }
    out
}

fn grid_rsi_midline() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 7, 10, 14, 21] {
        for &bull in &[35.0, 40.0, 45.0, 50.0, 55.0] {
            for &bear in &[45.0, 50.0, 55.0, 60.0, 65.0] {
                if bear <= bull { continue; }
                out.push((
                    leak_str(format!("RSI{} 中线 ({}/{})", period, bull as i32, bear as i32)),
                    StrategyParams { period, bull_level: bull, bear_level: bear, ..Default::default() },
                    period + 10,
                ));
            }
        }
    }
    out
}

fn grid_macd() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &fast in &[5, 8, 12] {
        for &slow in &[17, 21, 26, 35] {
            if slow <= fast { continue; }
            for &signal in &[3, 5, 7, 9] {
                out.push((
                    leak_str(format!("MACD {}/{}/{}", fast, slow, signal)),
                    StrategyParams { fast, slow, signal, ..Default::default() },
                    slow + signal + 10,
                ));
            }
        }
    }
    out
}

fn grid_boll(is_breakout: bool) -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    let name = if is_breakout { "突破" } else { "回归" };
    for &period in &[10, 14, 20, 25, 30] {
        for &k in &[1.5_f64, 1.8, 2.0, 2.2, 2.5] {
            out.push((
                leak_str(format!("BOLL{} {} k={}", period, name, k)),
                StrategyParams { period, k, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_kdj() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[5, 7, 9, 14, 21] {
        out.push((
            leak_str(format!("KDJ{} 金叉", period)),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn grid_cci_reversal() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 14, 20, 30] {
        for &threshold in &[80.0, 100.0, 150.0, 200.0] {
            out.push((
                leak_str(format!("CCI{} 反转 ±{}", period, threshold as i32)),
                StrategyParams { period, threshold, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_cci_midline() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 14, 20, 30] {
        out.push((
            leak_str(format!("CCI{} 穿零轴", period)),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn grid_price_ma() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &use_ema in &[false, true] {
        for &period in &[5, 10, 15, 20, 30, 50] {
            let label = if use_ema { "EMA" } else { "SMA" };
            out.push((
                leak_str(format!("Price/{} 穿{}", label, period)),
                StrategyParams { period, use_ema, ..Default::default() },
                period + 10,
            ));
        }
    }
    out
}

fn grid_donchian() -> Vec<(&'static str, StrategyParams, usize)> {
    let mut out = Vec::new();
    for &period in &[10, 15, 20, 30, 40, 55] {
        out.push((
            leak_str(format!("Donchian{} 突破", period)),
            StrategyParams { period, ..Default::default() },
            period + 10,
        ));
    }
    out
}

fn param_desc(kind: StrategyKind, p: &StrategyParams) -> String {
    match kind {
        StrategyKind::MaCross | StrategyKind::PriceMaCross =>
            format!("fast/period={},slow={},ema={}",
                if kind == StrategyKind::MaCross { p.fast } else { p.period },
                if kind == StrategyKind::MaCross { p.slow } else { 0 },
                p.use_ema),
        StrategyKind::RsiReversal => format!("p={},os={},ob={}", p.period, p.oversold, p.overbought),
        StrategyKind::RsiMidline => format!("p={},bull={},bear={}", p.period, p.bull_level, p.bear_level),
        StrategyKind::MacdCross => format!("{},{},{}", p.fast, p.slow, p.signal),
        StrategyKind::BollReversion | StrategyKind::BollBreakout => format!("p={},k={}", p.period, p.k),
        StrategyKind::KdjCross | StrategyKind::CciMidline => format!("p={}", p.period),
        StrategyKind::CciReversal => format!("p={},t={}", p.period, p.threshold),
        StrategyKind::DonchianBreakout => format!("p={}", p.period),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}

fn resample_to_4h(klines: &[Kline]) -> Vec<Kline> {
    let mut out = Vec::new();
    if klines.is_empty() { return out; }

    // 对齐到4h边界（UTC 00:00, 04:00, 08:00...）
    let chunk_ms = 4 * 60 * 60 * 1000; // 4h in ms
    let start_time = klines[0].open_time;
    // 找到第一个对齐到4h的时间
    let aligned_start = (start_time / chunk_ms) * chunk_ms;
    let mut current_chunk_start = aligned_start;

    let mut chunk: Vec<&Kline> = Vec::new();

    for k in klines {
        if k.open_time >= current_chunk_start + chunk_ms {
            // close current chunk
            if !chunk.is_empty() {
                out.push(aggregate_chunk(&chunk));
            }
            // advance to next chunk
            while k.open_time >= current_chunk_start + chunk_ms {
                current_chunk_start += chunk_ms;
            }
            chunk.clear();
        }
        chunk.push(k);
    }
    if !chunk.is_empty() && chunk.len() >= 60 { // 至少需要60根1mK线才算有效4h bar
        out.push(aggregate_chunk(&chunk));
    }
    out
}

fn aggregate_chunk(chunk: &[&Kline]) -> Kline {
    let open = chunk[0].open;
    let close = chunk[chunk.len() - 1].close;
    let high = chunk.iter().map(|k| k.high).fold(f64::MIN, f64::max);
    let low = chunk.iter().map(|k| k.low).fold(f64::MAX, f64::min);
    let volume: f64 = chunk.iter().map(|k| k.volume).sum();
    let quote_volume: f64 = chunk.iter().map(|k| k.quote_volume).sum();
    let trades: u64 = chunk.iter().map(|k| k.trades as u64).sum();
    let taker_buy_base: f64 = chunk.iter().map(|k| k.taker_buy_base).sum();
    let taker_buy_quote: f64 = chunk.iter().map(|k| k.taker_buy_quote).sum();

    Kline {
        open_time: chunk[0].open_time,
        open,
        high,
        low,
        close,
        volume,
        close_time: chunk[chunk.len() - 1].close_time,
        quote_volume,
        trades,
        taker_buy_base,
        taker_buy_quote,
    }
}

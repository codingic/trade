use anyhow::{anyhow, Context, Result};
use backtest::catalog::{common_strategy_presets, strategy_param_schema, StrategyParams};
use backtest::config::BacktestConfig;
use backtest::engine;
use backtest::sweep::run_sweep as run_sweep_lib;
use serde_json::{json, Value};
use std::collections::HashMap;
use trade_common::storage;

use crate::server::types::{BacktestRequest, CompoundBacktestRequest, CustomBacktestRequest, SweepQuery};

/// 返回内置策略目录（元信息 + 参数 schema，无需回测、无需数据库）
pub fn list_strategies() -> Result<Value> {
    let presets = common_strategy_presets();

    let mut categories: Vec<String> = Vec::new();
    for p in &presets {
        if !categories.contains(&p.category.to_string()) {
            categories.push(p.category.to_string());
        }
    }

    let strategies: Vec<Value> = presets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let schema = strategy_param_schema(p.kind);
            json!({
                "index": i + 1,
                "id": p.id,
                "name": p.name,
                "category": p.category,
                "description": p.description,
                "lookback": p.lookback,
                "kind": p.kind,
                "defaultParams": p.default_params,
                "paramSchema": schema,
            })
        })
        .collect();

    Ok(json!({
        "total": presets.len(),
        "categories": categories,
        "strategies": strategies,
    }))
}

pub async fn run_backtest_preview(request: BacktestRequest) -> Result<Value> {
    let config = build_backtest_config(request)?;
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

    let result = engine::run_backtest(&config, &klines);
    let recent_trades: Vec<Value> = result
        .trades
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|trade| {
            json!({
                "side": trade.side,
                "entryTime": trade.entry_time,
                "entryPrice": trade.entry_price,
                "exitTime": trade.exit_time,
                "exitPrice": trade.exit_price,
                "grossPnl": trade.gross_pnl,
                "netPnl": trade.net_pnl,
                "fee": trade.fee,
                "barsHeld": trade.bars_held,
            })
        })
        .collect();

    Ok(json!({
        "parameters": {
            "symbol": config.symbol,
            "interval": config.interval,
            "capital": config.initial_capital,
            "quantity": config.quantity,
            "marginPerTrade": config.margin_per_trade(),
            "leverage": config.leverage,
            "notionalPerTrade": config.notional_per_trade(),
            "feeRate": config.fee_rate,
            "fastMa": config.fast_ma_period,
            "slowMa": config.slow_ma_period,
            "lookback": config.lookback,
            "limit": config.limit,
        },
        "summary": {
            "bars": result.bars,
            "firstOpenTime": result.first_open_time,
            "lastCloseTime": result.last_close_time,
            "tradeCount": result.trades.len(),
            "winCount": result.win_count,
            "lossCount": result.loss_count,
            "winRatePct": result.win_rate_pct,
            "totalFees": result.total_fees,
            "netProfit": result.net_profit,
            "finalEquity": result.final_equity,
            "returnPct": result.return_pct,
            "maxDrawdownPct": result.max_drawdown_pct,
        },
        "recentTrades": recent_trades,
    }))
}

pub async fn run_strategy_catalog(request: BacktestRequest) -> Result<Value> {
    let config = build_catalog_base_config(&request)?;
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

    let mut rows = Vec::new();
    for preset in common_strategy_presets() {
        let mut run_config = config.clone();
        run_config.lookback = preset.lookback;
        let result = engine::run_backtest_with_preset(&run_config, &klines, &preset);

        rows.push(json!({
            "id": preset.id,
            "name": preset.name,
            "category": preset.category,
            "description": preset.description,
            "lookback": preset.lookback,
            "kind": preset.kind,
            "defaultParams": preset.default_params,
            "bars": result.bars,
            "tradeCount": result.trades.len(),
            "winCount": result.win_count,
            "lossCount": result.loss_count,
            "winRatePct": result.win_rate_pct,
            "totalFees": result.total_fees,
            "netProfit": result.net_profit,
            "finalEquity": result.final_equity,
            "returnPct": result.return_pct,
            "maxDrawdownPct": result.max_drawdown_pct,
        }));
    }

    rows.sort_by(|a, b| {
        let right = b.get("returnPct").and_then(|v| v.as_f64()).unwrap_or(f64::NEG_INFINITY);
        let left = a.get("returnPct").and_then(|v| v.as_f64()).unwrap_or(f64::NEG_INFINITY);
        right.partial_cmp(&left).unwrap_or(std::cmp::Ordering::Equal)
    });

    for (index, row) in rows.iter_mut().enumerate() {
        row["rank"] = json!(index + 1);
    }

    Ok(json!({
        "parameters": {
            "symbol": config.symbol,
            "interval": config.interval,
            "capital": config.initial_capital,
            "quantity": config.quantity,
            "marginPerTrade": config.margin_per_trade(),
            "leverage": config.leverage,
            "notionalPerTrade": config.notional_per_trade(),
            "feeRate": config.fee_rate,
            "limit": config.limit,
        },
        "strategies": rows,
    }))
}

/// 使用自定义参数对单个策略进行回测
pub async fn run_custom_backtest(request: CustomBacktestRequest) -> Result<Value> {
    let presets = common_strategy_presets();
    let preset = presets
        .iter()
        .find(|p| p.id == request.strategy_id)
        .ok_or_else(|| anyhow!("策略不存在: {}", request.strategy_id))?;

    let mut config = build_custom_config(&request)?;
    if config.lookback == 0 {
        config.lookback = preset.lookback;
    }

    let mut params = preset.default_params.clone();
    if let Some(overrides) = &request.params {
        apply_json_params(&mut params, overrides);
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

    let lookback = config.lookback.max(required_lookback(preset.kind, &params) + 5);
    let result = engine::run_backtest_with_dynamic_params(&config, &klines, lookback, preset.kind, &params);

    let recent_trades: Vec<Value> = result
        .trades
        .iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|trade| {
            json!({
                "side": trade.side,
                "entryTime": trade.entry_time,
                "entryPrice": trade.entry_price,
                "exitTime": trade.exit_time,
                "exitPrice": trade.exit_price,
                "grossPnl": trade.gross_pnl,
                "netPnl": trade.net_pnl,
                "fee": trade.fee,
                "barsHeld": trade.bars_held,
            })
        })
        .collect();

    Ok(json!({
        "strategy": {
            "id": preset.id,
            "name": preset.name,
            "category": preset.category,
            "description": preset.description,
            "kind": preset.kind,
            "params": params,
        },
        "parameters": {
            "symbol": config.symbol,
            "interval": config.interval,
            "capital": config.initial_capital,
            "quantity": config.quantity,
            "marginPerTrade": config.margin_per_trade(),
            "leverage": config.leverage,
            "notionalPerTrade": config.notional_per_trade(),
            "feeRate": config.fee_rate,
            "lookback": lookback,
            "limit": config.limit,
        },
        "summary": {
            "bars": result.bars,
            "firstOpenTime": result.first_open_time,
            "lastCloseTime": result.last_close_time,
            "tradeCount": result.trades.len(),
            "winCount": result.win_count,
            "lossCount": result.loss_count,
            "winRatePct": result.win_rate_pct,
            "totalFees": result.total_fees,
            "netProfit": result.net_profit,
            "finalEquity": result.final_equity,
            "returnPct": result.return_pct,
            "maxDrawdownPct": result.max_drawdown_pct,
        },
        "recentTrades": recent_trades,
    }))
}

fn required_lookback(_kind: backtest::catalog::StrategyKind, p: &StrategyParams) -> usize {
    // 取策略参数中最大的周期值作为最小 lookback
    let mut m = p.fast.max(p.slow).max(p.signal).max(p.period);
    // MACD 额外需要 slow + signal 根
    // boll/cci/rsi/kdj 需要 period 根，已经包含
    // MA cross 需要 slow 根
    if p.slow > m { m = p.slow; }
    if p.fast > m { m = p.fast; }
    m.max(p.period)
}

fn apply_json_params(params: &mut StrategyParams, overrides: &HashMap<String, Value>) {
    for (key, val) in overrides {
        match key.as_str() {
            "period" => if let Some(v) = val.as_u64() { params.period = v as usize; }
                else if let Some(v) = val.as_f64() { params.period = v as usize; },
            "fast" => if let Some(v) = val.as_u64() { params.fast = v as usize; }
                else if let Some(v) = val.as_f64() { params.fast = v as usize; },
            "slow" => if let Some(v) = val.as_u64() { params.slow = v as usize; }
                else if let Some(v) = val.as_f64() { params.slow = v as usize; },
            "signal" => if let Some(v) = val.as_u64() { params.signal = v as usize; }
                else if let Some(v) = val.as_f64() { params.signal = v as usize; },
            "k" => if let Some(v) = val.as_f64() { params.k = v; }
                else if let Some(v) = val.as_u64() { params.k = v as f64; },
            "oversold" => if let Some(v) = val.as_f64() { params.oversold = v; }
                else if let Some(v) = val.as_u64() { params.oversold = v as f64; },
            "overbought" => if let Some(v) = val.as_f64() { params.overbought = v; }
                else if let Some(v) = val.as_u64() { params.overbought = v as f64; },
            "bullLevel" | "bull_level" => if let Some(v) = val.as_f64() { params.bull_level = v; }
                else if let Some(v) = val.as_u64() { params.bull_level = v as f64; },
            "bearLevel" | "bear_level" => if let Some(v) = val.as_f64() { params.bear_level = v; }
                else if let Some(v) = val.as_u64() { params.bear_level = v as f64; },
            "threshold" => if let Some(v) = val.as_f64() { params.threshold = v; }
                else if let Some(v) = val.as_u64() { params.threshold = v as f64; },
            "useEma" | "use_ema" => if let Some(v) = val.as_bool() { params.use_ema = v; }
                else if let Some(v) = val.as_f64() { params.use_ema = v != 0.0; }
                else if let Some(v) = val.as_u64() { params.use_ema = v != 0; },
            _ => {}
        }
    }
}

fn build_backtest_config(request: BacktestRequest) -> Result<BacktestConfig> {
    let mut config = BacktestConfig::from_settings(None)?;

    if let Some(symbol) = request.symbol {
        config.symbol = symbol;
    }
    if let Some(interval) = request.interval {
        config.interval = interval;
    }
    if let Some(capital) = request.capital {
        config.initial_capital = capital;
    }
    if let Some(quantity) = request.quantity {
        config.quantity = quantity;
    }
    if let Some(leverage) = request.leverage {
        config.leverage = leverage;
    }
    if let Some(fee) = request.fee {
        config.fee_rate = fee;
    }
    if let Some(fast_ma) = request.fast_ma {
        config.fast_ma_period = fast_ma;
    }
    if let Some(slow_ma) = request.slow_ma {
        config.slow_ma_period = slow_ma;
    }
    if let Some(lookback) = request.lookback {
        config.lookback = lookback;
    }
    if let Some(limit) = request.limit {
        config.limit = if limit == 0 { None } else { Some(limit) };
    }

    config.export_csv = None;
    config.export_summary_csv = None;
    validate_basic(&config)?;
    Ok(config)
}

fn build_catalog_base_config(request: &BacktestRequest) -> Result<BacktestConfig> {
    let mut config = BacktestConfig::from_settings(None)?;

    if let Some(symbol) = &request.symbol {
        config.symbol = symbol.clone();
    }
    if let Some(interval) = &request.interval {
        config.interval = interval.clone();
    }
    if let Some(capital) = request.capital {
        config.initial_capital = capital;
    }
    if let Some(quantity) = request.quantity {
        config.quantity = quantity;
    }
    if let Some(leverage) = request.leverage {
        config.leverage = leverage;
    }
    if let Some(fee) = request.fee {
        config.fee_rate = fee;
    }
    if let Some(limit) = request.limit {
        config.limit = if limit == 0 { None } else { Some(limit) };
    }

    config.export_csv = None;
    config.export_summary_csv = None;
    validate_basic(&config)?;
    Ok(config)
}

fn build_custom_config(request: &CustomBacktestRequest) -> Result<BacktestConfig> {
    let mut config = BacktestConfig::from_settings(None)?;

    if let Some(symbol) = &request.symbol {
        config.symbol = symbol.clone();
    }
    if let Some(interval) = &request.interval {
        config.interval = interval.clone();
    }
    if let Some(capital) = request.capital {
        config.initial_capital = capital;
    }
    if let Some(quantity) = request.quantity {
        config.quantity = quantity;
    }
    if let Some(leverage) = request.leverage {
        config.leverage = leverage;
    }
    if let Some(fee) = request.fee {
        config.fee_rate = fee;
    }
    if let Some(lookback) = request.lookback {
        config.lookback = lookback;
    }
    if let Some(limit) = request.limit {
        config.limit = if limit == 0 { None } else { Some(limit) };
    }

    config.export_csv = None;
    config.export_summary_csv = None;
    validate_basic(&config)?;
    Ok(config)
}

fn validate_basic(config: &BacktestConfig) -> Result<()> {
    if config.quantity <= 0.0 {
        return Err(anyhow!("保证金必须大于0"));
    }
    if config.initial_capital <= 0.0 {
        return Err(anyhow!("初始资金必须大于0"));
    }
    if config.leverage <= 0.0 {
        return Err(anyhow!("杠杆倍数必须大于0"));
    }
    if config.fee_rate < 0.0 {
        return Err(anyhow!("手续费不能为负"));
    }
    Ok(())
}

/// 用与 sweep 完全相同的复利引擎，对指定策略参数组合跑一次回测。
///
/// 数据窗口由 `days` + `interval` 决定：先读取该周期全部 K 线，再取最近 `days` 天的根数。
pub async fn run_compound_backtest_request(request: CompoundBacktestRequest) -> Result<Value> {
    let symbol = request.symbol.unwrap_or_else(|| "BTCUSDT".to_string());
    let interval = request.interval.unwrap_or_else(|| "4h".to_string());
    let days = request.days.unwrap_or(120).max(1);
    let lookback = request.lookback.unwrap_or(0);
    let initial_capital = request.capital.unwrap_or(100.0).max(1.0);
    let leverage = request.leverage.unwrap_or(1.0).max(0.01);
    let fee_rate = request.fee.unwrap_or(0.0004).max(0.0);
    let take_profit_pct = request.take_profit_pct;

    let kind = parse_kind(&request.kind)?;

    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let mut klines = storage::klines(&conn, &symbol, &interval).with_context(|| {
        format!("读取历史数据失败: {} {}", symbol, interval)
    })?;

    if klines.is_empty() {
        return Err(anyhow!(
            "数据库里没有 {} {} 的历史数据，请先运行 `cargo run -p collector -- backfill 120`",
            symbol, interval,
        ));
    }

    // 按 interval 计算每天根数，然后切片到最近 days 天
    let bars_per_day = bars_per_day_for_interval(&interval);
    let take = days.saturating_mul(bars_per_day);
    if klines.len() > take {
        klines = klines.split_off(klines.len() - take);
    }

    let mut params = StrategyParams::default();
    if let Some(overrides) = &request.params {
        apply_json_params(&mut params, overrides);
    }

    // lookback 为 0 时，用参数所需最小 lookback + 5 的缓冲
    let effective_lookback = if lookback == 0 {
        required_lookback(kind, &params) + 5
    } else {
        lookback
    };

    let result = backtest::sweep::run_compound_backtest(
        &klines,
        effective_lookback,
        kind,
        &params,
        initial_capital,
        leverage,
        fee_rate,
        take_profit_pct,
    );

    // 与 custom backtest 输出结构对齐，前端可用同一套 renderCustomBacktestResult
    let recent_trades: Vec<Value> = result
        .trades
        .iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|trade| {
            json!({
                "side": trade.side,
                "entryTime": trade.entry_time,
                "entryPrice": trade.entry_price,
                "exitTime": trade.exit_time,
                "exitPrice": trade.exit_price,
                "grossPnl": trade.gross_pnl,
                "netPnl": trade.net_pnl,
                "fee": trade.fee,
                "barsHeld": trade.bars_held,
            })
        })
        .collect();

    Ok(json!({
        "strategy": {
            "id": request.kind,
            "name": format!("{} ({})", request.kind, params_desc(kind, &params)),
            "category": kind_category(kind),
            "description": params_desc(kind, &params),
            "kind": request.kind,
            "params": params,
        },
        "parameters": {
            "symbol": symbol,
            "interval": interval,
            "capital": initial_capital,
            "quantity": initial_capital * 0.95,
            "marginPerTrade": initial_capital * 0.95,
            "leverage": leverage,
            "notionalPerTrade": initial_capital * 0.95 * leverage,
            "feeRate": fee_rate,
            "lookback": effective_lookback,
            "limit": take,
            "days": days,
        },
        "summary": {
            "bars": result.bars,
            "firstOpenTime": result.first_open_time,
            "lastCloseTime": result.last_close_time,
            "tradeCount": result.trades.len(),
            "winCount": result.win_count,
            "lossCount": result.loss_count,
            "winRatePct": result.win_rate_pct,
            "totalFees": result.total_fees,
            "netProfit": result.net_profit,
            "finalEquity": result.final_equity,
            "returnPct": result.return_pct,
            "maxDrawdownPct": result.max_drawdown_pct,
        },
        "recentTrades": recent_trades,
    }))
}

fn parse_kind(kind: &str) -> Result<backtest::catalog::StrategyKind> {
    use backtest::catalog::StrategyKind;
    match kind {
        "maCross" => Ok(StrategyKind::MaCross),
        "rsiReversal" => Ok(StrategyKind::RsiReversal),
        "rsiMidline" => Ok(StrategyKind::RsiMidline),
        "rsiLongOnly" => Ok(StrategyKind::RsiLongOnly),
        "macdCross" => Ok(StrategyKind::MacdCross),
        "bollReversion" => Ok(StrategyKind::BollReversion),
        "bollBreakout" => Ok(StrategyKind::BollBreakout),
        "kdjCross" => Ok(StrategyKind::KdjCross),
        "cciReversal" => Ok(StrategyKind::CciReversal),
        "cciMidline" => Ok(StrategyKind::CciMidline),
        "priceMaCross" => Ok(StrategyKind::PriceMaCross),
        "donchianBreakout" => Ok(StrategyKind::DonchianBreakout),
        "rsiTakeProfit" => Ok(StrategyKind::RsiTakeProfit),
        other => Err(anyhow!("未知策略 kind: {}", other)),
    }
}

fn bars_per_day_for_interval(interval: &str) -> usize {
    match interval {
        "1m" => 60 * 24,
        "3m" => 20 * 24,
        "5m" => 12 * 24,
        "15m" => 4 * 24,
        "30m" => 2 * 24,
        "1h" => 24,
        "2h" => 12,
        "4h" => 6,
        "6h" => 4,
        "8h" => 3,
        "12h" => 2,
        "1d" => 1,
        _ => 6, // 默认按 4h 处理
    }
}

fn kind_category(kind: backtest::catalog::StrategyKind) -> &'static str {
    use backtest::catalog::StrategyKind;
    match kind {
        StrategyKind::MaCross => "均线交叉",
        StrategyKind::RsiReversal => "RSI反转",
        StrategyKind::RsiMidline => "RSI中线",
        StrategyKind::RsiLongOnly => "RSI做多",
        StrategyKind::MacdCross => "MACD",
        StrategyKind::BollReversion => "布林回归",
        StrategyKind::BollBreakout => "布林突破",
        StrategyKind::KdjCross => "KDJ",
        StrategyKind::CciReversal => "CCI反转",
        StrategyKind::CciMidline => "CCI中线",
        StrategyKind::PriceMaCross => "价格/MA",
        StrategyKind::DonchianBreakout => "唐奇安突破",
        StrategyKind::RsiTakeProfit => "RSI止盈",
    }
}

fn params_desc(kind: backtest::catalog::StrategyKind, p: &StrategyParams) -> String {
    use backtest::catalog::StrategyKind;
    match kind {
        StrategyKind::MaCross | StrategyKind::PriceMaCross => format!(
            "fast/period={},slow={},ema={}",
            if kind == StrategyKind::MaCross { p.fast } else { p.period },
            if kind == StrategyKind::MaCross { p.slow } else { 0 },
            p.use_ema,
        ),
        StrategyKind::RsiReversal => format!("p={},os={},ob={}", p.period, p.oversold, p.overbought),
        StrategyKind::RsiMidline => format!("p={},bull={},bear={}", p.period, p.bull_level, p.bear_level),
        StrategyKind::MacdCross => format!("{},{},{}", p.fast, p.slow, p.signal),
        StrategyKind::RsiLongOnly => format!("p={},os={},ob={}", p.period, p.oversold, p.overbought),
        StrategyKind::BollReversion | StrategyKind::BollBreakout => format!("p={},k={}", p.period, p.k),
        StrategyKind::KdjCross | StrategyKind::CciMidline => format!("p={}", p.period),
        StrategyKind::CciReversal => format!("p={},t={}", p.period, p.threshold),
        StrategyKind::DonchianBreakout => format!("p={}", p.period),
        StrategyKind::RsiTakeProfit => format!("p={},多={},空={}", p.period, p.oversold, p.overbought),
    }
}

/// 对所有内置策略做参数网格扫描，取最近 `days` 天内收益率前 `top` 名的组合。
pub async fn run_sweep(request: SweepQuery) -> Result<Value> {
    let symbol = request.symbol.unwrap_or_else(|| "BTCUSDT".to_string());
    let interval = request.interval.unwrap_or_else(|| "4h".to_string());
    let days = request.days.unwrap_or(120).max(1);
    let top = request.top.unwrap_or(20).max(1);

    let initial_capital = 100.0;
    let leverage = 1.0;
    let fee_rate = 0.0004;

    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let mut klines = storage::klines(&conn, &symbol, &interval).with_context(|| {
        format!("读取历史数据失败: {} {}", symbol, interval)
    })?;

    let bars_per_day = bars_per_day_for_interval(&interval);
    let take = days.saturating_mul(bars_per_day);
    let window: Vec<_> = if klines.len() > take {
        klines.split_off(klines.len() - take)
    } else {
        klines
    };

    if window.is_empty() {
        return Err(anyhow!(
            "数据库里没有 {} {} 的足够历史数据，请先运行 `cargo run -p collector -- backfill 120`",
            symbol, interval,
        ));
    }

    let rows = run_sweep_lib(&window, initial_capital, leverage, fee_rate, top);

    Ok(json!({
        "symbol": symbol,
        "interval": interval,
        "days": days,
        "bars": window.len(),
        "actualDays": (window.len() as f64 / bars_per_day as f64).round(),
        "firstOpenTime": window.first().map(|k| k.open_time),
        "lastCloseTime": window.last().map(|k| k.close_time),
        "capital": initial_capital,
        "leverage": leverage,
        "feeRate": fee_rate,
        "rows": rows,
    }))
}

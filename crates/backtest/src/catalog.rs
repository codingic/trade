use serde::Serialize;
use trade_common::binance::types::Kline;
use trade_common::indicators::{boll, cci, ema, kdj, ma, macd, rsi};

#[path = "boll_breakout.rs"] mod boll_breakout;
#[path = "boll_reversion.rs"] mod boll_reversion;
#[path = "cci20_midline.rs"] mod cci20_midline;
#[path = "cci20_reversal.rs"] mod cci20_reversal;
#[path = "donchian20.rs"] mod donchian20;
#[path = "ema_12_26.rs"] mod ema_12_26;
#[path = "ema_20_50.rs"] mod ema_20_50;
#[path = "ema_9_21.rs"] mod ema_9_21;
#[path = "kdj14_cross.rs"] mod kdj14_cross;
#[path = "kdj9_cross.rs"] mod kdj9_cross;
#[path = "macd_12_26_9.rs"] mod macd_12_26_9;
#[path = "macd_8_21_5.rs"] mod macd_8_21_5;
#[path = "price_ema50.rs"] mod price_ema50;
#[path = "price_ma20.rs"] mod price_ma20;
#[path = "rsi14_midline.rs"] mod rsi14_midline;
#[path = "rsi14_reversal.rs"] mod rsi14_reversal;
#[path = "rsi7_reversal.rs"] mod rsi7_reversal;
#[path = "rsi_long_only.rs"] mod rsi_long_only;
#[path = "sma_10_30.rs"] mod sma_10_30;
#[path = "sma_20_60.rs"] mod sma_20_60;
#[path = "sma_7_25.rs"] mod sma_7_25;

pub type StrategyEvaluator = fn(&[Kline]) -> Option<&'static str>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StrategyKind {
    MaCross,
    RsiReversal,
    RsiMidline,
    RsiLongOnly,
    MacdCross,
    BollReversion,
    BollBreakout,
    KdjCross,
    CciReversal,
    CciMidline,
    PriceMaCross,
    DonchianBreakout,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyParams {
    pub period: usize,
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
    pub k: f64,
    pub oversold: f64,
    pub overbought: f64,
    pub bull_level: f64,
    pub bear_level: f64,
    pub threshold: f64,
    pub use_ema: bool,
}

impl Default for StrategyParams {
    fn default() -> Self {
        Self {
            period: 20,
            fast: 7,
            slow: 25,
            signal: 9,
            k: 2.0,
            oversold: 30.0,
            overbought: 70.0,
            bull_level: 50.0,
            bear_level: 50.0,
            threshold: 100.0,
            use_ema: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrategyPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub lookback: usize,
    pub kind: StrategyKind,
    pub default_params: StrategyParams,
    pub evaluator: StrategyEvaluator,
}

pub fn common_strategy_presets() -> Vec<StrategyPreset> {
    let mut result = Vec::new();
    result.extend(sma_7_25::strategies());
    result.extend(sma_10_30::strategies());
    result.extend(sma_20_60::strategies());
    result.extend(ema_9_21::strategies());
    result.extend(ema_12_26::strategies());
    result.extend(ema_20_50::strategies());
    result.extend(rsi14_reversal::strategies());
    result.extend(rsi7_reversal::strategies());
    result.extend(rsi14_midline::strategies());
    result.extend(rsi_long_only::strategies());
    result.extend(macd_12_26_9::strategies());
    result.extend(macd_8_21_5::strategies());
    result.extend(boll_reversion::strategies());
    result.extend(boll_breakout::strategies());
    result.extend(kdj9_cross::strategies());
    result.extend(kdj14_cross::strategies());
    result.extend(cci20_reversal::strategies());
    result.extend(cci20_midline::strategies());
    result.extend(price_ma20::strategies());
    result.extend(price_ema50::strategies());
    result.extend(donchian20::strategies());
    result
}

pub fn evaluate_preset(preset: &StrategyPreset, window: &[Kline]) -> Option<&'static str> {
    (preset.evaluator)(window)
}

pub fn evaluate_with_params(kind: StrategyKind, p: &StrategyParams, window: &[Kline]) -> Option<&'static str> {
    match kind {
        StrategyKind::MaCross => evaluate_ma_cross(window, p.fast, p.slow, p.use_ema),
        StrategyKind::RsiReversal => evaluate_rsi_reversal(window, p.period, p.oversold, p.overbought),
        StrategyKind::RsiMidline => evaluate_rsi_midline(window, p.period, p.bull_level, p.bear_level),
        StrategyKind::RsiLongOnly => evaluate_rsi_long_only(window, p.period, p.oversold, p.overbought),
        StrategyKind::MacdCross => evaluate_macd_cross(window, p.fast, p.slow, p.signal),
        StrategyKind::BollReversion => evaluate_boll_reversion(window, p.period, p.k),
        StrategyKind::BollBreakout => evaluate_boll_breakout(window, p.period, p.k),
        StrategyKind::KdjCross => evaluate_kdj_cross(window, p.period),
        StrategyKind::CciReversal => evaluate_cci_reversal(window, p.period, p.threshold),
        StrategyKind::CciMidline => evaluate_cci_midline(window, p.period),
        StrategyKind::PriceMaCross => evaluate_price_ma_cross(window, p.period, p.use_ema),
        StrategyKind::DonchianBreakout => evaluate_donchian_breakout(window, p.period),
    }
}

pub fn strategy_param_schema(kind: StrategyKind) -> Vec<ParamDef> {
    match kind {
        StrategyKind::MaCross => vec![
            ParamDef::int("fast", "快线周期", 2, 200, 1),
            ParamDef::int("slow", "慢线周期", 3, 500, 1),
            ParamDef::bool("useEma", "使用 EMA"),
        ],
        StrategyKind::RsiReversal => vec![
            ParamDef::int("period", "RSI 周期", 2, 100, 1),
            ParamDef::float("oversold", "超卖阈值", 0.0, 50.0, 0.1),
            ParamDef::float("overbought", "超买阈值", 50.0, 100.0, 0.1),
        ],
        StrategyKind::RsiMidline => vec![
            ParamDef::int("period", "RSI 周期", 2, 100, 1),
            ParamDef::float("bullLevel", "多头阈值", 0.0, 60.0, 0.1),
            ParamDef::float("bearLevel", "空头阈值", 40.0, 100.0, 0.1),
        ],
        StrategyKind::RsiLongOnly => vec![
            ParamDef::int("period", "RSI 周期", 2, 100, 1),
            ParamDef::float("oversold", "开多阈值", 0.0, 50.0, 0.1),
            ParamDef::float("overbought", "平多阈值", 50.0, 100.0, 0.1),
        ],
        StrategyKind::MacdCross => vec![
            ParamDef::int("fast", "快线 EMA", 2, 100, 1),
            ParamDef::int("slow", "慢线 EMA", 3, 200, 1),
            ParamDef::int("signal", "信号线", 2, 100, 1),
        ],
        StrategyKind::BollReversion | StrategyKind::BollBreakout => vec![
            ParamDef::int("period", "布林周期", 5, 100, 1),
            ParamDef::float("k", "带宽倍数 k", 0.5, 5.0, 0.1),
        ],
        StrategyKind::KdjCross => vec![
            ParamDef::int("period", "KDJ 周期", 2, 100, 1),
        ],
        StrategyKind::CciReversal => vec![
            ParamDef::int("period", "CCI 周期", 2, 100, 1),
            ParamDef::float("threshold", "极值阈值", 50.0, 300.0, 1.0),
        ],
        StrategyKind::CciMidline => vec![
            ParamDef::int("period", "CCI 周期", 2, 100, 1),
        ],
        StrategyKind::PriceMaCross => vec![
            ParamDef::int("period", "MA 周期", 2, 200, 1),
            ParamDef::bool("useEma", "使用 EMA"),
        ],
        StrategyKind::DonchianBreakout => vec![
            ParamDef::int("period", "通道周期", 3, 200, 1),
        ],
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ParamDef {
    Int { key: &'static str, label: &'static str, min: i64, max: i64, step: i64 },
    Float { key: &'static str, label: &'static str, min: f64, max: f64, step: f64 },
    Bool { key: &'static str, label: &'static str },
}

impl ParamDef {
    fn int(key: &'static str, label: &'static str, min: i64, max: i64, step: i64) -> Self {
        Self::Int { key, label, min, max, step }
    }
    fn float(key: &'static str, label: &'static str, min: f64, max: f64, step: f64) -> Self {
        Self::Float { key, label, min, max, step }
    }
    fn bool(key: &'static str, label: &'static str) -> Self {
        Self::Bool { key, label }
    }
    pub fn key(&self) -> &'static str {
        match self {
            ParamDef::Int { key, .. } | ParamDef::Float { key, .. } | ParamDef::Bool { key, .. } => key,
        }
    }
}

pub(crate) fn preset(
    id: &'static str,
    name: &'static str,
    category: &'static str,
    description: &'static str,
    lookback: usize,
    kind: StrategyKind,
    default_params: StrategyParams,
    evaluator: StrategyEvaluator,
) -> StrategyPreset {
    StrategyPreset {
        id,
        name,
        category,
        description,
        lookback,
        kind,
        default_params,
        evaluator,
    }
}

pub(crate) fn ma_cross_params(fast: usize, slow: usize, use_ema: bool) -> StrategyParams {
    StrategyParams { fast, slow, use_ema, ..Default::default() }
}

pub(crate) fn rsi_reversal_params(period: usize, oversold: f64, overbought: f64) -> StrategyParams {
    StrategyParams { period, oversold, overbought, ..Default::default() }
}

pub(crate) fn rsi_midline_params(period: usize, bull_level: f64, bear_level: f64) -> StrategyParams {
    StrategyParams { period, bull_level, bear_level, ..Default::default() }
}

pub(crate) fn macd_cross_params(fast: usize, slow: usize, signal: usize) -> StrategyParams {
    StrategyParams { fast, slow, signal, ..Default::default() }
}

pub(crate) fn boll_params(period: usize, k: f64) -> StrategyParams {
    StrategyParams { period, k, ..Default::default() }
}

pub(crate) fn kdj_params(period: usize) -> StrategyParams {
    StrategyParams { period, ..Default::default() }
}

pub(crate) fn cci_reversal_params(period: usize, threshold: f64) -> StrategyParams {
    StrategyParams { period, threshold, ..Default::default() }
}

pub(crate) fn cci_midline_params(period: usize) -> StrategyParams {
    StrategyParams { period, ..Default::default() }
}

pub(crate) fn price_ma_params(period: usize, use_ema: bool) -> StrategyParams {
    StrategyParams { period, use_ema, ..Default::default() }
}

pub(crate) fn donchian_params(period: usize) -> StrategyParams {
    StrategyParams { period, ..Default::default() }
}

pub(crate) fn evaluate_ma_cross(
    window: &[Kline],
    fast: usize,
    slow: usize,
    use_ema: bool,
) -> Option<&'static str> {
    let closes = closes(window);
    let fast_series = if use_ema { ema(&closes, fast) } else { ma(&closes, fast) };
    let slow_series = if use_ema { ema(&closes, slow) } else { ma(&closes, slow) };

    crossover_signal(
        fast_series[fast_series.len() - 2],
        fast_series[fast_series.len() - 1],
        slow_series[slow_series.len() - 2],
        slow_series[slow_series.len() - 1],
    )
}

pub(crate) fn evaluate_rsi_reversal(
    window: &[Kline],
    period: usize,
    oversold: f64,
    overbought: f64,
) -> Option<&'static str> {
    let series = rsi(&closes(window), period);
    let (Some(prev), Some(curr)) = (series[series.len() - 2], series[series.len() - 1]) else {
        return None;
    };

    if prev <= oversold && curr > oversold {
        Some("BUY")
    } else if prev >= overbought && curr < overbought {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_rsi_midline(
    window: &[Kline],
    period: usize,
    bull_level: f64,
    bear_level: f64,
) -> Option<&'static str> {
    let series = rsi(&closes(window), period);
    let (Some(prev), Some(curr)) = (series[series.len() - 2], series[series.len() - 1]) else {
        return None;
    };

    if prev <= bull_level && curr > bull_level {
        Some("BUY")
    } else if prev >= bear_level && curr < bear_level {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_rsi_long_only(
    window: &[Kline],
    period: usize,
    entry_level: f64,
    exit_level: f64,
) -> Option<&'static str> {
    let series = rsi(&closes(window), period);
    let (Some(prev), Some(curr)) = (series[series.len() - 2], series[series.len() - 1]) else {
        return None;
    };

    if prev <= entry_level && curr > entry_level {
        Some("BUY")
    } else if prev >= exit_level && curr < exit_level {
        // 只做多策略，平仓但不开空
        Some("EXIT")
    } else {
        None
    }
}

pub(crate) fn evaluate_macd_cross(
    window: &[Kline],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Option<&'static str> {
    let result = macd(&closes(window), fast, slow, signal);
    crossover_signal(
        result.dif[result.dif.len() - 2],
        result.dif[result.dif.len() - 1],
        result.dea[result.dea.len() - 2],
        result.dea[result.dea.len() - 1],
    )
}

pub(crate) fn evaluate_boll_reversion(window: &[Kline], period: usize, k: f64) -> Option<&'static str> {
    let closes = closes(window);
    let bands = boll(&closes, period, k);
    let prev_close = closes[closes.len() - 2];
    let curr_close = closes[closes.len() - 1];
    let (Some(prev_lower), Some(curr_lower), Some(prev_upper), Some(curr_upper)) = (
        bands.lower[bands.lower.len() - 2],
        bands.lower[bands.lower.len() - 1],
        bands.upper[bands.upper.len() - 2],
        bands.upper[bands.upper.len() - 1],
    ) else {
        return None;
    };

    if prev_close < prev_lower && curr_close >= curr_lower {
        Some("BUY")
    } else if prev_close > prev_upper && curr_close <= curr_upper {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_boll_breakout(window: &[Kline], period: usize, k: f64) -> Option<&'static str> {
    let closes = closes(window);
    let bands = boll(&closes, period, k);
    let prev_close = closes[closes.len() - 2];
    let curr_close = closes[closes.len() - 1];
    let (Some(prev_lower), Some(curr_lower), Some(prev_upper), Some(curr_upper)) = (
        bands.lower[bands.lower.len() - 2],
        bands.lower[bands.lower.len() - 1],
        bands.upper[bands.upper.len() - 2],
        bands.upper[bands.upper.len() - 1],
    ) else {
        return None;
    };

    if prev_close <= prev_upper && curr_close > curr_upper {
        Some("BUY")
    } else if prev_close >= prev_lower && curr_close < curr_lower {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_kdj_cross(window: &[Kline], period: usize) -> Option<&'static str> {
    let result = kdj(&highs(window), &lows(window), &closes(window), period);
    crossover_signal(
        result.k[result.k.len() - 2],
        result.k[result.k.len() - 1],
        result.d[result.d.len() - 2],
        result.d[result.d.len() - 1],
    )
}

pub(crate) fn evaluate_cci_reversal(window: &[Kline], period: usize, threshold: f64) -> Option<&'static str> {
    let series = cci(&highs(window), &lows(window), &closes(window), period);
    let (Some(prev), Some(curr)) = (series[series.len() - 2], series[series.len() - 1]) else {
        return None;
    };

    if prev <= -threshold && curr > -threshold {
        Some("BUY")
    } else if prev >= threshold && curr < threshold {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_cci_midline(window: &[Kline], period: usize) -> Option<&'static str> {
    let series = cci(&highs(window), &lows(window), &closes(window), period);
    let (Some(prev), Some(curr)) = (series[series.len() - 2], series[series.len() - 1]) else {
        return None;
    };

    if prev <= 0.0 && curr > 0.0 {
        Some("BUY")
    } else if prev >= 0.0 && curr < 0.0 {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_price_ma_cross(window: &[Kline], period: usize, use_ema: bool) -> Option<&'static str> {
    let closes = closes(window);
    let ma_series = if use_ema { ema(&closes, period) } else { ma(&closes, period) };
    let prev_close = closes[closes.len() - 2];
    let curr_close = closes[closes.len() - 1];
    let (Some(prev_line), Some(curr_line)) = (
        ma_series[ma_series.len() - 2],
        ma_series[ma_series.len() - 1],
    ) else {
        return None;
    };

    if prev_close <= prev_line && curr_close > curr_line {
        Some("BUY")
    } else if prev_close >= prev_line && curr_close < curr_line {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn evaluate_donchian_breakout(window: &[Kline], period: usize) -> Option<&'static str> {
    if window.len() < period + 1 {
        return None;
    }

    let prev_close = window[window.len() - 2].close;
    let curr_close = window[window.len() - 1].close;
    let channel = &window[window.len() - 1 - period..window.len() - 1];
    let breakout_high = channel.iter().map(|k| k.high).fold(f64::MIN, f64::max);
    let breakout_low = channel.iter().map(|k| k.low).fold(f64::MAX, f64::min);

    if prev_close <= breakout_high && curr_close > breakout_high {
        Some("BUY")
    } else if prev_close >= breakout_low && curr_close < breakout_low {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn crossover_signal(
    prev_fast: Option<f64>,
    curr_fast: Option<f64>,
    prev_slow: Option<f64>,
    curr_slow: Option<f64>,
) -> Option<&'static str> {
    let (Some(prev_fast), Some(curr_fast), Some(prev_slow), Some(curr_slow)) =
        (prev_fast, curr_fast, prev_slow, curr_slow)
    else {
        return None;
    };

    if prev_fast <= prev_slow && curr_fast > curr_slow {
        Some("BUY")
    } else if prev_fast >= prev_slow && curr_fast < curr_slow {
        Some("SELL")
    } else {
        None
    }
}

pub(crate) fn closes(window: &[Kline]) -> Vec<f64> {
    window.iter().map(|item| item.close).collect()
}

pub(crate) fn highs(window: &[Kline]) -> Vec<f64> {
    window.iter().map(|item| item.high).collect()
}

pub(crate) fn lows(window: &[Kline]) -> Vec<f64> {
    window.iter().map(|item| item.low).collect()
}

#[cfg(test)]
mod tests {
    use super::common_strategy_presets;

    #[test]
    fn common_strategy_catalog_has_21_presets() {
        assert_eq!(common_strategy_presets().len(), 105);
    }
}

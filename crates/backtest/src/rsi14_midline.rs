use trade_common::binance::types::Kline;

use super::{evaluate_rsi_midline, preset, rsi_midline_params, StrategyKind, StrategyPreset};

/// RSI 中轴趋势策略组：围绕 RSI 中轴阈值的趋势跟随版本。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("rsi10_midline", "RSI10 中轴趋势", "RSI 趋势", "10 周期 RSI 中轴突破", 30, StrategyKind::RsiMidline, rsi_midline_params(10, 52.0, 48.0), evaluate::<10, 52, 48>),
        preset("rsi12_midline", "RSI12 中轴趋势", "RSI 趋势", "12 周期 RSI 中轴突破", 35, StrategyKind::RsiMidline, rsi_midline_params(12, 55.0, 45.0), evaluate::<12, 55, 45>),
        preset("rsi14_midline", "RSI14 中轴趋势", "RSI 趋势", "RSI 站上/跌破中轴区间判断趋势", 40, StrategyKind::RsiMidline, rsi_midline_params(14, 55.0, 45.0), evaluate::<14, 55, 45>),
        preset("rsi18_midline", "RSI18 中轴趋势", "RSI 趋势", "更平滑的 RSI 中轴跟随", 50, StrategyKind::RsiMidline, rsi_midline_params(18, 55.0, 45.0), evaluate::<18, 55, 45>),
        preset("rsi21_midline", "RSI21 强趋势", "RSI 趋势", "使用更宽中轴区间的趋势版本", 60, StrategyKind::RsiMidline, rsi_midline_params(21, 60.0, 40.0), evaluate::<21, 60, 40>),
    ]
}

fn evaluate<const PERIOD: usize, const BULL: usize, const BEAR: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_rsi_midline(window, PERIOD, BULL as f64, BEAR as f64)
}

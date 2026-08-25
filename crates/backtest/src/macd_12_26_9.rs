use trade_common::binance::types::Kline;

use super::{evaluate_macd_cross, macd_cross_params, preset, StrategyKind, StrategyPreset};

/// 标准 MACD 趋势策略组：围绕经典 12/26/9 的多组中周期 MACD 参数。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("macd_10_22_7", "MACD 10/22/7", "MACD 趋势", "更灵敏的中期 MACD", 70, StrategyKind::MacdCross, macd_cross_params(10, 22, 7), evaluate::<10, 22, 7>),
        preset("macd_12_26_9", "MACD 12/26/9", "MACD 趋势", "经典 MACD DIF/DEA 金叉死叉", 90, StrategyKind::MacdCross, macd_cross_params(12, 26, 9), evaluate::<12, 26, 9>),
        preset("macd_13_28_9", "MACD 13/28/9", "MACD 趋势", "略平滑的 MACD 版本", 95, StrategyKind::MacdCross, macd_cross_params(13, 28, 9), evaluate::<13, 28, 9>),
        preset("macd_14_30_9", "MACD 14/30/9", "MACD 趋势", "14/30 的中期 MACD", 100, StrategyKind::MacdCross, macd_cross_params(14, 30, 9), evaluate::<14, 30, 9>),
        preset("macd_16_34_9", "MACD 16/34/9", "MACD 趋势", "更慢节奏的 MACD 趋势跟随", 110, StrategyKind::MacdCross, macd_cross_params(16, 34, 9), evaluate::<16, 34, 9>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize, const SIGNAL: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_macd_cross(window, FAST, SLOW, SIGNAL)
}

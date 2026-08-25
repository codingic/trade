use trade_common::binance::types::Kline;

use super::{evaluate_macd_cross, macd_cross_params, preset, StrategyKind, StrategyPreset};

/// 快速 MACD 策略组：更短的均线和 signal 周期用于捕捉短趋势。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("macd_5_13_4", "MACD 5/13/4", "MACD 趋势", "极快的短线 MACD", 45, StrategyKind::MacdCross, macd_cross_params(5, 13, 4), evaluate::<5, 13, 4>),
        preset("macd_6_19_5", "MACD 6/19/5", "MACD 趋势", "短线 MACD 趋势版本", 55, StrategyKind::MacdCross, macd_cross_params(6, 19, 5), evaluate::<6, 19, 5>),
        preset("macd_8_17_5", "MACD 8/17/5", "MACD 趋势", "偏短节奏的 MACD 交叉", 55, StrategyKind::MacdCross, macd_cross_params(8, 17, 5), evaluate::<8, 17, 5>),
        preset("macd_8_21_5", "MACD 8/21/5", "MACD 趋势", "更快的 MACD 交叉组合", 70, StrategyKind::MacdCross, macd_cross_params(8, 21, 5), evaluate::<8, 21, 5>),
        preset("macd_9_24_6", "MACD 9/24/6", "MACD 趋势", "稍慢一点的快速 MACD", 80, StrategyKind::MacdCross, macd_cross_params(9, 24, 6), evaluate::<9, 24, 6>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize, const SIGNAL: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_macd_cross(window, FAST, SLOW, SIGNAL)
}

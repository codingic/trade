use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 标准 EMA 交叉策略组：围绕 12/26 一带的中期趋势跟随。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("ema_11_25", "EMA 11/25 趋势", "EMA 趋势", "接近 MACD 的前置 EMA 组合", 75, StrategyKind::MaCross, ma_cross_params(11, 25, true), evaluate::<11, 25>),
        preset("ema_12_26", "EMA 12/26 趋势", "EMA 趋势", "接近 MACD 基础均线组合", 80, StrategyKind::MaCross, ma_cross_params(12, 26, true), evaluate::<12, 26>),
        preset("ema_13_30", "EMA 13/30 趋势", "EMA 趋势", "更平滑的 EMA 中期趋势组合", 90, StrategyKind::MaCross, ma_cross_params(13, 30, true), evaluate::<13, 30>),
        preset("ema_14_33", "EMA 14/33 趋势", "EMA 趋势", "14/33 EMA 中期趋势版本", 95, StrategyKind::MaCross, ma_cross_params(14, 33, true), evaluate::<14, 33>),
        preset("ema_15_35", "EMA 15/35 趋势", "EMA 趋势", "15/35 EMA 更偏波段跟随", 100, StrategyKind::MaCross, ma_cross_params(15, 35, true), evaluate::<15, 35>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, true)
}

use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 波段 SMA 交叉策略组：偏向中长周期的趋势轮换。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("sma_15_45", "SMA 15/45 波段", "均线趋势", "中周期波段 SMA 交叉", 120, StrategyKind::MaCross, ma_cross_params(15, 45, false), evaluate::<15, 45>),
        preset("sma_18_54", "SMA 18/54 波段", "均线趋势", "18/54 SMA 的波段趋势版本", 135, StrategyKind::MaCross, ma_cross_params(18, 54, false), evaluate::<18, 54>),
        preset("sma_20_60", "SMA 20/60 波段", "均线趋势", "适合更长一点的波段跟随", 140, StrategyKind::MaCross, ma_cross_params(20, 60, false), evaluate::<20, 60>),
        preset("sma_25_75", "SMA 25/75 波段", "均线趋势", "更慢的长波段趋势切换", 170, StrategyKind::MaCross, ma_cross_params(25, 75, false), evaluate::<25, 75>),
        preset("sma_30_90", "SMA 30/90 长波段", "均线趋势", "极慢速长波段均线跟随", 200, StrategyKind::MaCross, ma_cross_params(30, 90, false), evaluate::<30, 90>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, false)
}

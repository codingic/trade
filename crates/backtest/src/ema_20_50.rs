use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 波段 EMA 交叉策略组：偏向中长周期趋势跟随。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("ema_18_45", "EMA 18/45 波段", "EMA 趋势", "18/45 EMA 的中周期波段版本", 110, StrategyKind::MaCross, ma_cross_params(18, 45, true), evaluate::<18, 45>),
        preset("ema_20_50", "EMA 20/50 波段", "EMA 趋势", "中周期 EMA 趋势跟随", 120, StrategyKind::MaCross, ma_cross_params(20, 50, true), evaluate::<20, 50>),
        preset("ema_21_55", "EMA 21/55 波段", "EMA 趋势", "21/55 EMA 更平滑的波段组合", 130, StrategyKind::MaCross, ma_cross_params(21, 55, true), evaluate::<21, 55>),
        preset("ema_24_60", "EMA 24/60 波段", "EMA 趋势", "24/60 EMA 更慢的波段跟随", 145, StrategyKind::MaCross, ma_cross_params(24, 60, true), evaluate::<24, 60>),
        preset("ema_30_90", "EMA 30/90 长波段", "EMA 趋势", "长周期 EMA 趋势版本", 200, StrategyKind::MaCross, ma_cross_params(30, 90, true), evaluate::<30, 90>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, true)
}

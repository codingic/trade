use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 快速 EMA 交叉策略组：适合较灵敏的短趋势切换。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("ema_5_13", "EMA 5/13 快速", "EMA 趋势", "更短周期的快速 EMA 交叉", 40, StrategyKind::MaCross, ma_cross_params(5, 13, true), evaluate::<5, 13>),
        preset("ema_6_15", "EMA 6/15 快速", "EMA 趋势", "6/15 EMA 用于更平滑的短线切换", 45, StrategyKind::MaCross, ma_cross_params(6, 15, true), evaluate::<6, 15>),
        preset("ema_8_21", "EMA 8/21 趋势", "EMA 趋势", "8/21 EMA 的经典短趋势跟随", 55, StrategyKind::MaCross, ma_cross_params(8, 21, true), evaluate::<8, 21>),
        preset("ema_9_21", "EMA 9/21 金叉", "EMA 趋势", "更灵敏的 EMA 交叉", 60, StrategyKind::MaCross, ma_cross_params(9, 21, true), evaluate::<9, 21>),
        preset("ema_10_24", "EMA 10/24 趋势", "EMA 趋势", "10/24 EMA 的更稳健短趋势切换", 70, StrategyKind::MaCross, ma_cross_params(10, 24, true), evaluate::<10, 24>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, true)
}

use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 短周期 SMA 交叉策略组：覆盖更敏感的短中期双均线组合。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("sma_5_20", "SMA 5/20 金叉", "均线趋势", "更灵敏的短线 SMA 交叉", 50, StrategyKind::MaCross, ma_cross_params(5, 20, false), evaluate::<5, 20>),
        preset("sma_6_18", "SMA 6/18 趋势", "均线趋势", "短周期 SMA 趋势切换", 45, StrategyKind::MaCross, ma_cross_params(6, 18, false), evaluate::<6, 18>),
        preset("sma_7_25", "SMA 7/25 金叉", "均线趋势", "经典短中期双均线交叉", 60, StrategyKind::MaCross, ma_cross_params(7, 25, false), evaluate::<7, 25>),
        preset("sma_8_24", "SMA 8/24 趋势", "均线趋势", "略平滑的短中期 SMA 交叉", 60, StrategyKind::MaCross, ma_cross_params(8, 24, false), evaluate::<8, 24>),
        preset("sma_9_27", "SMA 9/27 趋势", "均线趋势", "适合更稳健的短波段切换", 70, StrategyKind::MaCross, ma_cross_params(9, 27, false), evaluate::<9, 27>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, false)
}

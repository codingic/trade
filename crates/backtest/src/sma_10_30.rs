use trade_common::binance::types::Kline;

use super::{evaluate_ma_cross, ma_cross_params, preset, StrategyKind, StrategyPreset};

/// 中短周期 SMA 交叉策略组：在 10/30 基础上扩出多组平滑趋势参数。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("sma_10_30", "SMA 10/30 趋势", "均线趋势", "更平滑的双均线趋势跟随", 80, StrategyKind::MaCross, ma_cross_params(10, 30, false), evaluate::<10, 30>),
        preset("sma_11_33", "SMA 11/33 趋势", "均线趋势", "11/33 SMA 的中短趋势版本", 90, StrategyKind::MaCross, ma_cross_params(11, 33, false), evaluate::<11, 33>),
        preset("sma_12_36", "SMA 12/36 趋势", "均线趋势", "12/36 SMA 的中短趋势版本", 100, StrategyKind::MaCross, ma_cross_params(12, 36, false), evaluate::<12, 36>),
        preset("sma_13_39", "SMA 13/39 趋势", "均线趋势", "13/39 SMA 的更平滑组合", 105, StrategyKind::MaCross, ma_cross_params(13, 39, false), evaluate::<13, 39>),
        preset("sma_14_42", "SMA 14/42 趋势", "均线趋势", "14/42 SMA 的更长周期趋势跟随", 110, StrategyKind::MaCross, ma_cross_params(14, 42, false), evaluate::<14, 42>),
    ]
}

fn evaluate<const FAST: usize, const SLOW: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_ma_cross(window, FAST, SLOW, false)
}

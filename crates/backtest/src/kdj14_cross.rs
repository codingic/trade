use trade_common::binance::types::Kline;

use super::{evaluate_kdj_cross, kdj_params, preset, StrategyKind, StrategyPreset};

/// 平滑 KDJ 动量策略组：更长的 KDJ 周期减少噪声。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("kdj12_cross", "KDJ12 平滑交叉", "KDJ 动量", "12 周期的平滑 KDJ", 36, StrategyKind::KdjCross, kdj_params(12), evaluate::<12>),
        preset("kdj14_cross", "KDJ14 平滑交叉", "KDJ 动量", "更平滑的 KDJ 交叉策略", 40, StrategyKind::KdjCross, kdj_params(14), evaluate::<14>),
        preset("kdj16_cross", "KDJ16 平滑交叉", "KDJ 动量", "16 周期 KDJ 交叉", 45, StrategyKind::KdjCross, kdj_params(16), evaluate::<16>),
        preset("kdj18_cross", "KDJ18 平滑交叉", "KDJ 动量", "更慢速的 KDJ 切换", 50, StrategyKind::KdjCross, kdj_params(18), evaluate::<18>),
        preset("kdj21_cross", "KDJ21 平滑交叉", "KDJ 动量", "21 周期 KDJ 用于长一点的转折", 60, StrategyKind::KdjCross, kdj_params(21), evaluate::<21>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_kdj_cross(window, PERIOD)
}

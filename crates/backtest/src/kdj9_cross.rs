use trade_common::binance::types::Kline;

use super::{evaluate_kdj_cross, kdj_params, preset, StrategyKind, StrategyPreset};

/// 快速 KDJ 动量策略组：用较短 KDJ 周期捕捉快速转折。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("kdj7_cross", "KDJ7 交叉", "KDJ 动量", "更灵敏的 KDJ 交叉", 25, StrategyKind::KdjCross, kdj_params(7), evaluate::<7>),
        preset("kdj8_cross", "KDJ8 交叉", "KDJ 动量", "8 周期 KDJ 交叉", 28, StrategyKind::KdjCross, kdj_params(8), evaluate::<8>),
        preset("kdj9_cross", "KDJ9 交叉", "KDJ 动量", "K 线上穿/下穿 D 线", 30, StrategyKind::KdjCross, kdj_params(9), evaluate::<9>),
        preset("kdj10_cross", "KDJ10 交叉", "KDJ 动量", "10 周期 KDJ 动量切换", 32, StrategyKind::KdjCross, kdj_params(10), evaluate::<10>),
        preset("kdj11_cross", "KDJ11 交叉", "KDJ 动量", "更平滑的短周期 KDJ", 35, StrategyKind::KdjCross, kdj_params(11), evaluate::<11>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_kdj_cross(window, PERIOD)
}

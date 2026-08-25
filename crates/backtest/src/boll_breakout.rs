use trade_common::binance::types::Kline;

use super::{boll_params, evaluate_boll_breakout, preset, StrategyKind, StrategyPreset};

/// 布林带突破策略组：不同布林带窗口和带宽下的顺势突破版本。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("boll14_breakout_18", "BOLL14 突破 1.8", "布林带", "短周期布林突破", 40, StrategyKind::BollBreakout, boll_params(14, 1.8), evaluate::<14, 18>),
        preset("boll20_breakout_18", "BOLL20 突破 1.8", "布林带", "更紧带宽的突破版本", 45, StrategyKind::BollBreakout, boll_params(20, 1.8), evaluate::<20, 18>),
        preset("boll20_breakout_20", "BOLL20 突破", "布林带", "价格突破上轨追多，跌破下轨追空", 50, StrategyKind::BollBreakout, boll_params(20, 2.0), evaluate::<20, 20>),
        preset("boll25_breakout_20", "BOLL25 突破", "布林带", "更慢速的布林突破策略", 60, StrategyKind::BollBreakout, boll_params(25, 2.0), evaluate::<25, 20>),
        preset("boll30_breakout_22", "BOLL30 宽带突破", "布林带", "更宽带宽的趋势突破版本", 70, StrategyKind::BollBreakout, boll_params(30, 2.2), evaluate::<30, 22>),
    ]
}

fn evaluate<const PERIOD: usize, const K_X10: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_boll_breakout(window, PERIOD, K_X10 as f64 / 10.0)
}

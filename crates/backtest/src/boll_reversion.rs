use trade_common::binance::types::Kline;

use super::{boll_params, evaluate_boll_reversion, preset, StrategyKind, StrategyPreset};

/// 布林带均值回归策略组：不同布林带窗口和带宽下的回归版本。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("boll14_reversion_18", "BOLL14 均值回归 1.8", "布林带", "更灵敏的短周期布林回归", 40, StrategyKind::BollReversion, boll_params(14, 1.8), evaluate::<14, 18>),
        preset("boll20_reversion_18", "BOLL20 均值回归 1.8", "布林带", "更紧带宽的回归策略", 45, StrategyKind::BollReversion, boll_params(20, 1.8), evaluate::<20, 18>),
        preset("boll20_reversion_20", "BOLL20 均值回归", "布林带", "价格跌破下轨回归买入，上轨回落卖出", 50, StrategyKind::BollReversion, boll_params(20, 2.0), evaluate::<20, 20>),
        preset("boll20_reversion_22", "BOLL20 宽带回归", "布林带", "更宽带宽的布林回归版本", 55, StrategyKind::BollReversion, boll_params(20, 2.2), evaluate::<20, 22>),
        preset("boll25_reversion_20", "BOLL25 均值回归", "布林带", "更慢速窗口的均值回归", 60, StrategyKind::BollReversion, boll_params(25, 2.0), evaluate::<25, 20>),
    ]
}

fn evaluate<const PERIOD: usize, const K_X10: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_boll_reversion(window, PERIOD, K_X10 as f64 / 10.0)
}

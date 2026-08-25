use trade_common::binance::types::Kline;

use super::{donchian_params, evaluate_donchian_breakout, preset, StrategyKind, StrategyPreset};

/// Donchian 通道突破策略组：用不同通道长度做趋势突破。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("donchian10", "Donchian10 突破", "通道突破", "突破最近 10 根高低点通道", 35, StrategyKind::DonchianBreakout, donchian_params(10), evaluate::<10>),
        preset("donchian15", "Donchian15 突破", "通道突破", "突破最近 15 根高低点通道", 45, StrategyKind::DonchianBreakout, donchian_params(15), evaluate::<15>),
        preset("donchian20", "Donchian20 突破", "通道突破", "突破最近 20 根高低点通道", 60, StrategyKind::DonchianBreakout, donchian_params(20), evaluate::<20>),
        preset("donchian30", "Donchian30 突破", "通道突破", "突破最近 30 根高低点通道", 80, StrategyKind::DonchianBreakout, donchian_params(30), evaluate::<30>),
        preset("donchian40", "Donchian40 突破", "通道突破", "突破最近 40 根高低点通道", 100, StrategyKind::DonchianBreakout, donchian_params(40), evaluate::<40>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_donchian_breakout(window, PERIOD)
}

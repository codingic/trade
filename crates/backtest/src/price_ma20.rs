use trade_common::binance::types::Kline;

use super::{evaluate_price_ma_cross, preset, price_ma_params, StrategyKind, StrategyPreset};

/// 价格穿越 SMA 策略组：用收盘价穿越不同周期 SMA 来做方向切换。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("price_ma10", "价格穿越 MA10", "价格趋势", "价格上穿/下穿 MA10", 30, StrategyKind::PriceMaCross, price_ma_params(10, false), evaluate::<10>),
        preset("price_ma15", "价格穿越 MA15", "价格趋势", "价格上穿/下穿 MA15", 40, StrategyKind::PriceMaCross, price_ma_params(15, false), evaluate::<15>),
        preset("price_ma20", "价格穿越 MA20", "价格趋势", "价格上穿/下穿 MA20", 50, StrategyKind::PriceMaCross, price_ma_params(20, false), evaluate::<20>),
        preset("price_ma30", "价格穿越 MA30", "价格趋势", "价格上穿/下穿 MA30", 70, StrategyKind::PriceMaCross, price_ma_params(30, false), evaluate::<30>),
        preset("price_ma40", "价格穿越 MA40", "价格趋势", "价格上穿/下穿 MA40", 90, StrategyKind::PriceMaCross, price_ma_params(40, false), evaluate::<40>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_price_ma_cross(window, PERIOD, false)
}

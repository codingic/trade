use trade_common::binance::types::Kline;

use super::{evaluate_price_ma_cross, preset, price_ma_params, StrategyKind, StrategyPreset};

/// 价格穿越 EMA 策略组：用收盘价穿越不同周期 EMA 来做方向切换。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("price_ema21", "价格穿越 EMA21", "价格趋势", "价格上穿/下穿 EMA21", 60, StrategyKind::PriceMaCross, price_ma_params(21, true), evaluate::<21>),
        preset("price_ema34", "价格穿越 EMA34", "价格趋势", "价格上穿/下穿 EMA34", 80, StrategyKind::PriceMaCross, price_ma_params(34, true), evaluate::<34>),
        preset("price_ema50", "价格穿越 EMA50", "价格趋势", "价格上穿/下穿 EMA50", 120, StrategyKind::PriceMaCross, price_ma_params(50, true), evaluate::<50>),
        preset("price_ema55", "价格穿越 EMA55", "价格趋势", "价格上穿/下穿 EMA55", 130, StrategyKind::PriceMaCross, price_ma_params(55, true), evaluate::<55>),
        preset("price_ema89", "价格穿越 EMA89", "价格趋势", "价格上穿/下穿 EMA89", 180, StrategyKind::PriceMaCross, price_ma_params(89, true), evaluate::<89>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_price_ma_cross(window, PERIOD, true)
}

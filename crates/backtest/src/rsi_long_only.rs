use trade_common::binance::types::Kline;

use super::{evaluate_rsi_long_only, preset, rsi_reversal_params, StrategyKind, StrategyPreset};

/// RSI 超卖只做多策略组：RSI 跌破低位后回升开多，涨到高位后回落平多。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("rsi14_long_20_90", "RSI14 20多90平", "RSI 做多", "RSI14 跌破20后回升开多，涨到90后回落平多", 40, StrategyKind::RsiLongOnly, rsi_reversal_params(14, 20.0, 90.0), evaluate::<14, 20, 90>),
        preset("rsi14_long_25_85", "RSI14 25多85平", "RSI 做多", "RSI14 跌破25后回升开多，涨到85后回落平多", 40, StrategyKind::RsiLongOnly, rsi_reversal_params(14, 25.0, 85.0), evaluate::<14, 25, 85>),
        preset("rsi14_long_30_80", "RSI14 30多80平", "RSI 做多", "RSI14 跌破30后回升开多，涨到80后回落平多", 40, StrategyKind::RsiLongOnly, rsi_reversal_params(14, 30.0, 80.0), evaluate::<14, 30, 80>),
        preset("rsi7_long_20_80", "RSI7 20多80平", "RSI 做多", "更灵敏的7周期RSI超卖做多", 30, StrategyKind::RsiLongOnly, rsi_reversal_params(7, 20.0, 80.0), evaluate::<7, 20, 80>),
        preset("rsi21_long_20_90", "RSI21 20多90平", "RSI 做多", "更平滑的21周期RSI超卖做多", 50, StrategyKind::RsiLongOnly, rsi_reversal_params(21, 20.0, 90.0), evaluate::<21, 20, 90>),
    ]
}

fn evaluate<const PERIOD: usize, const ENTRY: usize, const EXIT: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_rsi_long_only(window, PERIOD, ENTRY as f64, EXIT as f64)
}

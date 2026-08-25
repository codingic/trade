use trade_common::binance::types::Kline;

use super::{evaluate_rsi_reversal, preset, rsi_reversal_params, StrategyKind, StrategyPreset};

/// 标准 RSI 反转策略组：通过不同 RSI 周期和阈值做超买超卖回归。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("rsi10_reversal", "RSI10 超卖反转", "RSI 反转", "更短周期的 RSI 反转", 30, StrategyKind::RsiReversal, rsi_reversal_params(10, 30.0, 70.0), evaluate::<10, 30, 70>),
        preset("rsi12_reversal", "RSI12 超卖反转", "RSI 反转", "12 周期 RSI 的反转版本", 35, StrategyKind::RsiReversal, rsi_reversal_params(12, 30.0, 70.0), evaluate::<12, 30, 70>),
        preset("rsi14_reversal", "RSI14 超卖反转", "RSI 反转", "RSI 跌破超卖后回升买入，超买后回落卖出", 40, StrategyKind::RsiReversal, rsi_reversal_params(14, 30.0, 70.0), evaluate::<14, 30, 70>),
        preset("rsi16_reversal", "RSI16 宽阈值反转", "RSI 反转", "更平滑、阈值更宽的反转版本", 45, StrategyKind::RsiReversal, rsi_reversal_params(16, 35.0, 65.0), evaluate::<16, 35, 65>),
        preset("rsi18_reversal", "RSI18 宽阈值反转", "RSI 反转", "更慢速 RSI 反转策略", 50, StrategyKind::RsiReversal, rsi_reversal_params(18, 35.0, 65.0), evaluate::<18, 35, 65>),
    ]
}

fn evaluate<const PERIOD: usize, const OVERSOLD: usize, const OVERBOUGHT: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_rsi_reversal(window, PERIOD, OVERSOLD as f64, OVERBOUGHT as f64)
}

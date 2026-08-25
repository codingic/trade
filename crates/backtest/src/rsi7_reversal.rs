use trade_common::binance::types::Kline;

use super::{evaluate_rsi_reversal, preset, rsi_reversal_params, StrategyKind, StrategyPreset};

/// 快速 RSI 反转策略组：用更短周期的 RSI 捕捉短线情绪反转。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("rsi5_reversal", "RSI5 极快反转", "RSI 反转", "极短周期 RSI 的情绪反转", 20, StrategyKind::RsiReversal, rsi_reversal_params(5, 20.0, 80.0), evaluate::<5, 20, 80>),
        preset("rsi6_reversal", "RSI6 短线反转", "RSI 反转", "6 周期 RSI 的短线回归", 22, StrategyKind::RsiReversal, rsi_reversal_params(6, 25.0, 75.0), evaluate::<6, 25, 75>),
        preset("rsi7_reversal", "RSI7 短线反转", "RSI 反转", "更快的 RSI 短线反转", 25, StrategyKind::RsiReversal, rsi_reversal_params(7, 25.0, 75.0), evaluate::<7, 25, 75>),
        preset("rsi8_reversal", "RSI8 短线反转", "RSI 反转", "8 周期 RSI 的短线反转", 30, StrategyKind::RsiReversal, rsi_reversal_params(8, 30.0, 70.0), evaluate::<8, 30, 70>),
        preset("rsi9_reversal", "RSI9 短线反转", "RSI 反转", "9 周期 RSI 的平滑短线反转", 32, StrategyKind::RsiReversal, rsi_reversal_params(9, 30.0, 70.0), evaluate::<9, 30, 70>),
    ]
}

fn evaluate<const PERIOD: usize, const OVERSOLD: usize, const OVERBOUGHT: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_rsi_reversal(window, PERIOD, OVERSOLD as f64, OVERBOUGHT as f64)
}

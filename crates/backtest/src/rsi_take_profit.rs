use trade_common::binance::types::Kline;

use super::{evaluate_rsi_take_profit, preset, rsi_reversal_params, StrategyKind, StrategyPreset};

/// RSI 带止盈策略组：RSI 上穿低位开多，上穿高位开空，盈利达到目标后止盈平仓。
///
/// 止盈百分比由回测引擎的 `take_profit_pct` 参数控制，本文件只定义入场信号。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset(
            "rsi14_tp_20_90",
            "RSI14 20多90空 止盈",
            "RSI止盈",
            "RSI14 上穿20开多，上穿90开空，盈利10%止盈（4h推荐）",
            30,
            StrategyKind::RsiTakeProfit,
            rsi_reversal_params(14, 20.0, 90.0),
            evaluate::<14, 20, 90>,
        ),
        preset(
            "rsi14_tp_25_85",
            "RSI14 25多85空 止盈",
            "RSI止盈",
            "RSI14 上穿25开多，上穿85开空，盈利10%止盈",
            30,
            StrategyKind::RsiTakeProfit,
            rsi_reversal_params(14, 25.0, 85.0),
            evaluate::<14, 25, 85>,
        ),
        preset(
            "rsi7_tp_20_80",
            "RSI7 20多80空 止盈",
            "RSI止盈",
            "更灵敏的7周期RSI，上穿20开多，上穿80开空，盈利10%止盈",
            20,
            StrategyKind::RsiTakeProfit,
            rsi_reversal_params(7, 20.0, 80.0),
            evaluate::<7, 20, 80>,
        ),
    ]
}

fn evaluate<const PERIOD: usize, const LONG: usize, const SHORT: usize>(
    window: &[Kline],
) -> Option<&'static str> {
    evaluate_rsi_take_profit(window, PERIOD, LONG as f64, SHORT as f64)
}

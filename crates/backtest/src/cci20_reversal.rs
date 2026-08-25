use trade_common::binance::types::Kline;

use super::{cci_reversal_params, evaluate_cci_reversal, preset, StrategyKind, StrategyPreset};

/// CCI 反转策略组：不同窗口和阈值下的极值回归版本。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("cci14_reversal_100", "CCI14 反转 100", "CCI 反转", "更快的 CCI 极值回归", 35, StrategyKind::CciReversal, cci_reversal_params(14, 100.0), evaluate::<14, 100>),
        preset("cci20_reversal_100", "CCI20 反转 100", "CCI 反转", "CCI 从极值区回归时反向入场", 50, StrategyKind::CciReversal, cci_reversal_params(20, 100.0), evaluate::<20, 100>),
        preset("cci20_reversal_150", "CCI20 反转 150", "CCI 反转", "更极端阈值的 CCI 反转", 55, StrategyKind::CciReversal, cci_reversal_params(20, 150.0), evaluate::<20, 150>),
        preset("cci25_reversal_120", "CCI25 反转 120", "CCI 反转", "25 周期的 CCI 反转版本", 60, StrategyKind::CciReversal, cci_reversal_params(25, 120.0), evaluate::<25, 120>),
        preset("cci30_reversal_150", "CCI30 反转 150", "CCI 反转", "更慢速、更极端的 CCI 回归", 70, StrategyKind::CciReversal, cci_reversal_params(30, 150.0), evaluate::<30, 150>),
    ]
}

fn evaluate<const PERIOD: usize, const THRESHOLD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_cci_reversal(window, PERIOD, THRESHOLD as f64)
}

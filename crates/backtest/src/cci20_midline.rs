use trade_common::binance::types::Kline;

use super::{cci_midline_params, evaluate_cci_midline, preset, StrategyKind, StrategyPreset};

/// CCI 中轴趋势策略组：不同窗口下的 CCI 0 轴趋势跟随。
pub fn strategies() -> Vec<StrategyPreset> {
    vec![
        preset("cci10_midline", "CCI10 中轴", "CCI 趋势", "10 周期 CCI 的 0 轴切换", 30, StrategyKind::CciMidline, cci_midline_params(10), evaluate::<10>),
        preset("cci14_midline", "CCI14 中轴", "CCI 趋势", "14 周期 CCI 中轴跟随", 35, StrategyKind::CciMidline, cci_midline_params(14), evaluate::<14>),
        preset("cci20_midline", "CCI20 中轴", "CCI 趋势", "CCI 上穿/下穿 0 轴判断趋势", 50, StrategyKind::CciMidline, cci_midline_params(20), evaluate::<20>),
        preset("cci26_midline", "CCI26 中轴", "CCI 趋势", "26 周期 CCI 的平滑趋势版本", 60, StrategyKind::CciMidline, cci_midline_params(26), evaluate::<26>),
        preset("cci30_midline", "CCI30 中轴", "CCI 趋势", "更慢速的 CCI 中轴趋势", 70, StrategyKind::CciMidline, cci_midline_params(30), evaluate::<30>),
    ]
}

fn evaluate<const PERIOD: usize>(window: &[Kline]) -> Option<&'static str> {
    evaluate_cci_midline(window, PERIOD)
}

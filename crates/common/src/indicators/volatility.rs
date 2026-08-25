/// 布林带（Bollinger Bands）
///
/// 中轨 = MA(period)，上/下轨 = 中轨 ± `k` 倍标准差。
/// - `period`：常用 20，`k`：常用 2
/// - 价格触及上轨 = 超买/可能回落，触及下轨 = 超卖/可能反弹
pub struct Boll {
    pub upper: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn boll(closes: &[f64], period: usize, k: f64) -> Boll {
    let n = closes.len();
    let mut upper = vec![None; n];
    let mut middle = vec![None; n];
    let mut lower = vec![None; n];

    for i in period - 1..n {
        let window = &closes[i + 1 - period..=i];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window.iter().map(|close| (close - mean).powi(2)).sum::<f64>()
            / period as f64;
        let std = variance.sqrt();

        middle[i] = Some(mean);
        upper[i] = Some(mean + k * std);
        lower[i] = Some(mean - k * std);
    }

    Boll { upper, middle, lower }
}

/// 真实波幅（ATR，Average True Range）
///
/// 衡量波动率，常用于设置止损（如止损 = 入场价 ± 2*ATR）。
/// - `period`：常用 14
/// - 用 Wilder 平滑法（和 RSI 一致）
pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut result = vec![None; n];
    if n <= period {
        return result;
    }

    let mut true_ranges = vec![0.0; n];
    for i in 1..n {
        let high_low = highs[i] - lows[i];
        let high_prev_close = (highs[i] - closes[i - 1]).abs();
        let low_prev_close = (lows[i] - closes[i - 1]).abs();
        true_ranges[i] = high_low.max(high_prev_close).max(low_prev_close);
    }

    let mut atr_val = true_ranges[1..=period].iter().sum::<f64>() / period as f64;
    result[period] = Some(atr_val);

    for i in period + 1..n {
        atr_val = (atr_val * (period as f64 - 1.0) + true_ranges[i]) / period as f64;
        result[i] = Some(atr_val);
    }

    result
}
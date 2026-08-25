/// 顺势指标（CCI，Commodity Channel Index）
///
/// 衡量价格偏离其统计均值（典型价格）的程度。
/// - `period`：常用 20
/// - CCI > +100 超买，CCI < -100 超卖
pub fn cci(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut result = vec![None; n];

    let typical_prices: Vec<f64> = (0..n)
        .map(|i| (highs[i] + lows[i] + closes[i]) / 3.0)
        .collect();

    for i in period - 1..n {
        let window = &typical_prices[i + 1 - period..=i];
        let mean = window.iter().sum::<f64>() / period as f64;
        let mean_deviation = window.iter().map(|price| (price - mean).abs()).sum::<f64>()
            / period as f64;

        let cci_val = if mean_deviation == 0.0 {
            0.0
        } else {
            (typical_prices[i] - mean) / (0.015 * mean_deviation)
        };
        result[i] = Some(cci_val);
    }
    result
}
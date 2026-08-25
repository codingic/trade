/// 相对强弱指标（RSI），用 Wilder 平滑法
///
/// - `period`：常用 14
/// - 返回 `Vec<Option<f64>>`，取值 0~100
pub fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; closes.len()];
    if closes.len() <= period {
        return result;
    }

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for i in 1..=period {
        let diff = closes[i] - closes[i - 1];
        if diff >= 0.0 {
            avg_gain += diff;
        } else {
            avg_loss += -diff;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;

    result[period] = Some(compute_rsi(avg_gain, avg_loss));

    for i in period + 1..closes.len() {
        let diff = closes[i] - closes[i - 1];
        let gain = if diff >= 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
        result[i] = Some(compute_rsi(avg_gain, avg_loss));
    }
    result
}

fn compute_rsi(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        100.0
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - 100.0 / (1.0 + rs)
    }
}
/// 简单移动平均线（MA）
///
/// 返回与输入等长的 `Vec<Option<f64>>`，前 `period-1` 位是 None。
pub fn ma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; values.len()];
    let mut sum = 0.0;
    for i in 0..values.len() {
        sum += values[i];
        if i >= period {
            sum -= values[i - period];
        }
        if i + 1 >= period {
            result[i] = Some(sum / period as f64);
        }
    }
    result
}

/// 指数移动平均线（EMA）
///
/// MACD 的基础。返回等长 `Vec<Option<f64>>`。
pub fn ema(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; values.len()];
    if values.len() < period {
        return result;
    }

    let mut sum = 0.0;
    for &value in &values[..period] {
        sum += value;
    }
    let mut ema_val = sum / period as f64;
    result[period - 1] = Some(ema_val);

    let k = 2.0 / (period as f64 + 1.0);
    for i in period..values.len() {
        ema_val = values[i] * k + ema_val * (1.0 - k);
        result[i] = Some(ema_val);
    }
    result
}
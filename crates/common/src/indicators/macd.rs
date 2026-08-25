use super::moving_average::ema;

/// MACD 指标结果
pub struct Macd {
    /// DIF 线（快线 - 慢线）
    pub dif: Vec<Option<f64>>,
    /// DEA 线（DIF 的再平滑）
    pub dea: Vec<Option<f64>>,
    /// MACD 柱（2 倍 DIF-DEA）
    pub macd: Vec<Option<f64>>,
}

/// MACD 指标
///
/// - `fast`/`slow`：常用 12 / 26
/// - `signal`：DEA 平滑周期，常用 9
pub fn macd(closes: &[f64], fast: usize, slow: usize, signal: usize) -> Macd {
    let ema_fast = ema(closes, fast);
    let ema_slow = ema(closes, slow);

    let n = closes.len();
    let mut dif = vec![None; n];
    for i in 0..n {
        if let (Some(fast_value), Some(slow_value)) = (ema_fast[i], ema_slow[i]) {
            dif[i] = Some(fast_value - slow_value);
        }
    }

    let dif_vals: Vec<f64> = dif.iter().flatten().copied().collect();
    let dea_vals = ema(&dif_vals, signal);

    let mut dea = vec![None; n];
    let offset = slow - 1;
    for (j, value) in dea_vals.iter().enumerate() {
        if let Some(value) = value {
            dea[offset + j] = Some(*value);
        }
    }

    let mut macd_hist = vec![None; n];
    for i in 0..n {
        if let (Some(dif_value), Some(dea_value)) = (dif[i], dea[i]) {
            macd_hist[i] = Some((dif_value - dea_value) * 2.0);
        }
    }

    Macd {
        dif,
        dea,
        macd: macd_hist,
    }
}
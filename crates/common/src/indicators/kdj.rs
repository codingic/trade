/// KDJ 指标结果
pub struct Kdj {
    /// K 值
    pub k: Vec<Option<f64>>,
    /// D 值
    pub d: Vec<Option<f64>>,
    /// J 值
    pub j: Vec<Option<f64>>,
}

/// KDJ 随机指标
///
/// 需要最高价、最低价、收盘价三条序列。
/// - `period`：常用 9
pub fn kdj(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Kdj {
    let n = closes.len();
    let mut k = vec![None; n];
    let mut d = vec![None; n];
    let mut j = vec![None; n];

    let mut k_prev = 50.0;
    let mut d_prev = 50.0;

    for i in 0..n {
        if i + 1 < period {
            continue;
        }

        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for m in i + 1 - period..=i {
            highest = highest.max(highs[m]);
            lowest = lowest.min(lows[m]);
        }

        let rsv = if highest == lowest {
            50.0
        } else {
            (closes[i] - lowest) / (highest - lowest) * 100.0
        };

        k_prev = k_prev * 2.0 / 3.0 + rsv / 3.0;
        d_prev = d_prev * 2.0 / 3.0 + k_prev / 3.0;
        let j_val = 3.0 * k_prev - 2.0 * d_prev;
        k[i] = Some(k_prev);
        d[i] = Some(d_prev);
        j[i] = Some(j_val);
    }

    Kdj { k, d, j }
}
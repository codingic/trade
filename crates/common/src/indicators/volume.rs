use super::moving_average::ma;

/// 能量潮（OBV，On-Balance Volume）
///
/// 用成交量验证价格趋势：价涨则加成交量，价跌则减成交量。
/// 若价格创新高但 OBV 未同步，说明上涨缺乏量能支撑（背离）。
pub fn obv(closes: &[f64], volumes: &[f64]) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut result = vec![None; n];
    if n == 0 {
        return result;
    }

    let mut obv_val = 0.0;
    result[0] = Some(0.0);

    for i in 1..n {
        if closes[i] > closes[i - 1] {
            obv_val += volumes[i];
        } else if closes[i] < closes[i - 1] {
            obv_val -= volumes[i];
        }
        result[i] = Some(obv_val);
    }
    result
}

/// 成交量均线（VOL MA）
///
/// 成交量的 N 根移动平均，用于对比「当前成交量是否放量/缩量」。
/// 通常叠加在成交量柱上，判断突破是否放量。
pub fn volume_ma(volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    ma(volumes, period)
}
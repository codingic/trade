use trade_common::indicators::{atr, boll, cci, kdj, ma, macd, obv, rsi, volume_ma};

use super::data::KlineSeries;

pub struct Indicators {
    pub ma7: Vec<Option<f64>>,
    pub ma25: Vec<Option<f64>>,
    pub rsi14: Vec<Option<f64>>,
    pub macd_dif: Vec<Option<f64>>,
    pub macd_dea: Vec<Option<f64>>,
    pub macd_hist: Vec<Option<f64>>,
    pub kdj_k: Vec<Option<f64>>,
    pub kdj_d: Vec<Option<f64>>,
    pub kdj_j: Vec<Option<f64>>,
    pub boll_upper: Vec<Option<f64>>,
    pub boll_middle: Vec<Option<f64>>,
    pub boll_lower: Vec<Option<f64>>,
    pub atr14: Vec<Option<f64>>,
    pub obv: Vec<Option<f64>>,
    pub cci20: Vec<Option<f64>>,
    pub vol_ma5: Vec<Option<f64>>,
    pub vol_ma20: Vec<Option<f64>>,
}

pub fn compute_indicators(series: &KlineSeries) -> Indicators {
    let macd_result = macd(&series.closes, 12, 26, 9);
    let kdj_result = kdj(&series.highs, &series.lows, &series.closes, 9);
    let boll_result = boll(&series.closes, 20, 2.0);

    Indicators {
        ma7: ma(&series.closes, 7),
        ma25: ma(&series.closes, 25),
        rsi14: rsi(&series.closes, 14),
        macd_dif: macd_result.dif,
        macd_dea: macd_result.dea,
        macd_hist: macd_result.macd,
        kdj_k: kdj_result.k,
        kdj_d: kdj_result.d,
        kdj_j: kdj_result.j,
        boll_upper: boll_result.upper,
        boll_middle: boll_result.middle,
        boll_lower: boll_result.lower,
        atr14: atr(&series.highs, &series.lows, &series.closes, 14),
        obv: obv(&series.closes, &series.volumes),
        cci20: cci(&series.highs, &series.lows, &series.closes, 20),
        vol_ma5: volume_ma(&series.volumes, 5),
        vol_ma20: volume_ma(&series.volumes, 20),
    }
}
use serde_json::{json, Value};

use super::data::KlineSeries;
use super::indicators::Indicators;

pub fn build_kline_chart(
    symbol: &str,
    interval: &str,
    series: &KlineSeries,
    indicators: Indicators,
) -> Value {
    json!({
        "symbol": symbol,
        "interval": interval,
        "times": series.times,
        "candles": series.candles,
        "close": series.closes,
        "volume": series.volumes,
        "ma7": indicators.ma7,
        "ma25": indicators.ma25,
        "rsi14": indicators.rsi14,
        "macd": { "dif": indicators.macd_dif, "dea": indicators.macd_dea, "hist": indicators.macd_hist },
        "kdj": { "k": indicators.kdj_k, "d": indicators.kdj_d, "j": indicators.kdj_j },
        "boll": { "upper": indicators.boll_upper, "middle": indicators.boll_middle, "lower": indicators.boll_lower },
        "atr14": indicators.atr14,
        "obv": indicators.obv,
        "cci20": indicators.cci20,
        "vol_ma5": indicators.vol_ma5,
        "vol_ma20": indicators.vol_ma20,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::build_kline_chart;
    use crate::service::market::data::KlineSeries;
    use crate::service::market::indicators::Indicators;

    #[test]
    fn build_kline_chart_exposes_expected_json_shape() {
        let series = KlineSeries {
            times: vec![1, 2],
            candles: vec![vec![10.0, 11.0, 9.0, 12.0], vec![11.0, 12.0, 10.0, 13.0]],
            closes: vec![11.0, 12.0],
            highs: vec![12.0, 13.0],
            lows: vec![9.0, 10.0],
            volumes: vec![100.0, 120.0],
        };
        let indicators = Indicators {
            ma7: vec![None, Some(10.5)],
            ma25: vec![None, Some(10.0)],
            rsi14: vec![None, Some(55.0)],
            macd_dif: vec![None, Some(1.1)],
            macd_dea: vec![None, Some(0.9)],
            macd_hist: vec![None, Some(0.4)],
            kdj_k: vec![None, Some(60.0)],
            kdj_d: vec![None, Some(58.0)],
            kdj_j: vec![None, Some(64.0)],
            boll_upper: vec![None, Some(13.0)],
            boll_middle: vec![None, Some(11.5)],
            boll_lower: vec![None, Some(10.0)],
            atr14: vec![None, Some(2.2)],
            obv: vec![Some(0.0), Some(120.0)],
            cci20: vec![None, Some(88.0)],
            vol_ma5: vec![None, Some(110.0)],
            vol_ma20: vec![None, Some(105.0)],
        };

        let chart = build_kline_chart("BTCUSDT", "1m", &series, indicators);

        assert_eq!(chart["symbol"], json!("BTCUSDT"));
        assert_eq!(chart["interval"], json!("1m"));
        assert_eq!(chart["times"], json!([1, 2]));
        assert_eq!(chart["candles"], json!([[10.0, 11.0, 9.0, 12.0], [11.0, 12.0, 10.0, 13.0]]));
        assert_eq!(chart["macd"]["hist"], json!([null, 0.4]));
        assert_eq!(chart["boll"]["middle"], json!([null, 11.5]));
        assert_eq!(chart["vol_ma20"], json!([null, 105.0]));
    }
}
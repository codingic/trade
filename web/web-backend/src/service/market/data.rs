use anyhow::Result;

use trade_common::storage;

pub struct KlineSeries {
    pub times: Vec<u64>,
    pub candles: Vec<Vec<f64>>,
    pub closes: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub volumes: Vec<f64>,
}

pub fn load_kline_series(symbol: &str, interval: &str, limit: u32) -> Result<KlineSeries> {
    let conn = storage::open(storage::DEFAULT_DB_PATH)?;
    let klines = storage::latest_klines(&conn, symbol, interval, limit)?;

    Ok(KlineSeries {
        times: klines.iter().map(|k| k.open_time).collect(),
        candles: klines
            .iter()
            .map(|k| vec![k.open, k.close, k.low, k.high])
            .collect(),
        closes: klines.iter().map(|k| k.close).collect(),
        highs: klines.iter().map(|k| k.high).collect(),
        lows: klines.iter().map(|k| k.low).collect(),
        volumes: klines.iter().map(|k| k.volume).collect(),
    })
}
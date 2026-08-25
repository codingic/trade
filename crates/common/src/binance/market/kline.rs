use anyhow::Result;
use serde_json::Value;

use crate::binance::types::Kline;
use crate::binance::BinanceClient;

impl BinanceClient {
    /// 获取 K 线（OHLCV）历史数据（按数量）
    pub async fn klines(&self, symbol: &str, interval: &str, limit: u32) -> Result<Vec<Kline>> {
        let url = format!(
            "{}/fapi/v1/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );
        self.fetch_klines(&url).await
    }

    /// 获取 K 线历史数据（按时间范围）
    pub async fn klines_range(
        &self,
        symbol: &str,
        interval: &str,
        start: Option<u64>,
        end: Option<u64>,
        limit: u32,
    ) -> Result<Vec<Kline>> {
        let mut url = format!(
            "{}/fapi/v1/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );
        if let Some(s) = start {
            url.push_str(&format!("&startTime={}", s));
        }
        if let Some(e) = end {
            url.push_str(&format!("&endTime={}", e));
        }
        self.fetch_klines(&url).await
    }

    async fn fetch_klines(&self, url: &str) -> Result<Vec<Kline>> {
        let resp = self.http.get(url).send().await?;
        let raw: Vec<Value> = resp.json().await?;
        raw.iter().map(parse_kline).collect()
    }
}

fn parse_kline(arr: &Value) -> Result<Kline> {
    fn num(arr: &Value, i: usize) -> Result<f64> {
        let s = arr[i].as_str().unwrap_or("0");
        Ok(s.parse()?)
    }

    Ok(Kline {
        open_time: arr[0].as_u64().unwrap_or(0),
        open: num(arr, 1)?,
        high: num(arr, 2)?,
        low: num(arr, 3)?,
        close: num(arr, 4)?,
        volume: num(arr, 5)?,
        close_time: arr[6].as_u64().unwrap_or(0),
        quote_volume: num(arr, 7)?,
        trades: arr[8].as_u64().unwrap_or(0),
        taker_buy_base: num(arr, 9)?,
        taker_buy_quote: num(arr, 10)?,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse_kline;

    #[test]
    fn parse_kline_maps_binance_array_fields() {
        let raw = json!([
            1710000000000_u64,
            "62000.10",
            "62500.50",
            "61800.00",
            "62300.20",
            "12.34",
            1710000059999_u64,
            "770000.00",
            123_u64,
            "6.78",
            "420000.50",
            "0"
        ]);

        let kline = parse_kline(&raw).unwrap();

        assert_eq!(kline.open_time, 1710000000000);
        assert_eq!(kline.close_time, 1710000059999);
        assert_eq!(kline.open, 62000.10);
        assert_eq!(kline.high, 62500.50);
        assert_eq!(kline.low, 61800.00);
        assert_eq!(kline.close, 62300.20);
        assert_eq!(kline.volume, 12.34);
        assert_eq!(kline.quote_volume, 770000.00);
        assert_eq!(kline.trades, 123);
        assert_eq!(kline.taker_buy_base, 6.78);
        assert_eq!(kline.taker_buy_quote, 420000.50);
    }
}
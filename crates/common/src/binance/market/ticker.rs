use anyhow::Result;

use crate::binance::types::{Ticker24h, TickerPrice};
use crate::binance::BinanceClient;

impl BinanceClient {
    /// 获取指定合约的最新成交价
    pub async fn ticker_price(&self, symbol: &str) -> Result<TickerPrice> {
        let url = format!("{}/fapi/v1/ticker/price?symbol={}", self.base_url, symbol);
        let resp = self.http.get(&url).send().await?;
        let ticker = resp.json().await?;
        Ok(ticker)
    }

    /// 获取指定合约的 24 小时行情统计
    pub async fn ticker_24h(&self, symbol: &str) -> Result<Ticker24h> {
        let url = format!("{}/fapi/v1/ticker/24hr?symbol={}", self.base_url, symbol);
        let resp = self.http.get(&url).send().await?;
        let stats = resp.json().await?;
        Ok(stats)
    }
}
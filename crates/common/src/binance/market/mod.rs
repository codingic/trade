//! 行情接口：负责从币安获取实时价格、K 线等市场数据
//!
//! 这些方法挂在 `BinanceClient` 上（通过 `impl BinanceClient` 块），
//! 调用方式：`client.ticker_price("BTCUSDT").await?`

mod kline;
mod ticker;
use anyhow::Result;
use serde_json::Value;

use crate::binance::BinanceClient;

impl BinanceClient {
    /// 下单（MARKET 市价单）
    pub async fn place_market_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
    ) -> Result<Value> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("side", side.to_string()),
            ("type", "MARKET".to_string()),
            ("quantity", quantity.to_string()),
        ];

        self.signed_request("POST", "/fapi/v1/order", &mut params)
            .await
    }
}
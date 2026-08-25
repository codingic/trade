use anyhow::Result;
use serde_json::Value;

use crate::binance::BinanceClient;

impl BinanceClient {
    /// 查询账户余额（验证 API Key 是否有效的好方法）
    pub async fn account_balance(&self) -> Result<Value> {
        let mut params = vec![];
        self.signed_request("GET", "/fapi/v2/balance", &mut params)
            .await
    }
}
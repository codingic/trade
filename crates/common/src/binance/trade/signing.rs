use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

use crate::binance::BinanceClient;

type HmacSha256 = Hmac<Sha256>;

impl BinanceClient {
    pub(super) async fn signed_request(
        &self,
        method: &str,
        path: &str,
        params: &mut Vec<(&str, String)>,
    ) -> Result<Value> {
        let (api_key, secret_key) = match (&self.api_key, &self.secret_key) {
            (Some(k), Some(s)) => (k.clone(), s.clone()),
            _ => return Err(anyhow!("未配置 API Key，无法调用私有接口")),
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        params.push(("timestamp", timestamp));

        params.sort_by(|a, b| a.0.cmp(b.0));
        let query: String = params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");

        let signature = sign(&query, &secret_key);
        let url = format!("{}{}?{}&signature={}", self.base_url, path, query, signature);

        let resp = self
            .http
            .request(method.parse()?, &url)
            .header("X-MBX-APIKEY", &api_key)
            .send()
            .await?;

        let body: Value = resp.json().await?;
        if let Some(code) = body.get("code") {
            return Err(anyhow!(
                "币安接口错误 code={code} msg={}",
                body.get("msg").and_then(|m| m.as_str()).unwrap_or("")
            ));
        }
        Ok(body)
    }
}

fn sign(data: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC 可以接受任意长度的密钥");
    mac.update(data.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_matches_known_hmac_sha256_vector() {
        let digest = sign("The quick brown fox jumps over the lazy dog", "key");
        assert_eq!(
            digest,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[tokio::test]
    async fn signed_request_requires_api_keys_before_network() {
        let client = BinanceClient::testnet();
        let err = client
            .signed_request("GET", "/fapi/v2/balance", &mut vec![])
            .await
            .unwrap_err();

        assert!(err.to_string().contains("未配置 API Key"));
    }
}
//! 币安 USDT-M 永续合约客户端
//!
//! 模块结构：
//! - `types`：数据结构定义
//! - `market`：行情接口（价格、K线等，公开，无需签名）
//! - `trade`：下单/撤单接口（私有，需 API Key + 签名）
//! - `account`：账户/持仓接口（待实现）
//! - `stream`：WebSocket 实时行情流（待实现）

pub mod market;
pub mod trade;
pub mod types;

use reqwest::Client;

/// 币安合约客户端：封装 HTTP 连接、API 基础地址、API Key
///
/// `base_url` 指向主网或测试网。
/// `api_key`/`secret_key` 用于私有接口（下单、查账户）的签名，
/// 只读行情接口不需要，可留空。
pub struct BinanceClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) secret_key: Option<String>,
}

impl BinanceClient {
    /// 用自定义 base_url 创建客户端（主网/测试网通用）
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
            api_key: None,
            secret_key: None,
        }
    }

    /// 连主网（真实行情，只读安全）
    pub fn mainnet() -> Self {
        Self::new("https://fapi.binance.com")
    }

    /// 连测试网（下单练手用，全是假钱）
    pub fn testnet() -> Self {
        Self::new("https://testnet.binancefuture.com")
    }

    /// 设置 API Key（用于私有接口签名）
    ///
    /// 链式调用风格：`BinanceClient::testnet().with_api_key(k, s)`
    pub fn with_api_key(mut self, api_key: &str, secret_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self.secret_key = Some(secret_key.to_string());
        self
    }
}

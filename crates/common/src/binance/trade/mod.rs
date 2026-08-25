//! 交易接口：下单、撤单（私有接口，需要 API Key + 签名）
//!
//! 币安私有接口的认证机制：
//! 1. 请求头带上 `X-MBX-APIKEY`（身份标识）
//! 2. 请求参数加上 `timestamp`（毫秒时间戳，防重放）
//! 3. 把所有参数按字母排序、拼成 `key=value&...` 字符串
//! 4. 用 Secret Key 对这段字符串做 HMAC-SHA256，得到 `signature`
//!
//! 只有签名正确，币安才认为「这个请求真的是你发的」。

mod account;
mod order;
mod signing;
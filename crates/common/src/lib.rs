//! trade-common 共享库
//!
//! 被 collector、strategy、web-backend 三个程序共用的核心模块：
//! - `binance`：币安 API 客户端（行情 + 下单）
//! - `indicators`：技术指标计算
//! - `storage`：SQLite 数据持久化

pub mod binance;
pub mod indicators;
pub mod settings;
pub mod storage;

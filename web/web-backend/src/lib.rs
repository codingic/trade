//! web-backend Web 服务后端
//!
//! 纯展示后端：读库 → 算指标 → 返回 JSON 给前端页面。
//! - `server`：HTTP 层（收发请求）
//! - `service`：业务层（编排读库 + 指标计算）

pub mod server;
pub mod service;

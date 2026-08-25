//! 技术指标计算
//!
//! 输入是一串按时间升序的价格或成交量序列，输出对应长度的指标数组。
//! 约定：数据不足的前 N 根，指标值用 None 表示（对齐到 K 线索引）。

mod cci;
mod kdj;
mod macd;
mod moving_average;
mod rsi;
mod volatility;
mod volume;

pub use moving_average::{ema, ma};
pub use cci::cci;
pub use kdj::{kdj, Kdj};
pub use macd::{macd, Macd};
pub use rsi::rsi;
pub use volatility::{atr, boll, Boll};
pub use volume::{obv, volume_ma};
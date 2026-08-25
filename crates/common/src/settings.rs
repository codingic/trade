use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

pub const DEFAULT_CONFIG_PATH: &str = "trade.toml";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TradeConfigFile {
    #[serde(default)]
    pub strategy: StrategyFileConfig,
    #[serde(default)]
    pub backtest: BacktestFileConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StrategyFileConfig {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    pub quantity: Option<f64>,
    pub fast_ma: Option<usize>,
    pub slow_ma: Option<usize>,
    pub lookback: Option<usize>,
    pub poll: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BacktestFileConfig {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    pub capital: Option<f64>,
    pub quantity: Option<f64>,
    pub leverage: Option<f64>,
    pub fee: Option<f64>,
    pub fast_ma: Option<usize>,
    pub slow_ma: Option<usize>,
    pub lookback: Option<usize>,
    pub limit: Option<usize>,
    pub export_csv: Option<String>,
    pub export_summary_csv: Option<String>,
}

pub fn load_trade_config(path: Option<&str>) -> Result<TradeConfigFile> {
    let config_path = path.unwrap_or(DEFAULT_CONFIG_PATH);
    if !Path::new(config_path).exists() {
        if path.is_some() {
            return Err(anyhow!("配置文件不存在: {config_path}"));
        }
        return Ok(TradeConfigFile::default());
    }

    let text = fs::read_to_string(config_path)
        .with_context(|| format!("读取配置文件失败: {config_path}"))?;
    toml::from_str(&text).with_context(|| format!("解析配置文件失败: {config_path}"))
}

#[cfg(test)]
mod tests {
    use super::TradeConfigFile;

    #[test]
    fn parse_trade_config_file_sections() {
        let raw = r#"
[strategy]
symbol = "ETHUSDT"
fast_ma = 9
slow_ma = 30

[backtest]
capital = 20000.0
leverage = 3.0
fee = 0.0002
export_csv = "tradedata/backtests/eth.csv"
export_summary_csv = "tradedata/backtests/eth_summary.csv"
"#;

        let config: TradeConfigFile = toml::from_str(raw).unwrap();

        assert_eq!(config.strategy.symbol.as_deref(), Some("ETHUSDT"));
        assert_eq!(config.strategy.fast_ma, Some(9));
        assert_eq!(config.strategy.slow_ma, Some(30));
        assert_eq!(config.backtest.capital, Some(20000.0));
        assert_eq!(config.backtest.leverage, Some(3.0));
        assert_eq!(config.backtest.fee, Some(0.0002));
        assert_eq!(
            config.backtest.export_csv.as_deref(),
            Some("tradedata/backtests/eth.csv")
        );
        assert_eq!(
            config.backtest.export_summary_csv.as_deref(),
            Some("tradedata/backtests/eth_summary.csv")
        );
    }
}
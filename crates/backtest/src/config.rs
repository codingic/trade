use anyhow::{anyhow, Result};
use strategy::SignalParams;
use trade_common::settings::{load_trade_config, BacktestFileConfig, DEFAULT_CONFIG_PATH};

pub const DEFAULT_SYMBOL: &str = "BTCUSDT";
pub const DEFAULT_INTERVAL: &str = "1m";
pub const DEFAULT_INITIAL_CAPITAL: f64 = 100.0;
pub const DEFAULT_QUANTITY: f64 = 10.0;
pub const DEFAULT_LEVERAGE: f64 = 1.0;
pub const DEFAULT_FEE_RATE: f64 = 0.0004;
pub const DEFAULT_LOOKBACK: usize = 50;

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub symbol: String,
    pub interval: String,
    pub initial_capital: f64,
    pub quantity: f64,
    pub leverage: f64,
    pub fee_rate: f64,
    pub lookback: usize,
    pub fast_ma_period: usize,
    pub slow_ma_period: usize,
    pub limit: Option<usize>,
    pub export_csv: Option<String>,
    pub export_summary_csv: Option<String>,
    pub list_strategies: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        let signal = SignalParams::default();
        Self {
            symbol: DEFAULT_SYMBOL.to_string(),
            interval: DEFAULT_INTERVAL.to_string(),
            initial_capital: DEFAULT_INITIAL_CAPITAL,
            quantity: DEFAULT_QUANTITY,
            leverage: DEFAULT_LEVERAGE,
            fee_rate: DEFAULT_FEE_RATE,
            lookback: DEFAULT_LOOKBACK,
            fast_ma_period: signal.fast_ma_period,
            slow_ma_period: signal.slow_ma_period,
            limit: None,
            export_csv: None,
            export_summary_csv: None,
            list_strategies: false,
        }
    }
}

impl BacktestConfig {
    pub fn signal_params(&self) -> SignalParams {
        SignalParams {
            fast_ma_period: self.fast_ma_period,
            slow_ma_period: self.slow_ma_period,
        }
    }

    pub fn margin_per_trade(&self) -> f64 {
        self.quantity
    }

    pub fn notional_per_trade(&self) -> f64 {
        self.quantity * self.leverage
    }

    pub fn from_settings(path: Option<&str>) -> Result<Self> {
        let file_config = load_trade_config(path)?;
        let mut config = Self::default();
        config.apply_file_config(&file_config.backtest);
        config.validate()?;
        Ok(config)
    }

    pub fn from_args() -> Result<Self> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let config_path = find_config_path(&args)?;
        let mut config = Self::from_settings(config_path.as_deref())?;
        config.apply_cli_args(&args)?;

        config.validate()?;
        Ok(config)
    }

    fn apply_file_config(&mut self, file: &BacktestFileConfig) {
        if let Some(symbol) = &file.symbol {
            self.symbol = symbol.clone();
        }
        if let Some(interval) = &file.interval {
            self.interval = interval.clone();
        }
        if let Some(capital) = file.capital {
            self.initial_capital = capital;
        }
        if let Some(quantity) = file.quantity {
            self.quantity = quantity;
        }
        if let Some(leverage) = file.leverage {
            self.leverage = leverage;
        }
        if let Some(fee) = file.fee {
            self.fee_rate = fee;
        }
        if let Some(fast_ma) = file.fast_ma {
            self.fast_ma_period = fast_ma;
        }
        if let Some(slow_ma) = file.slow_ma {
            self.slow_ma_period = slow_ma;
        }
        if let Some(lookback) = file.lookback {
            self.lookback = lookback;
        }
        if let Some(limit) = file.limit {
            self.limit = Some(limit);
        }
        if let Some(export_csv) = &file.export_csv {
            self.export_csv = Some(export_csv.clone());
        }
        if let Some(export_summary_csv) = &file.export_summary_csv {
            self.export_summary_csv = Some(export_summary_csv.clone());
        }
    }

    fn apply_cli_args(&mut self, args: &[String]) -> Result<()> {
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--config" => {
                    index += 2;
                    continue;
                }
                "--symbol" => self.symbol = args_value(args, &mut index, "--symbol")?,
                "--interval" => self.interval = args_value(args, &mut index, "--interval")?,
                "--capital" => {
                    self.initial_capital = parse_f64(args, &mut index, "--capital")?
                }
                "--quantity" => self.quantity = parse_f64(args, &mut index, "--quantity")?,
                "--leverage" => self.leverage = parse_f64(args, &mut index, "--leverage")?,
                "--fee" => self.fee_rate = parse_f64(args, &mut index, "--fee")?,
                "--fast-ma" => self.fast_ma_period = parse_usize(args, &mut index, "--fast-ma")?,
                "--slow-ma" => self.slow_ma_period = parse_usize(args, &mut index, "--slow-ma")?,
                "--lookback" => self.lookback = parse_usize(args, &mut index, "--lookback")?,
                "--limit" => self.limit = Some(parse_usize(args, &mut index, "--limit")?),
                "--export-csv" => {
                    self.export_csv = Some(args_value(args, &mut index, "--export-csv")?)
                }
                "--export-summary-csv" => {
                    self.export_summary_csv = Some(args_value(
                        args,
                        &mut index,
                        "--export-summary-csv",
                    )?)
                }
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--list-strategies" => {
                    self.list_strategies = true;
                }
                other => return Err(anyhow!("未知参数: {other}")),
            }
            index += 1;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.quantity <= 0.0 {
            return Err(anyhow!("--quantity 必须大于 0"));
        }
        if self.initial_capital <= 0.0 {
            return Err(anyhow!("--capital 必须大于 0"));
        }
        if self.leverage <= 0.0 {
            return Err(anyhow!("--leverage 必须大于 0"));
        }
        if self.fee_rate < 0.0 {
            return Err(anyhow!("--fee 不能小于 0"));
        }
        if self.fast_ma_period == 0 {
            return Err(anyhow!("--fast-ma 必须大于 0"));
        }
        if self.slow_ma_period <= self.fast_ma_period {
            return Err(anyhow!("--slow-ma 必须大于 --fast-ma"));
        }
        if self.lookback < self.signal_params().minimum_bars() {
            return Err(anyhow!(
                "--lookback 不能小于 {}（当前 slow MA 需要至少 slow+1 根）",
                self.signal_params().minimum_bars()
            ));
        }
        if let Some(limit) = self.limit {
            if limit < self.lookback {
                return Err(anyhow!("--limit 不能小于 --lookback"));
            }
        }

        Ok(())
    }
}

fn find_config_path(args: &[String]) -> Result<Option<String>> {
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == "--config" {
            if index + 1 >= args.len() {
                return Err(anyhow!("--config 缺少参数值"));
            }
            return Ok(Some(args[index + 1].clone()));
        }
        index += 1;
    }
    Ok(None)
}

fn args_value(args: &[String], index: &mut usize, flag: &str) -> Result<String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| anyhow!("{flag} 缺少参数值"))
}

fn parse_f64(args: &[String], index: &mut usize, flag: &str) -> Result<f64> {
    let raw = args_value(args, index, flag)?;
    raw.parse()
        .map_err(|_| anyhow!("{flag} 需要数字参数，收到: {raw}"))
}

fn parse_usize(args: &[String], index: &mut usize, flag: &str) -> Result<usize> {
    let raw = args_value(args, index, flag)?;
    raw.parse()
        .map_err(|_| anyhow!("{flag} 需要整数参数，收到: {raw}"))
}

fn print_usage() {
    println!("用法:");
    println!("  cargo run -p backtest -- [options]");
    println!("  默认会读取 {} 中的 [backtest] 配置", DEFAULT_CONFIG_PATH);
    println!();
    println!("可选参数:");
    println!("  --config <PATH>        指定配置文件路径，默认 trade.toml");
    println!("  --symbol <SYMBOL>      回测合约，默认 BTCUSDT");
    println!("  --interval <INTERVAL>  K 线周期，默认 1m");
    println!("  --capital <AMOUNT>     初始资金，默认 100");
    println!("  --quantity <QTY>       单次保证金(U)，默认 10");
    println!("  --leverage <N>         仓位倍数，默认 1");
    println!("  --fee <RATE>           单边手续费率，默认 0.0004");
    println!("  --fast-ma <N>          快线 MA 周期，默认 7");
    println!("  --slow-ma <N>          慢线 MA 周期，默认 25");
    println!("  --lookback <N>         信号回看根数，默认 50");
    println!("  --limit <N>            只回测最近 N 根 K 线");
    println!("  --export-csv <PATH>    导出成交明细 CSV 文件");
    println!("  --export-summary-csv <PATH> 导出回测汇总 CSV 文件");
    println!("  --list-strategies      列出所有内置策略预设后退出");
}
use anyhow::{anyhow, Result};

use strategy::SignalParams;
use trade_common::settings::{load_trade_config, StrategyFileConfig, DEFAULT_CONFIG_PATH};

pub const DEFAULT_SYMBOL: &str = "BTCUSDT";
pub const DEFAULT_INTERVAL: &str = "1m";
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;
pub const DEFAULT_LOOKBACK: usize = 50;
pub const DEFAULT_QUANTITY: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct StrategyConfig {
	pub symbol: String,
	pub interval: String,
	pub poll_interval_secs: u64,
	pub lookback: usize,
	pub quantity: f64,
	pub signal: SignalParams,
}

impl Default for StrategyConfig {
	fn default() -> Self {
		Self {
			symbol: DEFAULT_SYMBOL.to_string(),
			interval: DEFAULT_INTERVAL.to_string(),
			poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
			lookback: DEFAULT_LOOKBACK,
			quantity: DEFAULT_QUANTITY,
			signal: SignalParams::default(),
		}
	}
}

impl StrategyConfig {
	pub fn from_args() -> Result<Self> {
		let args: Vec<String> = std::env::args().skip(1).collect();
		let config_path = find_config_path(&args)?;
		let file_config = load_trade_config(config_path.as_deref())?;

		let mut config = Self::default();
		config.apply_file_config(&file_config.strategy);
		config.apply_cli_args(&args)?;

		config.validate()?;
		Ok(config)
	}

	fn apply_file_config(&mut self, file: &StrategyFileConfig) {
		if let Some(symbol) = &file.symbol {
			self.symbol = symbol.clone();
		}
		if let Some(interval) = &file.interval {
			self.interval = interval.clone();
		}
		if let Some(quantity) = file.quantity {
			self.quantity = quantity;
		}
		if let Some(fast_ma) = file.fast_ma {
			self.signal.fast_ma_period = fast_ma;
		}
		if let Some(slow_ma) = file.slow_ma {
			self.signal.slow_ma_period = slow_ma;
		}
		if let Some(lookback) = file.lookback {
			self.lookback = lookback;
		}
		if let Some(poll) = file.poll {
			self.poll_interval_secs = poll;
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
				"--quantity" => self.quantity = parse_f64(args, &mut index, "--quantity")?,
				"--fast-ma" => {
					self.signal.fast_ma_period = parse_usize(args, &mut index, "--fast-ma")?
				}
				"--slow-ma" => {
					self.signal.slow_ma_period = parse_usize(args, &mut index, "--slow-ma")?
				}
				"--lookback" => self.lookback = parse_usize(args, &mut index, "--lookback")?,
				"--poll" => {
					self.poll_interval_secs = parse_u64(args, &mut index, "--poll")?
				}
				"-h" | "--help" => {
					print_usage();
					std::process::exit(0);
				}
				other => return Err(anyhow!("未知参数: {other}")),
			}
			index += 1;
		}
		Ok(())
	}

	fn validate(&self) -> Result<()> {
		if self.quantity <= 0.0 {
			return Err(anyhow!("--quantity 必须大于 0"));
		}
		if self.poll_interval_secs == 0 {
			return Err(anyhow!("--poll 必须大于 0"));
		}
		if self.signal.fast_ma_period == 0 {
			return Err(anyhow!("--fast-ma 必须大于 0"));
		}
		if self.signal.slow_ma_period <= self.signal.fast_ma_period {
			return Err(anyhow!("--slow-ma 必须大于 --fast-ma"));
		}
		if self.lookback < self.signal.minimum_bars() {
			return Err(anyhow!(
				"--lookback 不能小于 {}（当前 slow MA 需要至少 slow+1 根）",
				self.signal.minimum_bars()
			));
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

fn parse_u64(args: &[String], index: &mut usize, flag: &str) -> Result<u64> {
	let raw = args_value(args, index, flag)?;
	raw.parse()
		.map_err(|_| anyhow!("{flag} 需要整数参数，收到: {raw}"))
}

fn print_usage() {
	println!("用法:");
	println!("  BINANCE_API_KEY=xxx BINANCE_SECRET_KEY=yyy cargo run -p strategy -- [options]");
	println!("  默认会读取 {} 中的 [strategy] 配置", DEFAULT_CONFIG_PATH);
	println!();
	println!("可选参数:");
	println!("  --config <PATH>        指定配置文件路径，默认 trade.toml");
	println!("  --symbol <SYMBOL>      交易合约，默认 BTCUSDT");
	println!("  --interval <INTERVAL>  K 线周期，默认 1m");
	println!("  --quantity <QTY>       每次开仓数量，默认 1.0");
	println!("  --fast-ma <N>          快线 MA 周期，默认 7");
	println!("  --slow-ma <N>          慢线 MA 周期，默认 25");
	println!("  --lookback <N>         每轮信号读取根数，默认 50");
	println!("  --poll <SECS>          轮询间隔秒数，默认 60");
}
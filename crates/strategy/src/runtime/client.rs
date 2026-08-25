use std::env;

use trade_common::binance::BinanceClient;

pub fn build_client() -> BinanceClient {
    let api_key = env::var("BINANCE_API_KEY").ok();
    let secret_key = env::var("BINANCE_SECRET_KEY").ok();

    match (api_key, secret_key) {
        (Some(k), Some(s)) => BinanceClient::testnet().with_api_key(&k, &s),
        _ => {
            eprintln!("❌ 未设置 API Key！");
            eprintln!("请用环境变量启动：");
            eprintln!("  BINANCE_API_KEY=xxx BINANCE_SECRET_KEY=yyy cargo run --bin strategy");
            std::process::exit(1);
        }
    }
}

pub async fn verify_api_key(client: &BinanceClient) {
    match client.account_balance().await {
        Ok(_) => println!("✅ API Key 有效，账户余额接口正常\n"),
        Err(e) => {
            eprintln!("❌ API Key 校验失败: {e}");
            eprintln!("请检查 Key 是否正确（测试网 Key 需在 testnet.binancefuture.com 申请）");
            std::process::exit(1);
        }
    }
}
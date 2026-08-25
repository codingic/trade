//! 参数扫描命令行工具：对所有策略做参数网格扫描，输出收益率前 N 名。
//!
//! 核心扫描逻辑已抽到 `backtest::sweep`，本文件只负责取数、切片与打印。

use anyhow::Result;
use backtest::sweep::run_sweep;
use trade_common::storage;

fn main() -> Result<()> {
    let days: usize = std::env::args()
        .skip(1)
        .find_map(|a| a.parse::<usize>().ok())
        .unwrap_or(120);
    let top_n: usize = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse::<usize>().ok())
        .nth(1)
        .unwrap_or(20);

    let conn = storage::open(storage::DEFAULT_DB_PATH)?;

    // 读取 1m 基础数据，由 storage 聚合成 4h，再取最近 N 天的 4h 根
    let bars_per_day = 6usize; // 4h 周期每天 6 根
    let klines_4h = storage::klines(&conn, "BTCUSDT", "4h")?;
    let take = days.saturating_mul(bars_per_day);
    let window: Vec<_> = if klines_4h.len() > take {
        klines_4h.split_off(klines_4h.len() - take)
    } else {
        klines_4h
    };

    println!(
        "4h 数据: {} 根 (约 {:.0} 天)，扫描中...\n",
        window.len(),
        window.len() as f64 / 6.0
    );

    let start = std::time::Instant::now();
    let rows = run_sweep(&window, 100.0, 1.0, 0.0004, top_n);
    let elapsed = start.elapsed();

    println!(
        "{:<4} {:<28} {:<20} {:>9} {:>8} {:>6} {:>9} {:>10}",
        "排名", "策略名称", "参数", "收益率%", "胜率%", "笔数", "最大回撤%", "净收益U"
    );
    println!("{}", "-".repeat(95));

    for r in &rows {
        println!(
            "{:<4} {:<28} {:<20} {:>9.2} {:>8.1} {:>6} {:>9.2} {:>10.4}",
            r.rank,
            truncate(&r.name, 28),
            truncate(&r.params_desc, 20),
            r.return_pct,
            r.win_rate,
            r.trades,
            r.max_dd,
            r.net_profit,
        );
    }

    println!("\n扫描完成，耗时 {:.2?}，共 {} 个组合进入排名。", elapsed, rows.len());
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "\u{2026}"
    }
}

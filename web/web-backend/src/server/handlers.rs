use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::service;

use super::types::{BacktestRequest, CustomBacktestRequest, KlineQuery};

/// 处理 /api/klines：解析参数 → 调业务层 → 返回 JSON
pub async fn handle_klines(
    Query(query): Query<KlineQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (symbol, interval, limit) = normalize_kline_query(query);

    let body = service::get_kline_chart(&symbol, &interval, limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("获取数据失败: {e}")))?;

    Ok(axum::Json(body))
}

fn normalize_kline_query(query: KlineQuery) -> (String, String, u32) {
    let symbol = query.symbol.unwrap_or_else(|| "BTCUSDT".to_string());
    let interval = query.interval.unwrap_or_else(|| "1m".to_string());
    let limit = query.limit.unwrap_or(300).min(1500).max(10);
    (symbol, interval, limit)
}

#[cfg(test)]
mod tests {
    use super::normalize_kline_query;
    use crate::server::types::KlineQuery;

    #[test]
    fn normalize_kline_query_applies_defaults() {
        let query = KlineQuery {
            symbol: None,
            interval: None,
            limit: None,
        };

        let normalized = normalize_kline_query(query);
        assert_eq!(normalized, ("BTCUSDT".to_string(), "1m".to_string(), 300));
    }

    #[test]
    fn normalize_kline_query_clamps_limit_to_bounds() {
        let low = KlineQuery {
            symbol: Some("ETHUSDT".to_string()),
            interval: Some("5m".to_string()),
            limit: Some(1),
        };
        let high = KlineQuery {
            symbol: Some("ETHUSDT".to_string()),
            interval: Some("5m".to_string()),
            limit: Some(9_999),
        };

        assert_eq!(normalize_kline_query(low), ("ETHUSDT".to_string(), "5m".to_string(), 10));
        assert_eq!(normalize_kline_query(high), ("ETHUSDT".to_string(), "5m".to_string(), 1500));
    }
}

/// 处理 /api/db：返回数据库存储概览
pub async fn handle_db_overview() -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = service::get_db_overview().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("获取数据库概览失败: {e}"),
        )
    })?;

    Ok(axum::Json(body))
}

pub async fn handle_backtest(
    Json(request): Json<BacktestRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = service::run_backtest_preview(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("回测失败: {e}")))?;

    Ok(axum::Json(body))
}

pub async fn handle_backtest_catalog(
    Json(request): Json<BacktestRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = service::run_strategy_catalog(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("批量回测失败: {e}")))?;

    Ok(axum::Json(body))
}

/// 处理 GET /api/strategies：返回内置 100 个策略的目录（元信息，无需回测）
pub async fn handle_strategies() -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = service::list_strategies()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("获取策略列表失败: {e}")))?;
    Ok(axum::Json(body))
}

/// 处理 POST /api/backtest/custom：使用自定义参数对单个策略回测
pub async fn handle_backtest_custom(
    Json(request): Json<CustomBacktestRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = service::run_custom_backtest(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("自定义回测失败: {e}")))?;
    Ok(axum::Json(body))
}
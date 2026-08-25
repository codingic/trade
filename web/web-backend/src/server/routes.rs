use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use super::handlers::{
    handle_backtest,
    handle_backtest_catalog,
    handle_backtest_custom,
    handle_db_overview,
    handle_klines,
    handle_strategies,
};

pub fn build_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("web-client");

    Router::new()
        .route("/api/klines", get(handle_klines))
        .route("/api/db", get(handle_db_overview))
        .route("/api/strategies", get(handle_strategies))
        .route("/api/backtest", post(handle_backtest))
        .route("/api/backtest/catalog", post(handle_backtest_catalog))
        .route("/api/backtest/custom", post(handle_backtest_custom))
        .fallback_service(ServeDir::new(static_dir))
        .layer(cors)
}
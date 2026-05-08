use axum::{routing::get, Router, extract::{Query, Path, Json}};
use crate::db::{get_assets, insert_data};
use crate::api::stocks::fetch_daily_ohlcv;
use crate::models::Models;

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(test))
}

async fn test() -> String {
    String::from("test, test!")
}

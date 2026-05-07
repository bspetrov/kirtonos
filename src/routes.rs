use axum::{routing::get, Router};
use crate::db::{get_assets, insert_data};
use crate::api::stocks::fetch_daily_ohlcv;
use crate::models::Models;

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(test))
        .route("/ingest_macro", get(collect_macro_data))
}

async fn test() -> String {
    String::from("test, test!")
}

// TODO-> Finish this
async fn collect_macro_data() -> &'static str {
    let assets = get_assets().await.unwrap();

    for asset in assets {
        let twelve_data = fetch_daily_ohlcv(&asset).await.unwrap();
        // insert_data(Models::Pricing(twelve_data), None);
    }
    "Macro data collected inside DuckDB!"
}
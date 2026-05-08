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

async fn collect_macro_data() -> String{
    let assets = get_assets().unwrap();

    for asset in assets {
        let twelve_data = fetch_daily_ohlcv(&asset).await.unwrap();
        println!("Fetching data for -> {}", asset);
        let _ = insert_data(Models::Pricing(twelve_data), None);
        println!("Fetched data successfully for -> {}", asset);
    }
    String::from("Fetched data successfully!")
}
mod api;
mod db;
mod metrics;
mod models;
mod report;
mod backtest;
use dotenv;


#[tokio::main]
async fn main () {
    dotenv::dotenv().ok();
    println!("Welcome, to Kirtonos (but faster!)");

    match db::initialize_db() {
        Ok(_) => println!("Database initialized and running.."),
        Err(e) => println!("Error -> {e}")
    }


    // let spy_data = api::stocks::fetch_daily_ohlcv("SPY").await.unwrap();
    // let qqq_data = api::stocks::fetch_daily_ohlcv("QQQ").await.unwrap();
    // let iwm_data = api::stocks::fetch_daily_ohlcv("IWM").await.unwrap();
}
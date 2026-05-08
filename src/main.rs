mod api;
mod backtest;
mod db;
mod metrics;
mod models;
mod report;
use dotenv;
mod routes;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    println!("Starting app...");

    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    match db::initialize_db() {
        Ok(_) => println!("Database initialized and running.."),
        Err(e) => println!("Error -> {e}"),
    }
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    
    let sync_result = sync_market_data().await;
    match sync_result {
        Ok(v) => println!("{}", v),
        Err(e) => println!("Problem with syncing market data - {}", e) 
    }
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    let app = routes::build_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Kirtonos REST listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn sync_market_data() -> anyhow::Result<String> {
    println!("Syncing database with market data for all default assets..");
    let assets = db::get_assets()?;
    for asset in assets {
        let twelve_data = api::stocks::fetch_daily_ohlcv(&asset).await?;
        println!("Fetching data for -> {}", asset);
        let _ = db::insert_data(models::Models::Pricing(twelve_data), None)?;
        println!("Fetched data successfully for -> {}", asset);
    }
    Ok(String::from("Market data synced successfully"))
}

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
    let app = routes::build_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Kirtonos REST listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

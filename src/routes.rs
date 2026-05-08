use crate::db::get_symbol_prices;
use crate::models::Pricing;
use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::Deserialize;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}

#[derive(Deserialize)]
struct PricingRequest {
    symbols: Vec<String>,
    start_date: String,
    end_date: String,
}

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(test))
        .route("/pricing_range", post(get_symbol_pricing))
}

async fn test() -> String {
    String::from("test, test!")
}

async fn get_symbol_pricing(
    Json(payload): Json<PricingRequest>,
) -> Result<Json<Vec<Pricing>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;
    let results = get_symbol_prices(payload.symbols, start_date, end_date, None)?;
    Ok(Json(results))
}

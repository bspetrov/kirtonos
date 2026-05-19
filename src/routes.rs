use crate::db::{get_risk_free_rate, get_symbol_prices};
use crate::metrics::{ratios, returns, risk};
use crate::models::Pricing;
use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Deserialize)]
struct MetricsRequest {
    symbols: Vec<String>,
    start_date: String,
    end_date: String,
}

#[derive(Serialize)]
struct ReturnsResponse {
    symbol: String,
    simple_returns: f32,
    log_returns: f32,
    cummulative_returns: f32,
    annualized_returns: f32,
}

#[derive(Serialize)]
struct RiskResponse {
    symbol: String,
    volatility: f64,
    max_drawdown: f64,
}

#[derive(Serialize)]
struct RatioResponse {
    symbol: String,
    value: f64,
}

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(test))
        .route("/pricing_range", post(get_symbol_pricing))
        .route("/metrics/returns", post(get_returns))
        .route("/metrics/risk", post(get_risk))
        .route("/ratios/sharpe", post(get_sharpe))
        .route("/ratios/sortino", post(get_sortino))
        .route("/ratios/calmar", post(get_calmar))
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

async fn get_returns(
    Json(payload): Json<MetricsRequest>,
) -> Result<Json<Vec<ReturnsResponse>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;

    let symbols_pricing = get_symbol_prices(payload.symbols, start_date, end_date, None)?;

    // Group close prices by symbol — DB returns a flat list with all symbols interleaved
    let mut by_symbol: HashMap<String, Vec<f64>> = HashMap::new();
    for p in &symbols_pricing {
        by_symbol.entry(p.symbol.clone()).or_default().push(p.close);
    }

    let mut results: Vec<ReturnsResponse> = by_symbol
        .into_iter()
        .map(|(symbol, closes)| {
            let simple = returns::calculate_simple_returns(&closes);
            let log = returns::calculate_log_returns(&closes);
            let cumulative = returns::cumulative_return(&simple);
            let annualized = returns::annualized_return(&simple);
            let log_cumulative: f64 = log.iter().sum();
            ReturnsResponse {
                symbol,
                simple_returns: cumulative as f32,
                log_returns: log_cumulative as f32,
                cummulative_returns: cumulative as f32,
                annualized_returns: annualized as f32,
            }
        })
        .collect();

    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Json(results))
}

async fn get_risk(
    Json(payload): Json<MetricsRequest>,
) -> Result<Json<Vec<RiskResponse>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;

    let symbols_pricing = get_symbol_prices(payload.symbols, start_date, end_date, None)?;

    // Group close prices by symbol — DB returns a flat list with all symbols interleaved
    let mut by_symbol: HashMap<String, Vec<f64>> = HashMap::new();
    for p in &symbols_pricing {
        by_symbol.entry(p.symbol.clone()).or_default().push(p.close);
    }

    let mut results: Vec<RiskResponse> = by_symbol
        .into_iter()
        .map(|(symbol, closes)| {
            let log_returns = returns::calculate_log_returns(&closes);
            let volatility = risk::volatility(&log_returns);
            let max_drawdown = risk::max_drawdown(&closes);

            RiskResponse {
                symbol,
                volatility,
                max_drawdown,
            }
        })
        .collect();

    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Json(results))
}

async fn get_sharpe(
    Json(payload): Json<MetricsRequest>,
) -> Result<Json<Vec<RatioResponse>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;
    let risk_free_rate = get_risk_free_rate(None)?;
    let symbols_pricing = get_symbol_prices(payload.symbols, start_date, end_date, None)?;

    let mut by_symbol: HashMap<String, Vec<f64>> = HashMap::new();
    for p in &symbols_pricing {
        by_symbol.entry(p.symbol.clone()).or_default().push(p.close);
    }

    let mut results: Vec<RatioResponse> = by_symbol
        .into_iter()
        .map(|(symbol, closes)| {
            let simple = returns::calculate_simple_returns(&closes);
            let log = returns::calculate_log_returns(&closes);
            let ann_return = returns::annualized_return(&simple);
            let vol = risk::volatility(&log);
            RatioResponse {
                symbol,
                value: ratios::sharpe(ann_return, vol, risk_free_rate),
            }
        })
        .collect();

    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Json(results))
}

async fn get_sortino(
    Json(payload): Json<MetricsRequest>,
) -> Result<Json<Vec<RatioResponse>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;
    let risk_free_rate = get_risk_free_rate(None)?;
    let symbols_pricing = get_symbol_prices(payload.symbols, start_date, end_date, None)?;

    let mut by_symbol: HashMap<String, Vec<f64>> = HashMap::new();
    for p in &symbols_pricing {
        by_symbol.entry(p.symbol.clone()).or_default().push(p.close);
    }

    let mut results: Vec<RatioResponse> = by_symbol
        .into_iter()
        .map(|(symbol, closes)| {
            let simple = returns::calculate_simple_returns(&closes);
            let log = returns::calculate_log_returns(&closes);
            let ann_return = returns::annualized_return(&simple);
            RatioResponse {
                symbol,
                value: ratios::sortino(ann_return, &log, risk_free_rate),
            }
        })
        .collect();

    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Json(results))
}

async fn get_calmar(
    Json(payload): Json<MetricsRequest>,
) -> Result<Json<Vec<RatioResponse>>, AppError> {
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")?;
    let symbols_pricing = get_symbol_prices(payload.symbols, start_date, end_date, None)?;

    let mut by_symbol: HashMap<String, Vec<f64>> = HashMap::new();
    for p in &symbols_pricing {
        by_symbol.entry(p.symbol.clone()).or_default().push(p.close);
    }

    let mut results: Vec<RatioResponse> = by_symbol
        .into_iter()
        .map(|(symbol, closes)| {
            let simple = returns::calculate_simple_returns(&closes);
            let ann_return = returns::annualized_return(&simple);
            let max_dd = risk::max_drawdown(&closes);
            RatioResponse {
                symbol,
                value: ratios::calmar(ann_return, max_dd),
            }
        })
        .collect();

    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Json(results))
}

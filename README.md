# Kirtonos v2

A Rust REST API for quantitative financial analysis and backtesting. Fetches market, macro, and crypto data from external APIs, stores it locally in DuckDB, and exposes endpoints for portfolio metrics and strategy simulation.

## Features

- Fetches daily OHLCV data for stocks, ETFs, and commodities via Twelve Data
- Fetches cryptocurrency data (BTC, ETH) via Binance
- Fetches macro and economic data (yields, inflation, unemployment, GDP, oil) via FRED
- Persists all data locally in an embedded DuckDB database
- REST API built with axum for consumption by quantitative analysts
- Portfolio metrics: returns, risk, Sharpe, Sortino, Calmar ratios (live risk-free rate from DTB3)
- Diversification and benchmarking analytics (in progress)
- Strategy backtesting engine (in progress)

## Asset Coverage

| Category | Tickers |
|---|---|
| Broad Market ETFs | SPY, QQQ, IWM |
| Tech | NVDA |
| Energy | XOM, CVX, COP |
| Commodities | SLV |
| Crypto | BTCUSDT, ETHUSDT (Binance) |
| Macro | DGS10, DGS2, DTB3, FEDFUNDS, CPIAUCSL, UNRATE, GDP, T10Y2Y, DCOILWTICO (FRED) |

## Requirements

- Rust (edition 2024)
- A `.env` file in the project root (see below)

## Setup

```bash
git clone https://github.com/yourusername/kirtonos_v2
cd kirtonos_v2
```

Create a `.env` file:

```
DATABASE_URL=data/data.duckdb
TWELVE_KEY=your_twelve_data_api_key
FRED_KEY=your_fred_api_key
```

API keys:
- Twelve Data: [twelvedata.com](https://twelvedata.com) — free tier, 800 requests/day
- FRED: [fred.stlouisfed.org/docs/api](https://fred.stlouisfed.org/docs/api/fred/) — free, requires registration
- Binance: no API key required for public market data endpoints

```bash
cargo run -- sync   # sync all market/macro data, then start the server
cargo run           # start the server only (skip sync)
```

The server starts on `http://0.0.0.0:3000`.

## Endpoints

All metrics and ratio endpoints accept a JSON body with `symbols`, `start_date`, and `end_date`.

| Method | Path | Description |
|---|---|---|
| GET | `/healthcheck` | Health check |
| POST | `/pricing_range` | OHLCV prices for one or more symbols over a date range |
| POST | `/metrics/returns` | Simple, log, cumulative, and annualized returns per symbol |
| POST | `/metrics/risk` | Annualized volatility and max drawdown per symbol |
| POST | `/ratios/sharpe` | Sharpe ratio per symbol (risk-free rate from live DTB3) |
| POST | `/ratios/sortino` | Sortino ratio per symbol (penalises downside volatility only) |
| POST | `/ratios/calmar` | Calmar ratio per symbol (annualized return / max drawdown) |

### Request body (all metrics and ratio endpoints)

```json
{
  "symbols": ["SPY", "QQQ"],
  "start_date": "2024-01-01",
  "end_date": "2024-12-31"
}
```

### Example responses

`POST /metrics/risk`
```json
[
  { "symbol": "QQQ", "volatility": 0.182, "max_drawdown": -0.083 },
  { "symbol": "SPY", "volatility": 0.141, "max_drawdown": -0.061 }
]
```

`POST /ratios/sharpe`
```json
[
  { "symbol": "QQQ", "value": 1.24 },
  { "symbol": "SPY", "value": 0.97 }
]
```

## Commands

```bash
cargo build       # compile
cargo run         # run the server
cargo test        # run all tests
cargo clippy      # lint
cargo fmt         # format
```

## Stack

| Concern | Crate |
|---|---|
| Async runtime | `tokio` |
| REST API | `axum` |
| HTTP client | `reqwest` |
| Database | `duckdb` (embedded, columnar) |
| Serialization | `serde` / `serde_json` |
| Error handling | `anyhow` |
| Date/time | `chrono` |
| CLI | `clap` |

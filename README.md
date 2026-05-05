# Kirtonos v2

A Rust REST API for quantitative financial analysis and backtesting. Fetches market, macro, and crypto data from external APIs, stores it locally in DuckDB, and exposes endpoints for portfolio metrics and strategy simulation.

## Features

- Fetches daily OHLCV data for stocks, ETFs, and commodities via Twelve Data
- Fetches cryptocurrency data (BTC, ETH) via CoinGecko
- Fetches macro data and risk-free rates (3-month T-bill, 10-year Treasury) via FRED
- Persists all data locally in an embedded DuckDB database
- REST API for consumption by quantitative analysts
- Portfolio metrics: returns, risk, Sharpe ratio, diversification, benchmarking (in progress)
- Strategy backtesting engine (in progress)

## Asset Coverage

| Category | Tickers |
|---|---|
| Indices/ETFs | SPY, QQQ, IWM |
| Tech | NVDA |
| Energy | XOM, CVX, COP |
| Commodities | GLD, SLV |
| Crypto | BTC, ETH |
| Macro / Risk-free | 10Y Treasury, 3M T-bill (FRED) |

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

```bash
cargo run
```

## Commands

```bash
cargo build       # compile
cargo run         # run
cargo test        # run all tests
cargo clippy      # lint
cargo fmt         # format
```

## Stack

| Concern | Crate |
|---|---|
| Async runtime | `tokio` |
| HTTP client | `reqwest` |
| Database | `duckdb` (embedded, columnar) |
| Serialization | `serde` / `serde_json` |
| Error handling | `anyhow` |
| Date/time | `chrono` |
| CLI | `clap` |
| Terminal output | `colored` |

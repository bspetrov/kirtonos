use crate::models::{MacroData, Pricing};
use chrono::{DateTime, NaiveDate};
use reqwest;
use serde::Deserialize;
use std::env;

pub fn get_api_key(provider: &str) -> String {
    let var = match provider {
        "twelve" => "TWELVE_KEY",
        "fred" => "FRED_KEY",
        "coingecko" => "COINGECKO_KEY",
        _ => panic!("Unknown provider: {}", provider),
    };
    env::var(var).expect(&format!("Missing env var: {}", var))
}

pub mod stocks {
    const SYMBOLS: &[(&str, &str)] = &[
        ("SPY", "ETF"),
        ("QQQ", "ETF"),
        ("IWM", "ETF"),
        ("NVDA", "Stock"),
        ("XOM", "Stock"),
        ("CVX", "Stock"),
        ("COP", "Stock"),
        ("GLD", "ETF"),
        ("SLV", "ETF"),
        ("XLE", "ETF"),
        ("XLK", "ETF"),
        ("XLF", "ETF"),
    ];

    #[derive(super::Deserialize, Debug)]
    pub struct DailyBar {
        pub datetime: String,
        pub open: String,
        pub high: String,
        pub low: String,
        pub close: String,
        pub volume: String,
    }

    #[derive(super::Deserialize, Debug)]
    pub struct TimeSeriesResponse {
        pub values: Vec<DailyBar>,
    }

    pub async fn fetch_daily_ohlcv(symbol: &str, start_date: super::NaiveDate) -> anyhow::Result<Vec<super::Pricing>> {
        let api_key = super::get_api_key("twelve");
        let api_url = format!(
            "https://api.twelvedata.com/time_series?symbol={}&start_date={}&interval=1day&adjust=all&apikey={}",
            symbol, start_date, api_key
        );
        let response = reqwest::get(api_url).await?;
        let text: TimeSeriesResponse = response.json().await?;
        let mut query_results = Vec::new();

        for row in &text.values {
            query_results.push(super::Pricing {
                datetime: super::NaiveDate::parse_from_str(&row.datetime, "%Y-%m-%d")?,
                symbol: symbol.to_string(),
                open: row.open.parse()?,
                high: row.high.parse()?,
                low: row.low.parse()?,
                close: row.close.parse()?,
                volume: row.volume.parse()?,
            })
        }
        Ok(query_results)
    }
}

pub mod crypto {
    pub const SYMBOLS: &[&str] = &["BTCUSDT", "ETHUSDT"];
    pub const START_MS: i64 = 1483228800000; // 2017-01-01 UTC

    pub fn parse_binance_row(
        row: &[serde_json::Value],
        symbol: &str,
    ) -> anyhow::Result<super::Pricing> {
        let timestamp_ms = row[0].as_i64().unwrap();
        let date = super::DateTime::from_timestamp(timestamp_ms / 1000, 0)
            .unwrap()
            .date_naive();
        Ok(super::Pricing {
            datetime: date,
            symbol: symbol.to_string(),
            open: row[1].as_str().unwrap().parse()?,
            high: row[2].as_str().unwrap().parse()?,
            low: row[3].as_str().unwrap().parse()?,
            close: row[4].as_str().unwrap().parse()?,
            volume: row[5].as_str().unwrap().parse::<f64>()? as i64,
        })
    }

    pub async fn fetch_crypto(symbol: &str, start_date: super::NaiveDate) -> anyhow::Result<Vec<super::Pricing>> {
        let mut query_results = Vec::new();
        let mut start_ms = start_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();

        loop {
            let url = format!(
                "https://api.binance.com/api/v3/klines?symbol={}&interval=1d&limit=1000&startTime={}",
                symbol, start_ms
            );
            let response = reqwest::get(url).await?;
            let raw_data: Vec<Vec<serde_json::Value>> = response.json().await?;

            if raw_data.is_empty() {
                break;
            }

            let last_ts = raw_data.last().unwrap()[0].as_i64().unwrap();

            for row in &raw_data {
                query_results.push(parse_binance_row(row, symbol)?);
            }

            if raw_data.len() < 1000 {
                break;
            }

            start_ms = last_ts + 1;
        }
        Ok(query_results)
    }
}

pub mod economic {
    pub const MACRO_SERIES: &[(&str, &str)] = &[
        ("DGS10", "daily"),
        ("DGS2", "daily"),
        ("DTB3", "daily"),
        ("FEDFUNDS", "monthly"),
        ("CPIAUCSL", "monthly"),
        ("UNRATE", "monthly"),
        ("A191RL1Q225SBEA", "quarterly"),
        ("T10Y2Y", "daily"),
        ("DCOILWTICO", "daily"),
    ];

    #[derive(super::Deserialize, Debug)]
    pub struct FedData {
        pub realtime_start: String,
        pub realtime_end: String,
        pub date: String,
        pub value: String,
    }

    #[derive(super::Deserialize, Debug)]
    pub struct FedResponse {
        pub observations: Vec<FedData>,
    }

    pub async fn fetch_fred_data(series_id: &str, start_date: super::NaiveDate) -> anyhow::Result<Vec<super::MacroData>> {
        let fred_api_key = super::get_api_key("fred");
        let fred_url = format!(
            "https://api.stlouisfed.org/fred/series/observations?series_id={}&observation_start={}&api_key={}&file_type=json",
            series_id, start_date, fred_api_key
        );

        let frequency = MACRO_SERIES
            .iter()
            .find(|(id, _)| *id == series_id)
            .map(|(_, freq)| *freq)
            .unwrap_or("unknown");
        let response = reqwest::get(fred_url).await?;

        let response_json: FedResponse = response.json().await?;
        let mut query_results = Vec::new();

        for row in &response_json.observations {
            if row.value == "." {
                continue;
            }
            query_results.push(super::MacroData {
                date: super::NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")?,
                series_id: series_id.to_string(),
                value: row.value.parse()?,
                frequency: frequency.to_string(),
            })
        }
        Ok(query_results)
    }
}

pub mod datasets {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pricing_parses_correctly() {
        let bar = stocks::DailyBar {
            datetime: "2024-01-15".to_string(),
            open: "185.0".to_string(),
            high: "191.0".to_string(),
            low: "181.2".to_string(),
            close: "189.4".to_string(),
            volume: "70350000".to_string(),
        };

        let pricing = Pricing {
            datetime: NaiveDate::parse_from_str(&bar.datetime, "%Y-%m-%d").unwrap(),
            symbol: "SPY".to_string(),
            open: bar.open.parse().unwrap(),
            high: bar.high.parse().unwrap(),
            low: bar.low.parse().unwrap(),
            close: bar.close.parse().unwrap(),
            volume: bar.volume.parse().unwrap(),
        };

        assert_eq!(
            pricing.datetime,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        assert_eq!(pricing.symbol, "SPY");
        assert_eq!(pricing.open, 185.0);
        assert_eq!(pricing.high, 191.0);
        assert_eq!(pricing.low, 181.2);
        assert_eq!(pricing.close, 189.4);
        assert_eq!(pricing.volume, 70350000);
    }

    #[test]
    fn test_start_ms_is_2017_jan_01() {
        let date = chrono::DateTime::from_timestamp(crypto::START_MS / 1000, 0)
            .unwrap()
            .date_naive();
        assert_eq!(date, NaiveDate::from_ymd_opt(2017, 1, 1).unwrap());
    }

    #[test]
    fn test_volume_float_truncates_to_i64() {
        let raw = "12345.678";
        let volume = raw.parse::<f64>().unwrap() as i64;
        assert_eq!(volume, 12345);
    }

    #[test]
    fn test_macro_series_lookup_daily() {
        let freq = economic::MACRO_SERIES
            .iter()
            .find(|(id, _)| *id == "DGS10")
            .map(|(_, freq)| *freq);
        assert_eq!(freq, Some("daily"));
    }

    #[test]
    fn test_macro_series_lookup_monthly() {
        let freq = economic::MACRO_SERIES
            .iter()
            .find(|(id, _)| *id == "CPIAUCSL")
            .map(|(_, freq)| *freq);
        assert_eq!(freq, Some("monthly"));
    }

    #[test]
    fn test_macro_series_lookup_quarterly() {
        let freq = economic::MACRO_SERIES
            .iter()
            .find(|(id, _)| *id == "A191RL1Q225SBEA")
            .map(|(_, freq)| *freq);
        assert_eq!(freq, Some("quarterly"));
    }

    #[test]
    fn test_macro_series_unknown_returns_none() {
        let freq = economic::MACRO_SERIES
            .iter()
            .find(|(id, _)| *id == "NONEXISTENT")
            .map(|(_, freq)| *freq);
        assert_eq!(freq, None);
    }

    #[test]
    fn test_fed_data_parses_correctly() {
        let obs = economic::FedData {
            realtime_start: "2024-01-15".to_string(),
            realtime_end: "9999-01-01".to_string(),
            date: "2024-01-15".to_string(),
            value: "4.15".to_string(),
        };

        let date = NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d").unwrap();
        let value: f64 = obs.value.parse().unwrap();

        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(value, 4.15);
    }

    #[test]
    fn test_fred_missing_value_is_dot() {
        let obs = economic::FedData {
            realtime_start: "2024-01-15".to_string(),
            realtime_end: "9999-01-01".to_string(),
            date: "2024-01-15".to_string(),
            value: ".".to_string(),
        };
        assert!(obs.value.parse::<f64>().is_err());
    }

    #[test]
    fn test_binance_row_parses_to_pricing() {
        let symbol = "BTCUSDT";
        let row = serde_json::json!([                                                                                  
            1483228800,
            "95000.0",                                                                                                 
            "96000.0",
            "94000.0",                                                                                                 
            "95500.0",                                                                                               
            "1234.5"                                                                                                 
        ]);
        let result = crypto::parse_binance_row(row.as_array().unwrap(), "BTCUSDT").unwrap();

        let timestamp_ms = row[0].as_i64().unwrap();
        let date = super::DateTime::from_timestamp(timestamp_ms / 1000, 0)
            .unwrap()
            .date_naive();
        let pricing_mock = Pricing {
            datetime: date,
            symbol: symbol.to_string(),
            open: row[1].as_str().unwrap().parse().unwrap(),
            high: row[2].as_str().unwrap().parse().unwrap(),
            low: row[3].as_str().unwrap().parse().unwrap(),
            close: row[4].as_str().unwrap().parse().unwrap(),
            volume: row[5].as_str().unwrap().parse::<f64>().unwrap() as i64,
        };

        assert_eq!(pricing_mock, result);

    }

}

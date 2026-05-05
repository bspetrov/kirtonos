use reqwest;
use std::env;
use chrono::NaiveDate;
use crate::models::Pricing;


pub fn get_twelve_key() -> String {
    let api_key: String = env::var("TWELVE_KEY").expect("Missing Twelve Data key!");
    api_key
}

pub fn get_fred_key() -> String {
    let api_key: String = env::var("FRED_KEY").expect("Missing FRED API KEY");
    api_key
}

pub mod stocks {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct DailyBar {
        pub datetime: String,
        pub open: String,
        pub high: String,
        pub low: String,
        pub close: String,
        pub volume: String
    }

    #[derive(Deserialize, Debug)]
    pub struct TimeSeriesResponse {
        pub values: Vec<DailyBar>
    }

    pub async fn fetch_daily_ohlcv(symbol: &str) -> anyhow::Result<Vec<super::Pricing>> {
        let api_key = super::get_twelve_key();
        let api_url= format!("https://api.twelvedata.com/time_series?symbol={}&start_date=2010-01-01&interval=1day&apikey={}", symbol, api_key);
        let response = reqwest::get(api_url).await?;

        let text: TimeSeriesResponse = response.json().await?;
        let mut query_results = Vec::new();

        for row in &text.values {
            query_results.push(
                super::Pricing {
                    datetime: super::NaiveDate::parse_from_str(&row.datetime, "%Y-%m-%d")?,
                    symbol: symbol.to_string(),
                    open: row.open.parse()?,
                    high: row.high.parse()?,
                    low: row.low.parse()?,
                    close: row.close.parse()?,
                    volume: row.volume.parse()?
                }
            )
        }
        Ok(query_results)
    }

}

pub mod crypto {

}

pub mod economic {
    pub async fn fetch_fred_data(series_id: &str) -> anyhow::Result<()>{
        let fred_api_key = super::get_fred_key();
        let fred_url = format!("https://api.stlouisfed.org/fred/series/observations?series_id={}&observation_start=2016-01-01&api_key={}&file_type=json", series_id, fred_api_key);

        let response = reqwest::get(fred_url).await?;

        let text = response.text().await?;
        println!("{}", text);

        Ok(())
    }
}

pub mod datasets {  

}


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
            volume: "70350000".to_string()
        };

        let pricing = Pricing {
            datetime: NaiveDate::parse_from_str(&bar.datetime, "%Y-%m-%d").unwrap(),
            symbol: "SPY".to_string(),
            open: bar.open.parse().unwrap(),
            high: bar.high.parse().unwrap(),
            low: bar.low.parse().unwrap(),
            close: bar.close.parse().unwrap(),
            volume: bar.volume.parse().unwrap()
        };

        assert_eq!(pricing.datetime, NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap());
        assert_eq!(pricing.symbol, "SPY");
        assert_eq!(pricing.open, 185.0);
        assert_eq!(pricing.high, 191.0);
        assert_eq!(pricing.low, 181.2);
        assert_eq!(pricing.close, 189.4);
        assert_eq!(pricing.volume, 70350000);
    }
    
}
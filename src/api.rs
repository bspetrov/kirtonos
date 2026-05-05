use reqwest;
use dotenv;
use serde::Deserialize;
use std::env;


#[derive(Deserialize)]
#[derive(Debug)]
pub struct DailyBar {
    pub datetime: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct TimeSeriesResponse {
    pub values: Vec<DailyBar>
}

pub fn get_api_key() -> String {
    let api_key: String = env::var("TWELVE_KEY").expect("Missing Twelve Data key!");
    api_key
}

pub mod stocks {
    pub async fn fetch_daily_ohlcv(symbol: &str) -> anyhow::Result<super::TimeSeriesResponse> {
        let api_key = super::get_api_key();
        let api_url= format!("https://api.twelvedata.com/time_series?symbol={}&interval=1day&apikey={}", symbol, api_key);
        let response = reqwest::get(api_url).await?;

        let text: super::TimeSeriesResponse = response.json().await?;
        print!("{:?}", text.values);
        Ok(text)
    }

}

pub mod crypto {

}

pub mod datasets {  

}


#[cfg(test)]
mod tests {
    use super::*;

    
}
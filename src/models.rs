use chrono::NaiveDate;

pub enum Models {
      Pricing(Vec<Pricing>),
      Assets(Vec<Assets>),
      RiskFreeRates(Vec<RiskFreeRates>)
}

pub struct Pricing {
    pub datetime: NaiveDate,
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

pub struct Assets {
    pub symbol: String,
    pub name: String,
    pub asset_class: String,
    pub sector: String,
    pub currency: String
}

pub struct RiskFreeRates {
    pub date: NaiveDate,
    pub rate_3m: f64,
    pub rate_10y: f64
}
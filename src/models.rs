use chrono::NaiveDate;

pub enum Models {
    Pricing(Vec<Pricing>),
    Assets(Vec<Assets>),
    MacroData(Vec<MacroData>),
}

#[derive(Debug, PartialEq)]
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
    pub currency: String,
}

pub struct MacroData {
    pub date: NaiveDate,
    pub series_id: String,
    pub value: f64,
    pub frequency: String,
}

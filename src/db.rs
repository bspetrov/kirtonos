use crate::models::{Assets, MacroData, Models, Pricing};
use chrono::NaiveDate;
use duckdb::{Connection, Result, params};
use std::env;

pub enum DataTable {
    Pricing,
    MacroData,
}

fn get_db_url() -> String {
    let db_url = env::var("DATABASE_URL").expect("Missing DB location -> must set!");
    db_url
}

pub fn default_assets() -> Vec<Assets> {
    vec![
        Assets {
            symbol: "SPY".to_string(),
            name: "SPDR S&P 500 ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "QQQ".to_string(),
            name: "Invesco Nasdaq 100 ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "IWM".to_string(),
            name: "iShares Russell 2000 ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "NVDA".to_string(),
            name: "NVIDIA Corporation".to_string(),
            asset_class: "Stock".to_string(),
            sector: "Technology".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "XOM".to_string(),
            name: "ExxonMobil Corporation".to_string(),
            asset_class: "Stock".to_string(),
            sector: "Energy".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "CVX".to_string(),
            name: "Chevron Corporation".to_string(),
            asset_class: "Stock".to_string(),
            sector: "Energy".to_string(),
            currency: "USD".to_string(),
        },
        Assets {
            symbol: "COP".to_string(),
            name: "ConocoPhillips".to_string(),
            asset_class: "Stock".to_string(),
            sector: "Energy".to_string(),
            currency: "USD".to_string(),
        },
        // Assets {
        //     symbol: "GLD".to_string(),
        //     name: "SPDR Gold Shares ETF".to_string(),
        //     asset_class: "ETF".to_string(),
        //     sector: "Commodity".to_string(),
        //     currency: "USD".to_string(),
        // },
        Assets {
            symbol: "SLV".to_string(),
            name: "iShares Silver Trust ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Commodity".to_string(),
            currency: "USD".to_string(),
        },
    ]
}

pub fn initialize_db() -> Result<()> {
    let db_url = get_db_url();
    let conn = Connection::open(db_url)?;

    conn.execute_batch(
        "BEGIN;
             CREATE TABLE IF NOT EXISTS pricing (
                     datetime DATE NOT NULL,
                     symbol TEXT NOT NULL,
                     open DOUBLE NOT NULL,
                     high DOUBLE NOT NULL,
                     low DOUBLE NOT NULL,
                     close DOUBLE NOT NULL,
                     volume BIGINT NOT NULL,
                     PRIMARY KEY (datetime, symbol)
                     );
             CREATE TABLE IF NOT EXISTS assets (
                     symbol TEXT NOT NULL,
                     name TEXT NOT NULL,
                     asset_class TEXT NOT NULL,
                     sector TEXT NOT NULL,
                     currency TEXT NOT NULL,
                     PRIMARY KEY (symbol)
                     );
             CREATE TABLE IF NOT EXISTS macro_data (
                     date DATE NOT NULL,
                     series_id TEXT NOT NULL,
                     value DOUBLE NOT NULL,
                     frequency TEXT NOT NULL,
                     PRIMARY KEY (date, series_id)
                     );
             COMMIT;
        ",
    )?;

    let assets = Models::Assets(default_assets());
    insert_data(assets, Some(&conn))?;
    Ok(())
}

pub fn insert_data(data_model: Models, conn: Option<&Connection>) -> Result<()> {
    let owned;
    let conn: &Connection = match conn {
        Some(c) => c,
        None => {
            owned = Connection::open(get_db_url())?;
            &owned
        }
    };
    match data_model {
        Models::Assets(assets) => {
            conn.execute_batch("BEGIN")?;
            for asset in assets {
                conn.execute(
                                            "INSERT OR IGNORE INTO assets (symbol, name, asset_class, sector, currency) VALUES (?, ?, ?, ?, ?)",
                                            params![asset.symbol, asset.name, asset.asset_class, asset.sector, asset.currency]
                                    )?;
            }
            conn.execute_batch("COMMIT")?;
        }

        Models::Pricing(pricing) => {
            conn.execute_batch("BEGIN")?;
            for p in pricing {
                conn.execute(
                                            "INSERT INTO pricing (datetime, symbol, open, high, low, close, volume) VALUES (?, ?, ?, ?, ?, ?, ?)",
                                            params![p.datetime.to_string(), p.symbol, p.open, p.high, p.low, p.close, p.volume]
                                    )?;
            }
            conn.execute_batch("COMMIT")?;
        }

        Models::MacroData(macro_data) => {
            conn.execute_batch("BEGIN")?;
            for m in macro_data {
                conn.execute(
                                            "INSERT OR IGNORE INTO macro_data (date, series_id, value, frequency) VALUES (?, ?, ?, ?)",
                                            params![m.date.to_string(), m.series_id, m.value, m.frequency]
                                    )?;
            }
            conn.execute_batch("COMMIT")?;
        }
    }
    Ok(())
}

pub fn get_symbol_prices(
    symbols: Vec<String>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    conn: Option<&Connection>,
) -> anyhow::Result<Vec<Pricing>> {
    let owned;
    let conn: &Connection = match conn {
        Some(c) => c,
        None => {
            owned = Connection::open(get_db_url())?;
            &owned
        }
    };
    let placeholders = symbols.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT CAST(datetime AS VARCHAR), symbol, open, high, low, close, volume FROM pricing WHERE symbol IN ({}) AND datetime >= ? AND datetime <= ?",
        placeholders
    );
    let mut stmt = conn.prepare(&sql)?;

    let mut all_params: Vec<String> = symbols;
    all_params.push(start_date.to_string());
    all_params.push(end_date.to_string());

    let mut rows = stmt.query(duckdb::params_from_iter(all_params))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        results.push(Pricing {
            datetime: NaiveDate::parse_from_str(&row.get::<_, String>(0)?, "%Y-%m-%d")?,
            symbol: row.get(1)?,
            open: row.get(2)?,
            high: row.get(3)?,
            low: row.get(4)?,
            close: row.get(5)?,
            volume: row.get(6)?,
        });
    }
    Ok(results)
}

pub fn get_latest_date(id: &str, table: DataTable) -> anyhow::Result<Option<NaiveDate>> {
    let conn = Connection::open(get_db_url())?;
    let sql = match table {
        DataTable::Pricing => "SELECT CAST(MAX(datetime) AS VARCHAR) FROM pricing WHERE symbol = ?",
        DataTable::MacroData => {
            "SELECT CAST(MAX(date) AS VARCHAR) FROM macro_data WHERE series_id = ?"
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        let val: Option<String> = row.get(0)?;
        return match val {
            Some(s) => Ok(Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?)),
            None => Ok(None),
        };
    }
    Ok(None)
}

pub fn get_assets() -> Result<Vec<String>> {
    let db_url = get_db_url();
    let conn = Connection::open(db_url)?;

    let mut stmt = conn.prepare("SELECT symbol from assets")?;
    let mut rows = stmt.query([])?;

    let mut assets = Vec::new();

    while let Some(row) = rows.next()? {
        assets.push(row.get(0)?)
    }

    Ok(assets)
}

pub fn get_risk_free_rate(conn: Option<&Connection>) -> anyhow::Result<f64> {
    let owned;
    let conn: &Connection = match conn {
        Some(c) => c,
        None => {
            owned = Connection::open(get_db_url())?;
            &owned
        }
    };
    let mut stmt = conn.prepare(
        "SELECT value FROM macro_data WHERE series_id = 'DTB3' ORDER BY date DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let value: f64 = row.get(0)?;
        return Ok(value / 100.0);
    }
    anyhow::bail!("No DTB3 data found in macro_data")
}

// This was simply a test function to see if entities get created..
pub fn show_tables() -> Result<()> {
    let db_url = get_db_url();
    let conn = Connection::open(db_url)?;
    let mut stmt = conn.prepare("DESCRIBE pricing")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let col_name: String = row.get(0)?;
        let col_type: String = row.get(1)?;
        println!("{}: {}", col_name, col_type);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Pricing;
    use chrono::NaiveDate;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "BEGIN;
                CREATE TABLE pricing (
                        datetime DATE NOT NULL,
                        symbol TEXT NOT NULL,
                        open DOUBLE NOT NULL,
                        high DOUBLE NOT NULL,
                        low DOUBLE NOT NULL,
                        close DOUBLE NOT NULL,
                        volume BIGINT NOT NULL,
                        PRIMARY KEY (datetime, symbol)
                        );
                CREATE TABLE assets (
                        symbol TEXT NOT NULL,
                        name TEXT NOT NULL,
                        asset_class TEXT NOT NULL,
                        sector TEXT NOT NULL,
                        currency TEXT NOT NULL,
                        PRIMARY KEY (symbol)
                        );
                CREATE TABLE macro_data (
                        date DATE NOT NULL,
                        series_id TEXT NOT NULL,
                        value DOUBLE NOT NULL,
                        frequency TEXT NOT NULL,
                        PRIMARY KEY (date, series_id)
                        );
                COMMIT;
            ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_pricing_insert_roundtrip() {
        let conn = setup_conn();
        let row = Pricing {
            datetime: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            symbol: "AAPL".to_string(),
            open: 185.0,
            high: 188.5,
            low: 184.2,
            close: 187.0,
            volume: 1_000_000,
        };

        insert_data(Models::Pricing(vec![row]), Some(&conn));

        let mut stmt = conn.prepare(
            "SELECT CAST(datetime AS VARCHAR), symbol, open, high, low, close, volume FROM pricing"
        ).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let r = rows.next().unwrap().unwrap();

        assert_eq!(r.get::<_, String>(0).unwrap(), "2024-01-15");
        assert_eq!(r.get::<_, String>(1).unwrap(), "AAPL");
        assert_eq!(r.get::<_, f64>(2).unwrap(), 185.0);
        assert_eq!(r.get::<_, f64>(3).unwrap(), 188.5);
        assert_eq!(r.get::<_, f64>(4).unwrap(), 184.2);
        assert_eq!(r.get::<_, f64>(5).unwrap(), 187.0);
        assert_eq!(r.get::<_, i64>(6).unwrap(), 1_000_000);
    }

    #[test]
    fn test_assets_insert_roundtrip() {
        let conn: Connection = setup_conn();

        let row: Assets = Assets {
            symbol: "SPY".to_string(),
            name: "SPDR S&P 500 ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        };
        insert_data(Models::Assets(vec![row]), Some(&conn)).unwrap();

        let mut stmt = conn
            .prepare("SELECT symbol, name, asset_class, sector, currency FROM assets")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let r = rows.next().unwrap().unwrap();

        assert_eq!(r.get::<_, String>(0).unwrap(), "SPY");
        assert_eq!(r.get::<_, String>(1).unwrap(), "SPDR S&P 500 ETF");
        assert_eq!(r.get::<_, String>(2).unwrap(), "ETF");
        assert_eq!(r.get::<_, String>(3).unwrap(), "Broad Market");
        assert_eq!(r.get::<_, String>(4).unwrap(), "USD");
    }

    #[test]
    fn test_pricing_duplicate_pk_fails() {
        let conn = setup_conn();
        let row = Pricing {
            datetime: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            symbol: "AAPL".to_string(),
            open: 185.0,
            high: 188.5,
            low: 184.2,
            close: 187.0,
            volume: 1_000_000,
        };

        insert_data(Models::Pricing(vec![row]), Some(&conn)).unwrap();

        let duplicate = Pricing {
            datetime: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            symbol: "AAPL".to_string(),
            open: 190.0,
            high: 192.0,
            low: 188.0,
            close: 191.0,
            volume: 500_000,
        };

        assert!(insert_data(Models::Pricing(vec![duplicate]), Some(&conn)).is_err());
    }

    #[test]
    fn test_assets_duplicate_pk_ignored() {
        let conn = setup_conn();
        let row = Assets {
            symbol: "SPY".to_string(),
            name: "SPDR S&P 500 ETF".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        };

        insert_data(Models::Assets(vec![row]), Some(&conn)).unwrap();

        let duplicate = Assets {
            symbol: "SPY".to_string(),
            name: "Different Name".to_string(),
            asset_class: "ETF".to_string(),
            sector: "Broad Market".to_string(),
            currency: "USD".to_string(),
        };

        insert_data(Models::Assets(vec![duplicate]), Some(&conn)).unwrap();

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM assets WHERE symbol = 'SPY'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let count: i64 = rows.next().unwrap().unwrap().get(0).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_macro_data_insert_roundtrip() {
        use crate::models::MacroData;
        let conn = setup_conn();
        let row = MacroData {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            series_id: "DGS10".to_string(),
            value: 4.15,
            frequency: "daily".to_string(),
        };

        insert_data(Models::MacroData(vec![row]), Some(&conn)).unwrap();

        let mut stmt = conn
            .prepare("SELECT CAST(date AS VARCHAR), series_id, value, frequency FROM macro_data")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let r = rows.next().unwrap().unwrap();

        assert_eq!(r.get::<_, String>(0).unwrap(), "2024-01-15");
        assert_eq!(r.get::<_, String>(1).unwrap(), "DGS10");
        assert_eq!(r.get::<_, f64>(2).unwrap(), 4.15);
        assert_eq!(r.get::<_, String>(3).unwrap(), "daily");
    }

    #[test]
    fn test_macro_data_duplicate_pk_ignored() {
        use crate::models::MacroData;
        let conn = setup_conn();
        let row = MacroData {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            series_id: "DGS10".to_string(),
            value: 4.15,
            frequency: "daily".to_string(),
        };

        insert_data(Models::MacroData(vec![row]), Some(&conn)).unwrap();

        let duplicate = MacroData {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            series_id: "DGS10".to_string(),
            value: 9.99,
            frequency: "daily".to_string(),
        };

        insert_data(Models::MacroData(vec![duplicate]), Some(&conn)).unwrap();

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM macro_data WHERE series_id = 'DGS10'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let count: i64 = rows.next().unwrap().unwrap().get(0).unwrap();
        assert_eq!(count, 1);
    }
}

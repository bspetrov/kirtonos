mod api;
mod backtest;
mod db;
mod metrics;
mod models;
mod report;
use dotenv;
mod routes;
use chrono::{Datelike, Duration, Local, NaiveDate};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    println!("Starting app...");

    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    match db::initialize_db() {
        Ok(_) => println!("Database initialized and running.."),
        Err(e) => println!("Error -> {e}"),
    }
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");

    match sync_all_data().await {
        Ok(_) => println!("All data synced successfully"),
        Err(e) => println!("Problem syncing data: {}", e),
    }
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - -");
    let app = routes::build_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Kirtonos REST listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

fn resolve_start_date(
    id: &str,
    table: db::DataTable,
    default: NaiveDate,
    today: NaiveDate,
) -> anyhow::Result<Option<NaiveDate>> {
    match db::get_latest_date(id, table)? {
        Some(date) => {
            let next = date + Duration::days(1);
            if next >= today {
                Ok(None)
            } else {
                Ok(Some(next))
            }
        }
        None => Ok(Some(default)),
    }
}

async fn sync_all_data() -> anyhow::Result<()> {
    let today = Local::now().date_naive();
    println!("- - - Syncing stocks - - -");
    for asset in db::get_assets()? {
        let Some(start_date) = resolve_start_date(
            &asset,
            db::DataTable::Pricing,
            NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(),
            today,
        )?
        else {
            println!("{} is up to date, skipping..", asset);
            continue;
        };
        println!("Fetching {} from {}..", asset, start_date);
        let results = api::stocks::fetch_daily_ohlcv(&asset, start_date).await?;
        db::insert_data(models::Models::Pricing(results), None)?;
    }

    println!("- - - Syncing crypto - - -");
    for symbol in api::crypto::SYMBOLS {
        let Some(start_date) = resolve_start_date(
            symbol,
            db::DataTable::Pricing,
            NaiveDate::from_ymd_opt(2017, 1, 1).unwrap(),
            today,
        )?
        else {
            println!("{} is up to date, skipping..", symbol);
            continue;
        };
        println!("Fetching {} from {}..", symbol, start_date);
        let results = api::crypto::fetch_crypto(symbol, start_date).await?;
        db::insert_data(models::Models::Pricing(results), None)?;
    }

    println!("- - - Syncing macro - - -");
    for (series_id, frequency) in api::economic::MACRO_SERIES {
        let start_date = match db::get_latest_date(series_id, db::DataTable::MacroData)? {
            Some(date) => {
                let is_stale = match *frequency {
                    "daily" => date + Duration::days(1) < today,
                    "monthly" => date.year() < today.year() || date.month() < today.month(),
                    "quarterly" => {
                        let last_q = (date.month() - 1) / 3;
                        let curr_q = (today.month() - 1) / 3;
                        date.year() < today.year() || last_q < curr_q
                    }
                    _ => date + Duration::days(1) < today,
                };
                if !is_stale {
                    println!("{} is up to date, skipping..", series_id);
                    continue;
                }
                date + Duration::days(1)
            }
            None => NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
        };
        println!(
            "Fetching {} ({}) from {}..",
            series_id, frequency, start_date
        );
        let results = api::economic::fetch_fred_data(series_id, start_date).await?;
        db::insert_data(models::Models::MacroData(results), None)?;
    }

    Ok(())
}

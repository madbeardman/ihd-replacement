use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

type AppError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Deserialize)]
pub struct OctopusConsumptionResponse {
    pub count: Option<u32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<OctopusConsumptionSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctopusConsumptionSlot {
    pub consumption: f64,
    pub interval_start: String,
    pub interval_end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConsumptionDay {
    pub date: String,
    pub fuel: String,
    pub slots: Vec<OctopusConsumptionSlot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YesterdayHistoryResponse {
    pub electricity: Option<StoredConsumptionDay>,
    pub gas: Option<StoredConsumptionDay>,
}

#[derive(Debug, Clone)]
pub struct OctopusConfig {
    pub api_key: String,
    pub electricity_mpan: String,
    pub electricity_serial: String,
    pub gas_mprn: String,
    pub gas_serial: String,
}

pub fn load_octopus_config() -> Result<OctopusConfig, Box<dyn std::error::Error>> {
    Ok(OctopusConfig {
        api_key: std::env::var("OCTOPUS_API_KEY")?,
        electricity_mpan: std::env::var("OCTOPUS_ELECTRICITY_MPAN")?,
        electricity_serial: std::env::var("OCTOPUS_ELECTRICITY_SERIAL")?,
        gas_mprn: std::env::var("OCTOPUS_GAS_MPRN")?,
        gas_serial: std::env::var("OCTOPUS_GAS_SERIAL")?,
    })
}

pub async fn fetch_electricity_usage_for_day(
    config: &OctopusConfig,
    day: NaiveDate,
) -> Result<StoredConsumptionDay, AppError> {
    let period_from = day.and_hms_opt(0, 0, 0).ok_or("Invalid start date")?;
    let period_to = (day + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .ok_or("Invalid end date")?;

    let url = format!(
        "https://api.octopus.energy/v1/electricity-meter-points/{}/meters/{}/consumption/?page_size=250&period_from={}&period_to={}&order_by=period",
        config.electricity_mpan,
        config.electricity_serial,
        period_from.format("%Y-%m-%dT%H:%M:%S"),
        period_to.format("%Y-%m-%dT%H:%M:%S"),
    );

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .basic_auth(&config.api_key, Some(""))
        .send()
        .await?
        .error_for_status()?
        .json::<OctopusConsumptionResponse>()
        .await?;

    Ok(StoredConsumptionDay {
        date: day.to_string(),
        fuel: "electricity".to_string(),
        slots: response.results,
    })
}

pub async fn fetch_gas_usage_for_day(
    config: &OctopusConfig,
    day: NaiveDate,
) -> Result<StoredConsumptionDay, AppError> {
    let period_from = day.and_hms_opt(0, 0, 0).ok_or("Invalid start date")?;
    let period_to = (day + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .ok_or("Invalid end date")?;

    let url = format!(
        "https://api.octopus.energy/v1/gas-meter-points/{}/meters/{}/consumption/?page_size=250&period_from={}&period_to={}&order_by=period",
        config.gas_mprn,
        config.gas_serial,
        period_from.format("%Y-%m-%dT%H:%M:%S"),
        period_to.format("%Y-%m-%dT%H:%M:%S"),
    );

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .basic_auth(&config.api_key, Some(""))
        .send()
        .await?
        .error_for_status()?
        .json::<OctopusConsumptionResponse>()
        .await?;

    Ok(StoredConsumptionDay {
        date: day.to_string(),
        fuel: "gas".to_string(),
        slots: response.results,
    })
}

pub fn save_consumption_day(
    base_dir: &Path,
    day: &StoredConsumptionDay,
) -> Result<PathBuf, AppError> {
    let fuel_dir = base_dir.join(&day.fuel);
    fs::create_dir_all(&fuel_dir)?;

    let path = fuel_dir.join(format!("{}.json", day.date));
    let json = serde_json::to_string_pretty(day)?;
    fs::write(&path, json)?;

    Ok(path)
}

pub fn load_consumption_day(
    base_dir: &Path,
    fuel: &str,
    day: NaiveDate,
) -> Result<Option<StoredConsumptionDay>, AppError> {
    let path = base_dir.join(fuel).join(format!("{}.json", day));

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)?;
    let data = serde_json::from_str::<StoredConsumptionDay>(&json)?;
    Ok(Some(data))
}

pub fn load_yesterday_history(history_dir: &Path) -> Result<YesterdayHistoryResponse, AppError> {
    let yesterday = Local::now().date_naive() - Duration::days(1);

    Ok(YesterdayHistoryResponse {
        electricity: load_consumption_day(history_dir, "electricity", yesterday)?,
        gas: load_consumption_day(history_dir, "gas", yesterday)?,
    })
}

pub async fn fetch_and_store_yesterday_history(
    history_dir: &Path,
    config: &OctopusConfig,
    dev_mode: bool,
) -> Result<(), AppError> {
    let yesterday = Local::now().date_naive() - Duration::days(1);

    let electricity = fetch_electricity_usage_for_day(config, yesterday).await?;
    let electricity_path = save_consumption_day(history_dir, &electricity)?;

    if dev_mode {
        println!(
            "Saved {} electricity slots to {}",
            electricity.slots.len(),
            electricity_path.display()
        );
    }

    let gas = fetch_gas_usage_for_day(config, yesterday).await?;
    let gas_path = save_consumption_day(history_dir, &gas)?;

    if dev_mode {
        println!(
            "Saved {} gas slots to {}",
            gas.slots.len(),
            gas_path.display()
        );
    }

    Ok(())
}

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::agile::{load_stored_day, stored_day_to_day_slots};

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
    #[serde(alias = "consumption")]
    pub consumption_kwh: f64,
    pub interval_start: String,
    pub interval_end: String,
    pub unit_rate_p_per_kwh: Option<f64>,
    pub cost_gbp: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConsumptionDay {
    pub date: String,
    pub fuel: String,
    pub total_consumption_kwh: Option<f64>,
    pub total_unit_cost_gbp: Option<f64>,
    pub standing_charge_gbp: Option<f64>,
    pub total_cost_gbp: Option<f64>,
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
    pub gas_unit_rate_p_per_kwh: Option<f64>,
    pub electricity_standing_charge_p_per_day: Option<f64>,
    pub gas_standing_charge_p_per_day: Option<f64>,
    pub gas_correction_factor: f64,
    pub gas_calorific_value: f64,
}

fn parse_timestamp_key(value: &str) -> Result<i64, AppError> {
    let dt = chrono::DateTime::parse_from_rfc3339(value)?;
    Ok(dt.timestamp())
}

pub fn load_octopus_config() -> Result<OctopusConfig, Box<dyn std::error::Error>> {
    Ok(OctopusConfig {
        api_key: std::env::var("OCTOPUS_API_KEY")?,
        electricity_mpan: std::env::var("OCTOPUS_ELECTRICITY_MPAN")?,
        electricity_serial: std::env::var("OCTOPUS_ELECTRICITY_SERIAL")?,
        gas_mprn: std::env::var("OCTOPUS_GAS_MPRN")?,
        gas_serial: std::env::var("OCTOPUS_GAS_SERIAL")?,
        gas_unit_rate_p_per_kwh: std::env::var("OCTOPUS_GAS_UNIT_RATE_P_PER_KWH")
            .ok()
            .and_then(|v| v.parse::<f64>().ok()),
        electricity_standing_charge_p_per_day: std::env::var(
            "OCTOPUS_ELECTRICITY_STANDING_CHARGE_P_PER_DAY",
        )
        .ok()
        .and_then(|v| v.parse::<f64>().ok()),
        gas_standing_charge_p_per_day: std::env::var("OCTOPUS_GAS_STANDING_CHARGE_P_PER_DAY")
            .ok()
            .and_then(|v| v.parse::<f64>().ok()),
        gas_correction_factor: std::env::var("OCTOPUS_GAS_CORRECTION_FACTOR")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.02264),

        gas_calorific_value: std::env::var("OCTOPUS_GAS_CALORIFIC_VALUE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(39.1),
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
        total_consumption_kwh: None,
        total_unit_cost_gbp: None,
        standing_charge_gbp: None,
        total_cost_gbp: None,
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
        total_consumption_kwh: None,
        total_unit_cost_gbp: None,
        standing_charge_gbp: None,
        total_cost_gbp: None,
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

fn trim_slots_to_day(mut day_data: StoredConsumptionDay, day: NaiveDate) -> StoredConsumptionDay {
    let day_prefix = day.to_string();

    day_data
        .slots
        .retain(|slot| slot.interval_start.starts_with(&day_prefix));

    day_data
}

fn build_electricity_rate_map(
    agile_dir: &Path,
    day: NaiveDate,
) -> Result<HashMap<i64, f64>, AppError> {
    let stored_day = load_stored_day(agile_dir, day)?
        .ok_or_else(|| format!("No Agile data found for {}", day))?;

    let day_slots = stored_day_to_day_slots(&stored_day);

    let rate_map = day_slots
        .into_iter()
        .map(|slot| (slot.valid_from.timestamp(), slot.value_inc_vat))
        .collect::<HashMap<_, _>>();

    Ok(rate_map)
}

fn enrich_electricity_day_with_costs(
    mut day_data: StoredConsumptionDay,
    agile_dir: &Path,
    day: NaiveDate,
) -> Result<StoredConsumptionDay, AppError> {
    let rate_map = build_electricity_rate_map(agile_dir, day)?;

    for slot in &mut day_data.slots {
        let slot_key = parse_timestamp_key(&slot.interval_start)?;

        if let Some(unit_rate_p_per_kwh) = rate_map.get(&slot_key) {
            slot.unit_rate_p_per_kwh = Some(*unit_rate_p_per_kwh);
            slot.cost_gbp = Some(slot.consumption_kwh * (*unit_rate_p_per_kwh / 100.0));
        } else {
            slot.unit_rate_p_per_kwh = None;
            slot.cost_gbp = None;
        }
    }

    Ok(day_data)
}

fn enrich_gas_day_with_costs(
    mut day_data: StoredConsumptionDay,
    config: &OctopusConfig,
) -> StoredConsumptionDay {
    for slot in &mut day_data.slots {
        let raw_m3 = slot.consumption_kwh; // <-- currently misnamed

        // Convert m³ → kWh
        let kwh = raw_m3 * config.gas_correction_factor * config.gas_calorific_value / 3.6;

        slot.consumption_kwh = kwh;

        if let Some(unit_rate_p_per_kwh) = config.gas_unit_rate_p_per_kwh {
            slot.unit_rate_p_per_kwh = Some(unit_rate_p_per_kwh);
            slot.cost_gbp = Some(kwh * (unit_rate_p_per_kwh / 100.0));
        } else {
            slot.unit_rate_p_per_kwh = None;
            slot.cost_gbp = None;
        }
    }

    day_data
}

fn finalise_day_totals(
    mut day_data: StoredConsumptionDay,
    standing_charge_gbp: Option<f64>,
) -> StoredConsumptionDay {
    let total_consumption_kwh = day_data
        .slots
        .iter()
        .map(|slot| slot.consumption_kwh)
        .sum::<f64>();

    let total_unit_cost_gbp = day_data
        .slots
        .iter()
        .filter_map(|slot| slot.cost_gbp)
        .sum::<f64>();

    let total_cost_gbp = Some(total_unit_cost_gbp + standing_charge_gbp.unwrap_or(0.0));

    day_data.total_consumption_kwh = Some(total_consumption_kwh);
    day_data.total_unit_cost_gbp = Some(total_unit_cost_gbp);
    day_data.standing_charge_gbp = standing_charge_gbp;
    day_data.total_cost_gbp = total_cost_gbp;

    day_data
}

pub async fn fetch_and_store_yesterday_history(
    history_dir: &Path,
    agile_dir: &Path,
    config: &OctopusConfig,
    dev_mode: bool,
) -> Result<(), AppError> {
    let yesterday = Local::now().date_naive() - Duration::days(1);

    let electricity_raw = fetch_electricity_usage_for_day(config, yesterday).await?;
    let electricity_trimmed = trim_slots_to_day(electricity_raw, yesterday);
    let electricity_enriched =
        enrich_electricity_day_with_costs(electricity_trimmed, agile_dir, yesterday)?;
    let electricity = finalise_day_totals(
        electricity_enriched,
        config
            .electricity_standing_charge_p_per_day
            .map(|p| p / 100.0),
    );
    let electricity_path = save_consumption_day(history_dir, &electricity)?;

    if dev_mode {
        println!(
            "Saved {} electricity slots to {} (total cost incl. standing charge: £{:.3})",
            electricity.slots.len(),
            electricity_path.display(),
            electricity.total_cost_gbp.unwrap_or(0.0),
        );
    }

    let gas_raw = fetch_gas_usage_for_day(config, yesterday).await?;
    let gas_trimmed = trim_slots_to_day(gas_raw, yesterday);
    let gas_enriched = enrich_gas_day_with_costs(gas_trimmed, config);
    let gas = finalise_day_totals(
        gas_enriched,
        config.gas_standing_charge_p_per_day.map(|p| p / 100.0),
    );
    let gas_path = save_consumption_day(history_dir, &gas)?;

    if dev_mode {
        println!(
            "Saved {} gas slots to {} (total cost incl. standing charge: £{:.3})",
            gas.slots.len(),
            gas_path.display(),
            gas.total_cost_gbp.unwrap_or(0.0),
        );
    }

    Ok(())
}

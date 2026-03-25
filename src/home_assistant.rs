use serde::Deserialize;

type AppError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct HaConfig {
    pub base_url: String,
    pub token: String,
    pub dev_mode: bool,
}

#[derive(Debug, Deserialize)]
pub struct HaState {
    pub entity_id: String,
    pub state: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LiveState {
    pub house_power_w: Option<f64>,
    pub solar_generation_w: Option<f64>,
    pub dishwasher_power_w: Option<f64>,
    pub washing_machine_power_w: Option<f64>,
    pub tumble_dryer_power_w: Option<f64>,
}

pub fn load_ha_config() -> Result<HaConfig, Box<dyn std::error::Error>> {
    let base_url = std::env::var("HA_BASE_URL")?;
    let token = std::env::var("HA_TOKEN")?;

    let dev_mode = std::env::var("DEV_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    Ok(HaConfig {
        base_url,
        token,
        dev_mode,
    })
}

pub async fn fetch_all_states(config: &HaConfig) -> Result<Vec<HaState>, AppError> {
    let url = format!("{}/api/states", config.base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .bearer_auth(&config.token)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<HaState>>()
        .await?;

    Ok(response)
}

pub fn get_numeric_state(states: &[HaState], entity_id: &str) -> Option<f64> {
    states
        .iter()
        .find(|state| state.entity_id == entity_id)
        .and_then(|state| state.state.parse::<f64>().ok())
}

pub fn extract_live_state(states: &[HaState]) -> LiveState {
    LiveState {
        house_power_w: get_numeric_state(states, "sensor.total_power_being_used"),
        solar_generation_w: get_numeric_state(states, "sensor.solar_panel_led_sensor_power"),
        dishwasher_power_w: get_numeric_state(states, "sensor.dishwasher_power"),
        washing_machine_power_w: get_numeric_state(states, "sensor.washing_machine_power"),
        tumble_dryer_power_w: get_numeric_state(states, "sensor.tumble_dryer_power"),
    }
}

pub fn is_appliance_running(power_w: Option<f64>) -> bool {
    power_w.unwrap_or(0.0) > 10.0
}

pub fn log_dev(config: &HaConfig, message: impl AsRef<str>) {
    if config.dev_mode {
        println!("{}", message.as_ref());
    }
}

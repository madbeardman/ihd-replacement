use serde::Deserialize;

type AppError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct HaConfig {
    pub base_url: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct HaStateResponse {
    state: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LiveState {
    pub house_power_w: Option<f64>,
    pub solar_generation_w: Option<f64>,
}

pub fn load_ha_config() -> Result<HaConfig, AppError> {
    let base_url = std::env::var("HA_BASE_URL")?;
    let token = std::env::var("HA_TOKEN")?;

    Ok(HaConfig { base_url, token })
}

pub async fn fetch_numeric_entity_state(
    config: &HaConfig,
    entity_id: &str,
) -> Result<Option<f64>, AppError> {
    let url = format!(
        "{}/api/states/{}",
        config.base_url.trim_end_matches('/'),
        entity_id
    );

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .bearer_auth(&config.token)
        .send()
        .await?
        .error_for_status()?
        .json::<HaStateResponse>()
        .await?;

    let value = response.state.parse::<f64>().ok();

    Ok(value)
}

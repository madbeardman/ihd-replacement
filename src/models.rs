use crate::agile::RollingWindow;
use crate::home_assistant::LiveState;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DashboardState {
    pub dev_mode: bool,
    pub live: LiveState,
    pub agile: RollingWindow,
    pub appliances: ApplianceRecommendations,
    pub usage_metrics: UsageRotationMetrics,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UsageRotationMetrics {
    pub current_power_w: Option<f64>,
    pub current_price_p_per_kwh: Option<f64>,
    pub current_cost_per_hour_gbp: Option<f64>,
    pub cost_today_gbp: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchMarker {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplianceRecommendation {
    pub name: String,
    pub power_w: Option<f64>,
    pub running: bool,
    pub best_start: Option<String>,
    pub display: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplianceRecommendations {
    pub dishwasher: ApplianceRecommendation,
    pub washing_machine: ApplianceRecommendation,
    pub tumble_dryer: ApplianceRecommendation,
}

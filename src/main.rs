mod agile;
mod home_assistant;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, extract::State, response::Html, routing::get};
use chrono::{Datelike, Local, Timelike};
use dotenvy::dotenv;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::agile::{
    RollingWindow, build_rolling_window, build_stored_days, fetch_latest_agile_rates,
    load_stored_day, save_stored_day, stored_day_to_day_slots,
};
use crate::home_assistant::log_dev;
use crate::home_assistant::{
    HaConfig, LiveState, extract_live_state, fetch_all_states, load_ha_config,
};

type AppError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, serde::Serialize)]
struct DashboardState {
    dev_mode: bool,
    live: LiveState,
    agile: RollingWindow,
    appliances: ApplianceRecommendations,
    usage_metrics: UsageRotationMetrics,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UsageRotationMetrics {
    current_power_w: Option<f64>,
    current_price_p_per_kwh: Option<f64>,
    current_cost_per_hour_gbp: Option<f64>,
    cost_today_gbp: Option<f64>,
}

#[derive(Clone)]
struct AppState {
    dashboard: Arc<RwLock<DashboardState>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FetchMarker {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ApplianceRecommendation {
    name: String,
    power_w: Option<f64>,
    running: bool,
    best_start: Option<String>,
    display: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ApplianceRecommendations {
    dishwasher: ApplianceRecommendation,
    washing_machine: ApplianceRecommendation,
    tumble_dryer: ApplianceRecommendation,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let agile_dir = PathBuf::from("data/agile");
    let ha_config = load_ha_config().expect("Failed to load Home Assistant config");

    fetch_and_store_latest_agile(&agile_dir, &ha_config)
        .await
        .expect("Failed to fetch/store Agile data at startup");

    let dashboard = load_dashboard_state(&agile_dir, &ha_config)
        .await
        .expect("Failed to load dashboard state");

    println!(
        "Loaded dashboard with {} Agile slots",
        dashboard.agile.slot_count
    );

    let state = AppState {
        dashboard: Arc::new(RwLock::new(dashboard)),
    };

    start_home_assistant_polling(state.clone(), ha_config.clone());
    start_scheduler(state.clone(), agile_dir.clone(), ha_config.clone());

    let app = Router::new()
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/agile", get(get_agile))
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind server");

    println!("----------------------------------------");
    println!("Frontend:      http://127.0.0.1:3000");
    println!("Dashboard API: http://127.0.0.1:3000/api/dashboard");
    println!("Agile API:     http://127.0.0.1:3000/api/agile");

    println!(
        "Logging:       {}",
        if ha_config.dev_mode {
            "ENABLED (DEV_MODE=true)"
        } else {
            "disabled (DEV_MODE=false)"
        }
    );
    println!("----------------------------------------");

    axum::serve(listener, app).await.expect("Server failed");
}

fn start_scheduler(state: AppState, agile_dir: PathBuf, ha_config: HaConfig) {
    tokio::spawn(async move {
        let mut last_run: Option<FetchMarker> = None;

        loop {
            let now = Local::now();
            let hour = now.hour();
            let minute = now.minute();

            let should_run_agile = (hour == 5 || hour == 17) && minute == 0;

            if should_run_agile {
                let marker = FetchMarker {
                    year: now.year(),
                    month: now.month(),
                    day: now.day(),
                    hour,
                };

                let already_ran = last_run.as_ref() == Some(&marker);

                if !already_ran {
                    println!(
                        "Scheduled Agile fetch triggered at {:04}-{:02}-{:02} {:02}:{:02}",
                        marker.year, marker.month, marker.day, hour, minute
                    );

                    match fetch_and_store_latest_agile(&agile_dir, &ha_config).await {
                        Ok(()) => {
                            println!("Scheduled Agile fetch/store completed");
                            last_run = Some(marker);
                        }
                        Err(err) => {
                            eprintln!("Scheduled Agile fetch failed: {err}");
                        }
                    }
                }
            }

            match load_dashboard_state(&agile_dir, &ha_config).await {
                Ok(updated_dashboard) => {
                    let mut guard = state.dashboard.write().await;
                    *guard = updated_dashboard;
                }
                Err(err) => {
                    eprintln!("Failed to refresh dashboard state: {err}");
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

async fn fetch_and_store_latest_agile(
    agile_dir: &Path,
    ha_config: &HaConfig,
) -> Result<(), AppError> {
    let api = fetch_latest_agile_rates().await?;

    if ha_config.dev_mode {
        println!("Fetched {} raw Agile slots", api.results.len());
    }

    let days = build_stored_days(&api);

    for day in &days {
        let path = save_stored_day(agile_dir, day)?;
        if ha_config.dev_mode {
            println!("Saved {} slots to {}", day.slots.len(), path.display());
        }
    }

    Ok(())
}

async fn load_dashboard_state(
    agile_dir: &Path,
    ha_config: &HaConfig,
) -> Result<DashboardState, AppError> {
    let agile = load_rolling_window_from_store(agile_dir)?;

    let live = match fetch_all_states(ha_config).await {
        Ok(states) => extract_live_state(&states),
        Err(err) => {
            eprintln!("Failed to fetch live HA state: {err}");
            LiveState {
                house_power_w: None,
                solar_generation_w: None,
                dishwasher_power_w: None,
                washing_machine_power_w: None,
                tumble_dryer_power_w: None,
            }
        }
    };

    let appliances = build_appliance_recommendations(&live, &agile);
    let usage_metrics = build_usage_rotation_metrics(&live, &agile);

    Ok(DashboardState {
        dev_mode: ha_config.dev_mode,
        live,
        agile,
        appliances,
        usage_metrics,
    })
}

fn load_rolling_window_from_store(agile_dir: &Path) -> Result<RollingWindow, AppError> {
    let today = Local::now().date_naive();
    let tomorrow = today.succ_opt().ok_or("Failed to calculate tomorrow")?;

    let today_day = load_stored_day(agile_dir, today)?;
    let tomorrow_day = load_stored_day(agile_dir, tomorrow)?;

    let today_slots = today_day
        .as_ref()
        .map(stored_day_to_day_slots)
        .unwrap_or_default();

    let tomorrow_slots = tomorrow_day
        .as_ref()
        .map(stored_day_to_day_slots)
        .unwrap_or_default();

    let now_local = Local::now();
    let current_slot_index = (now_local.hour() * 2 + now_local.minute() / 30) as u8;

    let rolling_slots = build_rolling_window(&today_slots, &tomorrow_slots, current_slot_index);

    Ok(RollingWindow {
        current_slot_index,
        slot_count: rolling_slots.len(),
        slots: rolling_slots,
    })
}

fn find_best_start_time(
    slots: &[crate::agile::RollingSlot],
    required_slots: usize,
) -> Option<String> {
    if slots.len() < required_slots || required_slots == 0 {
        return None;
    }

    let mut best_total = f64::MAX;
    let mut best_start = None;

    for window_start in 0..=(slots.len() - required_slots) {
        let window = &slots[window_start..window_start + required_slots];
        let total: f64 = window.iter().map(|slot| slot.value_inc_vat).sum();

        if total < best_total {
            best_total = total;
            best_start = window.first().map(|slot| slot.valid_from);
        }
    }

    best_start.map(|utc_time| utc_time.with_timezone(&Local).format("%H:%M").to_string())
}

fn build_appliance_recommendation(
    name: &str,
    power_w: Option<f64>,
    required_slots: usize,
    rolling_slots: &[crate::agile::RollingSlot],
) -> ApplianceRecommendation {
    let running = crate::home_assistant::is_appliance_running(power_w);
    let best_start = if running {
        None
    } else {
        find_best_start_time(rolling_slots, required_slots)
    };

    let display = if running {
        "ON".to_string()
    } else {
        best_start.clone().unwrap_or_else(|| "--".to_string())
    };

    ApplianceRecommendation {
        name: name.to_string(),
        power_w,
        running,
        best_start,
        display,
    }
}

fn build_appliance_recommendations(
    live: &LiveState,
    agile: &RollingWindow,
) -> ApplianceRecommendations {
    ApplianceRecommendations {
        dishwasher: build_appliance_recommendation(
            "Dishwasher",
            live.dishwasher_power_w,
            8,
            &agile.slots,
        ),
        washing_machine: build_appliance_recommendation(
            "Washing Machine",
            live.washing_machine_power_w,
            2,
            &agile.slots,
        ),
        tumble_dryer: build_appliance_recommendation(
            "Tumble Dryer",
            live.tumble_dryer_power_w,
            5,
            &agile.slots,
        ),
    }
}

fn start_home_assistant_polling(state: AppState, ha_config: HaConfig) {
    tokio::spawn(async move {
        loop {
            let now = chrono::Local::now().format("%H:%M:%S");

            let live = match fetch_all_states(&ha_config).await {
                Ok(states) => extract_live_state(&states),
                Err(err) => {
                    eprintln!("[{now}] HA fetch failed: {err}");
                    LiveState {
                        house_power_w: None,
                        solar_generation_w: None,
                        dishwasher_power_w: None,
                        washing_machine_power_w: None,
                        tumble_dryer_power_w: None,
                    }
                }
            };

            {
                let mut dashboard = state.dashboard.write().await;
                dashboard.live = live.clone();
                dashboard.appliances =
                    build_appliance_recommendations(&dashboard.live, &dashboard.agile);
                dashboard.usage_metrics =
                    build_usage_rotation_metrics(&dashboard.live, &dashboard.agile);
            }

            let house_text = live
                .house_power_w
                .map(|v| format!("{v:.2}W"))
                .unwrap_or_else(|| "unavailable".to_string());

            let solar_text = live
                .solar_generation_w
                .map(|v| format!("{v:.2}W"))
                .unwrap_or_else(|| "unavailable".to_string());

            let dishwasher_text = live
                .dishwasher_power_w
                .map(|v| format!("{v:.2}W"))
                .unwrap_or_else(|| "unavailable".to_string());

            let washer_text = live
                .washing_machine_power_w
                .map(|v| format!("{v:.2}W"))
                .unwrap_or_else(|| "unavailable".to_string());

            let dryer_text = live
                .tumble_dryer_power_w
                .map(|v| format!("{v:.2}W"))
                .unwrap_or_else(|| "unavailable".to_string());

            log_dev(
                &ha_config,
                format!(
                    "[{now}] HA poll | house: {house_text} | solar: {solar_text} | dishwasher: {dishwasher_text} | washer: {washer_text} | dryer: {dryer_text}"
                ),
            );

            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}

fn build_usage_rotation_metrics(live: &LiveState, agile: &RollingWindow) -> UsageRotationMetrics {
    let current_power_w = live.house_power_w;
    let current_price_p_per_kwh = agile.slots.first().map(|slot| slot.value_inc_vat);

    let current_cost_per_hour_gbp = match (current_power_w, current_price_p_per_kwh) {
        (Some(power_w), Some(price_p_per_kwh)) => {
            let pounds_per_kwh = price_p_per_kwh / 100.0;
            Some((power_w / 1000.0) * pounds_per_kwh)
        }
        _ => None,
    };

    UsageRotationMetrics {
        current_power_w,
        current_price_p_per_kwh,
        current_cost_per_hour_gbp,
        cost_today_gbp: None, // placeholder until wired in
    }
}

async fn get_dashboard(State(state): State<AppState>) -> Json<DashboardState> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.clone())
}

async fn get_agile(State(state): State<AppState>) -> Json<RollingWindow> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.agile.clone())
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let html = include_str!("../static/index.html");

    let dev_mode = {
        let dashboard = state.dashboard.read().await;
        if dashboard.dev_mode { "true" } else { "false" }
    };

    Html(html.replace(
        r#"data-dev-mode="false""#,
        &format!(r#"data-dev-mode="{}""#, dev_mode),
    ))
}

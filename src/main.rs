mod agile;
mod app_state;
mod dashboard;
mod handlers;
mod history;
mod home_assistant;
mod models;
mod scheduler;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::app_state::AppState;
use crate::dashboard::{fetch_and_store_latest_agile, load_dashboard_state};
use crate::handlers::{get_agile, get_dashboard, get_history_yesterday, index};
use crate::home_assistant::load_ha_config;
use crate::scheduler::{start_home_assistant_polling, start_scheduler};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let agile_dir = PathBuf::from("data/agile");
    let history_dir = PathBuf::from("data/history");

    let ha_config = load_ha_config().expect("Failed to load Home Assistant config");
    let octopus_config = history::load_octopus_config().expect("Failed to load Octopus config");

    fetch_and_store_latest_agile(&agile_dir, &ha_config)
        .await
        .expect("Failed to fetch/store Agile data at startup");

    history::fetch_and_store_yesterday_history(&history_dir, &octopus_config, ha_config.dev_mode)
        .await
        .expect("Failed to fetch/store yesterday history");

    let dashboard = load_dashboard_state(&agile_dir, &ha_config)
        .await
        .expect("Failed to load dashboard state");

    println!(
        "Loaded dashboard with {} Agile slots",
        dashboard.agile.slot_count
    );

    let state = AppState {
        dashboard: Arc::new(RwLock::new(dashboard)),
        history_dir: history_dir.clone(),
    };

    start_home_assistant_polling(state.clone(), ha_config.clone());
    start_scheduler(
        state.clone(),
        agile_dir.clone(),
        history_dir.clone(),
        ha_config.clone(),
        octopus_config.clone(),
    );

    let app = Router::new()
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/agile", get(get_agile))
        .route("/api/history/yesterday", get(get_history_yesterday))
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind server");

    println!("----------------------------------------");
    println!("Frontend:      http://0.0.0.0:3000");
    println!("Dashboard API: http://0.0.0.0:3000/api/dashboard");
    println!("Agile API:     http://0.0.0.0:3000/api/agile");
    println!("History API:   http://0.0.0.0:3000/api/history/yesterday");
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

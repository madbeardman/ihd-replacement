mod agile;
mod app_state;
mod dashboard;
mod handlers;
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
use crate::handlers::{get_agile, get_dashboard, index};
use crate::home_assistant::load_ha_config;
use crate::scheduler::{start_home_assistant_polling, start_scheduler};

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind server");

    println!("----------------------------------------");
    println!("Frontend:      http://0.0.0.0:3000");
    println!("Dashboard API: http://0.0.0.0:3000/api/dashboard");
    println!("Agile API:     http://0.0.0.0:3000/api/agile");
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

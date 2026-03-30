use axum::{Json, extract::State, response::Html};

use crate::agile::RollingWindow;
use crate::app_state::AppState;
use crate::models::DashboardState;

pub async fn get_dashboard(State(state): State<AppState>) -> Json<DashboardState> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.clone())
}

pub async fn get_agile(State(state): State<AppState>) -> Json<RollingWindow> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.agile.clone())
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
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

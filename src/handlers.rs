use axum::{Json, extract::State, response::Html};

use crate::agile::RollingWindow;
use crate::app_state::AppState;
use crate::history::{YesterdayHistoryResponse, load_yesterday_history};
use crate::models::DashboardState;

pub async fn get_dashboard(State(state): State<AppState>) -> Json<DashboardState> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.clone())
}

pub async fn get_agile(State(state): State<AppState>) -> Json<RollingWindow> {
    let dashboard = state.dashboard.read().await;
    Json(dashboard.agile.clone())
}

pub async fn get_history_yesterday(
    State(state): State<AppState>,
) -> Json<YesterdayHistoryResponse> {
    let history = load_yesterday_history(&state.history_dir).unwrap_or(YesterdayHistoryResponse {
        electricity: None,
        gas: None,
    });

    Json(history)
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

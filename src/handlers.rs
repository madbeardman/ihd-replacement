use axum::{
    extract::{Query, State},
    response::Html,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::agile::RollingWindow;
use crate::app_state::AppState;
use crate::history::{
    load_history_for_day, load_history_for_month, load_history_for_week, load_yesterday_history,
    MonthHistoryResponse, WeekHistoryResponse, YesterdayHistoryResponse,
};
use crate::models::DashboardState;

#[derive(Debug, Deserialize)]
pub struct HistoryDayQuery {
    pub date: String,
}

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

pub async fn get_history_day(
    State(state): State<AppState>,
    Query(query): Query<HistoryDayQuery>,
) -> Result<Json<YesterdayHistoryResponse>, (axum::http::StatusCode, String)> {
    let day = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid date format: {}. Expected YYYY-MM-DD", query.date),
        )
    })?;

    let history = load_history_for_day(&state.history_dir, day).map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load history for day: {err}"),
        )
    })?;

    Ok(Json(history))
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let html = include_str!("../static/index.html");

    let dev_mode = {
        let dashboard = state.dashboard.read().await;
        if dashboard.dev_mode {
            "true"
        } else {
            "false"
        }
    };

    Html(html.replace(
        r#"data-dev-mode="false""#,
        &format!(r#"data-dev-mode="{}""#, dev_mode),
    ))
}

pub async fn get_history_week(
    State(state): State<AppState>,
    Query(query): Query<HistoryDayQuery>,
) -> Result<Json<WeekHistoryResponse>, (axum::http::StatusCode, String)> {
    let day = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid date format: {}. Expected YYYY-MM-DD", query.date),
        )
    })?;

    let history = load_history_for_week(&state.history_dir, day).map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load history for week: {err}"),
        )
    })?;

    Ok(Json(history))
}

pub async fn get_history_month(
    State(state): State<AppState>,
    Query(query): Query<HistoryDayQuery>,
) -> Result<Json<MonthHistoryResponse>, (axum::http::StatusCode, String)> {
    let day = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid date format: {}. Expected YYYY-MM-DD", query.date),
        )
    })?;

    let history = load_history_for_month(&state.history_dir, day).map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load history for month: {err}"),
        )
    })?;

    Ok(Json(history))
}

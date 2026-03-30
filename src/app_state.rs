use std::sync::Arc;

use tokio::sync::RwLock;

use crate::models::DashboardState;

#[derive(Clone)]
pub struct AppState {
    pub dashboard: Arc<RwLock<DashboardState>>,
}

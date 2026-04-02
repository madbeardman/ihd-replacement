use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::models::DashboardState;

#[derive(Clone)]
pub struct AppState {
    pub dashboard: Arc<RwLock<DashboardState>>,
    pub history_dir: PathBuf,
}

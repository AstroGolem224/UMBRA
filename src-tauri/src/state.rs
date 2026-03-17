use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: PathBuf,
}

impl AppState {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }
}

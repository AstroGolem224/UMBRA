use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};

use crate::commands::config::AppConfig;
use crate::uap::AgentRegistry;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: PathBuf,
    pub github_cache: Arc<Mutex<Option<(serde_json::Value, Instant)>>>,
    pub agent_registry: AgentRegistry,
}

impl AppState {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            github_cache: Arc::new(Mutex::new(None)),
            agent_registry: AgentRegistry::new(),
        }
    }
}

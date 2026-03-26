use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};

use crate::commands::config::AppConfig;
use crate::errors::AppError;
use crate::uap::AgentRegistry;
use crate::workbench::store::WorkbenchStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: PathBuf,
    pub github_cache: Arc<Mutex<Option<(serde_json::Value, Instant)>>>,
    pub agent_registry: AgentRegistry,
    pub workbench_store: WorkbenchStore,
    pub active_run_registry: ActiveRunRegistry,
}

impl AppState {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Result<Self, AppError> {
        let base_dir = config_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let workbench_store = WorkbenchStore::new(base_dir.join("workbench.sqlite3"))?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            github_cache: Arc::new(Mutex::new(None)),
            agent_registry: AgentRegistry::new(),
            workbench_store,
            active_run_registry: ActiveRunRegistry::default(),
        })
    }
}

#[derive(Clone, Default)]
pub struct ActiveRunRegistry {
    pids: Arc<Mutex<HashMap<String, u32>>>,
}

impl ActiveRunRegistry {
    pub async fn register(&self, run_id: String, pid: u32) {
        self.pids.lock().await.insert(run_id, pid);
    }

    pub async fn remove(&self, run_id: &str) -> Option<u32> {
        self.pids.lock().await.remove(run_id)
    }
}

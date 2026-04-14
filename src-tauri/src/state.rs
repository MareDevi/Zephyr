use std::sync::RwLock;

use anyhow::anyhow;

use crate::error::AppResult;
use crate::ipc::dto::{DaemonHealthSnapshot, DashboardSnapshotDto};

pub struct AppState {
    dashboard: RwLock<DashboardSnapshotDto>,
}

impl AppState {
    pub fn new(initial_dashboard: DashboardSnapshotDto) -> Self {
        Self {
            dashboard: RwLock::new(initial_dashboard),
        }
    }

    pub fn get_health(&self) -> AppResult<DaemonHealthSnapshot> {
        let guard = self
            .dashboard
            .read()
            .map_err(|_| anyhow!("state lock poisoned while reading health"))?;
        Ok(guard.health.clone())
    }

    pub fn get_dashboard(&self) -> AppResult<DashboardSnapshotDto> {
        let guard = self
            .dashboard
            .read()
            .map_err(|_| anyhow!("state lock poisoned while reading dashboard"))?;
        Ok(guard.clone())
    }

    pub fn set_dashboard(&self, dashboard: DashboardSnapshotDto) -> AppResult<()> {
        let mut guard = self
            .dashboard
            .write()
            .map_err(|_| anyhow!("state lock poisoned while writing dashboard"))?;
        *guard = dashboard;
        Ok(())
    }
}

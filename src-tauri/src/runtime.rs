use std::future::Future;

use anyhow::{anyhow, Context};
use tauri::{AppHandle, Emitter, Manager, Wry};
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;

use crate::error::AppResult;
use crate::ipc::dto::DashboardSnapshotDto;
use crate::state::AppState;

pub struct BackendRuntime {
    runtime: Runtime,
    system_bus: OnceCell<zbus::Connection>,
}

impl BackendRuntime {
    pub fn new() -> AppResult<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()
            .context("tokio: failed to initialize runtime")?;
        Ok(Self {
            runtime,
            system_bus: OnceCell::const_new(),
        })
    }

    pub fn spawn_task<F>(&self, name: &'static str, task: F) -> tokio::task::JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(async move {
            tracing::debug!(task = name, "tokio task started");
            task.await;
            tracing::debug!(task = name, "tokio task finished");
        })
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.runtime.block_on(future)
    }

    pub async fn system_bus(&self) -> AppResult<zbus::Connection> {
        let connection = self
            .system_bus
            .get_or_try_init(|| async {
                zbus::Connection::system()
                    .await
                    .context("D-Bus: failed to connect to system bus (async)")
            })
            .await?;
        Ok(connection.clone())
    }
}

pub fn publish_dashboard_update(
    app: &AppHandle<Wry>,
    snapshot: &DashboardSnapshotDto,
) -> AppResult<()> {
    app.state::<AppState>()
        .set_dashboard(snapshot.clone())
        .context("runtime: failed to persist dashboard snapshot")?;
    app.emit(crate::ipc::events::DASHBOARD_UPDATED_EVENT, snapshot)
        .map_err(|error| anyhow!("runtime: failed to emit dashboard event: {error}"))?;
    crate::tray::sync_from_snapshot(app, snapshot)
}

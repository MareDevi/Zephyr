use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DashboardSnapshotDto, GpuStatusSnapshot};
use std::process::Command;
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
) -> GpuStatusSnapshot {
    if !daemon_available {
        if detect_nvidia() {
            return GpuStatusSnapshot {
                mode: Some("Nvidia".to_owned()),
                vendor: Some("NVIDIA".to_owned()),
                last_error: None,
                ..GpuStatusSnapshot::default()
            };
        }
        return GpuStatusSnapshot {
            last_error: Some("supergfxd unavailable".to_owned()),
            ..GpuStatusSnapshot::default()
        };
    }

    match dbus::supergfxd::get_status(connection) {
        Ok(status) => status,
        Err(error) => GpuStatusSnapshot {
            last_error: Some(error.to_string()),
            ..GpuStatusSnapshot::default()
        },
    }
}

fn detect_nvidia() -> bool {
    Command::new("nvidia-smi")
        .arg("-L")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub async fn set_mode_async(runtime: &crate::runtime::BackendRuntime, mode: &str) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::supergfxd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("supergfxd unavailable"));
    }
    dbus::supergfxd::set_mode_async(&connection, mode).await?;
    Ok(())
}

pub fn set_mode(app: &AppHandle<Wry>, mode: &str) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_mode_async(&runtime, mode))?;
    crate::services::dashboard::collect_dashboard()
}

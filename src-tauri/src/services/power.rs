use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DashboardSnapshotDto, PowerStatusSnapshot};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
) -> PowerStatusSnapshot {
    if !daemon_available {
        return PowerStatusSnapshot {
            last_error: Some("power-profiles-daemon unavailable".to_owned()),
            ..PowerStatusSnapshot::default()
        };
    }

    match dbus::ppd::get_status(connection) {
        Ok(status) => status,
        Err(error) => PowerStatusSnapshot {
            last_error: Some(error.to_string()),
            ..PowerStatusSnapshot::default()
        },
    }
}

pub async fn set_active_profile_async(
    runtime: &crate::runtime::BackendRuntime,
    profile: &str,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::ppd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("power-profiles-daemon unavailable"));
    }
    dbus::ppd::set_active_profile_async(&connection, profile).await?;
    Ok(())
}

pub fn set_active_profile(app: &AppHandle<Wry>, profile: &str) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_active_profile_async(&runtime, profile))?;
    crate::services::dashboard::collect_dashboard()
}

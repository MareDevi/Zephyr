#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{ArmouryStatusSnapshot, DashboardSnapshotDto};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> ArmouryStatusSnapshot {
    if !daemon_available {
        return ArmouryStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..ArmouryStatusSnapshot::default()
        };
    }
    if !interface_available {
        return ArmouryStatusSnapshot {
            last_error: Some("asusd armoury interface unavailable".to_owned()),
            ..ArmouryStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_armoury_status(connection) {
        Ok(status) => status,
        Err(error) => ArmouryStatusSnapshot {
            last_error: Some(error.to_string()),
            ..ArmouryStatusSnapshot::default()
        },
    }
}

pub fn set_current_value(path: &str, value: i32) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    dbus::asusd::set_armoury_current_value(&connection, path, value)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn set_current_value_with_app(
    app: &AppHandle<Wry>,
    path: &str,
    value: i32,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_current_value_async(&runtime, path, value))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_current_value_async(
    runtime: &crate::runtime::BackendRuntime,
    path: &str,
    value: i32,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    dbus::asusd::set_armoury_current_value_async(&connection, path, value).await?;
    Ok(())
}

fn ensure_available(connection: &zbus::blocking::Connection) -> AppResult<()> {
    if !dbus::asusd::probe(connection)? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::armoury_interface_available(connection)? {
        return Err(anyhow::anyhow!("asusd armoury interface unavailable"));
    }
    Ok(())
}

async fn ensure_available_async(runtime: &crate::runtime::BackendRuntime) -> AppResult<zbus::Connection> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::armoury_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd armoury interface unavailable"));
    }
    Ok(connection)
}

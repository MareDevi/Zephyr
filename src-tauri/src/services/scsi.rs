#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DashboardSnapshotDto, ScsiStatusSnapshot};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> ScsiStatusSnapshot {
    if !daemon_available {
        return ScsiStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..ScsiStatusSnapshot::default()
        };
    }
    if !interface_available {
        return ScsiStatusSnapshot {
            last_error: Some("asusd scsi interface unavailable".to_owned()),
            ..ScsiStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_scsi_status(connection) {
        Ok(status) => status,
        Err(error) => ScsiStatusSnapshot {
            last_error: Some(error.to_string()),
            ..ScsiStatusSnapshot::default()
        },
    }
}

pub fn set_enabled(enabled: bool) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    dbus::asusd::set_scsi_enabled(&connection, enabled)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn set_enabled_with_app(app: &AppHandle<Wry>, enabled: bool) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_enabled_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_enabled_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    dbus::asusd::set_scsi_enabled_async(&connection, enabled).await?;
    Ok(())
}

pub fn set_mode(mode: u8) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    dbus::asusd::set_scsi_mode(&connection, mode)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn set_mode_with_app(app: &AppHandle<Wry>, mode: u8) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_mode_async(&runtime, mode))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_mode_async(runtime: &crate::runtime::BackendRuntime, mode: u8) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    dbus::asusd::set_scsi_mode_async(&connection, mode).await?;
    Ok(())
}

fn ensure_available(connection: &zbus::blocking::Connection) -> AppResult<()> {
    if !dbus::asusd::probe(connection)? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::scsi_interface_available(connection)? {
        return Err(anyhow::anyhow!("asusd scsi interface unavailable"));
    }
    Ok(())
}

async fn ensure_available_async(runtime: &crate::runtime::BackendRuntime) -> AppResult<zbus::Connection> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::scsi_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd scsi interface unavailable"));
    }
    Ok(connection)
}

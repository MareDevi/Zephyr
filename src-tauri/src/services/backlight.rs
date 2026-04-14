#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{BacklightStatusSnapshot, DashboardSnapshotDto};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> BacklightStatusSnapshot {
    if !daemon_available {
        return BacklightStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..BacklightStatusSnapshot::default()
        };
    }
    if !interface_available {
        return BacklightStatusSnapshot {
            last_error: Some("asusd backlight interface unavailable".to_owned()),
            ..BacklightStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_backlight_status(connection) {
        Ok(status) => status,
        Err(error) => BacklightStatusSnapshot {
            last_error: Some(error.to_string()),
            ..BacklightStatusSnapshot::default()
        },
    }
}

pub fn set(
    screenpad_brightness: Option<i32>,
    screenpad_gamma: Option<f32>,
    sync_screenpad_brightness: Option<bool>,
) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    if let Some(value) = screenpad_brightness {
        dbus::asusd::set_backlight_screenpad_brightness(&connection, value)?;
    }
    if let Some(value) = screenpad_gamma {
        dbus::asusd::set_backlight_screenpad_gamma(&connection, value)?;
    }
    if let Some(value) = sync_screenpad_brightness {
        dbus::asusd::set_backlight_sync_screenpad_brightness(&connection, value)?;
    }
    crate::services::dashboard::collect_dashboard()
}

pub fn set_with_app(
    app: &AppHandle<Wry>,
    screenpad_brightness: Option<i32>,
    screenpad_gamma: Option<f32>,
    sync_screenpad_brightness: Option<bool>,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_async(
        &runtime,
        screenpad_brightness,
        screenpad_gamma,
        sync_screenpad_brightness,
    ))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_async(
    runtime: &crate::runtime::BackendRuntime,
    screenpad_brightness: Option<i32>,
    screenpad_gamma: Option<f32>,
    sync_screenpad_brightness: Option<bool>,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    if let Some(value) = screenpad_brightness {
        dbus::asusd::set_backlight_screenpad_brightness_async(&connection, value).await?;
    }
    if let Some(value) = screenpad_gamma {
        dbus::asusd::set_backlight_screenpad_gamma_async(&connection, value).await?;
    }
    if let Some(value) = sync_screenpad_brightness {
        dbus::asusd::set_backlight_sync_screenpad_brightness_async(&connection, value).await?;
    }
    Ok(())
}

fn ensure_available(connection: &zbus::blocking::Connection) -> AppResult<()> {
    if !dbus::asusd::probe(connection)? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::backlight_interface_available(connection)? {
        return Err(anyhow::anyhow!("asusd backlight interface unavailable"));
    }
    Ok(())
}

async fn ensure_available_async(runtime: &crate::runtime::BackendRuntime) -> AppResult<zbus::Connection> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::backlight_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd backlight interface unavailable"));
    }
    Ok(connection)
}

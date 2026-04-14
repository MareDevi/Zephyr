#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{AnimeStatusSnapshot, DashboardSnapshotDto};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> AnimeStatusSnapshot {
    if !daemon_available {
        return AnimeStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..AnimeStatusSnapshot::default()
        };
    }
    if !interface_available {
        return AnimeStatusSnapshot {
            last_error: Some("asusd anime interface unavailable".to_owned()),
            ..AnimeStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_anime_status(connection) {
        Ok(status) => status,
        Err(error) => AnimeStatusSnapshot {
            last_error: Some(error.to_string()),
            ..AnimeStatusSnapshot::default()
        },
    }
}

pub fn set_display_enabled(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_display_enabled_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_display_enabled_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_enable_display_async(&connection, enabled).await
}

pub fn set_builtins_enabled(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_builtins_enabled_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_builtins_enabled_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_builtins_enabled_async(&connection, enabled).await
}

pub fn set_brightness(app: &AppHandle<Wry>, level: u8) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_brightness_async(&runtime, level))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
    level: u8,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_brightness_async(&connection, level).await
}

pub fn set_off_when_lid_closed(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_off_when_lid_closed_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_off_when_lid_closed_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_off_when_lid_closed_async(&connection, enabled).await
}

pub fn set_off_when_suspended(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_off_when_suspended_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_off_when_suspended_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_off_when_suspended_async(&connection, enabled).await
}

pub fn set_off_when_unplugged(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_off_when_unplugged_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_off_when_unplugged_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_available_async(&connection).await?;
    dbus::asusd::set_anime_off_when_unplugged_async(&connection, enabled).await
}

async fn ensure_available_async(connection: &zbus::Connection) -> AppResult<()> {
    if !dbus::asusd::probe_async(connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::anime_interface_available_async(connection).await? {
        return Err(anyhow::anyhow!("asusd anime interface unavailable"));
    }
    Ok(())
}

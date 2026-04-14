#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{AuraStatusSnapshot, DashboardSnapshotDto, SlashStatusSnapshot};
use tauri::{AppHandle, Manager, Wry};

pub fn read_aura_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> AuraStatusSnapshot {
    if !daemon_available {
        return AuraStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..AuraStatusSnapshot::default()
        };
    }
    if !interface_available {
        return AuraStatusSnapshot {
            last_error: Some("asusd aura interface unavailable".to_owned()),
            ..AuraStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_aura_status(connection) {
        Ok(status) => status,
        Err(error) => AuraStatusSnapshot {
            last_error: Some(error.to_string()),
            ..AuraStatusSnapshot::default()
        },
    }
}

pub fn read_slash_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> SlashStatusSnapshot {
    if !daemon_available {
        return SlashStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..SlashStatusSnapshot::default()
        };
    }
    if !interface_available {
        return SlashStatusSnapshot {
            last_error: Some("asusd slash interface unavailable".to_owned()),
            ..SlashStatusSnapshot::default()
        };
    }
    match dbus::asusd::get_slash_status(connection) {
        Ok(status) => status,
        Err(error) => SlashStatusSnapshot {
            last_error: Some(error.to_string()),
            ..SlashStatusSnapshot::default()
        },
    }
}

pub fn set_aura_brightness(app: &AppHandle<Wry>, level: u8) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_aura_brightness_async(&runtime, level))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_aura_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
    level: u8,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_aura_available_async(&connection).await?;
    dbus::asusd::set_aura_brightness_async(&connection, level).await
}

pub fn set_aura_mode(app: &AppHandle<Wry>, mode: &str) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_aura_mode_async(&runtime, mode))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_aura_mode_async(
    runtime: &crate::runtime::BackendRuntime,
    mode: &str,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_aura_available_async(&connection).await?;
    dbus::asusd::set_aura_mode_async(&connection, mode).await
}

pub fn set_leds_brightness(app: &AppHandle<Wry>, level: &str) -> AppResult<DashboardSnapshotDto> {
    let code = parse_brightness_level(level)
        .ok_or_else(|| anyhow::anyhow!("unsupported leds brightness '{level}'"))?;
    set_aura_brightness(app, code)
}

pub async fn set_leds_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
    level: &str,
) -> AppResult<()> {
    let code = parse_brightness_level(level)
        .ok_or_else(|| anyhow::anyhow!("unsupported leds brightness '{level}'"))?;
    set_aura_brightness_async(runtime, code).await
}

pub fn next_leds_brightness(app: &AppHandle<Wry>) -> AppResult<DashboardSnapshotDto> {
    cycle_leds_brightness(app, true)
}

pub async fn next_leds_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
) -> AppResult<()> {
    cycle_leds_brightness_async(runtime, true).await
}

pub fn prev_leds_brightness(app: &AppHandle<Wry>) -> AppResult<DashboardSnapshotDto> {
    cycle_leds_brightness(app, false)
}

pub async fn prev_leds_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
) -> AppResult<()> {
    cycle_leds_brightness_async(runtime, false).await
}

pub fn set_slash_enabled(app: &AppHandle<Wry>, enabled: bool) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_enabled_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_enabled_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_enabled_async(&connection, enabled).await
}

pub fn set_slash_brightness(
    app: &AppHandle<Wry>,
    brightness: u8,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_brightness_async(&runtime, brightness))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
    brightness: u8,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_brightness_async(&connection, brightness).await
}

pub fn set_slash_interval(app: &AppHandle<Wry>, interval: u8) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_interval_async(&runtime, interval))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_interval_async(
    runtime: &crate::runtime::BackendRuntime,
    interval: u8,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_interval_async(&connection, interval).await
}

pub fn set_slash_mode(app: &AppHandle<Wry>, mode: &str) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_mode_async(&runtime, mode))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_mode_async(
    runtime: &crate::runtime::BackendRuntime,
    mode: &str,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_mode_async(&connection, mode).await
}

pub fn set_slash_show_on_boot(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_on_boot_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_on_boot_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_on_boot_async(&connection, enabled).await
}

pub fn set_slash_show_on_shutdown(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_on_shutdown_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_on_shutdown_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_on_shutdown_async(&connection, enabled).await
}

pub fn set_slash_show_on_sleep(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_on_sleep_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_on_sleep_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_on_sleep_async(&connection, enabled).await
}

pub fn set_slash_show_on_battery(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_on_battery_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_on_battery_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_on_battery_async(&connection, enabled).await
}

pub fn set_slash_show_battery_warning(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_battery_warning_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_battery_warning_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_battery_warning_async(&connection, enabled).await
}

pub fn set_slash_show_on_lid_closed(
    app: &AppHandle<Wry>,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_slash_show_on_lid_closed_async(&runtime, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_slash_show_on_lid_closed_async(
    runtime: &crate::runtime::BackendRuntime,
    enabled: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_slash_available_async(&connection).await?;
    dbus::asusd::set_slash_show_on_lid_closed_async(&connection, enabled).await
}

async fn ensure_aura_available_async(connection: &zbus::Connection) -> AppResult<()> {
    if !dbus::asusd::probe_async(connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::aura_interface_available_async(connection).await? {
        return Err(anyhow::anyhow!("asusd aura interface unavailable"));
    }
    Ok(())
}

async fn ensure_slash_available_async(connection: &zbus::Connection) -> AppResult<()> {
    if !dbus::asusd::probe_async(connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::slash_interface_available_async(connection).await? {
        return Err(anyhow::anyhow!("asusd slash interface unavailable"));
    }
    Ok(())
}

fn parse_brightness_level(level: &str) -> Option<u8> {
    let normalized = level.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "off" => Some(0),
        "low" => Some(1),
        "med" | "medium" => Some(2),
        "high" => Some(3),
        _ => normalized.parse::<u8>().ok().filter(|value| *value <= 3),
    }
}

fn cycle_leds_brightness(app: &AppHandle<Wry>, next: bool) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(cycle_leds_brightness_async(&runtime, next))?;
    crate::services::dashboard::collect_dashboard()
}

async fn cycle_leds_brightness_async(
    runtime: &crate::runtime::BackendRuntime,
    next: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    ensure_aura_available_async(&connection).await?;
    let current = dbus::asusd::get_aura_status_async(&connection)
        .await
        .ok()
        .and_then(|status| status.brightness)
        .and_then(|value| parse_brightness_level(&value))
        .unwrap_or(0);
    let target = if next {
        (current + 1) % 4
    } else {
        (current + 3) % 4
    };
    dbus::asusd::set_aura_brightness_async(&connection, target).await
}

#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DashboardSnapshotDto, PlatformStatusSnapshot};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
) -> PlatformStatusSnapshot {
    if !daemon_available {
        return PlatformStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..PlatformStatusSnapshot::default()
        };
    }
    if !interface_available {
        return PlatformStatusSnapshot {
            last_error: Some("asusd platform interface unavailable".to_owned()),
            ..PlatformStatusSnapshot::default()
        };
    }

    match dbus::asusd::get_platform_status(connection) {
        Ok(status) => status,
        Err(error) => PlatformStatusSnapshot {
            last_error: Some(error.to_string()),
            ..PlatformStatusSnapshot::default()
        },
    }
}

pub fn set_platform_profile(
    app: &AppHandle<Wry>,
    profile: &str,
    on_ac: bool,
    on_battery: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_platform_profile_async(
        &runtime, profile, on_ac, on_battery,
    ))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_platform_profile_async(
    runtime: &crate::runtime::BackendRuntime,
    profile: &str,
    on_ac: bool,
    on_battery: bool,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::platform_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd platform interface unavailable"));
    }
    // Keep behavior intuitive: "set profile" always updates current active profile.
    // Optional AC/Battery flags additionally persist per-power-source overrides.
    dbus::asusd::set_platform_profile_async(&connection, profile).await?;
    if on_ac {
        dbus::asusd::set_platform_profile_on_ac_async(&connection, profile).await?;
    }
    if on_battery {
        dbus::asusd::set_platform_profile_on_battery_async(&connection, profile).await?;
    }
    Ok(())
}

pub fn next_platform_profile(app: &AppHandle<Wry>) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(next_platform_profile_async(&runtime))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn next_platform_profile_async(
    runtime: &crate::runtime::BackendRuntime,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::platform_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd platform interface unavailable"));
    }
    dbus::asusd::next_platform_profile_async(&connection).await?;
    Ok(())
}

pub fn battery_one_shot_charge(
    app: &AppHandle<Wry>,
    percent: Option<u8>,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(battery_one_shot_charge_async(&runtime, percent))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn battery_one_shot_charge_async(
    runtime: &crate::runtime::BackendRuntime,
    percent: Option<u8>,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::platform_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd platform interface unavailable"));
    }
    if !dbus::asusd::supports_charge_control_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd charge limit control unsupported"));
    }
    if let Some(percent) = percent {
        if !(1..=100).contains(&percent) {
            return Err(anyhow::anyhow!("charge limit must be in 1..=100"));
        }
        dbus::asusd::set_charge_control_end_threshold_async(&connection, percent).await?;
    }
    dbus::asusd::one_shot_full_charge_async(&connection).await?;
    Ok(())
}

pub fn set_charge_limit(app: &AppHandle<Wry>, percent: u8) -> AppResult<DashboardSnapshotDto> {
    if !(1..=100).contains(&percent) {
        return Err(anyhow::anyhow!("charge limit must be in 1..=100"));
    }

    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_charge_limit_async(&runtime, percent))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_charge_limit_async(
    runtime: &crate::runtime::BackendRuntime,
    percent: u8,
) -> AppResult<()> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::platform_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd platform interface unavailable"));
    }
    if !dbus::asusd::supports_charge_control_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd charge limit control unsupported"));
    }
    dbus::asusd::set_charge_control_end_threshold_async(&connection, percent).await?;
    Ok(())
}

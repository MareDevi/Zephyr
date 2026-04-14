#![allow(dead_code)]
// Legacy sync wrappers are kept for non-IPC integration paths.

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DashboardSnapshotDto, FanCurvesStatusSnapshot, SetFanCurvePointRequest};
use tauri::{AppHandle, Manager, Wry};

pub fn read_snapshot(
    connection: &zbus::blocking::Connection,
    daemon_available: bool,
    interface_available: bool,
    active_profile: Option<&str>,
) -> FanCurvesStatusSnapshot {
    if !daemon_available {
        return FanCurvesStatusSnapshot {
            last_error: Some("asusd unavailable".to_owned()),
            ..FanCurvesStatusSnapshot::default()
        };
    }
    if !interface_available {
        return FanCurvesStatusSnapshot {
            last_error: Some("asusd fan curves interface unavailable".to_owned()),
            ..FanCurvesStatusSnapshot::default()
        };
    }

    let profile_code = parse_profile_code(active_profile.unwrap_or("balanced")).unwrap_or(0);
    match dbus::asusd::get_fan_curves_status(connection, profile_code) {
        Ok(status) => status,
        Err(error) => FanCurvesStatusSnapshot {
            last_error: Some(error.to_string()),
            ..FanCurvesStatusSnapshot::default()
        },
    }
}

pub fn read_profile(profile: &str) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    dbus::asusd::get_fan_curves_status(&connection, profile_code)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn reset_to_defaults(profile: &str) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    dbus::asusd::set_curves_to_defaults(&connection, profile_code)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn reset_to_defaults_with_app(
    app: &AppHandle<Wry>,
    profile: &str,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(reset_to_defaults_async(&runtime, profile))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn reset_to_defaults_async(
    runtime: &crate::runtime::BackendRuntime,
    profile: &str,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    dbus::asusd::set_curves_to_defaults_async(&connection, profile_code).await?;
    Ok(())
}

pub fn set_enabled(profile: &str, enabled: bool) -> AppResult<DashboardSnapshotDto> {
    let connection = zbus::blocking::Connection::system()
        .map_err(|error| anyhow::anyhow!("D-Bus: failed to connect to system bus: {error}"))?;
    ensure_available(&connection)?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    dbus::asusd::set_fan_curves_enabled(&connection, profile_code, enabled)?;
    crate::services::dashboard::collect_dashboard()
}

pub fn set_enabled_with_app(
    app: &AppHandle<Wry>,
    profile: &str,
    enabled: bool,
) -> AppResult<DashboardSnapshotDto> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    runtime.block_on(set_enabled_async(&runtime, profile, enabled))?;
    crate::services::dashboard::collect_dashboard()
}

pub async fn set_enabled_async(
    runtime: &crate::runtime::BackendRuntime,
    profile: &str,
    enabled: bool,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    dbus::asusd::set_fan_curves_enabled_async(&connection, profile_code, enabled).await?;
    Ok(())
}

pub async fn set_curve_async(
    runtime: &crate::runtime::BackendRuntime,
    profile: &str,
    fan: &str,
    points: &[SetFanCurvePointRequest],
    enabled: bool,
) -> AppResult<()> {
    let connection = ensure_available_async(runtime).await?;
    let profile_code = parse_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan profile '{profile}'"))?;
    let profile_name = normalize_profile_name(profile);
    let fan_name = normalize_fan_name(fan)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported fan '{fan}' (expected cpu/gpu/mid)"))?;
    let (temps, pwms) = validate_curve_points(points)?;
    dbus::asusd::set_fan_curve_async(
        &connection,
        profile_code,
        &profile_name,
        &fan_name,
        pwms,
        temps,
        enabled,
    )
    .await?;
    Ok(())
}

fn ensure_available(connection: &zbus::blocking::Connection) -> AppResult<()> {
    if !dbus::asusd::probe(connection)? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::fan_curves_interface_available(connection)? {
        return Err(anyhow::anyhow!("asusd fan curves interface unavailable"));
    }
    Ok(())
}

async fn ensure_available_async(runtime: &crate::runtime::BackendRuntime) -> AppResult<zbus::Connection> {
    let connection = runtime.system_bus().await?;
    if !dbus::asusd::probe_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd unavailable"));
    }
    if !dbus::asusd::fan_curves_interface_available_async(&connection).await? {
        return Err(anyhow::anyhow!("asusd fan curves interface unavailable"));
    }
    Ok(connection)
}

fn validate_curve_points(points: &[SetFanCurvePointRequest]) -> AppResult<([u8; 8], [u8; 8])> {
    if points.len() != 8 {
        return Err(anyhow::anyhow!(
            "asusd: fan curve requires exactly 8 points, got {}",
            points.len()
        ));
    }

    let mut temps = [0u8; 8];
    let mut pwms = [0u8; 8];
    for (index, point) in points.iter().enumerate() {
        if point.temperature > 100 {
            return Err(anyhow::anyhow!(
                "asusd: fan curve point {index} has invalid temperature {} (expected 0..=100)",
                point.temperature
            ));
        }
        if point.pwm > 100 {
            return Err(anyhow::anyhow!(
                "asusd: fan curve point {index} has invalid pwm {} (expected 0..=100)",
                point.pwm
            ));
        }
        temps[index] = point.temperature;
        pwms[index] = point.pwm;
    }

    for index in 1..8 {
        if temps[index] < temps[index - 1] {
            return Err(anyhow::anyhow!(
                "asusd: fan curve temperatures must be non-decreasing (index {index})"
            ));
        }
        if pwms[index] < pwms[index - 1] {
            return Err(anyhow::anyhow!(
                "asusd: fan curve pwm values must be non-decreasing (index {index})"
            ));
        }
    }

    Ok((temps, pwms))
}

fn normalize_profile_name(profile: &str) -> String {
    profile
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .replace(' ', "-")
}

fn normalize_fan_name(fan: &str) -> Option<String> {
    let normalized = fan
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");
    match normalized.as_str() {
        "cpu" => Some("CPU".to_owned()),
        "gpu" => Some("GPU".to_owned()),
        "mid" => Some("MID".to_owned()),
        _ => None,
    }
}

fn parse_profile_code(profile: &str) -> Option<u32> {
    let normalized = profile
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");
    match normalized.as_str() {
        "balanced" => Some(0),
        "performance" => Some(1),
        "quiet" => Some(2),
        "lowpower" => Some(3),
        "custom" => Some(4),
        _ => None,
    }
}

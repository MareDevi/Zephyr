#![allow(dead_code)]
// Keep sync D-Bus adapters for compatibility with non-IPC callers.

use anyhow::Context;

use crate::error::AppResult;
use crate::ipc::dto::GpuStatusSnapshot;

pub const SERVICE_NAME: &str = "org.supergfxctl.Daemon";
pub const INTERFACE: &str = "org.supergfxctl.Daemon";
pub const PATH: &str = "/org/supergfxctl/Gfx";

pub fn probe(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    let proxy = zbus::blocking::fdo::DBusProxy::new(connection)
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for supergfxd probe")?;
    let service_name = zbus::names::BusName::try_from(SERVICE_NAME)
        .context("D-Bus: invalid supergfxd service name")?;
    proxy
        .name_has_owner(service_name)
        .context("D-Bus: failed to query supergfxd service owner")
}

pub async fn probe_async(connection: &zbus::Connection) -> AppResult<bool> {
    let proxy = zbus::fdo::DBusProxy::new(connection)
        .await
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for supergfxd probe")?;
    let service_name = zbus::names::BusName::try_from(SERVICE_NAME)
        .context("D-Bus: invalid supergfxd service name")?;
    proxy
        .name_has_owner(service_name)
        .await
        .context("D-Bus: failed to query supergfxd service owner")
}

pub fn get_status(connection: &zbus::blocking::Connection) -> AppResult<GpuStatusSnapshot> {
    let proxy = zbus::blocking::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .context("D-Bus: failed to create supergfxd proxy")?;

    Ok(GpuStatusSnapshot {
        mode: call_method_u32(&proxy, "Mode", "mode").map(format_mode_code),
        power: call_method_u32(&proxy, "Power", "power").map(format_power_code),
        pending_mode: call_method_u32(&proxy, "PendingMode", "pending_mode").map(format_mode_code),
        pending_action: call_method_u32(&proxy, "PendingUserAction", "pending_user_action")
            .map(format_action_code),
        daemon_version: call_method_string(&proxy, "Version", "version"),
        vendor: call_method_string(&proxy, "Vendor", "vendor"),
        supported_modes: call_method_u32_array(&proxy, "Supported", "supported").map(format_modes),
        last_error: None,
    })
}

pub async fn get_status_async(connection: &zbus::Connection) -> AppResult<GpuStatusSnapshot> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .await
        .context("D-Bus: failed to create supergfxd proxy")?;

    Ok(GpuStatusSnapshot {
        mode: call_method_u32_async(&proxy, "Mode", "mode")
            .await
            .map(format_mode_code),
        power: call_method_u32_async(&proxy, "Power", "power")
            .await
            .map(format_power_code),
        pending_mode: call_method_u32_async(&proxy, "PendingMode", "pending_mode")
            .await
            .map(format_mode_code),
        pending_action: call_method_u32_async(&proxy, "PendingUserAction", "pending_user_action")
            .await
            .map(format_action_code),
        daemon_version: call_method_string_async(&proxy, "Version", "version").await,
        vendor: call_method_string_async(&proxy, "Vendor", "vendor").await,
        supported_modes: call_method_u32_array_async(&proxy, "Supported", "supported")
            .await
            .map(format_modes),
        last_error: None,
    })
}

pub fn get_mode(connection: &zbus::blocking::Connection) -> AppResult<Option<String>> {
    let proxy = zbus::blocking::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .context("D-Bus: failed to create supergfxd proxy for mode read")?;
    Ok(call_method_u32(&proxy, "Mode", "mode").map(format_mode_code))
}

pub async fn get_mode_async(connection: &zbus::Connection) -> AppResult<Option<String>> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .await
        .context("D-Bus: failed to create supergfxd proxy for mode read")?;
    Ok(call_method_u32_async(&proxy, "Mode", "mode")
        .await
        .map(format_mode_code))
}

pub fn set_mode(connection: &zbus::blocking::Connection, mode: &str) -> AppResult<String> {
    let proxy = zbus::blocking::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .context("D-Bus: failed to create supergfxd proxy for mode write")?;
    let mode_code = parse_mode_code(mode)
        .ok_or_else(|| anyhow::anyhow!("supergfxd: unsupported mode '{mode}'"))?;
    let message = proxy
        .call_method("SetMode", &(mode_code,))
        .or_else(|_| proxy.call_method("set_mode", &(mode_code,)))
        .context("D-Bus: failed to set supergfxd mode")?;
    let action = message
        .body()
        .deserialize::<zbus::zvariant::OwnedValue>()
        .map(|value| format_value(&value))
        .unwrap_or_else(|_| "unknown".to_owned());
    Ok(action)
}

pub async fn set_mode_async(connection: &zbus::Connection, mode: &str) -> AppResult<String> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PATH, INTERFACE)
        .await
        .context("D-Bus: failed to create supergfxd proxy for mode write")?;
    let mode_code = parse_mode_code(mode)
        .ok_or_else(|| anyhow::anyhow!("supergfxd: unsupported mode '{mode}'"))?;
    let message = match proxy.call_method("SetMode", &(mode_code,)).await {
        Ok(message) => message,
        Err(_) => proxy
            .call_method("set_mode", &(mode_code,))
            .await
            .context("D-Bus: failed to set supergfxd mode")?,
    };
    let action = message
        .body()
        .deserialize::<zbus::zvariant::OwnedValue>()
        .map(|value| format_value(&value))
        .unwrap_or_else(|_| "unknown".to_owned());
    Ok(action)
}

fn call_method_u32(proxy: &zbus::blocking::Proxy<'_>, method: &str, legacy: &str) -> Option<u32> {
    let message = proxy
        .call_method(method, &())
        .or_else(|_| proxy.call_method(legacy, &()))
        .ok()?;
    message.body().deserialize::<u32>().ok()
}

async fn call_method_u32_async(proxy: &zbus::Proxy<'_>, method: &str, legacy: &str) -> Option<u32> {
    let message = match proxy.call_method(method, &()).await {
        Ok(message) => message,
        Err(_) => proxy.call_method(legacy, &()).await.ok()?,
    };
    message.body().deserialize::<u32>().ok()
}

fn call_method_u32_array(
    proxy: &zbus::blocking::Proxy<'_>,
    method: &str,
    legacy: &str,
) -> Option<Vec<u32>> {
    let message = proxy
        .call_method(method, &())
        .or_else(|_| proxy.call_method(legacy, &()))
        .ok()?;
    message.body().deserialize::<Vec<u32>>().ok()
}

async fn call_method_u32_array_async(
    proxy: &zbus::Proxy<'_>,
    method: &str,
    legacy: &str,
) -> Option<Vec<u32>> {
    let message = match proxy.call_method(method, &()).await {
        Ok(message) => message,
        Err(_) => proxy.call_method(legacy, &()).await.ok()?,
    };
    message.body().deserialize::<Vec<u32>>().ok()
}

fn call_method_string(
    proxy: &zbus::blocking::Proxy<'_>,
    method: &str,
    legacy: &str,
) -> Option<String> {
    let message = proxy
        .call_method(method, &())
        .or_else(|_| proxy.call_method(legacy, &()))
        .ok()?;
    message.body().deserialize::<String>().ok()
}

async fn call_method_string_async(
    proxy: &zbus::Proxy<'_>,
    method: &str,
    legacy: &str,
) -> Option<String> {
    let message = match proxy.call_method(method, &()).await {
        Ok(message) => message,
        Err(_) => proxy.call_method(legacy, &()).await.ok()?,
    };
    message.body().deserialize::<String>().ok()
}

fn format_value(value: &zbus::zvariant::OwnedValue) -> String {
    let debug_text = format!("{value:?}");
    debug_text.trim_matches('"').to_owned()
}

fn parse_mode_code(mode: &str) -> Option<u32> {
    let normalized = mode
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");
    match normalized.as_str() {
        "hybrid" | "optimus" => Some(0),
        "integrated" | "igpu" => Some(1),
        "nvidianomodeset" | "nvidia" => Some(2),
        "vfio" => Some(3),
        "asusegpu" | "egpu" => Some(4),
        "asusmuxdgpu" | "ultimate" | "dgpu" => Some(5),
        "none" | "unknown" => Some(6),
        _ => None,
    }
}

fn format_mode_code(code: u32) -> String {
    match code {
        0 => "Hybrid".to_owned(),
        1 => "Integrated".to_owned(),
        2 => "NvidiaNoModeset".to_owned(),
        3 => "Vfio".to_owned(),
        4 => "AsusEgpu".to_owned(),
        5 => "AsusMuxDgpu".to_owned(),
        _ => "Unknown".to_owned(),
    }
}

fn format_power_code(code: u32) -> String {
    match code {
        0 => "active".to_owned(),
        1 => "suspended".to_owned(),
        2 => "off".to_owned(),
        3 => "dgpu_disabled".to_owned(),
        4 => "asus_mux_discreet".to_owned(),
        _ => "unknown".to_owned(),
    }
}

fn format_action_code(code: u32) -> String {
    match code {
        0 => "Logout".to_owned(),
        1 => "Reboot".to_owned(),
        2 => "SwitchToIntegrated".to_owned(),
        3 => "AsusEgpuDisable".to_owned(),
        _ => "Nothing".to_owned(),
    }
}

fn format_modes(modes: Vec<u32>) -> String {
    modes
        .into_iter()
        .map(format_mode_code)
        .collect::<Vec<_>>()
        .join(", ")
}

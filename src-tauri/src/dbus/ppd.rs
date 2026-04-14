#![allow(dead_code)]
// Keep sync D-Bus adapters for compatibility with non-IPC callers.

use anyhow::Context;
use zbus::names::InterfaceName;
use zbus::zvariant::OwnedValue;

use crate::error::AppResult;
use crate::ipc::dto::PowerStatusSnapshot;

pub const SERVICE_NAME: &str = "org.freedesktop.UPower.PowerProfiles";
pub const INTERFACE: &str = "org.freedesktop.UPower.PowerProfiles";
pub const PATH: &str = "/org/freedesktop/UPower/PowerProfiles";

pub fn probe(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    let proxy = zbus::blocking::fdo::DBusProxy::new(connection)
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for ppd probe")?;
    let service_name =
        zbus::names::BusName::try_from(SERVICE_NAME).context("D-Bus: invalid ppd service name")?;
    proxy
        .name_has_owner(service_name)
        .context("D-Bus: failed to query ppd service owner")
}

pub async fn probe_async(connection: &zbus::Connection) -> AppResult<bool> {
    let proxy = zbus::fdo::DBusProxy::new(connection)
        .await
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for ppd probe")?;
    let service_name =
        zbus::names::BusName::try_from(SERVICE_NAME).context("D-Bus: invalid ppd service name")?;
    proxy
        .name_has_owner(service_name)
        .await
        .context("D-Bus: failed to query ppd service owner")
}

pub fn get_status(connection: &zbus::blocking::Connection) -> AppResult<PowerStatusSnapshot> {
    let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid ppd destination for properties")?
        .path(PATH)
        .context("D-Bus: invalid ppd path")?
        .build()
        .context("D-Bus: failed to create ppd properties proxy")?;

    Ok(PowerStatusSnapshot {
        active_profile: get_property_value(&properties, "ActiveProfile")
            .and_then(|value| format_ppd_profile_value(&value)),
        profiles: get_property_value(&properties, "Profiles")
            .and_then(|value| format_ppd_profiles_value(&value)),
        performance_degraded: get_property_value(&properties, "PerformanceDegraded")
            .and_then(|value| format_ppd_optional_text(&value)),
        last_error: None,
    })
}

pub async fn get_status_async(connection: &zbus::Connection) -> AppResult<PowerStatusSnapshot> {
    let properties = zbus::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid ppd destination for properties")?
        .path(PATH)
        .context("D-Bus: invalid ppd path")?
        .build()
        .await
        .context("D-Bus: failed to create ppd properties proxy")?;

    Ok(PowerStatusSnapshot {
        active_profile: get_property_value_async(&properties, "ActiveProfile")
            .await
            .and_then(|value| format_ppd_profile_value(&value)),
        profiles: get_property_value_async(&properties, "Profiles")
            .await
            .and_then(|value| format_ppd_profiles_value(&value)),
        performance_degraded: get_property_value_async(&properties, "PerformanceDegraded")
            .await
            .and_then(|value| format_ppd_optional_text(&value)),
        last_error: None,
    })
}

pub fn set_active_profile(connection: &zbus::blocking::Connection, profile: &str) -> AppResult<()> {
    let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid ppd destination for properties write")?
        .path(PATH)
        .context("D-Bus: invalid ppd path for properties write")?
        .build()
        .context("D-Bus: failed to create ppd properties proxy for write")?;
    let interface =
        InterfaceName::try_from(INTERFACE).context("D-Bus: invalid ppd interface name")?;
    let normalized = normalize_profile(profile)
        .ok_or_else(|| anyhow::anyhow!("ppd: unsupported profile '{profile}'"))?;
    properties
        .set(
            interface,
            "ActiveProfile",
            zbus::zvariant::Value::from(normalized),
        )
        .context("D-Bus: failed to set ppd ActiveProfile")
}

pub async fn set_active_profile_async(
    connection: &zbus::Connection,
    profile: &str,
) -> AppResult<()> {
    let properties = zbus::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid ppd destination for properties write")?
        .path(PATH)
        .context("D-Bus: invalid ppd path for properties write")?
        .build()
        .await
        .context("D-Bus: failed to create ppd properties proxy for write")?;
    let interface =
        InterfaceName::try_from(INTERFACE).context("D-Bus: invalid ppd interface name")?;
    let normalized = normalize_profile(profile)
        .ok_or_else(|| anyhow::anyhow!("ppd: unsupported profile '{profile}'"))?;
    properties
        .set(
            interface,
            "ActiveProfile",
            zbus::zvariant::Value::from(normalized),
        )
        .await
        .context("D-Bus: failed to set ppd ActiveProfile")
}

fn get_property_value(
    properties: &zbus::blocking::fdo::PropertiesProxy<'_>,
    property_name: &str,
) -> Option<OwnedValue> {
    let interface = InterfaceName::try_from(INTERFACE).ok()?;
    properties.get(interface, property_name).ok()
}

async fn get_property_value_async(
    properties: &zbus::fdo::PropertiesProxy<'_>,
    property_name: &str,
) -> Option<OwnedValue> {
    let interface = InterfaceName::try_from(INTERFACE).ok()?;
    properties.get(interface, property_name).await.ok()
}

fn format_value(value: &OwnedValue) -> String {
    let debug_text = format!("{value:?}");
    debug_text.trim_matches('"').to_owned()
}

fn format_ppd_profile_value(value: &OwnedValue) -> Option<String> {
    if let Ok(profile) = String::try_from(value.clone()) {
        return Some(profile);
    }
    Some(format_value(value))
}

fn format_ppd_optional_text(value: &OwnedValue) -> Option<String> {
    let text = format_ppd_profile_value(value)?;
    let trimmed = text.trim().to_owned();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed)
}

fn format_ppd_profiles_value(value: &OwnedValue) -> Option<String> {
    if let Ok(list) = Vec::<String>::try_from(value.clone()) {
        if !list.is_empty() {
            return Some(list.join(", "));
        }
    }

    let debug = format_value(value);
    let known = ["power-saver", "balanced", "performance"];
    let mut found: Vec<&str> = Vec::new();
    for name in known {
        if debug.to_ascii_lowercase().contains(name) {
            found.push(name);
        }
    }
    if !found.is_empty() {
        return Some(found.join(", "));
    }

    Some(debug)
}

fn normalize_profile(profile: &str) -> Option<&'static str> {
    let normalized = profile.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    match normalized.as_str() {
        "power-saver" | "powersaver" => Some("power-saver"),
        "balanced" => Some("balanced"),
        "performance" => Some("performance"),
        _ => None,
    }
}

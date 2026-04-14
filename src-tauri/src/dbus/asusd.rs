#![allow(dead_code)]
// Keep sync D-Bus adapters for compatibility with non-IPC callers.

use anyhow::Context;
use zbus::names::InterfaceName;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value as ZValue;

use crate::error::AppResult;
use crate::ipc::dto::{
    AnimeStatusSnapshot, ArmouryAttributeSnapshot, ArmouryStatusSnapshot, AuraStatusSnapshot,
    BacklightStatusSnapshot, FanCurvePointSnapshot, FanCurveSeriesSnapshot,
    FanCurvesStatusSnapshot, PlatformStatusSnapshot, ScsiStatusSnapshot, SlashStatusSnapshot,
};

pub const SERVICE_NAME: &str = "xyz.ljones.Asusd";
pub const PLATFORM_INTERFACE: &str = "xyz.ljones.Platform";
pub const FAN_CURVES_INTERFACE: &str = "xyz.ljones.FanCurves";
pub const AURA_INTERFACE: &str = "xyz.ljones.Aura";
pub const ANIME_INTERFACE: &str = "xyz.ljones.Anime";
pub const SLASH_INTERFACE: &str = "xyz.ljones.Slash";
pub const SCSI_INTERFACE: &str = "xyz.ljones.ScsiAura";
pub const BACKLIGHT_INTERFACE: &str = "xyz.ljones.Backlight";
pub const ARMOURY_INTERFACE: &str = "xyz.ljones.AsusArmoury";
pub const ROOT_PATH: &str = "/";
pub const PLATFORM_PATH: &str = "/xyz/ljones";
pub const FAN_CURVES_PATH: &str = "/xyz/ljones";
pub const ARMOURY_PATH_PREFIX: &str = "/xyz/ljones/asus_armoury/";

pub fn probe(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    let proxy = zbus::blocking::fdo::DBusProxy::new(connection)
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for asusd probe")?;
    let service_name = zbus::names::BusName::try_from(SERVICE_NAME)
        .context("D-Bus: invalid asusd service name")?;
    proxy
        .name_has_owner(service_name)
        .context("D-Bus: failed to query asusd service owner")
}

pub async fn probe_async(connection: &zbus::Connection) -> AppResult<bool> {
    let proxy = zbus::fdo::DBusProxy::new(connection)
        .await
        .context("D-Bus: failed to create org.freedesktop.DBus proxy for asusd probe")?;
    let service_name = zbus::names::BusName::try_from(SERVICE_NAME)
        .context("D-Bus: invalid asusd service name")?;
    proxy
        .name_has_owner(service_name)
        .await
        .context("D-Bus: failed to query asusd service owner")
}

pub fn platform_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, PLATFORM_INTERFACE)
}

pub async fn platform_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, PLATFORM_INTERFACE).await
}

pub fn fan_curves_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, FAN_CURVES_INTERFACE)
}

pub async fn fan_curves_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, FAN_CURVES_INTERFACE).await
}

pub fn aura_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, AURA_INTERFACE)
}

pub async fn aura_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, AURA_INTERFACE).await
}

pub fn anime_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, ANIME_INTERFACE)
}

pub async fn anime_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, ANIME_INTERFACE).await
}

pub fn slash_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, SLASH_INTERFACE)
}

pub async fn slash_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, SLASH_INTERFACE).await
}

pub fn armoury_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    let paths = list_armoury_paths(connection)?;
    Ok(!paths.is_empty())
}

pub async fn armoury_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    let paths = list_armoury_paths_async(connection).await?;
    Ok(!paths.is_empty())
}

pub fn scsi_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, SCSI_INTERFACE)
}

pub async fn scsi_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, SCSI_INTERFACE).await
}

pub fn backlight_interface_available(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    interface_available(connection, BACKLIGHT_INTERFACE)
}

pub async fn backlight_interface_available_async(connection: &zbus::Connection) -> AppResult<bool> {
    interface_available_async(connection, BACKLIGHT_INTERFACE).await
}

pub fn get_platform_status(
    connection: &zbus::blocking::Connection,
) -> AppResult<PlatformStatusSnapshot> {
    let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for platform properties")?
        .path(PLATFORM_PATH)
        .context("D-Bus: invalid asusd platform path")?
        .build()
        .context("D-Bus: failed to create asusd platform properties proxy")?;

    Ok(PlatformStatusSnapshot {
        platform_profile: get_property_value(&properties, "PlatformProfile")
            .and_then(|value| format_platform_profile_value(&value)),
        platform_profile_choices: get_property_value(&properties, "PlatformProfileChoices")
            .and_then(|value| format_platform_profile_choices_value(&value)),
        platform_profile_on_ac: get_property_value(&properties, "PlatformProfileOnAc")
            .and_then(|value| format_platform_profile_value(&value)),
        platform_profile_on_battery: get_property_value(&properties, "PlatformProfileOnBattery")
            .and_then(|value| format_platform_profile_value(&value)),
        charge_control_end_threshold: get_property_value(&properties, "ChargeControlEndThreshold")
            .and_then(|value| u8::try_from(value).ok()),
        platform_profile_linked_epp: get_property_value(&properties, "PlatformProfileLinkedEpp")
            .and_then(|value| bool::try_from(value).ok()),
        last_error: None,
    })
}

pub fn set_platform_profile(
    connection: &zbus::blocking::Connection,
    profile: &str,
) -> AppResult<()> {
    let profile_code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    let proxy =
        zbus::blocking::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
            .context("D-Bus: failed to create asusd platform proxy for set profile")?;
    tracing::debug!(
        profile = %profile,
        profile_code = profile_code,
        "D-Bus call: asusd.SetPlatformProfile (u32)"
    );
    match proxy.call_method("SetPlatformProfile", &(profile_code,)) {
        Ok(_) => return Ok(()),
        Err(first_error) => {
            tracing::warn!(
                profile = %profile,
                profile_code = profile_code,
                error = %first_error,
                "D-Bus call asusd.SetPlatformProfile with u32 failed, retrying with i32"
            );
            let profile_code_i32 = i32::try_from(profile_code)
                .map_err(|_| anyhow::anyhow!("invalid profile code: {profile_code}"))?;
            if proxy
                .call_method("SetPlatformProfile", &(profile_code_i32,))
                .is_ok()
            {
                return Ok(());
            }
            // Compatibility fallback for older method naming.
            if proxy
                .call_method("set_platform_profile", &(profile_code,))
                .is_ok()
            {
                return Ok(());
            }
            tracing::warn!(
                profile = %profile,
                "D-Bus call asusd.SetPlatformProfile failed, falling back to PlatformProfile property write"
            );
        }
    }
    set_property_value(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfile",
        ZValue::from(profile_code),
    )
    .context("D-Bus: failed to set asusd PlatformProfile")
}

pub async fn set_platform_profile_async(
    connection: &zbus::Connection,
    profile: &str,
) -> AppResult<()> {
    let profile_code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd platform proxy for set profile")?;
    tracing::debug!(
        profile = %profile,
        profile_code = profile_code,
        "D-Bus call: asusd.SetPlatformProfile (u32)"
    );
    match proxy
        .call_method("SetPlatformProfile", &(profile_code,))
        .await
    {
        Ok(_) => return Ok(()),
        Err(first_error) => {
            tracing::warn!(
                profile = %profile,
                profile_code = profile_code,
                error = %first_error,
                "D-Bus call asusd.SetPlatformProfile with u32 failed, retrying with i32"
            );
            let profile_code_i32 = i32::try_from(profile_code)
                .map_err(|_| anyhow::anyhow!("invalid profile code: {profile_code}"))?;
            if proxy
                .call_method("SetPlatformProfile", &(profile_code_i32,))
                .await
                .is_ok()
            {
                return Ok(());
            }
            if proxy
                .call_method("set_platform_profile", &(profile_code,))
                .await
                .is_ok()
            {
                return Ok(());
            }
            tracing::warn!(
                profile = %profile,
                "D-Bus call asusd.SetPlatformProfile failed, falling back to PlatformProfile property write"
            );
        }
    }
    set_property_value_async(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfile",
        ZValue::from(profile_code),
    )
    .await
    .context("D-Bus: failed to set asusd PlatformProfile")
}

pub fn set_charge_control_end_threshold(
    connection: &zbus::blocking::Connection,
    threshold: u8,
) -> AppResult<()> {
    let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for charge limit write")?
        .path(PLATFORM_PATH)
        .context("D-Bus: invalid asusd platform path for charge limit write")?
        .build()
        .context("D-Bus: failed to create asusd properties proxy for charge limit write")?;
    let interface = InterfaceName::try_from(PLATFORM_INTERFACE)
        .context("D-Bus: invalid asusd platform interface name")?;
    properties
        .set(
            interface,
            "ChargeControlEndThreshold",
            zbus::zvariant::Value::from(threshold),
        )
        .context("D-Bus: failed to set asusd ChargeControlEndThreshold")
}

pub async fn set_charge_control_end_threshold_async(
    connection: &zbus::Connection,
    threshold: u8,
) -> AppResult<()> {
    let properties = zbus::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for charge limit write")?
        .path(PLATFORM_PATH)
        .context("D-Bus: invalid asusd platform path for charge limit write")?
        .build()
        .await
        .context("D-Bus: failed to create asusd properties proxy for charge limit write")?;
    let interface = InterfaceName::try_from(PLATFORM_INTERFACE)
        .context("D-Bus: invalid asusd platform interface name")?;
    properties
        .set(
            interface,
            "ChargeControlEndThreshold",
            zbus::zvariant::Value::from(threshold),
        )
        .await
        .context("D-Bus: failed to set asusd ChargeControlEndThreshold")
}

pub fn next_platform_profile(connection: &zbus::blocking::Connection) -> AppResult<()> {
    let proxy =
        zbus::blocking::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
            .context("D-Bus: failed to create asusd platform proxy for next profile")?;
    proxy
        .call_method("NextPlatformProfile", &())
        .or_else(|_| proxy.call_method("next_platform_profile", &()))
        .context("D-Bus: failed to call asusd NextPlatformProfile")?;
    Ok(())
}

pub async fn next_platform_profile_async(connection: &zbus::Connection) -> AppResult<()> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd platform proxy for next profile")?;
    match proxy.call_method("NextPlatformProfile", &()).await {
        Ok(_) => Ok(()),
        Err(_) => {
            proxy
                .call_method("next_platform_profile", &())
                .await
                .context("D-Bus: failed to call asusd NextPlatformProfile")?;
            Ok(())
        }
    }
}

pub fn set_platform_profile_on_ac(
    connection: &zbus::blocking::Connection,
    profile: &str,
) -> AppResult<()> {
    let code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    set_property_value(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfileOnAc",
        ZValue::from(code),
    )
    .context("D-Bus: failed to set asusd PlatformProfileOnAc")
}

pub async fn set_platform_profile_on_ac_async(
    connection: &zbus::Connection,
    profile: &str,
) -> AppResult<()> {
    let code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    set_property_value_async(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfileOnAc",
        ZValue::from(code),
    )
    .await
    .context("D-Bus: failed to set asusd PlatformProfileOnAc")
}

pub fn set_platform_profile_on_battery(
    connection: &zbus::blocking::Connection,
    profile: &str,
) -> AppResult<()> {
    let code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    set_property_value(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfileOnBattery",
        ZValue::from(code),
    )
    .context("D-Bus: failed to set asusd PlatformProfileOnBattery")
}

pub async fn set_platform_profile_on_battery_async(
    connection: &zbus::Connection,
    profile: &str,
) -> AppResult<()> {
    let code = parse_platform_profile_code(profile)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported platform profile '{profile}'"))?;
    set_property_value_async(
        connection,
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "PlatformProfileOnBattery",
        ZValue::from(code),
    )
    .await
    .context("D-Bus: failed to set asusd PlatformProfileOnBattery")
}

pub fn supports_charge_control(connection: &zbus::blocking::Connection) -> AppResult<bool> {
    let proxy =
        zbus::blocking::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
            .context("D-Bus: failed to create asusd platform proxy for capability check")?;
    tracing::debug!("D-Bus call: asusd.SupportedProperties");
    match proxy
        .call_method("SupportedProperties", &())
        .or_else(|_| proxy.call_method("supported_properties", &()))
    {
        Ok(message) => {
            let summary = message
                .body()
                .deserialize::<OwnedValue>()
                .map(|value| format_value(&value))
                .context("D-Bus: failed to decode asusd supported_properties")?;
            tracing::debug!(payload = %summary, "D-Bus result: asusd.SupportedProperties");
            Ok(summary.contains("ChargeControlEndThreshold"))
        }
        Err(error) => {
            tracing::warn!(
                error = %error,
                "D-Bus query asusd.supported_properties failed, falling back to ChargeControlEndThreshold property probe"
            );
            let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
                .destination(SERVICE_NAME)
                .context("D-Bus: invalid asusd destination for charge support fallback")?
                .path(PLATFORM_PATH)
                .context("D-Bus: invalid asusd platform path for charge support fallback")?
                .build()
                .context(
                    "D-Bus: failed to create asusd properties proxy for charge support fallback",
                )?;
            Ok(get_property_value_with_interface(
                &properties,
                PLATFORM_INTERFACE,
                "ChargeControlEndThreshold",
            )
            .is_some())
        }
    }
}

pub async fn supports_charge_control_async(connection: &zbus::Connection) -> AppResult<bool> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd platform proxy for capability check")?;
    tracing::debug!("D-Bus call: asusd.SupportedProperties");
    let supported = match proxy.call_method("SupportedProperties", &()).await {
        Ok(message) => Ok(message),
        Err(_) => proxy.call_method("supported_properties", &()).await,
    };
    match supported {
        Ok(message) => {
            let summary = message
                .body()
                .deserialize::<OwnedValue>()
                .map(|value| format_value(&value))
                .context("D-Bus: failed to decode asusd supported_properties")?;
            tracing::debug!(payload = %summary, "D-Bus result: asusd.SupportedProperties");
            Ok(summary.contains("ChargeControlEndThreshold"))
        }
        Err(error) => {
            tracing::warn!(
                error = %error,
                "D-Bus query asusd.supported_properties failed, falling back to ChargeControlEndThreshold property probe"
            );
            let properties = zbus::fdo::PropertiesProxy::builder(connection)
                .destination(SERVICE_NAME)
                .context("D-Bus: invalid asusd destination for charge support fallback")?
                .path(PLATFORM_PATH)
                .context("D-Bus: invalid asusd platform path for charge support fallback")?
                .build()
                .await
                .context(
                    "D-Bus: failed to create asusd properties proxy for charge support fallback",
                )?;
            Ok(get_property_value_with_interface_async(
                &properties,
                PLATFORM_INTERFACE,
                "ChargeControlEndThreshold",
            )
            .await
            .is_some())
        }
    }
}

pub fn one_shot_full_charge(connection: &zbus::blocking::Connection) -> AppResult<()> {
    let proxy =
        zbus::blocking::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
            .context("D-Bus: failed to create asusd platform proxy for one-shot charge")?;
    proxy
        .call_method("OneShotFullCharge", &())
        .or_else(|_| proxy.call_method("one_shot_full_charge", &()))
        .context("D-Bus: failed to call asusd OneShotFullCharge")?;
    Ok(())
}

pub async fn one_shot_full_charge_async(connection: &zbus::Connection) -> AppResult<()> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, PLATFORM_PATH, PLATFORM_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd platform proxy for one-shot charge")?;
    match proxy.call_method("OneShotFullCharge", &()).await {
        Ok(_) => Ok(()),
        Err(_) => {
            proxy
                .call_method("one_shot_full_charge", &())
                .await
                .context("D-Bus: failed to call asusd OneShotFullCharge")?;
            Ok(())
        }
    }
}

pub fn get_fan_curves_status(
    connection: &zbus::blocking::Connection,
    profile_code: u32,
) -> AppResult<FanCurvesStatusSnapshot> {
    let proxy = zbus::blocking::Proxy::new(
        connection,
        SERVICE_NAME,
        FAN_CURVES_PATH,
        FAN_CURVES_INTERFACE,
    )
    .context("D-Bus: failed to create asusd fan curves proxy")?;

    let curve_message = call_fan_curve_data(&proxy, profile_code)
        .context("D-Bus: failed to call asusd fan_curve_data")?;
    let decoded_curve = decode_fan_curve_message(&curve_message)
        .context("D-Bus: failed to decode asusd fan_curve_data payload")?;

    let platform_properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for platform profile read")?
        .path(PLATFORM_PATH)
        .context("D-Bus: invalid asusd platform path for profile read")?
        .build()
        .context("D-Bus: failed to create asusd platform properties proxy")?;

    Ok(FanCurvesStatusSnapshot {
        active_profile: get_property_value(&platform_properties, "PlatformProfile")
            .and_then(|value| format_platform_profile_value(&value)),
        profile_choices: get_property_value(&platform_properties, "PlatformProfileChoices")
            .and_then(|value| format_platform_profile_choices_value(&value)),
        curve_data: Some(decoded_curve.raw),
        curve_series: decoded_curve.series,
        last_error: None,
    })
}

pub fn set_fan_curves_enabled(
    connection: &zbus::blocking::Connection,
    profile_code: u32,
    enabled: bool,
) -> AppResult<()> {
    let proxy = zbus::blocking::Proxy::new(
        connection,
        SERVICE_NAME,
        FAN_CURVES_PATH,
        FAN_CURVES_INTERFACE,
    )
    .context("D-Bus: failed to create asusd fan curves proxy for write")?;
    proxy
        .call_method("SetFanCurvesEnabled", &(profile_code, enabled))
        .or_else(|_| proxy.call_method("set_fan_curves_enabled", &(profile_code, enabled)))
        .context("D-Bus: failed to call asusd SetFanCurvesEnabled")?;
    Ok(())
}

pub async fn set_fan_curves_enabled_async(
    connection: &zbus::Connection,
    profile_code: u32,
    enabled: bool,
) -> AppResult<()> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, FAN_CURVES_PATH, FAN_CURVES_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd fan curves proxy for write")?;
    match proxy
        .call_method("SetFanCurvesEnabled", &(profile_code, enabled))
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => {
            proxy
                .call_method("set_fan_curves_enabled", &(profile_code, enabled))
                .await
                .context("D-Bus: failed to call asusd SetFanCurvesEnabled")?;
            Ok(())
        }
    }
}

pub async fn set_fan_curve_async(
    connection: &zbus::Connection,
    profile_code: u32,
    profile_name: &str,
    fan: &str,
    pwm: [u8; 8],
    temp: [u8; 8],
    enabled: bool,
) -> AppResult<()> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, FAN_CURVES_PATH, FAN_CURVES_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd fan curves proxy for set fan curve")?;

    let curve = fan_curve_write_tuple(fan, pwm, temp, enabled);
    if proxy
        .call_method("SetFanCurve", &(profile_code, curve.clone()))
        .await
        .is_ok()
    {
        return Ok(());
    }
    if proxy
        .call_method("set_fan_curve", &(profile_code, curve.clone()))
        .await
        .is_ok()
    {
        return Ok(());
    }

    let profile_code_i32 = i32::try_from(profile_code)
        .map_err(|_| anyhow::anyhow!("invalid profile code: {profile_code}"))?;
    if proxy
        .call_method("SetFanCurve", &(profile_code_i32, curve.clone()))
        .await
        .is_ok()
    {
        return Ok(());
    }
    if proxy
        .call_method("set_fan_curve", &(profile_code_i32, curve.clone()))
        .await
        .is_ok()
    {
        return Ok(());
    }

    match proxy
        .call_method("SetFanCurve", &(profile_name, curve.clone()))
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => {
            proxy
                .call_method("set_fan_curve", &(profile_name, curve))
                .await
                .context("D-Bus: failed to call asusd SetFanCurve")
                .map(|_| ())
        }
    }?;
    Ok(())
}

pub fn set_curves_to_defaults(
    connection: &zbus::blocking::Connection,
    profile_code: u32,
) -> AppResult<()> {
    let proxy = zbus::blocking::Proxy::new(
        connection,
        SERVICE_NAME,
        FAN_CURVES_PATH,
        FAN_CURVES_INTERFACE,
    )
    .context("D-Bus: failed to create asusd fan curves proxy for reset")?;
    proxy
        .call_method("SetCurvesToDefaults", &(profile_code,))
        .or_else(|_| proxy.call_method("set_curves_to_defaults", &(profile_code,)))
        .context("D-Bus: failed to call asusd SetCurvesToDefaults")?;
    Ok(())
}

pub async fn set_curves_to_defaults_async(
    connection: &zbus::Connection,
    profile_code: u32,
) -> AppResult<()> {
    let proxy = zbus::Proxy::new(connection, SERVICE_NAME, FAN_CURVES_PATH, FAN_CURVES_INTERFACE)
        .await
        .context("D-Bus: failed to create asusd fan curves proxy for reset")?;
    match proxy.call_method("SetCurvesToDefaults", &(profile_code,)).await {
        Ok(_) => Ok(()),
        Err(_) => {
            proxy
                .call_method("set_curves_to_defaults", &(profile_code,))
                .await
                .context("D-Bus: failed to call asusd SetCurvesToDefaults")?;
            Ok(())
        }
    }
}

pub fn get_aura_status(connection: &zbus::blocking::Connection) -> AppResult<AuraStatusSnapshot> {
    let path = find_first_interface_path(connection, AURA_INTERFACE)?;
    let properties = properties_proxy(connection, &path, AURA_INTERFACE, "aura read")?;
    Ok(AuraStatusSnapshot {
        brightness: get_property_summary_with_interface(&properties, AURA_INTERFACE, "Brightness"),
        led_mode: get_property_summary_with_interface(&properties, AURA_INTERFACE, "LedMode"),
        supported_brightness: get_property_summary_with_interface(
            &properties,
            AURA_INTERFACE,
            "SupportedBrightness",
        ),
        supported_basic_modes: get_property_summary_with_interface(
            &properties,
            AURA_INTERFACE,
            "SupportedBasicModes",
        ),
        last_error: None,
    })
}

pub async fn get_aura_status_async(connection: &zbus::Connection) -> AppResult<AuraStatusSnapshot> {
    let path = find_first_interface_path_async(connection, AURA_INTERFACE).await?;
    let properties = zbus::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for aura read")?
        .path(path.as_str())
        .with_context(|| format!("D-Bus: invalid asusd path '{path}' for aura read"))?
        .interface(AURA_INTERFACE)
        .with_context(|| format!("D-Bus: invalid asusd interface '{AURA_INTERFACE}' for aura read"))?
        .build()
        .await
        .context("D-Bus: failed to create asusd properties proxy for aura read")?;
    let brightness = get_property_value_with_interface_async(&properties, AURA_INTERFACE, "Brightness")
        .await
        .map(|value| format_value(&value));
    let led_mode = get_property_value_with_interface_async(&properties, AURA_INTERFACE, "LedMode")
        .await
        .map(|value| format_value(&value));
    let supported_brightness =
        get_property_value_with_interface_async(&properties, AURA_INTERFACE, "SupportedBrightness")
            .await
            .map(|value| format_value(&value));
    let supported_basic_modes =
        get_property_value_with_interface_async(&properties, AURA_INTERFACE, "SupportedBasicModes")
            .await
            .map(|value| format_value(&value));
    Ok(AuraStatusSnapshot {
        brightness,
        led_mode,
        supported_brightness,
        supported_basic_modes,
        last_error: None,
    })
}

pub fn set_aura_brightness(connection: &zbus::blocking::Connection, level: u8) -> AppResult<()> {
    let path = find_first_interface_path(connection, AURA_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        AURA_INTERFACE,
        "Brightness",
        ZValue::from(level),
    )
    .context("D-Bus: failed to set asusd aura Brightness")
}

pub async fn set_aura_brightness_async(connection: &zbus::Connection, level: u8) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, AURA_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        AURA_INTERFACE,
        "Brightness",
        ZValue::from(level),
    )
    .await
    .context("D-Bus: failed to set asusd aura Brightness")
}

pub fn set_aura_mode(connection: &zbus::blocking::Connection, mode: &str) -> AppResult<()> {
    let path = find_first_interface_path(connection, AURA_INTERFACE)?;
    let mode_codes = parse_aura_mode_codes(mode)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported aura mode '{mode}'"))?;
    for mode_code in mode_codes {
        tracing::debug!(mode = %mode, mode_code = mode_code, path = %path, "D-Bus write: asusd aura LedMode");
        if set_property_value(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(mode_code),
        )
        .is_ok()
        {
            return Ok(());
        }
        let mode_i32 = i32::from(mode_code);
        if set_property_value(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(mode_i32),
        )
        .is_ok()
        {
            return Ok(());
        }
        if set_property_value(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(u32::from(mode_code)),
        )
        .is_ok()
        {
            return Ok(());
        }
        tracing::warn!(
            mode = %mode,
            mode_code = mode_code,
            "D-Bus write asusd aura LedMode failed for this candidate code"
        );
    }
    Err(anyhow::anyhow!("D-Bus: failed to set asusd aura LedMode"))
}

pub async fn set_aura_mode_async(connection: &zbus::Connection, mode: &str) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, AURA_INTERFACE).await?;
    let mode_codes = parse_aura_mode_codes(mode)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported aura mode '{mode}'"))?;
    for mode_code in mode_codes {
        tracing::debug!(mode = %mode, mode_code = mode_code, path = %path, "D-Bus write: asusd aura LedMode");
        if set_property_value_async(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(mode_code),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        let mode_i32 = i32::from(mode_code);
        if set_property_value_async(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(mode_i32),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        if set_property_value_async(
            connection,
            &path,
            AURA_INTERFACE,
            "LedMode",
            ZValue::from(u32::from(mode_code)),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        tracing::warn!(
            mode = %mode,
            mode_code = mode_code,
            "D-Bus write asusd aura LedMode failed for this candidate code"
        );
    }
    Err(anyhow::anyhow!("D-Bus: failed to set asusd aura LedMode"))
}

pub fn set_anime_brightness(connection: &zbus::blocking::Connection, level: u8) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "Brightness",
        ZValue::from(level),
    )
    .context("D-Bus: failed to set asusd anime Brightness")
}

pub async fn set_anime_brightness_async(
    connection: &zbus::Connection,
    level: u8,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "Brightness",
        ZValue::from(level),
    )
    .await
    .context("D-Bus: failed to set asusd anime Brightness")
}

pub fn get_anime_status(connection: &zbus::blocking::Connection) -> AppResult<AnimeStatusSnapshot> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    let properties = properties_proxy(connection, &path, ANIME_INTERFACE, "anime read")?;
    let brightness = get_property_value_with_interface(&properties, ANIME_INTERFACE, "Brightness")
        .map(|value| format_value(&value));
    Ok(AnimeStatusSnapshot {
        enable_display: get_property_value_with_interface(
            &properties,
            ANIME_INTERFACE,
            "EnableDisplay",
        )
        .and_then(|value| bool::try_from(value).ok()),
        builtins_enabled: get_property_value_with_interface(
            &properties,
            ANIME_INTERFACE,
            "BuiltinsEnabled",
        )
        .and_then(|value| bool::try_from(value).ok()),
        brightness,
        off_when_lid_closed: get_property_value_with_interface(
            &properties,
            ANIME_INTERFACE,
            "OffWhenLidClosed",
        )
        .and_then(|value| bool::try_from(value).ok()),
        off_when_suspended: get_property_value_with_interface(
            &properties,
            ANIME_INTERFACE,
            "OffWhenSuspended",
        )
        .and_then(|value| bool::try_from(value).ok()),
        off_when_unplugged: get_property_value_with_interface(
            &properties,
            ANIME_INTERFACE,
            "OffWhenUnplugged",
        )
        .and_then(|value| bool::try_from(value).ok()),
        last_error: None,
    })
}

pub fn set_anime_enable_display(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "EnableDisplay",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd anime EnableDisplay")
}

pub async fn set_anime_enable_display_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "EnableDisplay",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd anime EnableDisplay")
}

pub fn set_anime_builtins_enabled(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "BuiltinsEnabled",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd anime BuiltinsEnabled")
}

pub async fn set_anime_builtins_enabled_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "BuiltinsEnabled",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd anime BuiltinsEnabled")
}

pub fn set_anime_off_when_lid_closed(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenLidClosed",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd anime OffWhenLidClosed")
}

pub async fn set_anime_off_when_lid_closed_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenLidClosed",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd anime OffWhenLidClosed")
}

pub fn set_anime_off_when_suspended(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenSuspended",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd anime OffWhenSuspended")
}

pub async fn set_anime_off_when_suspended_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenSuspended",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd anime OffWhenSuspended")
}

pub fn set_anime_off_when_unplugged(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, ANIME_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenUnplugged",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd anime OffWhenUnplugged")
}

pub async fn set_anime_off_when_unplugged_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, ANIME_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        ANIME_INTERFACE,
        "OffWhenUnplugged",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd anime OffWhenUnplugged")
}

pub fn get_slash_status(connection: &zbus::blocking::Connection) -> AppResult<SlashStatusSnapshot> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    let properties = properties_proxy(connection, &path, SLASH_INTERFACE, "slash read")?;
    Ok(SlashStatusSnapshot {
        enabled: get_property_value_with_interface(&properties, SLASH_INTERFACE, "Enabled")
            .and_then(|value| bool::try_from(value).ok()),
        brightness: get_property_value_with_interface(&properties, SLASH_INTERFACE, "Brightness")
            .and_then(|value| u8::try_from(value).ok()),
        interval: get_property_value_with_interface(&properties, SLASH_INTERFACE, "Interval")
            .and_then(|value| u8::try_from(value).ok()),
        mode: get_property_summary_with_interface(&properties, SLASH_INTERFACE, "Mode"),
        show_on_boot: get_property_value_with_interface(&properties, SLASH_INTERFACE, "ShowOnBoot")
            .and_then(|value| bool::try_from(value).ok()),
        show_on_sleep: get_property_value_with_interface(
            &properties,
            SLASH_INTERFACE,
            "ShowOnSleep",
        )
        .and_then(|value| bool::try_from(value).ok()),
        show_on_shutdown: get_property_value_with_interface(
            &properties,
            SLASH_INTERFACE,
            "ShowOnShutdown",
        )
        .and_then(|value| bool::try_from(value).ok()),
        show_on_battery: get_property_value_with_interface(
            &properties,
            SLASH_INTERFACE,
            "ShowOnBattery",
        )
        .and_then(|value| bool::try_from(value).ok()),
        show_battery_warning: get_property_value_with_interface(
            &properties,
            SLASH_INTERFACE,
            "ShowBatteryWarning",
        )
        .and_then(|value| bool::try_from(value).ok()),
        show_on_lid_closed: get_property_value_with_interface(
            &properties,
            SLASH_INTERFACE,
            "ShowOnLidClosed",
        )
        .and_then(|value| bool::try_from(value).ok()),
        last_error: None,
    })
}

pub fn set_slash_enabled(connection: &zbus::blocking::Connection, enabled: bool) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "Enabled",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash Enabled")
}

pub async fn set_slash_enabled_async(connection: &zbus::Connection, enabled: bool) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "Enabled",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash Enabled")
}

pub fn set_slash_brightness(
    connection: &zbus::blocking::Connection,
    brightness: u8,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "Brightness",
        ZValue::from(brightness),
    )
    .context("D-Bus: failed to set asusd slash Brightness")
}

pub async fn set_slash_brightness_async(
    connection: &zbus::Connection,
    brightness: u8,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "Brightness",
        ZValue::from(brightness),
    )
    .await
    .context("D-Bus: failed to set asusd slash Brightness")
}

pub fn set_slash_interval(connection: &zbus::blocking::Connection, interval: u8) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "Interval",
        ZValue::from(interval),
    )
    .context("D-Bus: failed to set asusd slash Interval")
}

pub async fn set_slash_interval_async(connection: &zbus::Connection, interval: u8) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "Interval",
        ZValue::from(interval),
    )
    .await
    .context("D-Bus: failed to set asusd slash Interval")
}

pub fn set_slash_mode(connection: &zbus::blocking::Connection, mode: &str) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    let mode_codes = parse_slash_mode_codes(mode)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported slash mode '{mode}'"))?;
    for mode_code in mode_codes {
        tracing::debug!(mode = %mode, mode_code = mode_code, path = %path, "D-Bus write: asusd slash Mode");
        if set_property_value(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(mode_code),
        )
        .is_ok()
        {
            return Ok(());
        }
        let mode_i32 = i32::from(mode_code);
        if set_property_value(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(mode_i32),
        )
        .is_ok()
        {
            return Ok(());
        }
        if set_property_value(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(u32::from(mode_code)),
        )
        .is_ok()
        {
            return Ok(());
        }
    }
    Err(anyhow::anyhow!("D-Bus: failed to set asusd slash Mode"))
}

pub async fn set_slash_mode_async(connection: &zbus::Connection, mode: &str) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    let mode_codes = parse_slash_mode_codes(mode)
        .ok_or_else(|| anyhow::anyhow!("asusd: unsupported slash mode '{mode}'"))?;
    for mode_code in mode_codes {
        tracing::debug!(mode = %mode, mode_code = mode_code, path = %path, "D-Bus write: asusd slash Mode");
        if set_property_value_async(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(mode_code),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        let mode_i32 = i32::from(mode_code);
        if set_property_value_async(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(mode_i32),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        if set_property_value_async(
            connection,
            &path,
            SLASH_INTERFACE,
            "Mode",
            ZValue::from(u32::from(mode_code)),
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
    }
    Err(anyhow::anyhow!("D-Bus: failed to set asusd slash Mode"))
}

pub fn set_slash_show_on_boot(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnBoot",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowOnBoot")
}

pub async fn set_slash_show_on_boot_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnBoot",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowOnBoot")
}

pub fn set_slash_show_on_shutdown(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnShutdown",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowOnShutdown")
}

pub async fn set_slash_show_on_shutdown_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnShutdown",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowOnShutdown")
}

pub fn set_slash_show_on_sleep(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnSleep",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowOnSleep")
}

pub async fn set_slash_show_on_sleep_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnSleep",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowOnSleep")
}

pub fn set_slash_show_on_battery(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnBattery",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowOnBattery")
}

pub async fn set_slash_show_on_battery_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnBattery",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowOnBattery")
}

pub fn set_slash_show_battery_warning(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowBatteryWarning",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowBatteryWarning")
}

pub async fn set_slash_show_battery_warning_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowBatteryWarning",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowBatteryWarning")
}

pub fn set_slash_show_on_lid_closed(
    connection: &zbus::blocking::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, SLASH_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnLidClosed",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd slash ShowOnLidClosed")
}

pub async fn set_slash_show_on_lid_closed_async(
    connection: &zbus::Connection,
    enabled: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SLASH_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SLASH_INTERFACE,
        "ShowOnLidClosed",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd slash ShowOnLidClosed")
}

pub fn get_backlight_status(
    connection: &zbus::blocking::Connection,
) -> AppResult<BacklightStatusSnapshot> {
    let path = find_first_interface_path(connection, BACKLIGHT_INTERFACE)?;
    let properties = properties_proxy(connection, &path, BACKLIGHT_INTERFACE, "backlight read")?;
    Ok(BacklightStatusSnapshot {
        screenpad_brightness: get_property_value_with_interface(
            &properties,
            BACKLIGHT_INTERFACE,
            "ScreenpadBrightness",
        )
        .and_then(|value| i32::try_from(value).ok()),
        screenpad_gamma: get_property_summary_with_interface(
            &properties,
            BACKLIGHT_INTERFACE,
            "ScreenpadGamma",
        ),
        sync_screenpad_brightness: get_property_value_with_interface(
            &properties,
            BACKLIGHT_INTERFACE,
            "ScreenpadSyncWithPrimary",
        )
        .and_then(|value| bool::try_from(value).ok()),
        last_error: None,
    })
}

pub fn set_backlight_screenpad_brightness(
    connection: &zbus::blocking::Connection,
    value: i32,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, BACKLIGHT_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadBrightness",
        ZValue::from(value),
    )
    .context("D-Bus: failed to set asusd backlight ScreenpadBrightness")
}

pub async fn set_backlight_screenpad_brightness_async(
    connection: &zbus::Connection,
    value: i32,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, BACKLIGHT_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadBrightness",
        ZValue::from(value),
    )
    .await
    .context("D-Bus: failed to set asusd backlight ScreenpadBrightness")
}

pub fn set_backlight_screenpad_gamma(
    connection: &zbus::blocking::Connection,
    value: f32,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, BACKLIGHT_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadGamma",
        ZValue::from(value.to_string()),
    )
    .context("D-Bus: failed to set asusd backlight ScreenpadGamma")
}

pub async fn set_backlight_screenpad_gamma_async(
    connection: &zbus::Connection,
    value: f32,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, BACKLIGHT_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadGamma",
        ZValue::from(value.to_string()),
    )
    .await
    .context("D-Bus: failed to set asusd backlight ScreenpadGamma")
}

pub fn set_backlight_sync_screenpad_brightness(
    connection: &zbus::blocking::Connection,
    value: bool,
) -> AppResult<()> {
    let path = find_first_interface_path(connection, BACKLIGHT_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadSyncWithPrimary",
        ZValue::from(value),
    )
    .context("D-Bus: failed to set asusd backlight ScreenpadSyncWithPrimary")
}

pub async fn set_backlight_sync_screenpad_brightness_async(
    connection: &zbus::Connection,
    value: bool,
) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, BACKLIGHT_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        BACKLIGHT_INTERFACE,
        "ScreenpadSyncWithPrimary",
        ZValue::from(value),
    )
    .await
    .context("D-Bus: failed to set asusd backlight ScreenpadSyncWithPrimary")
}

pub fn get_scsi_status(connection: &zbus::blocking::Connection) -> AppResult<ScsiStatusSnapshot> {
    let path = find_first_interface_path(connection, SCSI_INTERFACE)?;
    let properties = properties_proxy(connection, &path, SCSI_INTERFACE, "scsi read")?;
    Ok(ScsiStatusSnapshot {
        enabled: get_property_value_with_interface(&properties, SCSI_INTERFACE, "Enabled")
            .and_then(|value| bool::try_from(value).ok()),
        mode: get_property_summary_with_interface(&properties, SCSI_INTERFACE, "LedMode"),
        mode_data: get_property_summary_with_interface(&properties, SCSI_INTERFACE, "LedModeData"),
        last_error: None,
    })
}

pub fn set_scsi_enabled(connection: &zbus::blocking::Connection, enabled: bool) -> AppResult<()> {
    let path = find_first_interface_path(connection, SCSI_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SCSI_INTERFACE,
        "Enabled",
        ZValue::from(enabled),
    )
    .context("D-Bus: failed to set asusd scsi Enabled")
}

pub async fn set_scsi_enabled_async(connection: &zbus::Connection, enabled: bool) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SCSI_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SCSI_INTERFACE,
        "Enabled",
        ZValue::from(enabled),
    )
    .await
    .context("D-Bus: failed to set asusd scsi Enabled")
}

pub fn set_scsi_mode(connection: &zbus::blocking::Connection, mode: u8) -> AppResult<()> {
    let path = find_first_interface_path(connection, SCSI_INTERFACE)?;
    set_property_value(
        connection,
        &path,
        SCSI_INTERFACE,
        "LedMode",
        ZValue::from(mode),
    )
    .context("D-Bus: failed to set asusd scsi LedMode")
}

pub async fn set_scsi_mode_async(connection: &zbus::Connection, mode: u8) -> AppResult<()> {
    let path = find_first_interface_path_async(connection, SCSI_INTERFACE).await?;
    set_property_value_async(
        connection,
        &path,
        SCSI_INTERFACE,
        "LedMode",
        ZValue::from(mode),
    )
    .await
    .context("D-Bus: failed to set asusd scsi LedMode")
}

pub fn list_armoury_paths(connection: &zbus::blocking::Connection) -> AppResult<Vec<String>> {
    let object_manager = zbus::blocking::fdo::ObjectManagerProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for object manager")?
        .path(ROOT_PATH)
        .context("D-Bus: invalid asusd root path for object manager")?
        .build()
        .context("D-Bus: failed to create asusd object manager proxy")?;
    let objects = object_manager
        .get_managed_objects()
        .context("D-Bus: failed to query asusd managed objects")?;

    let mut paths = Vec::new();
    for (path, interfaces) in objects {
        let has_interface = interfaces
            .keys()
            .any(|interface| interface.as_str() == ARMOURY_INTERFACE);
        if has_interface {
            let path_text = path.to_string();
            if path_text.starts_with(ARMOURY_PATH_PREFIX) {
                paths.push(path_text);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

async fn list_armoury_paths_async(connection: &zbus::Connection) -> AppResult<Vec<String>> {
    let mut paths = list_interface_paths_async(connection, ARMOURY_INTERFACE).await?;
    paths.retain(|path| path.starts_with(ARMOURY_PATH_PREFIX));
    paths.sort();
    Ok(paths)
}

pub fn get_armoury_status(
    connection: &zbus::blocking::Connection,
) -> AppResult<ArmouryStatusSnapshot> {
    let paths = list_armoury_paths(connection)?;
    let mut attributes = Vec::with_capacity(paths.len());

    for path in paths {
        let properties =
            match properties_proxy(connection, &path, ARMOURY_INTERFACE, "armoury read") {
                Ok(proxy) => proxy,
                Err(error) => {
                    attributes.push(ArmouryAttributeSnapshot {
                        path: path.clone(),
                        last_error: Some(error.to_string()),
                        ..ArmouryAttributeSnapshot::default()
                    });
                    continue;
                }
            };

        let item = ArmouryAttributeSnapshot {
            path: path.clone(),
            name: get_property_summary_with_interface(&properties, ARMOURY_INTERFACE, "Name"),
            available_attrs: get_property_summary_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "AvailableAttrs",
            ),
            current_value: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "CurrentValue",
            )
            .and_then(|value| i32::try_from(value).ok()),
            default_value: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "DefaultValue",
            )
            .and_then(|value| i32::try_from(value).ok()),
            min_value: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "MinValue",
            )
            .and_then(|value| i32::try_from(value).ok()),
            max_value: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "MaxValue",
            )
            .and_then(|value| i32::try_from(value).ok()),
            possible_values: get_property_summary_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "PossibleValues",
            ),
            scalar_increment: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "ScalarIncrement",
            )
            .and_then(|value| i32::try_from(value).ok()),
            queued_gpu_value: get_property_value_with_interface(
                &properties,
                ARMOURY_INTERFACE,
                "QueuedGpuValue",
            )
            .and_then(|value| i32::try_from(value).ok()),
            last_error: None,
        };
        attributes.push(item);
    }

    Ok(ArmouryStatusSnapshot {
        attributes,
        last_error: None,
    })
}

pub fn set_armoury_current_value(
    connection: &zbus::blocking::Connection,
    path: &str,
    value: i32,
) -> AppResult<()> {
    set_property_value(
        connection,
        path,
        ARMOURY_INTERFACE,
        "CurrentValue",
        ZValue::from(value),
    )
    .context("D-Bus: failed to set asusd armoury CurrentValue")
}

pub async fn set_armoury_current_value_async(
    connection: &zbus::Connection,
    path: &str,
    value: i32,
) -> AppResult<()> {
    set_property_value_async(
        connection,
        path,
        ARMOURY_INTERFACE,
        "CurrentValue",
        ZValue::from(value),
    )
    .await
    .context("D-Bus: failed to set asusd armoury CurrentValue")
}

fn get_property_value(
    properties: &zbus::blocking::fdo::PropertiesProxy<'_>,
    property_name: &str,
) -> Option<OwnedValue> {
    get_property_value_with_interface(properties, PLATFORM_INTERFACE, property_name)
}

fn get_property_summary_with_interface(
    properties: &zbus::blocking::fdo::PropertiesProxy<'_>,
    interface_name: &str,
    property_name: &str,
) -> Option<String> {
    get_property_value_with_interface(properties, interface_name, property_name)
        .map(|value| format_value(&value))
}

fn get_property_value_with_interface(
    properties: &zbus::blocking::fdo::PropertiesProxy<'_>,
    interface_name: &str,
    property_name: &str,
) -> Option<OwnedValue> {
    let interface = InterfaceName::try_from(interface_name).ok()?;
    properties.get(interface, property_name).ok()
}

async fn get_property_value_with_interface_async(
    properties: &zbus::fdo::PropertiesProxy<'_>,
    interface_name: &str,
    property_name: &str,
) -> Option<OwnedValue> {
    let interface = InterfaceName::try_from(interface_name).ok()?;
    properties.get(interface, property_name).await.ok()
}

fn format_value(value: &OwnedValue) -> String {
    let debug_text = format!("{value:?}");
    debug_text.trim_matches('"').to_owned()
}

fn format_fan_curve_payload(value: &OwnedValue) -> String {
    if let Ok(lines) = Vec::<String>::try_from(value.clone()) {
        if !lines.is_empty() {
            return lines.join("\n");
        }
    }
    if let Ok(entries) = Vec::<OwnedValue>::try_from(value.clone()) {
        let lines: Vec<String> = entries.iter().map(format_value).collect();
        if !lines.is_empty() {
            return lines.join("\n");
        }
    }
    format_value(value)
}

type FanCurvePointTuple = (u8, u8, u8, u8, u8, u8, u8, u8);
type FanCurveEntryTuple = (String, FanCurvePointTuple, FanCurvePointTuple, bool);
type FanCurveDecodedPayload = Vec<FanCurveEntryTuple>;

struct FanCurveDecoded {
    raw: String,
    series: Vec<FanCurveSeriesSnapshot>,
}

fn fan_curve_write_tuple(
    fan: &str,
    pwm: [u8; 8],
    temp: [u8; 8],
    enabled: bool,
) -> FanCurveEntryTuple {
    (
        fan.to_owned(),
        (pwm[0], pwm[1], pwm[2], pwm[3], pwm[4], pwm[5], pwm[6], pwm[7]),
        (
            temp[0], temp[1], temp[2], temp[3], temp[4], temp[5], temp[6], temp[7],
        ),
        enabled,
    )
}

fn decode_fan_curve_message(message: &zbus::Message) -> AppResult<FanCurveDecoded> {
    if let Ok(entries) = message.body().deserialize::<FanCurveDecodedPayload>() {
        if !entries.is_empty() {
            let mut raw_lines: Vec<String> = Vec::new();
            let mut series: Vec<FanCurveSeriesSnapshot> = Vec::new();
            for (fan, pwm, temp, enabled) in entries {
                let pwm_values = [pwm.0, pwm.1, pwm.2, pwm.3, pwm.4, pwm.5, pwm.6, pwm.7];
                let temp_values = [
                    temp.0, temp.1, temp.2, temp.3, temp.4, temp.5, temp.6, temp.7,
                ];
                raw_lines.push(format!(
                    "{fan}: pwm={:?}, temp={:?}, enabled={enabled}",
                    pwm_values, temp_values
                ));
                let points = temp_values
                    .iter()
                    .zip(pwm_values.iter())
                    .map(|(temperature, pwm)| FanCurvePointSnapshot {
                        temperature: *temperature,
                        pwm: *pwm,
                    })
                    .collect();
                series.push(FanCurveSeriesSnapshot {
                    fan,
                    enabled,
                    points,
                });
            }
            return Ok(FanCurveDecoded {
                raw: raw_lines.join("\n"),
                series,
            });
        }
    }

    if let Ok(value) = message.body().deserialize::<OwnedValue>() {
        return Ok(FanCurveDecoded {
            raw: format_fan_curve_payload(&value),
            series: Vec::new(),
        });
    }

    Err(anyhow::anyhow!(
        "unsupported fan_curve_data payload signature: {:?}",
        message.body().signature()
    ))
}

fn call_fan_curve_data<'a>(
    proxy: &'a zbus::blocking::Proxy<'a>,
    profile_code: u32,
) -> AppResult<zbus::Message> {
    tracing::debug!(
        profile_code = profile_code,
        "D-Bus call: asusd.FanCurveData (u32)"
    );
    match proxy
        .call_method("FanCurveData", &(profile_code,))
        .or_else(|_| proxy.call_method("fan_curve_data", &(profile_code,)))
    {
        Ok(message) => Ok(message),
        Err(first_error) => {
            tracing::warn!(
                profile_code = profile_code,
                error = %first_error,
                "D-Bus call asusd.FanCurveData with u32 failed, retrying with i32"
            );
            let profile_code_i32 = i32::try_from(profile_code)
                .map_err(|_| anyhow::anyhow!("invalid profile code: {profile_code}"))?;
            tracing::debug!(
                profile_code = profile_code_i32,
                "D-Bus call: asusd.FanCurveData (i32 retry)"
            );
            proxy
                .call_method("FanCurveData", &(profile_code_i32,))
                .or_else(|_| proxy.call_method("fan_curve_data", &(profile_code_i32,)))
                .map_err(|second_error| {
                    anyhow::anyhow!(
                        "u32 call failed: {first_error}; i32 retry failed: {second_error}"
                    )
                })
        }
    }
}

fn interface_available(
    connection: &zbus::blocking::Connection,
    interface_name: &str,
) -> AppResult<bool> {
    Ok(!list_interface_paths(connection, interface_name)?.is_empty())
}

async fn interface_available_async(
    connection: &zbus::Connection,
    interface_name: &str,
) -> AppResult<bool> {
    Ok(!list_interface_paths_async(connection, interface_name)
        .await?
        .is_empty())
}

fn find_first_interface_path(
    connection: &zbus::blocking::Connection,
    interface_name: &str,
) -> AppResult<String> {
    let mut paths = list_interface_paths(connection, interface_name)?;
    paths.sort();
    paths
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("asusd interface '{interface_name}' is unavailable"))
}

async fn find_first_interface_path_async(
    connection: &zbus::Connection,
    interface_name: &str,
) -> AppResult<String> {
    let mut paths = list_interface_paths_async(connection, interface_name).await?;
    paths.sort();
    paths
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("asusd interface '{interface_name}' is unavailable"))
}

fn list_interface_paths(
    connection: &zbus::blocking::Connection,
    interface_name: &str,
) -> AppResult<Vec<String>> {
    let object_manager = zbus::blocking::fdo::ObjectManagerProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for object manager")?
        .path(ROOT_PATH)
        .context("D-Bus: invalid asusd root path for object manager")?
        .build()
        .context("D-Bus: failed to create asusd object manager proxy")?;
    let objects = object_manager
        .get_managed_objects()
        .context("D-Bus: failed to query asusd managed objects")?;
    let mut paths = Vec::new();
    for (path, interfaces) in objects {
        if interfaces
            .keys()
            .any(|name| name.as_str() == interface_name)
        {
            paths.push(path.to_string());
        }
    }
    Ok(paths)
}

async fn list_interface_paths_async(
    connection: &zbus::Connection,
    interface_name: &str,
) -> AppResult<Vec<String>> {
    let object_manager = zbus::fdo::ObjectManagerProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for object manager")?
        .path(ROOT_PATH)
        .context("D-Bus: invalid asusd root path for object manager")?
        .build()
        .await
        .context("D-Bus: failed to create asusd object manager proxy")?;
    let objects = object_manager
        .get_managed_objects()
        .await
        .context("D-Bus: failed to query asusd managed objects")?;
    let mut paths = Vec::new();
    for (path, interfaces) in objects {
        if interfaces
            .keys()
            .any(|name| name.as_str() == interface_name)
        {
            paths.push(path.to_string());
        }
    }
    Ok(paths)
}

fn properties_proxy<'a>(
    connection: &'a zbus::blocking::Connection,
    path: &'a str,
    interface_name: &'a str,
    op: &'a str,
) -> AppResult<zbus::blocking::fdo::PropertiesProxy<'a>> {
    zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .with_context(|| format!("D-Bus: invalid asusd destination for {op}"))?
        .path(path)
        .with_context(|| format!("D-Bus: invalid asusd path '{path}' for {op}"))?
        .interface(interface_name)
        .with_context(|| format!("D-Bus: invalid asusd interface '{interface_name}' for {op}"))?
        .build()
        .with_context(|| format!("D-Bus: failed to create asusd properties proxy for {op}"))
}

fn set_property_value(
    connection: &zbus::blocking::Connection,
    path: &str,
    interface_name: &str,
    property_name: &str,
    value: ZValue<'_>,
) -> AppResult<()> {
    let properties = zbus::blocking::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for property write")?
        .path(path)
        .with_context(|| format!("D-Bus: invalid asusd path '{path}' for property write"))?
        .build()
        .context("D-Bus: failed to create asusd properties proxy for property write")?;
    let interface = InterfaceName::try_from(interface_name)
        .with_context(|| format!("D-Bus: invalid asusd interface '{interface_name}'"))?;
    tracing::debug!(
        path = %path,
        interface = %interface_name,
        property = %property_name,
        "D-Bus property write"
    );
    let result = properties
        .set(interface, property_name, value)
        .with_context(|| format!("D-Bus: failed to set asusd property '{property_name}'"));
    if let Err(error) = &result {
        tracing::warn!(
            path = %path,
            interface = %interface_name,
            property = %property_name,
            error = %error,
            "D-Bus property write failed"
        );
    }
    result
}

async fn set_property_value_async(
    connection: &zbus::Connection,
    path: &str,
    interface_name: &str,
    property_name: &str,
    value: ZValue<'_>,
) -> AppResult<()> {
    let properties = zbus::fdo::PropertiesProxy::builder(connection)
        .destination(SERVICE_NAME)
        .context("D-Bus: invalid asusd destination for property write")?
        .path(path)
        .with_context(|| format!("D-Bus: invalid asusd path '{path}' for property write"))?
        .build()
        .await
        .context("D-Bus: failed to create asusd properties proxy for property write")?;
    let interface = InterfaceName::try_from(interface_name)
        .with_context(|| format!("D-Bus: invalid asusd interface '{interface_name}'"))?;
    tracing::debug!(
        path = %path,
        interface = %interface_name,
        property = %property_name,
        "D-Bus property write"
    );
    let result = properties
        .set(interface, property_name, value)
        .await
        .with_context(|| format!("D-Bus: failed to set asusd property '{property_name}'"));
    if let Err(error) = &result {
        tracing::warn!(
            path = %path,
            interface = %interface_name,
            property = %property_name,
            error = %error,
            "D-Bus property write failed"
        );
    }
    result
}

fn parse_platform_profile_code(profile: &str) -> Option<u32> {
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

fn format_platform_profile_value(value: &OwnedValue) -> Option<String> {
    if let Ok(code) = u32::try_from(value.clone()) {
        return Some(format_platform_profile_code(code).to_owned());
    }
    if let Ok(code) = i32::try_from(value.clone()) {
        if code >= 0 {
            return Some(format_platform_profile_code(code as u32).to_owned());
        }
    }
    if let Ok(text) = String::try_from(value.clone()) {
        return parse_platform_profile_code(&text).map_or(Some(text), |code| {
            Some(format_platform_profile_code(code).to_owned())
        });
    }
    None
}

fn format_platform_profile_choices_value(value: &OwnedValue) -> Option<String> {
    if let Ok(codes) = Vec::<u32>::try_from(value.clone()) {
        let names: Vec<&str> = codes
            .into_iter()
            .map(format_platform_profile_code)
            .collect();
        return Some(names.join(", "));
    }
    if let Ok(codes) = Vec::<i32>::try_from(value.clone()) {
        let names: Vec<&str> = codes
            .into_iter()
            .filter(|code| *code >= 0)
            .map(|code| format_platform_profile_code(code as u32))
            .collect();
        if !names.is_empty() {
            return Some(names.join(", "));
        }
    }
    if let Ok(values) = Vec::<OwnedValue>::try_from(value.clone()) {
        let names: Vec<String> = values
            .iter()
            .filter_map(format_platform_profile_value)
            .collect();
        if !names.is_empty() {
            return Some(names.join(", "));
        }
    }
    if let Ok(strings) = Vec::<String>::try_from(value.clone()) {
        let normalized: Vec<String> = strings
            .into_iter()
            .map(|item| {
                parse_platform_profile_code(&item)
                    .map(|code| format_platform_profile_code(code).to_owned())
                    .unwrap_or(item)
            })
            .collect();
        if !normalized.is_empty() {
            return Some(normalized.join(", "));
        }
    }
    Some(format_value(value))
}

fn format_platform_profile_code(code: u32) -> &'static str {
    match code {
        0 => "Balanced",
        1 => "Performance",
        2 => "Quiet",
        3 => "Low Power",
        4 => "Custom",
        _ => "Unknown",
    }
}

fn parse_aura_mode_codes(mode: &str) -> Option<Vec<u8>> {
    let normalized = mode
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");
    let mut codes = Vec::new();
    let primary = match normalized.as_str() {
        "static" => 0,
        "breathe" => 1,
        "rainbowcycle" => 2,
        "rainbowwave" => 3,
        "star" | "stars" => 4,
        "rain" => 5,
        "highlight" => 6,
        "laser" => 7,
        "ripple" => 8,
        "pulse" => 10,
        "comet" => 11,
        "flash" => 12,
        _ => return None,
    };
    codes.push(primary);
    if normalized == "pulse" {
        codes.push(4);
    }
    Some(codes)
}

fn parse_slash_mode_codes(mode: &str) -> Option<Vec<u8>> {
    let normalized = mode
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");
    let (enum_index, hw_code) = match normalized.as_str() {
        "static" => (0, 0x06),
        "bounce" => (1, 0x10),
        "slash" => (2, 0x12),
        "loading" => (3, 0x13),
        "bitstream" => (4, 0x1d),
        "transmission" => (5, 0x1a),
        "flow" => (6, 0x19),
        "flux" => (7, 0x25),
        "phantom" => (8, 0x24),
        "spectrum" => (9, 0x26),
        "hazard" => (10, 0x32),
        "interfacing" => (11, 0x33),
        "ramp" => (12, 0x34),
        "gameover" => (13, 0x42),
        "start" => (14, 0x43),
        "buzzer" => (15, 0x44),
        _ => return None,
    };
    let mut codes = vec![enum_index, hw_code];
    codes.sort_unstable();
    codes.dedup();
    Some(codes)
}

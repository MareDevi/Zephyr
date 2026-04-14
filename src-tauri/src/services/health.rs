#![allow(dead_code)]
// Legacy sync probes are kept for callers outside the Tauri IPC path.

use anyhow::Context;

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::DaemonHealthSnapshot;

pub fn probe_daemons() -> AppResult<DaemonHealthSnapshot> {
    let connection =
        zbus::blocking::Connection::system().context("D-Bus: failed to connect to system bus")?;
    Ok(DaemonHealthSnapshot {
        asusd_available: dbus::asusd::probe(&connection).context("D-Bus: asusd probe failed")?,
        supergfxd_available: dbus::supergfxd::probe(&connection)
            .context("D-Bus: supergfxd probe failed")?,
        ppd_available: dbus::ppd::probe(&connection).context("D-Bus: ppd probe failed")?,
        last_error: None,
    })
}

pub fn probe_daemons_resilient() -> DaemonHealthSnapshot {
    match probe_daemons() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "health probe bootstrap degraded");
            DaemonHealthSnapshot {
                asusd_available: false,
                supergfxd_available: false,
                ppd_available: false,
                last_error: Some(error.to_string()),
            }
        }
    }
}

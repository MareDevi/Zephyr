use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use futures_util::StreamExt;
use tauri::{Manager, Wry};
use tokio::time::MissedTickBehavior;

use crate::dbus;
use crate::error::AppResult;
use crate::ipc::dto::{DaemonHealthSnapshot, DaemonInterfaceSnapshot, DashboardSnapshotDto};
use crate::state::AppState;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

pub fn collect_dashboard() -> AppResult<DashboardSnapshotDto> {
    let connection =
        zbus::blocking::Connection::system().context("D-Bus: failed to connect to system bus")?;

    let mut bootstrap_errors: Vec<String> = Vec::new();
    let asusd_available = probe_with_note(
        dbus::asusd::probe(&connection),
        "asusd probe failed",
        &mut bootstrap_errors,
    );
    let supergfxd_available = probe_with_note(
        dbus::supergfxd::probe(&connection),
        "supergfxd probe failed",
        &mut bootstrap_errors,
    );
    let ppd_available = probe_with_note(
        dbus::ppd::probe(&connection),
        "ppd probe failed",
        &mut bootstrap_errors,
    );

    let asusd_platform_available = if asusd_available {
        probe_with_note(
            dbus::asusd::platform_interface_available(&connection),
            "asusd platform interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_fan_curves_available = if asusd_available {
        probe_with_note(
            dbus::asusd::fan_curves_interface_available(&connection),
            "asusd fan curves interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_aura_available = if asusd_available {
        probe_with_note(
            dbus::asusd::aura_interface_available(&connection),
            "asusd aura interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_anime_available = if asusd_available {
        probe_with_note(
            dbus::asusd::anime_interface_available(&connection),
            "asusd anime interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_slash_available = if asusd_available {
        probe_with_note(
            dbus::asusd::slash_interface_available(&connection),
            "asusd slash interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_scsi_available = if asusd_available {
        probe_with_note(
            dbus::asusd::scsi_interface_available(&connection),
            "asusd scsi interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_backlight_available = if asusd_available {
        probe_with_note(
            dbus::asusd::backlight_interface_available(&connection),
            "asusd backlight interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };
    let asusd_armoury_available = if asusd_available {
        probe_with_note(
            dbus::asusd::armoury_interface_available(&connection),
            "asusd armoury interface discovery failed",
            &mut bootstrap_errors,
        )
    } else {
        false
    };

    let health = DaemonHealthSnapshot {
        asusd_available,
        supergfxd_available,
        ppd_available,
        last_error: (!bootstrap_errors.is_empty()).then(|| bootstrap_errors.join("; ")),
    };
    let interfaces = DaemonInterfaceSnapshot {
        asusd_platform_available,
        asusd_fan_curves_available,
        asusd_aura_available,
        asusd_anime_available,
        asusd_slash_available,
        asusd_scsi_available,
        asusd_backlight_available,
        asusd_armoury_available,
        supergfxd_interface_available: supergfxd_available,
        ppd_interface_available: ppd_available,
    };

    let platform_snapshot = crate::services::platform::read_snapshot(
        &connection,
        asusd_available,
        asusd_platform_available,
    );
    let active_profile = platform_snapshot.platform_profile.clone();

    Ok(DashboardSnapshotDto {
        performance: crate::services::performance::read_snapshot(),
        gpu: crate::services::gpu::read_snapshot(&connection, supergfxd_available),
        power: crate::services::power::read_snapshot(&connection, ppd_available),
        fan_curves: crate::services::fan_curves::read_snapshot(
            &connection,
            asusd_available,
            asusd_fan_curves_available,
            active_profile.as_deref(),
        ),
        aura: crate::services::aura::read_aura_snapshot(
            &connection,
            asusd_available,
            asusd_aura_available,
        ),
        anime: crate::services::anime::read_snapshot(
            &connection,
            asusd_available,
            asusd_anime_available,
        ),
        slash: crate::services::aura::read_slash_snapshot(
            &connection,
            asusd_available,
            asusd_slash_available,
        ),
        scsi: crate::services::scsi::read_snapshot(
            &connection,
            asusd_available,
            asusd_scsi_available,
        ),
        backlight: crate::services::backlight::read_snapshot(
            &connection,
            asusd_available,
            asusd_backlight_available,
        ),
        armoury: crate::services::armoury::read_snapshot(
            &connection,
            asusd_available,
            asusd_armoury_available,
        ),
        platform: platform_snapshot,
        health,
        interfaces,
        updated_at_ms: now_ms(),
    })
}

pub fn collect_dashboard_resilient() -> DashboardSnapshotDto {
    match collect_dashboard() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "dashboard collection degraded");
            DashboardSnapshotDto::from_health(
                DaemonHealthSnapshot {
                    asusd_available: false,
                    supergfxd_available: false,
                    ppd_available: false,
                    last_error: Some(error.to_string()),
                },
                now_ms(),
            )
        }
    }
}

const WATCH_RECONNECT_DELAY: Duration = Duration::from_secs(3);
const FALLBACK_REFRESH_INTERVAL: Duration = Duration::from_secs(90);
const PERFORMANCE_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub fn start_dashboard_watchers(app: tauri::AppHandle<Wry>) {
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-fallback-refresh", {
            let app = app.clone();
            async move {
                run_fallback_refresh_loop(app).await;
            }
        });
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-watch-performance", {
            let app = app.clone();
            async move {
                run_performance_refresh_loop(app).await;
            }
        });
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-watch-supergfx-mode", {
            let app = app.clone();
            async move {
                run_proxy_signal_watch(
                    app,
                    dbus::supergfxd::SERVICE_NAME,
                    dbus::supergfxd::PATH,
                    dbus::supergfxd::INTERFACE,
                    "NotifyGfx",
                    "supergfxd mode signal",
                )
                .await;
            }
        });
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-watch-supergfx-pending", {
            let app = app.clone();
            async move {
                run_proxy_signal_watch(
                    app,
                    dbus::supergfxd::SERVICE_NAME,
                    dbus::supergfxd::PATH,
                    dbus::supergfxd::INTERFACE,
                    "NotifyAction",
                    "supergfxd pending action signal",
                )
                .await;
            }
        });
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-watch-asusd-platform", {
            let app = app.clone();
            async move {
                run_properties_changed_watch(
                    app,
                    dbus::asusd::SERVICE_NAME,
                    dbus::asusd::PLATFORM_PATH,
                    dbus::asusd::PLATFORM_INTERFACE,
                    "asusd platform properties changed",
                )
                .await;
            }
        });
    app.state::<crate::runtime::BackendRuntime>()
        .spawn_task("dashboard-watch-ppd-profile", {
            let app = app.clone();
            async move {
                run_properties_changed_watch(
                    app,
                    dbus::ppd::SERVICE_NAME,
                    dbus::ppd::PATH,
                    dbus::ppd::INTERFACE,
                    "ppd profile properties changed",
                )
                .await;
            }
        });
}

#[deprecated(note = "use start_dashboard_watchers")]
#[allow(dead_code)]
pub fn start_dashboard_refresh_loop(app: tauri::AppHandle<Wry>) {
    start_dashboard_watchers(app);
}

#[deprecated(note = "use start_dashboard_watchers")]
#[allow(dead_code)]
pub fn start_gpu_mode_watch_loop(app: tauri::AppHandle<Wry>) {
    start_dashboard_watchers(app);
}

async fn run_fallback_refresh_loop(app: tauri::AppHandle<Wry>) {
    let mut interval = tokio::time::interval(FALLBACK_REFRESH_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval.tick().await;
    loop {
        interval.tick().await;
        refresh_dashboard_if_changed(&app, "low-frequency fallback refresh").await;
    }
}

async fn run_performance_refresh_loop(app: tauri::AppHandle<Wry>) {
    let mut interval = tokio::time::interval(PERFORMANCE_REFRESH_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval.tick().await;
    loop {
        interval.tick().await;
        refresh_performance_if_changed(&app).await;
    }
}

async fn run_proxy_signal_watch(
    app: tauri::AppHandle<Wry>,
    service: &'static str,
    path: &'static str,
    interface: &'static str,
    member: &'static str,
    reason: &'static str,
) {
    loop {
        let runtime = app.state::<crate::runtime::BackendRuntime>();
        let connection = match runtime.system_bus().await {
            Ok(connection) => connection,
            Err(error) => {
                tracing::warn!(
                    service,
                    interface,
                    member,
                    error = %error,
                    "dashboard watcher failed to acquire async system bus"
                );
                tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
                continue;
            }
        };

        let proxy = match zbus::Proxy::new(&connection, service, path, interface).await {
            Ok(proxy) => proxy,
            Err(error) => {
                tracing::warn!(
                    service,
                    path,
                    interface,
                    member,
                    error = %error,
                    "dashboard watcher failed to create D-Bus signal proxy"
                );
                tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
                continue;
            }
        };

        let mut stream = match proxy.receive_signal(member).await {
            Ok(stream) => stream,
            Err(error) => {
                tracing::warn!(
                    service,
                    interface,
                    member,
                    error = %error,
                    "dashboard watcher failed to subscribe to signal"
                );
                tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
                continue;
            }
        };

        while stream.next().await.is_some() {
            refresh_dashboard_if_changed(&app, reason).await;
        }
        tracing::debug!(
            service,
            interface,
            member,
            "dashboard signal stream ended, reconnecting"
        );
        tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
    }
}

async fn run_properties_changed_watch(
    app: tauri::AppHandle<Wry>,
    service: &'static str,
    path: &'static str,
    watched_interface: &'static str,
    reason: &'static str,
) {
    loop {
        let runtime = app.state::<crate::runtime::BackendRuntime>();
        let connection = match runtime.system_bus().await {
            Ok(connection) => connection,
            Err(error) => {
                tracing::warn!(
                    service,
                    watched_interface,
                    error = %error,
                    "dashboard watcher failed to acquire async system bus"
                );
                tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
                continue;
            }
        };

        let rule = match zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender(service)
            .context("invalid D-Bus sender for property signal watcher")
            .and_then(|builder| {
                builder
                    .path(path)
                    .context("invalid D-Bus path for property signal watcher")
            })
            .and_then(|builder| {
                builder
                    .interface("org.freedesktop.DBus.Properties")
                    .context("invalid properties interface for property signal watcher")
            })
            .and_then(|builder| {
                builder
                    .member("PropertiesChanged")
                    .context("invalid PropertiesChanged member for property signal watcher")
            })
            .and_then(|builder| {
                builder
                    .add_arg(watched_interface)
                    .context("invalid watched interface argument for property signal watcher")
            }) {
            Ok(builder) => builder.build(),
            Err(error) => {
                tracing::warn!(
                    service,
                    path,
                    watched_interface,
                    error = %error,
                    "dashboard watcher has invalid property signal match rule"
                );
                return;
            }
        };

        let mut stream =
            match zbus::MessageStream::for_match_rule(rule, &connection, Some(32)).await {
                Ok(stream) => stream,
                Err(error) => {
                    tracing::warn!(
                        service,
                        path,
                        watched_interface,
                        error = %error,
                        "dashboard watcher failed to subscribe to PropertiesChanged"
                    );
                    tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
                    continue;
                }
            };

        while let Some(message) = stream.next().await {
            match message {
                Ok(_) => refresh_dashboard_if_changed(&app, reason).await,
                Err(error) => {
                    tracing::warn!(
                        service,
                        path,
                        watched_interface,
                        error = %error,
                        "dashboard watcher property stream error, reconnecting"
                    );
                    break;
                }
            }
        }
        tokio::time::sleep(WATCH_RECONNECT_DELAY).await;
    }
}

async fn refresh_dashboard_if_changed(app: &tauri::AppHandle<Wry>, reason: &'static str) {
    let snapshot = match tokio::task::spawn_blocking(collect_dashboard_resilient).await {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "dashboard refresh worker panicked");
            return;
        }
    };

    let state = app.state::<AppState>();
    let should_emit = state
        .get_dashboard()
        .map_or(true, |current| current != snapshot);
    if !should_emit {
        return;
    }

    if let Err(error) = crate::runtime::publish_dashboard_update(app, &snapshot) {
        tracing::warn!(reason, error = %error, "failed to publish refreshed dashboard snapshot");
    }
}

async fn refresh_performance_if_changed(app: &tauri::AppHandle<Wry>) {
    let performance = match tokio::task::spawn_blocking(crate::services::performance::read_snapshot).await {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "performance refresh worker panicked");
            return;
        }
    };

    let state = app.state::<AppState>();
    let mut snapshot = match state.get_dashboard() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "failed to read current dashboard snapshot");
            return;
        }
    };
    if snapshot.performance == performance {
        return;
    }

    snapshot.performance = performance;
    snapshot.updated_at_ms = now_ms();
    if let Err(error) = crate::runtime::publish_dashboard_update(app, &snapshot) {
        tracing::warn!(error = %error, "failed to publish realtime performance snapshot");
    }
}

fn probe_with_note(result: AppResult<bool>, note: &str, errors: &mut Vec<String>) -> bool {
    match result {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(probe = note, error = %error, "dashboard probe failed");
            errors.push(format!("{note}: {error}"));
            false
        }
    }
}

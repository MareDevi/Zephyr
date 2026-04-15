use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, Context};
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt as AutoStartExt;
use tauri_plugin_notification::NotificationExt;

use crate::error::AppResult;
use crate::ipc::dto::DashboardSnapshotDto;
use crate::state::AppState;

const TRAY_ID: &str = "zephyr-main-tray";
const MENU_SHOW_WINDOW: &str = "tray:show-window";
const MENU_HIDE_WINDOW: &str = "tray:hide-window";
const MENU_QUIT: &str = "tray:quit";
const GPU_PREFIX: &str = "tray:gpu:";
const POWER_PREFIX: &str = "tray:power:";
const PLATFORM_PREFIX: &str = "tray:platform:";
const DEFAULT_MINIMIZE_TO_TRAY: bool = true;
const DEFAULT_ENABLE_AUTOSTART: bool = false;

pub struct TrayState {
    minimize_to_tray_enabled: AtomicBool,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            minimize_to_tray_enabled: AtomicBool::new(DEFAULT_MINIMIZE_TO_TRAY),
        }
    }
}

impl TrayState {
    pub fn minimize_to_tray_enabled(&self) -> bool {
        self.minimize_to_tray_enabled.load(Ordering::Relaxed)
    }
}

pub fn init(app: &tauri::App<Wry>) -> AppResult<()> {
    let app_handle = app.handle().clone();
    let snapshot = app
        .state::<AppState>()
        .get_dashboard()
        .context("tray: failed to read initial dashboard snapshot")?;
    let autostart_enabled = app_handle.autolaunch().is_enabled().unwrap_or(false);
    if DEFAULT_ENABLE_AUTOSTART && !autostart_enabled {
        if let Err(error) = app_handle.autolaunch().enable() {
            tracing::warn!(error = %error, "tray: failed to enable autostart default");
        }
    }

    let menu = build_menu(&app_handle, &snapshot)?;
    let tooltip = tooltip_for_snapshot(&snapshot);
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
        .context("tray: failed to load tray icon image")?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(false)
        .tooltip(tooltip)
        .menu(&menu)
        .on_menu_event(|app, event| {
            if let Err(error) = handle_menu_event(app, event.id.as_ref()) {
                tracing::warn!(error = %error, menu_id = ?event.id, "tray: menu action failed");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .context("tray: failed to build tray icon")?;

    Ok(())
}

pub fn sync_from_snapshot(
    app: &tauri::AppHandle<Wry>,
    snapshot: &DashboardSnapshotDto,
) -> AppResult<()> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| anyhow!("tray: tray icon not initialized"))?;
    let menu = build_menu(app, snapshot)?;
    tray.set_menu(Some(menu))
        .context("tray: failed to update tray menu")?;
    tray.set_tooltip(Some(tooltip_for_snapshot(snapshot)))
        .context("tray: failed to update tray tooltip")?;
    Ok(())
}

pub fn should_minimize_to_tray(app: &tauri::AppHandle<Wry>) -> bool {
    app.state::<TrayState>().minimize_to_tray_enabled()
}

fn handle_menu_event(app: &tauri::AppHandle<Wry>, id: &str) -> AppResult<()> {
    match id {
        MENU_SHOW_WINDOW => {
            if let Some(window) = app.get_webview_window("main") {
                window.show().context("tray: failed to show main window")?;
                window
                    .unminimize()
                    .context("tray: failed to unminimize main window")?;
                window
                    .set_focus()
                    .context("tray: failed to focus main window")?;
            }
            Ok(())
        }
        MENU_HIDE_WINDOW => {
            if let Some(window) = app.get_webview_window("main") {
                window.hide().context("tray: failed to hide main window")?;
            }
            Ok(())
        }
        MENU_QUIT => {
            app.exit(0);
            Ok(())
        }
        _ if id.starts_with(GPU_PREFIX) => {
            let mode = id.trim_start_matches(GPU_PREFIX);
            run_control_action(
                app,
                &format!("GPU mode set to {mode}"),
                &format!("Failed to set GPU mode to {mode}"),
                || crate::services::gpu::set_mode(app, mode),
            )
        }
        _ if id.starts_with(POWER_PREFIX) => {
            let profile = id.trim_start_matches(POWER_PREFIX);
            run_control_action(
                app,
                &format!("Power profile set to {profile}"),
                &format!("Failed to set power profile to {profile}"),
                || crate::services::power::set_active_profile(app, profile),
            )
        }
        _ if id.starts_with(PLATFORM_PREFIX) => {
            let profile = id.trim_start_matches(PLATFORM_PREFIX);
            run_control_action(
                app,
                &format!("Platform profile set to {profile}"),
                &format!("Failed to set platform profile to {profile}"),
                || crate::services::platform::set_platform_profile(app, profile, false, false),
            )
        }
        _ => Ok(()),
    }
}

fn run_control_action<F>(
    app: &tauri::AppHandle<Wry>,
    success_message: &str,
    failure_message: &str,
    action: F,
) -> AppResult<()>
where
    F: FnOnce() -> AppResult<DashboardSnapshotDto>,
{
    match action() {
        Ok(snapshot) => {
            publish_snapshot(app, snapshot)?;
            notify(app, "AsusTone", success_message);
            Ok(())
        }
        Err(error) => {
            notify(app, "AsusTone", &format!("{failure_message}: {error}"));
            Err(error)
        }
    }
}

fn publish_snapshot(app: &tauri::AppHandle<Wry>, snapshot: DashboardSnapshotDto) -> AppResult<()> {
    app.state::<AppState>()
        .set_dashboard(snapshot.clone())
        .context("tray: failed to persist dashboard snapshot")?;
    app.emit(crate::ipc::events::DASHBOARD_UPDATED_EVENT, &snapshot)
        .map_err(|error| anyhow!("tray: failed to emit dashboard event: {error}"))?;
    sync_from_snapshot(app, &snapshot)
}

fn build_menu(
    app: &tauri::AppHandle<Wry>,
    snapshot: &DashboardSnapshotDto,
) -> AppResult<Menu<Wry>> {
    let status_text = MenuItem::with_id(
        app,
        "tray:status",
        status_line(snapshot),
        false,
        None::<&str>,
    )
    .context("tray: failed to build status menu item")?;
    let daemon_text = MenuItem::with_id(
        app,
        "tray:daemon-status",
        daemon_health_line(snapshot),
        false,
        None::<&str>,
    )
    .context("tray: failed to build daemon status menu item")?;

    let gpu_menu = build_profile_submenu(
        app,
        "GPU mode",
        GPU_PREFIX,
        snapshot.gpu.mode.as_deref(),
        extract_options(
            snapshot.gpu.supported_modes.as_deref(),
            &[
                "Hybrid",
                "Integrated",
                "NvidiaNoModeset",
                "Vfio",
                "AsusEgpu",
                "AsusMuxDgpu",
            ],
        ),
    )?;
    let power_menu = build_profile_submenu(
        app,
        "Power profile",
        POWER_PREFIX,
        snapshot.power.active_profile.as_deref(),
        extract_options(
            snapshot.power.profiles.as_deref(),
            &["power-saver", "balanced", "performance"],
        ),
    )?;
    let platform_menu = build_profile_submenu(
        app,
        "Platform profile",
        PLATFORM_PREFIX,
        snapshot.platform.platform_profile.as_deref(),
        extract_options(
            snapshot.platform.platform_profile_choices.as_deref(),
            &["balanced", "performance", "quiet", "low-power", "custom"],
        ),
    )?;

    let show_item = MenuItem::with_id(app, MENU_SHOW_WINDOW, "Show Window", true, None::<&str>)
        .context("tray: failed to build show-window item")?;
    let hide_item = MenuItem::with_id(app, MENU_HIDE_WINDOW, "Hide Window", true, None::<&str>)
        .context("tray: failed to build hide-window item")?;
    let quit_item = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)
        .context("tray: failed to build quit item")?;
    let separator_top =
        PredefinedMenuItem::separator(app).context("tray: failed to build top separator")?;
    let separator_bottom =
        PredefinedMenuItem::separator(app).context("tray: failed to build bottom separator")?;

    let items: [&dyn IsMenuItem<Wry>; 10] = [
        &status_text,
        &daemon_text,
        &separator_top,
        &gpu_menu,
        &power_menu,
        &platform_menu,
        &show_item,
        &hide_item,
        &separator_bottom,
        &quit_item,
    ];
    Menu::with_items(app, &items).context("tray: failed to compose tray menu")
}

fn build_profile_submenu(
    app: &tauri::AppHandle<Wry>,
    title: &str,
    prefix: &str,
    current: Option<&str>,
    options: Vec<String>,
) -> AppResult<Submenu<Wry>> {
    let mut menu_items: Vec<CheckMenuItem<Wry>> = Vec::new();
    let current_normalized = current.map(|value| value.to_ascii_lowercase());

    for option in options {
        let checked = current_normalized
            .as_deref()
            .is_some_and(|selected| selected == option.to_ascii_lowercase());
        let id = format!("{prefix}{option}");
        menu_items.push(
            CheckMenuItem::with_id(app, id, option, true, checked, None::<&str>)
                .context("tray: failed to build profile menu item")?,
        );
    }

    let item_refs: Vec<&dyn IsMenuItem<Wry>> = menu_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<Wry>)
        .collect();
    Submenu::with_items(app, title, true, &item_refs)
        .context("tray: failed to build profile submenu")
}

fn extract_options(raw: Option<&str>, fallback: &[&str]) -> Vec<String> {
    if let Some(text) = raw {
        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();
        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                current.push(ch);
            } else if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        }
        if !current.is_empty() {
            tokens.push(current);
        }
        tokens.sort();
        tokens.dedup();
        if !tokens.is_empty() {
            return tokens;
        }
    }
    fallback.iter().map(|item| (*item).to_owned()).collect()
}

fn status_line(snapshot: &DashboardSnapshotDto) -> String {
    format!(
        "GPU: {} | Power: {} | Platform: {}",
        snapshot.gpu.mode.as_deref().unwrap_or("unknown"),
        snapshot
            .power
            .active_profile
            .as_deref()
            .unwrap_or("unknown"),
        snapshot
            .platform
            .platform_profile
            .as_deref()
            .unwrap_or("unknown"),
    )
}

fn daemon_health_line(snapshot: &DashboardSnapshotDto) -> String {
    format!(
        "asusd:{} supergfxd:{} ppd:{}",
        yes_no(snapshot.health.asusd_available),
        yes_no(snapshot.health.supergfxd_available),
        yes_no(snapshot.health.ppd_available),
    )
}

fn tooltip_for_snapshot(snapshot: &DashboardSnapshotDto) -> String {
    let cpu = snapshot
        .performance
        .cpu
        .utilization_percent
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "unknown".to_owned());
    let gpu = snapshot
        .performance
        .gpu
        .utilization_percent
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "unknown".to_owned());
    format!(
        "AsusTone\nCPU: {cpu} | GPU: {gpu}\n{}\n{}",
        status_line(snapshot),
        daemon_health_line(snapshot)
    )
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "down"
    }
}

fn notify(app: &tauri::AppHandle<Wry>, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        tracing::warn!(error = %error, "tray: failed to show notification");
    }
}

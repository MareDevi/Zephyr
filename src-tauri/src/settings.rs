use std::sync::RwLock;

use anyhow::{anyhow, Context};
use tauri::Wry;
use tauri_plugin_autostart::ManagerExt as AutoStartExt;
use tauri_plugin_store::StoreExt;

use crate::error::AppResult;
use crate::ipc::dto::{ColorMode, LaunchBehavior, SettingsSnapshotDto, ThemeId, ThemeKind};

const SETTINGS_STORE_FILENAME: &str = "settings.json";
const SETTINGS_STORE_KEY: &str = "app";

pub struct SettingsState {
    settings: RwLock<SettingsSnapshotDto>,
}

impl SettingsState {
    pub fn new(settings: SettingsSnapshotDto) -> Self {
        Self {
            settings: RwLock::new(settings),
        }
    }

    pub fn get(&self) -> AppResult<SettingsSnapshotDto> {
        let guard = self
            .settings
            .read()
            .map_err(|_| anyhow!("state lock poisoned while reading settings"))?;
        Ok(guard.clone())
    }

    pub fn set(&self, next: SettingsSnapshotDto) -> AppResult<()> {
        let mut guard = self
            .settings
            .write()
            .map_err(|_| anyhow!("state lock poisoned while writing settings"))?;
        *guard = next;
        Ok(())
    }
}

pub fn load_settings(app: &tauri::AppHandle<Wry>) -> AppResult<SettingsSnapshotDto> {
    let store = app
        .store(SETTINGS_STORE_FILENAME)
        .context("settings: failed to open settings store")?;
    let Some(value) = store.get(SETTINGS_STORE_KEY.to_owned()) else {
        return Ok(SettingsSnapshotDto::default());
    };
    let settings: SettingsSnapshotDto =
        serde_json::from_value(value).context("settings: failed to parse store value")?;
    Ok(normalize_settings(settings))
}

pub fn save_settings(app: &tauri::AppHandle<Wry>, settings: &SettingsSnapshotDto) -> AppResult<()> {
    let store = app
        .store(SETTINGS_STORE_FILENAME)
        .context("settings: failed to open settings store")?;
    let value = serde_json::to_value(settings).context("settings: failed to serialize settings")?;
    store.set(SETTINGS_STORE_KEY.to_owned(), value);
    store.save().context("settings: failed to persist settings")
}

pub fn apply_autostart(app: &tauri::AppHandle<Wry>, enabled: bool) -> AppResult<()> {
    let manager = app.autolaunch();
    let current_enabled = manager
        .is_enabled()
        .context("settings: failed to query start-on-boot status")?;
    if enabled == current_enabled {
        return Ok(());
    }
    if enabled {
        manager
            .enable()
            .context("settings: failed to enable start-on-boot")?;
    } else {
        manager
            .disable()
            .context("settings: failed to disable start-on-boot")?;
    }
    Ok(())
}

pub fn set_autostart_enabled(
    app: &tauri::AppHandle<Wry>,
    state: &SettingsState,
    enabled: bool,
) -> AppResult<SettingsSnapshotDto> {
    let mut settings = state.get()?;
    apply_autostart(app, enabled)?;
    settings.autostart_enabled = enabled;
    save_settings(app, &settings)?;
    state.set(settings.clone())?;
    Ok(settings)
}

pub fn set_launch_behavior(
    app: &tauri::AppHandle<Wry>,
    state: &SettingsState,
    launch_behavior: LaunchBehavior,
) -> AppResult<SettingsSnapshotDto> {
    let mut settings = state.get()?;
    settings.launch_behavior = launch_behavior;
    save_settings(app, &settings)?;
    state.set(settings.clone())?;
    Ok(settings)
}

pub fn set_theme(
    app: &tauri::AppHandle<Wry>,
    state: &SettingsState,
    theme_kind: ThemeKind,
    theme_id: ThemeId,
    accent_color: Option<String>,
    color_mode: ColorMode,
) -> AppResult<SettingsSnapshotDto> {
    let mut settings = state.get()?;
    if let Some(color) = accent_color.as_ref() {
        if !is_valid_hex_color(color) {
            return Err(anyhow!("settings: accentColor must be a #RRGGBB hex color"));
        }
    }
    if theme_kind == ThemeKind::Catppuccin && theme_id == ThemeId::Default {
        return Err(anyhow!(
            "settings: catppuccin theme requires latte, frappe, macchiato, or mocha"
        ));
    }

    settings.theme_kind = theme_kind;
    settings.theme_id = if theme_kind == ThemeKind::Heroui {
        ThemeId::Default
    } else {
        theme_id
    };
    settings.accent_color = if theme_kind == ThemeKind::Heroui {
        accent_color
    } else {
        None
    };
    settings.color_mode = color_mode;
    save_settings(app, &settings)?;
    state.set(settings.clone())?;
    Ok(settings)
}

fn is_valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value
            .chars()
            .skip(1)
            .all(|ch| ch.is_ascii_hexdigit())
}

fn normalize_settings(mut settings: SettingsSnapshotDto) -> SettingsSnapshotDto {
    if settings.theme_kind == ThemeKind::Heroui && settings.theme_id != ThemeId::Default {
        settings.theme_kind = ThemeKind::Catppuccin;
    }
    if settings.theme_kind == ThemeKind::Heroui {
        settings.theme_id = ThemeId::Default;
    } else {
        settings.accent_color = None;
    }
    settings
}

use tauri::Emitter;
use tauri::Manager;
use tauri::State;

use crate::error::ApiError;
use crate::ipc::dto::{
    BacklightStatusSnapshot, BatteryOneShotRequest, BootstrapSnapshotDto, DashboardSnapshotDto,
    SettingsSnapshotDto, SetAutostartRequest, SetLaunchBehaviorRequest,
    ReadFanCurvesRequest, ResetFanCurvesRequest, ScsiStatusSnapshot, SetAnimeBrightnessRequest,
    SetAnimeBuiltinsEnabledRequest, SetAnimeDisplayEnabledRequest, SetAnimeOffConditionRequest,
    SetArmouryValueRequest, SetAuraBrightnessRequest, SetAuraModeRequest, SetBacklightRequest,
    SetChargeLimitRequest, SetFanCurveRequest, SetFanCurvesEnabledRequest, SetGpuModeRequest, SetThemeRequest,
    SetLedsBrightnessRequest,
    SetPlatformProfileRequest, SetPowerProfileRequest, SetScsiEnabledRequest, SetScsiModeRequest,
    SetSlashBrightnessRequest, SetSlashEnabledRequest, SetSlashFlagRequest,
    SetSlashIntervalRequest, SetSlashModeRequest,
};
use crate::services::dashboard;
use crate::settings::SettingsState;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn get_bootstrap_snapshot(
    state: State<'_, AppState>,
) -> Result<BootstrapSnapshotDto, ApiError> {
    let health = state.get_health().map_err(ApiError::from)?;
    Ok(BootstrapSnapshotDto { health })
}

#[tauri::command]
#[specta::specta]
pub fn refresh_bootstrap_snapshot(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<BootstrapSnapshotDto, ApiError> {
    let snapshot = dashboard::collect_dashboard_resilient();
    state
        .set_dashboard(snapshot.clone())
        .map_err(ApiError::from)?;
    app.emit(crate::ipc::events::DASHBOARD_UPDATED_EVENT, &snapshot)
        .map_err(|error| ApiError::from(anyhow::anyhow!("event emit failed: {error}")))?;
    let health = snapshot.health;
    Ok(BootstrapSnapshotDto { health })
}

#[tauri::command]
#[specta::specta]
pub fn get_dashboard_snapshot(
    state: State<'_, AppState>,
) -> Result<DashboardSnapshotDto, ApiError> {
    state.get_dashboard().map_err(ApiError::from)
}

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, SettingsState>) -> Result<SettingsSnapshotDto, ApiError> {
    state.get().map_err(ApiError::from)
}

#[tauri::command]
#[specta::specta]
pub fn set_autostart(
    request: SetAutostartRequest,
    state: State<'_, SettingsState>,
    app: tauri::AppHandle,
) -> Result<SettingsSnapshotDto, ApiError> {
    let settings = crate::settings::set_autostart_enabled(&app, &state, request.enabled)
        .map_err(ApiError::from)?;
    let snapshot = app
        .state::<AppState>()
        .get_dashboard()
        .map_err(ApiError::from)?;
    crate::tray::sync_from_snapshot(&app, &snapshot)
        .map_err(|error| ApiError::from(anyhow::anyhow!("tray sync failed: {error}")))?;
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub fn set_launch_behavior(
    request: SetLaunchBehaviorRequest,
    state: State<'_, SettingsState>,
    app: tauri::AppHandle,
) -> Result<SettingsSnapshotDto, ApiError> {
    crate::settings::set_launch_behavior(&app, &state, request.launch_behavior)
        .map_err(ApiError::from)
}

#[tauri::command]
#[specta::specta]
pub fn set_theme(
    request: SetThemeRequest,
    state: State<'_, SettingsState>,
    app: tauri::AppHandle,
) -> Result<SettingsSnapshotDto, ApiError> {
    crate::settings::set_theme(
        &app,
        &state,
        request.theme_kind,
        request.theme_id,
        request.accent_color,
        request.color_mode,
    )
        .map_err(ApiError::from)
}

#[tauri::command]
#[specta::specta]
pub fn refresh_dashboard_snapshot(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let snapshot = dashboard::collect_dashboard_resilient();
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_gpu_mode(
    request: SetGpuModeRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::gpu::set_mode_async(&runtime, &request.mode)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_power_profile(
    request: SetPowerProfileRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::power::set_active_profile_async(&runtime, &request.profile)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_platform_profile(
    request: SetPlatformProfileRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::platform::set_platform_profile_async(
        &runtime,
        &request.profile,
        request.ac,
        request.battery,
    )
    .await
    .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn next_platform_profile(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::platform::next_platform_profile_async(&runtime)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_charge_limit(
    request: SetChargeLimitRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::platform::set_charge_limit_async(&runtime, request.percent)
        .await
        .map_err(ApiError::from)?;
    let snapshot = dashboard::collect_dashboard_resilient();
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn battery_one_shot_charge(
    request: BatteryOneShotRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::platform::battery_one_shot_charge_async(&runtime, request.percent)
        .await
        .map_err(ApiError::from)?;
    let snapshot = dashboard::collect_dashboard_resilient();
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub fn read_fan_curves(
    request: ReadFanCurvesRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let snapshot =
        crate::services::fan_curves::read_profile(&request.profile).map_err(ApiError::from)?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn reset_fan_curves(
    request: ResetFanCurvesRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::fan_curves::reset_to_defaults_async(&runtime, &request.profile)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_fan_curves_enabled(
    request: SetFanCurvesEnabledRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::fan_curves::set_enabled_async(&runtime, &request.profile, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_fan_curve(
    request: SetFanCurveRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::fan_curves::set_curve_async(
        &runtime,
        &request.profile,
        &request.fan,
        &request.points,
        request.enabled,
    )
    .await
    .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_aura_brightness(
    request: SetAuraBrightnessRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_aura_brightness_async(&runtime, request.level)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_aura_mode(
    request: SetAuraModeRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_aura_mode_async(&runtime, &request.mode)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_leds_brightness(
    request: SetLedsBrightnessRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_leds_brightness_async(&runtime, &request.level)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn next_leds_brightness(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::next_leds_brightness_async(&runtime)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn prev_leds_brightness(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::prev_leds_brightness_async(&runtime)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_display_enabled(
    request: SetAnimeDisplayEnabledRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_display_enabled_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_brightness(
    request: SetAnimeBrightnessRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_brightness_async(&runtime, request.level)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_builtins_enabled(
    request: SetAnimeBuiltinsEnabledRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_builtins_enabled_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_off_when_lid_closed(
    request: SetAnimeOffConditionRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_off_when_lid_closed_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_off_when_suspended(
    request: SetAnimeOffConditionRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_off_when_suspended_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_anime_off_when_unplugged(
    request: SetAnimeOffConditionRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::anime::set_off_when_unplugged_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_enabled(
    request: SetSlashEnabledRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_enabled_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_brightness(
    request: SetSlashBrightnessRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_brightness_async(&runtime, request.brightness)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_interval(
    request: SetSlashIntervalRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_interval_async(&runtime, request.interval)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_mode(
    request: SetSlashModeRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_mode_async(&runtime, &request.mode)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_on_boot(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_on_boot_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_on_shutdown(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_on_shutdown_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_on_sleep(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_on_sleep_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_on_battery(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_on_battery_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_battery_warning(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_battery_warning_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_slash_show_on_lid_closed(
    request: SetSlashFlagRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::aura::set_slash_show_on_lid_closed_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_armoury_value(
    request: SetArmouryValueRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::armoury::set_current_value_async(&runtime, &request.path, request.value)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_backlight(
    request: SetBacklightRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::backlight::set_async(
        &runtime,
        request.screenpad_brightness,
        request.screenpad_gamma,
        request.sync_screenpad_brightness,
    )
    .await
    .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub fn read_backlight_snapshot(
    state: State<'_, AppState>,
) -> Result<BacklightStatusSnapshot, ApiError> {
    state
        .get_dashboard()
        .map(|snapshot| snapshot.backlight)
        .map_err(ApiError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn set_scsi_enabled(
    request: SetScsiEnabledRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::scsi::set_enabled_async(&runtime, request.enabled)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn set_scsi_mode(
    request: SetScsiModeRequest,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DashboardSnapshotDto, ApiError> {
    let runtime = app.state::<crate::runtime::BackendRuntime>();
    crate::services::scsi::set_mode_async(&runtime, request.mode)
        .await
        .map_err(ApiError::from)?;
    let snapshot = collect_dashboard_snapshot_async().await?;
    emit_dashboard_snapshot(&state, &app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub fn read_scsi_snapshot(state: State<'_, AppState>) -> Result<ScsiStatusSnapshot, ApiError> {
    state
        .get_dashboard()
        .map(|snapshot| snapshot.scsi)
        .map_err(ApiError::from)
}

fn emit_dashboard_snapshot(
    state: &State<'_, AppState>,
    app: &tauri::AppHandle,
    snapshot: &DashboardSnapshotDto,
) -> Result<(), ApiError> {
    state
        .set_dashboard(snapshot.clone())
        .map_err(ApiError::from)?;
    app.emit(crate::ipc::events::DASHBOARD_UPDATED_EVENT, &snapshot)
        .map_err(|error| ApiError::from(anyhow::anyhow!("event emit failed: {error}")))?;
    crate::tray::sync_from_snapshot(app, snapshot)
        .map_err(|error| ApiError::from(anyhow::anyhow!("tray sync failed: {error}")))
}

async fn collect_dashboard_snapshot_async() -> Result<DashboardSnapshotDto, ApiError> {
    tokio::task::spawn_blocking(dashboard::collect_dashboard_resilient)
        .await
        .map_err(|error| ApiError::from(anyhow::anyhow!("dashboard refresh worker failed: {error}")))
}

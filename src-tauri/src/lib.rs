mod dbus;
mod error;
mod ipc;
mod logging;
mod runtime;
mod services;
mod settings;
mod state;
mod tray;

use specta_typescript::Typescript;
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = logging::init_logging() {
        eprintln!("failed to initialize logging: {error}");
    }
    tracing::info!("starting zephyr backend");

    let initial_dashboard = services::dashboard::collect_dashboard_resilient();
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        ipc::commands::get_bootstrap_snapshot,
        ipc::commands::refresh_bootstrap_snapshot,
        ipc::commands::get_dashboard_snapshot,
        ipc::commands::get_settings,
        ipc::commands::set_autostart,
        ipc::commands::set_launch_behavior,
        ipc::commands::set_theme,
        ipc::commands::refresh_dashboard_snapshot,
        ipc::commands::set_gpu_mode,
        ipc::commands::set_power_profile,
        ipc::commands::set_platform_profile,
        ipc::commands::next_platform_profile,
        ipc::commands::set_charge_limit,
        ipc::commands::battery_one_shot_charge,
        ipc::commands::read_fan_curves,
        ipc::commands::reset_fan_curves,
        ipc::commands::set_fan_curves_enabled,
        ipc::commands::set_fan_curve,
        ipc::commands::set_aura_brightness,
        ipc::commands::set_aura_mode,
        ipc::commands::set_leds_brightness,
        ipc::commands::next_leds_brightness,
        ipc::commands::prev_leds_brightness,
        ipc::commands::set_anime_display_enabled,
        ipc::commands::set_anime_brightness,
        ipc::commands::set_anime_builtins_enabled,
        ipc::commands::set_anime_off_when_lid_closed,
        ipc::commands::set_anime_off_when_suspended,
        ipc::commands::set_anime_off_when_unplugged,
        ipc::commands::set_slash_enabled,
        ipc::commands::set_slash_brightness,
        ipc::commands::set_slash_interval,
        ipc::commands::set_slash_mode,
        ipc::commands::set_slash_show_on_boot,
        ipc::commands::set_slash_show_on_shutdown,
        ipc::commands::set_slash_show_on_sleep,
        ipc::commands::set_slash_show_on_battery,
        ipc::commands::set_slash_show_battery_warning,
        ipc::commands::set_slash_show_on_lid_closed,
        ipc::commands::set_backlight,
        ipc::commands::read_backlight_snapshot,
        ipc::commands::set_scsi_enabled,
        ipc::commands::set_scsi_mode,
        ipc::commands::read_scsi_snapshot,
        ipc::commands::set_armoury_value
    ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(tray::TrayState::default())
        .manage(state::AppState::new(initial_dashboard))
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. })
                && window.label() == "main"
                && tray::should_minimize_to_tray(window.app_handle())
            {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = window.hide() {
                        tracing::warn!(error = %error, "failed to hide main window to tray");
                    }
                }
            }
        })
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            let settings = settings::load_settings(app.handle())?;
            settings::apply_autostart(app.handle(), settings.autostart_enabled)?;
            let launch_behavior = settings.launch_behavior;
            app.manage(settings::SettingsState::new(settings));
            tray::init(app)?;
            if launch_behavior == ipc::dto::LaunchBehavior::Silent {
                if let Some(window) = app.get_webview_window("main") {
                    window
                        .hide()
                        .map_err(|error| anyhow::anyhow!("settings: failed to hide startup window: {error}"))?;
                }
            }
            app.manage(runtime::BackendRuntime::new()?);
            let app_handle = app.handle().clone();
            app.state::<runtime::BackendRuntime>().spawn_task(
                "dbus-system-bus-warmup",
                async move {
                    let runtime = app_handle.state::<runtime::BackendRuntime>();
                    if let Err(error) = runtime.system_bus().await {
                        tracing::debug!(error = %error, "failed to warm async D-Bus connection");
                    }
                },
            );
            services::dashboard::start_dashboard_watchers(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

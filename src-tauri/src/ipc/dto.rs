use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DaemonHealthSnapshot {
    pub asusd_available: bool,
    pub supergfxd_available: bool,
    pub ppd_available: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapSnapshotDto {
    pub health: DaemonHealthSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct DaemonInterfaceSnapshot {
    pub asusd_platform_available: bool,
    pub asusd_fan_curves_available: bool,
    pub asusd_aura_available: bool,
    pub asusd_anime_available: bool,
    pub asusd_slash_available: bool,
    pub asusd_scsi_available: bool,
    pub asusd_backlight_available: bool,
    pub asusd_armoury_available: bool,
    pub supergfxd_interface_available: bool,
    pub ppd_interface_available: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuStatusSnapshot {
    pub mode: Option<String>,
    pub power: Option<String>,
    pub pending_mode: Option<String>,
    pub pending_action: Option<String>,
    pub daemon_version: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PowerStatusSnapshot {
    pub active_profile: Option<String>,
    pub profiles: Option<String>,
    pub performance_degraded: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatusSnapshot {
    pub platform_profile: Option<String>,
    pub platform_profile_choices: Option<String>,
    pub platform_profile_on_ac: Option<String>,
    pub platform_profile_on_battery: Option<String>,
    pub charge_control_end_threshold: Option<u8>,
    pub platform_profile_linked_epp: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct FanCurvesStatusSnapshot {
    pub active_profile: Option<String>,
    pub profile_choices: Option<String>,
    pub curve_data: Option<String>,
    pub curve_series: Vec<FanCurveSeriesSnapshot>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct FanCurvePointSnapshot {
    pub temperature: u8,
    pub pwm: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct FanCurveSeriesSnapshot {
    pub fan: String,
    pub enabled: bool,
    pub points: Vec<FanCurvePointSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct CpuPerformanceSnapshot {
    pub frequency_mhz: Option<u32>,
    pub utilization_percent: Option<f32>,
    pub core_count: Option<u16>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuPerformanceSnapshot {
    pub frequency_mhz: Option<u32>,
    pub utilization_percent: Option<f32>,
    pub source: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct RamPerformanceSnapshot {
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub cpu: CpuPerformanceSnapshot,
    pub gpu: GpuPerformanceSnapshot,
    pub ram: RamPerformanceSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuraStatusSnapshot {
    pub brightness: Option<String>,
    pub led_mode: Option<String>,
    pub supported_brightness: Option<String>,
    pub supported_basic_modes: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnimeStatusSnapshot {
    pub enable_display: Option<bool>,
    pub builtins_enabled: Option<bool>,
    pub brightness: Option<String>,
    pub off_when_lid_closed: Option<bool>,
    pub off_when_suspended: Option<bool>,
    pub off_when_unplugged: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SlashStatusSnapshot {
    pub enabled: Option<bool>,
    pub brightness: Option<u8>,
    pub interval: Option<u8>,
    pub mode: Option<String>,
    pub show_on_boot: Option<bool>,
    pub show_on_sleep: Option<bool>,
    pub show_on_shutdown: Option<bool>,
    pub show_on_battery: Option<bool>,
    pub show_battery_warning: Option<bool>,
    pub show_on_lid_closed: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScsiStatusSnapshot {
    pub enabled: Option<bool>,
    pub mode: Option<String>,
    pub mode_data: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct BacklightStatusSnapshot {
    pub screenpad_brightness: Option<i32>,
    pub screenpad_gamma: Option<String>,
    pub sync_screenpad_brightness: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArmouryAttributeSnapshot {
    pub path: String,
    pub name: Option<String>,
    pub available_attrs: Option<String>,
    pub current_value: Option<i32>,
    pub default_value: Option<i32>,
    pub min_value: Option<i32>,
    pub max_value: Option<i32>,
    pub possible_values: Option<String>,
    pub scalar_increment: Option<i32>,
    pub queued_gpu_value: Option<i32>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArmouryStatusSnapshot {
    pub attributes: Vec<ArmouryAttributeSnapshot>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshotDto {
    pub health: DaemonHealthSnapshot,
    pub interfaces: DaemonInterfaceSnapshot,
    pub performance: PerformanceSnapshot,
    pub gpu: GpuStatusSnapshot,
    pub power: PowerStatusSnapshot,
    pub platform: PlatformStatusSnapshot,
    pub fan_curves: FanCurvesStatusSnapshot,
    pub aura: AuraStatusSnapshot,
    pub anime: AnimeStatusSnapshot,
    pub slash: SlashStatusSnapshot,
    pub scsi: ScsiStatusSnapshot,
    pub backlight: BacklightStatusSnapshot,
    pub armoury: ArmouryStatusSnapshot,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetGpuModeRequest {
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetPowerProfileRequest {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetPlatformProfileRequest {
    pub profile: String,
    #[serde(default)]
    pub ac: bool,
    #[serde(default)]
    pub battery: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetChargeLimitRequest {
    pub percent: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReadFanCurvesRequest {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ResetFanCurvesRequest {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetFanCurvesEnabledRequest {
    pub profile: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetFanCurvePointRequest {
    pub temperature: u8,
    pub pwm: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetFanCurveRequest {
    pub profile: String,
    pub fan: String,
    pub points: Vec<SetFanCurvePointRequest>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAuraBrightnessRequest {
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAuraModeRequest {
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetLedsBrightnessRequest {
    pub level: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAnimeDisplayEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAnimeBrightnessRequest {
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAnimeOffConditionRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAnimeBuiltinsEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetSlashEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetSlashBrightnessRequest {
    pub brightness: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetSlashIntervalRequest {
    pub interval: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetSlashModeRequest {
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetSlashFlagRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetArmouryValueRequest {
    pub path: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetBacklightRequest {
    pub screenpad_brightness: Option<i32>,
    pub screenpad_gamma: Option<f32>,
    pub sync_screenpad_brightness: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetScsiEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetScsiModeRequest {
    pub mode: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BatteryOneShotRequest {
    pub percent: Option<u8>,
}

impl DashboardSnapshotDto {
    pub fn from_health(health: DaemonHealthSnapshot, updated_at_ms: u64) -> Self {
        Self {
            health,
            interfaces: DaemonInterfaceSnapshot::default(),
            performance: PerformanceSnapshot::default(),
            gpu: GpuStatusSnapshot::default(),
            power: PowerStatusSnapshot::default(),
            platform: PlatformStatusSnapshot::default(),
            fan_curves: FanCurvesStatusSnapshot::default(),
            aura: AuraStatusSnapshot::default(),
            anime: AnimeStatusSnapshot::default(),
            slash: SlashStatusSnapshot::default(),
            scsi: ScsiStatusSnapshot::default(),
            backlight: BacklightStatusSnapshot::default(),
            armoury: ArmouryStatusSnapshot::default(),
            updated_at_ms,
        }
    }
}

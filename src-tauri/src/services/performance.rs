use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::Clock;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::ipc::dto::{
    CpuPerformanceSnapshot, GpuPerformanceSnapshot, PerformanceSnapshot, RamPerformanceSnapshot,
};

pub fn read_snapshot() -> PerformanceSnapshot {
    PerformanceSnapshot {
        cpu: read_cpu_snapshot(),
        gpu: read_gpu_snapshot(),
        ram: read_ram_snapshot(),
    }
}

fn read_cpu_snapshot() -> CpuPerformanceSnapshot {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
    );
    let mut snapshot = CpuPerformanceSnapshot::default();
    let mut errors: Vec<String> = Vec::new();

    system.refresh_cpu_all();
    std::thread::sleep(Duration::from_millis(120));
    system.refresh_cpu_usage();

    let cores = system.cpus().len();
    if cores > 0 {
        snapshot.core_count = Some(cores.min(u16::MAX as usize) as u16);
        let total_frequency: u64 = system
            .cpus()
            .iter()
            .map(|cpu| u64::from(cpu.frequency()))
            .sum();
        snapshot.frequency_mhz = Some((total_frequency / cores as u64) as u32);
    } else {
        errors.push("sysinfo reported zero cpu cores".to_owned());
    }

    let usage = system.global_cpu_usage();
    if usage.is_finite() {
        snapshot.utilization_percent = Some((usage * 10.0).round() / 10.0);
    } else {
        errors.push("sysinfo returned invalid cpu utilization".to_owned());
    }

    if !errors.is_empty() {
        snapshot.last_error = Some(errors.join("; "));
    }

    snapshot
}

fn read_gpu_snapshot() -> GpuPerformanceSnapshot {
    if let Some(snapshot) = read_nvml_gpu_snapshot() {
        return snapshot;
    }

    let mut snapshot = GpuPerformanceSnapshot::default();
    let mut errors: Vec<String> = Vec::new();

    let Some(card_path) = first_gpu_card_path() else {
        snapshot.last_error = Some("no drm card found".to_owned());
        return snapshot;
    };

    match read_gpu_frequency_mhz(&card_path) {
        Ok((value, source)) => {
            snapshot.frequency_mhz = value;
            snapshot.source = Some(source);
        }
        Err(error) => errors.push(error),
    }
    match read_gpu_utilization_percent(&card_path) {
        Ok((value, source)) => {
            snapshot.utilization_percent = value;
            if snapshot.source.is_none() {
                snapshot.source = Some(source);
            }
        }
        Err(error) => errors.push(error),
    }

    if !errors.is_empty() {
        snapshot.last_error = Some(errors.join("; "));
    }
    snapshot
}

fn read_ram_snapshot() -> RamPerformanceSnapshot {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    system.refresh_memory();

    let total = system.total_memory();
    if total == 0 {
        return RamPerformanceSnapshot {
            last_error: Some("sysinfo reported zero total memory".to_owned()),
            ..RamPerformanceSnapshot::default()
        };
    }

    let used = system.used_memory().min(total);
    let utilization = (used as f64 / total as f64) * 100.0;
    RamPerformanceSnapshot {
        total_bytes: Some(total),
        used_bytes: Some(used),
        utilization_percent: Some(((utilization * 10.0).round() / 10.0) as f32),
        last_error: None,
    }
}

fn read_nvml_gpu_snapshot() -> Option<GpuPerformanceSnapshot> {
    let nvml = Nvml::init().ok()?;

    let count = match nvml.device_count() {
        Ok(count) => count,
        Err(error) => {
            return Some(GpuPerformanceSnapshot {
                source: Some("nvml".to_owned()),
                last_error: Some(format!("nvml device count query failed: {error}")),
                ..GpuPerformanceSnapshot::default()
            });
        }
    };
    if count == 0 {
        return None;
    }

    let device = match nvml.device_by_index(0) {
        Ok(device) => device,
        Err(error) => {
            return Some(GpuPerformanceSnapshot {
                source: Some("nvml".to_owned()),
                last_error: Some(format!("nvml device query failed: {error}")),
                ..GpuPerformanceSnapshot::default()
            });
        }
    };

    let utilization_percent = match device.utilization_rates() {
        Ok(utilization) => Some((utilization.gpu as f32 * 10.0).round() / 10.0),
        Err(error) => {
            return Some(GpuPerformanceSnapshot {
                source: Some("nvml".to_owned()),
                last_error: Some(format!("nvml utilization query failed: {error}")),
                ..GpuPerformanceSnapshot::default()
            });
        }
    };
    let frequency_mhz = match device.clock_info(Clock::Graphics) {
        Ok(clock) => Some(clock),
        Err(error) => {
            return Some(GpuPerformanceSnapshot {
                utilization_percent,
                source: Some("nvml".to_owned()),
                last_error: Some(format!("nvml clock query failed: {error}")),
                ..GpuPerformanceSnapshot::default()
            });
        }
    };

    Some(GpuPerformanceSnapshot {
        frequency_mhz,
        utilization_percent,
        source: Some("nvml".to_owned()),
        last_error: None,
    })
}

fn first_gpu_card_path() -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    let mut cards: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("card") && !name.contains('-'))
        })
        .collect();
    cards.sort();
    cards.into_iter().next()
}

fn read_gpu_frequency_mhz(card_path: &Path) -> Result<(Option<u32>, String), String> {
    let candidates = [
        ("device/gt_cur_freq_mhz", 1_u32),
        ("device/pp_dpm_sclk", 1_u32),
        ("device/cur_freq", 1_000_000_u32),
        ("device/current_link_speed", 1_u32),
    ];
    for (relative_path, divisor) in candidates {
        let absolute = card_path.join(relative_path);
        let Ok(content) = fs::read_to_string(&absolute) else {
            continue;
        };
        if let Some(parsed) = parse_frequency_value(&content, divisor) {
            return Ok((Some(parsed), absolute.display().to_string()));
        }
    }
    Err("gpu frequency source unavailable".to_owned())
}

fn read_gpu_utilization_percent(card_path: &Path) -> Result<(Option<f32>, String), String> {
    let candidates = ["device/gpu_busy_percent", "device/mem_busy_percent"];
    for relative_path in candidates {
        let absolute = card_path.join(relative_path);
        let Ok(content) = fs::read_to_string(&absolute) else {
            continue;
        };
        if let Some(parsed) = parse_percent_value(&content) {
            return Ok((Some(parsed), absolute.display().to_string()));
        }
    }
    Err("gpu utilization source unavailable".to_owned())
}

fn parse_frequency_value(raw: &str, divisor: u32) -> Option<u32> {
    let line = raw
        .lines()
        .find(|entry| entry.contains('*'))
        .unwrap_or(raw.lines().next().unwrap_or(raw))
        .trim();
    let numeric = line
        .split(|char: char| !char.is_ascii_digit())
        .find(|part| !part.is_empty())?;
    let value = numeric.parse::<u64>().ok()?;
    if divisor == 0 {
        return None;
    }
    Some((value / u64::from(divisor)) as u32)
}

fn parse_percent_value(raw: &str) -> Option<f32> {
    let numeric = raw
        .trim()
        .split(|char: char| !char.is_ascii_digit() && char != '.')
        .find(|part| !part.is_empty())?;
    numeric
        .parse::<f32>()
        .ok()
        .map(|value| (value * 10.0).round() / 10.0)
}

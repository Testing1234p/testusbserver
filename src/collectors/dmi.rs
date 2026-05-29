use std::fs;
use crate::models::{SystemInfo, BiosInfo, CpuInfo, MemoryInfo, MemoryDimm};

fn read_sys_dmi(path: &str) -> String {
    let full = format!("/sys/class/dmi/id/{}", path);
    fs::read_to_string(&full).unwrap_or_default().trim().to_string()
}

pub fn collect_system_info() -> SystemInfo {
    SystemInfo {
        manufacturer: read_sys_dmi("sys_vendor"),
        product_name: read_sys_dmi("product_name"),
        version: read_sys_dmi("product_version"),
        serial_number: read_sys_dmi("product_serial"),
        sku: read_sys_dmi("product_sku"),
        asset_tag: read_sys_dmi("chassis_asset_tag"),
        uuid: read_sys_dmi("product_uuid"),
        family: read_sys_dmi("product_family"),
    }
}

pub fn collect_bios() -> BiosInfo {
    BiosInfo {
        vendor: read_sys_dmi("bios_vendor"),
        version: read_sys_dmi("bios_version"),
        date: read_sys_dmi("bios_date"),
        release: read_sys_dmi("bios_release"),
    }
}

pub fn collect_cpu() -> CpuInfo {
    let mut cpu = CpuInfo::default();
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        let mut flags_str = String::new();
        let mut seen_model = false;
        for line in content.lines() {
            if line.starts_with("vendor_id") {
                cpu.vendor_id = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
            if line.starts_with("model name") && !seen_model {
                cpu.model_name = line.split(':').nth(1).unwrap_or("").trim().to_string();
                seen_model = true;
            }
            if line.starts_with("flags") || line.starts_with("Features") {
                flags_str = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
        cpu.flags = flags_str.split_whitespace().map(|s| s.to_string()).collect();
    }

    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        cpu.cores = content.lines().filter(|l| l.starts_with("processor")).count();
        let siblings = content.lines()
            .find(|l| l.starts_with("siblings"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(cpu.cores);
        cpu.threads = siblings;
    }

    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        let unique_physical = content.lines()
            .filter(|l| l.starts_with("physical id"))
            .map(|l| l.split(':').nth(1).unwrap_or("0").trim().to_string())
            .collect::<std::collections::HashSet<_>>()
            .len();
        cpu.sockets = unique_physical.max(1);
    }

    cpu.architecture = std::env::consts::ARCH.to_string();
    cpu
}

pub fn collect_memory() -> MemoryInfo {
    let mut mem = MemoryInfo::default();
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(v) = line.split_whitespace().nth(1) {
                    mem.total_kb = v.parse().unwrap_or(0);
                }
            }
        }
    }
    mem.total_human = format_size_kb(mem.total_kb);

    // Tentativo DMI per DIMM
    if let Ok(entries) = fs::read_dir("/sys/devices/system/memory/") {
        let _ = entries;
    }

    // Fallback: dmidecode via comando se disponibile
    let output = std::process::Command::new("dmidecode")
        .args(["-t", "memory"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut current = MemoryDimm::default();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Memory Device") {
                if !current.locator.is_empty() {
                    mem.dimms.push(current);
                }
                current = MemoryDimm::default();
            } else if let Some(v) = trimmed.strip_prefix("Locator:") {
                current.locator = v.trim().to_string();
            } else if let Some(v) = trimmed.strip_prefix("Size:") {
                let s = v.trim();
                current.size_human = s.to_string();
                if s.contains("MB") {
                    current.size_kb = s.split_whitespace().next().unwrap_or("0").parse::<u64>().unwrap_or(0) * 1024;
                } else if s.contains("GB") {
                    current.size_kb = s.split_whitespace().next().unwrap_or("0").parse::<u64>().unwrap_or(0) * 1024 * 1024;
                }
            } else if let Some(v) = trimmed.strip_prefix("Speed:") {
                current.speed_mhz = v.trim().split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);
            } else if let Some(v) = trimmed.strip_prefix("Manufacturer:") {
                current.manufacturer = v.trim().to_string();
            } else if let Some(v) = trimmed.strip_prefix("Part Number:") {
                current.part_number = v.trim().to_string();
            } else if let Some(v) = trimmed.strip_prefix("Serial Number:") {
                current.serial_number = v.trim().to_string();
            } else if let Some(v) = trimmed.strip_prefix("Type:") {
                current.memory_type = v.trim().to_string();
            } else if let Some(v) = trimmed.strip_prefix("Rank:") {
                current.rank = v.trim().to_string();
            }
        }
        if !current.locator.is_empty() {
            mem.dimms.push(current);
        }
    }

    mem
}

fn format_size_kb(kb: u64) -> String {
    if kb >= 1024 * 1024 {
        format!("{:.1} GB", kb as f64 / (1024.0 * 1024.0))
    } else if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{} KB", kb)
    }
}

use crate::models::PciDevice;
use std::fs;

fn run_lspci() -> Option<String> {
    let out = std::process::Command::new("lspci")
        .args(["-nnk"])
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => None,
    }
}

pub fn collect_pci() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    if let Some(text) = run_lspci() {
        let mut current = PciDevice::default();
        for line in text.lines() {
            let trimmed = line.trim();
            // Riga che inizia con slot tipo "00:00.0 ..."
            if !line.starts_with('\t') && line.contains(':') {
                if !current.slot.is_empty() {
                    devices.push(current);
                }
                current = PciDevice::default();
                if let Some(pos) = trimmed.find(' ') {
                    current.slot = trimmed[..pos].to_string();
                    let rest = &trimmed[pos..];
                    // Estrai class e name [xxxx:xxxx]
                    if let Some(open) = rest.rfind('[') {
                        if let Some(close) = rest[open..].find(']') {
                            let ids = &rest[open+1..open+close];
                            if let Some(colon) = ids.find(':') {
                                current.vendor = ids[..colon].to_string();
                                current.device = ids[colon+1..].to_string();
                            }
                        }
                    }
                }
            } else if trimmed.starts_with("Subsystem:") {
                if let Some(open) = trimmed.rfind('[') {
                    if let Some(close) = trimmed[open..].find(']') {
                        current.subsystem = trimmed[open+1..open+close].to_string();
                    }
                }
            } else if trimmed.starts_with("Kernel driver in use:") {
                current.kernel_driver = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if trimmed.starts_with("Kernel modules:") {
                current.kernel_modules = trimmed.split(':').nth(1).unwrap_or("").split(',').map(|s| s.trim().to_string()).collect();
            } else if trimmed.starts_with("Class ") {
                current.class = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
        if !current.slot.is_empty() {
            devices.push(current);
        }
    }

    // Fallback parsing /sys se lspci non c'è
    if devices.is_empty() {
        if let Ok(dir) = fs::read_dir("/sys/bus/pci/devices/") {
            for entry in dir.flatten() {
                let path = entry.path();
                let slot = entry.file_name().to_string_lossy().to_string();
                let vendor = fs::read_to_string(path.join("vendor")).unwrap_or_default().trim().to_string();
                let device = fs::read_to_string(path.join("device")).unwrap_or_default().trim().to_string();
                let class = fs::read_to_string(path.join("class")).unwrap_or_default().trim().to_string();
                let driver = fs::read_to_string(path.join("driver")).ok().map(|_| {
                    fs::read_link(path.join("driver")).ok().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).unwrap_or_default()
                }).unwrap_or_default();

                devices.push(PciDevice {
                    slot,
                    class,
                    vendor,
                    device,
                    subsystem: String::new(),
                    kernel_driver: driver,
                    kernel_modules: Vec::new(),
                });
            }
        }
    }

    devices
}

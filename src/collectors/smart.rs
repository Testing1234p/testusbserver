use crate::models::{StorageDevice, SmartAttribute};

fn run_smartctl(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("smartctl")
        .args(args)
        .output();
    match out {
        Ok(o) => Some(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => None,
    }
}

pub fn collect_storage() -> Vec<StorageDevice> {
    let mut devices = Vec::new();

    // Scansiona dischi
    let scan = run_smartctl(&["--scan"]);
    if scan.is_none() {
        return devices;
    }

    for line in scan.unwrap().lines() {
        let dev = line.split_whitespace().next().unwrap_or("");
        if dev.is_empty() || !dev.starts_with("/dev/") {
            continue;
        }

        if let Some(text) = run_smartctl(&["--all", dev]) {
            let mut disk = StorageDevice {
                device: dev.to_string(),
                ..Default::default()
            };

            let mut in_attributes = false;
            let mut attrs: Vec<SmartAttribute> = Vec::new();

            for smart_line in text.lines() {
                let trimmed = smart_line.trim();

                if trimmed.starts_with("Device Model:") || trimmed.starts_with("Model Family:") {
                    disk.model = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("Serial Number:") {
                    disk.serial = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("Firmware Version:") {
                    disk.firmware = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("User Capacity:") {
                    disk.capacity_human = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("SMART overall-health") {
                    disk.smart_health = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("Power_On_Hours") {
                    if let Some(raw) = trimmed.split_whitespace().last() {
                        disk.power_on_hours = raw.parse().unwrap_or(0);
                    }
                } else if trimmed.starts_with("Power_Cycle_Count") {
                    if let Some(raw) = trimmed.split_whitespace().last() {
                        disk.power_cycle_count = raw.parse().unwrap_or(0);
                    }
                } else if trimmed.starts_with("Temperature") {
                    // Temperatura SMART
                    for word in trimmed.split_whitespace() {
                        if let Ok(t) = word.trim_end_matches(|c: char| !c.is_ascii_digit()).parse::<i32>() {
                            if t > 0 && t < 120 {
                                disk.temperature = t;
                                break;
                            }
                        }
                    }
                }

                // Sezione attributi
                if trimmed.starts_with("ID#") {
                    in_attributes = true;
                    continue;
                }
                if in_attributes && trimmed.starts_with("0x") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 10 {
                        let id = u8::from_str_radix(parts[0].trim_start_matches("0x"), 16).unwrap_or(0);
                        let attr = SmartAttribute {
                            id,
                            name: parts[1].to_string(),
                            flag: parts[2].to_string(),
                            value: parts[3].parse().unwrap_or(0),
                            worst: parts[4].parse().unwrap_or(0),
                            threshold: parts[5].parse().unwrap_or(0),
                            raw: parts[9..].join(" "),
                            status: "ok".to_string(),
                        };
                        attrs.push(attr);
                    }
                }
            }

            disk.smart_attributes = attrs;
            if disk.smart_health.is_empty() {
                disk.smart_health = "unknown".to_string();
            }
            devices.push(disk);
        }
    }

    devices
}

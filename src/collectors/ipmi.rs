use crate::models::{IpmiInfo, IpmiSensor, SelEntry};

fn run_ipmi(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("ipmitool")
        .args(args)
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => None,
    }
}

pub fn collect_ipmi() -> IpmiInfo {
    let mut info = IpmiInfo {
        available: false,
        bmc_firmware: String::new(),
        sensors: Vec::new(),
        sel_entries: Vec::new(),
    };

    // Verifica disponibilità
    if run_ipmi(&["mc", "info"]).is_none() {
        return info;
    }
    info.available = true;

    // BMC firmware version
    if let Some(text) = run_ipmi(&["mc", "info"]) {
        for line in text.lines() {
            if line.contains("Firmware Revision") {
                info.bmc_firmware = line.split(':').nth(1).unwrap_or("").trim().to_string();
                break;
            }
        }
    }

    // Sensori
    if let Some(text) = run_ipmi(&["sensor", "list"]) {
        for line in text.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                info.sensors.push(IpmiSensor {
                    name: parts[0].trim().to_string(),
                    value: parts[1].trim().to_string(),
                    unit: String::new(),
                    status: parts[3].trim().to_string(),
                });
            }
        }
    }

    // SEL log
    if let Some(text) = run_ipmi(&["sel", "list"]) {
        for line in text.lines() {
            if line.trim().is_empty() || line.starts_with("SEL") {
                continue;
            }
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if parts.len() >= 5 {
                let severity = if line.to_lowercase().contains("critical") {
                    "critical"
                } else if line.to_lowercase().contains("warning") {
                    "warning"
                } else {
                    "info"
                };
                info.sel_entries.push(SelEntry {
                    id: parts[0].trim().to_string(),
                    date: parts[1].trim().to_string(),
                    time: parts[2].trim().to_string(),
                    sensor: parts[4].trim().to_string(),
                    event: parts.get(5).unwrap_or(&"").trim().to_string(),
                    severity: severity.to_string(),
                });
            }
        }
    }

    info
}

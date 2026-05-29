use crate::models::{SensorReading, SensorLimits};
use std::fs;

fn run_sensors() -> Option<String> {
    let out = std::process::Command::new("sensors")
        .args(["-u"])
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => None,
    }
}

pub fn collect_sensors() -> Vec<SensorReading> {
    let mut readings = Vec::new();

    // Prova lm-sensors
    if let Some(text) = run_sensors() {
        let mut chip_name = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if !line.starts_with(' ') && !trimmed.is_empty() {
                chip_name = trimmed.trim_end_matches(':').to_string();
                continue;
            }
            if trimmed.ends_with(':') {
                continue;
            }
            if let Some(pos) = trimmed.find("_input:") {
                let sensor_name = format!("{}_{}", chip_name, &trimmed[..pos].trim());
                let val_str = trimmed[pos + 7..].trim();
                if let Ok(val) = val_str.parse::<f64>() {
                    let unit = if sensor_name.contains("temp") { "C" } else if sensor_name.contains("fan") { "RPM" } else if sensor_name.contains("in") { "V" } else { "" };
                    readings.push(SensorReading {
                        name: sensor_name,
                        value: val,
                        unit: unit.to_string(),
                        status: "ok".to_string(),
                        limits: SensorLimits::default(),
                    });
                }
            }
        }
    }

    // Fallback hwmon sysfs
    if readings.is_empty() {
        if let Ok(hwmon_dir) = fs::read_dir("/sys/class/hwmon/") {
            for entry in hwmon_dir.flatten() {
                let path = entry.path();
                let chip = fs::read_to_string(path.join("name")).unwrap_or_default().trim().to_string();
                if let Ok(files) = fs::read_dir(&path) {
                    for f in files.flatten() {
                        let fname = f.file_name().to_string_lossy().to_string();
                        if fname.ends_with("_input") {
                            let sensor = fname.trim_end_matches("_input");
                            let val = fs::read_to_string(f.path()).unwrap_or_default().trim().parse::<f64>().unwrap_or(0.0);
                            let value = if sensor.starts_with("temp") { val / 1000.0 } else { val };
                            let unit = if sensor.starts_with("temp") { "C" } else if sensor.starts_with("fan") { "RPM" } else if sensor.starts_with("in") { "V" } else { "" };
                            readings.push(SensorReading {
                                name: format!("{}_{}", chip, sensor),
                                value,
                                unit: unit.to_string(),
                                status: "ok".to_string(),
                                limits: SensorLimits::default(),
                            });
                        }
                    }
                }
            }
        }
    }

    readings
}

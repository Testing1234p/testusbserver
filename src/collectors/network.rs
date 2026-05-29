use crate::models::NetworkInterface;
use std::fs;

pub fn collect_network() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    if let Ok(dir) = fs::read_dir("/sys/class/net/") {
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }
            let path = entry.path();

            let mac = fs::read_to_string(path.join("address")).unwrap_or_default().trim().to_string();
            let is_virtual = path.join("device").exists() == false;

            let mut driver = String::new();
            if let Ok(link) = fs::read_link(path.join("device").join("driver")) {
                driver = link.file_name().unwrap_or_default().to_string_lossy().to_string();
            }

            let mut speed = 0u64;
            if let Ok(s) = fs::read_to_string(path.join("speed")) {
                speed = s.trim().parse().unwrap_or(0);
            }

            let mut duplex = "unknown".to_string();
            if let Ok(d) = fs::read_to_string(path.join("duplex")) {
                duplex = d.trim().to_string();
            }

            interfaces.push(NetworkInterface {
                name,
                mac_address: mac,
                driver,
                speed_mbps: speed,
                duplex,
                is_virtual,
            });
        }
    }

    interfaces
}

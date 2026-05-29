mod collectors;
mod models;
mod report;

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

fn main() {
    println!("========================================");
    println!("  HARDWARE AGENT USB v0.1.0");
    println!("  Lettura sicura, solo informativa");
    println!("========================================");

    let mut notes: Vec<String> = Vec::new();

    // Rileva percorso chiavetta USB (montaggio automatico)
    let usb_mount = find_usb_mount().unwrap_or_else(|| {
        println!("[WARN] Chiavetta USB non trovata. Uso /tmp per debug.");
        notes.push("USB stick not auto-detected; fallback to /tmp".to_string());
        PathBuf::from("/tmp")
    });

    println!("[INFO] Output directory: {}", usb_mount.display());

    // Raccolta dati (tutto read-only)
    println!("[INFO] Raccolta System Info...");
    let system_info = collectors::dmi::collect_system_info();

    println!("[INFO] Raccolta BIOS...");
    let bios = collectors::dmi::collect_bios();

    println!("[INFO] Raccolta CPU...");
    let cpu = collectors::dmi::collect_cpu();

    println!("[INFO] Raccolta Memoria...");
    let memory = collectors::dmi::collect_memory();

    println!("[INFO] Raccolta Storage SMART...");
    let storage = collectors::smart::collect_storage();

    println!("[INFO] Raccolta Rete...");
    let network = collectors::network::collect_network();

    println!("[INFO] Raccolta PCI...");
    let pci_devices = collectors::pci::collect_pci();

    println!("[INFO] Raccolta Sensori...");
    let sensors = collectors::sensors::collect_sensors();

    println!("[INFO] Raccolta IPMI...");
    let ipmi = collectors::ipmi::collect_ipmi();

    // Determina nome file base
    let serial = if system_info.serial_number.is_empty() {
        "unknown".to_string()
    } else {
        system_info.serial_number.clone()
    };
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let base_name = format!("Report_Server_{}_{}", serial, timestamp);

    // Costruisci report
    let mut report = models::SystemReport {
        scan_timestamp: Utc::now(),
        system_info,
        bios,
        cpu,
        memory,
        storage,
        network,
        pci_devices,
        sensors,
        ipmi,
        notes,
    };

    // Genera TXT
    let txt_path = usb_mount.join(format!("{}.txt", base_name));
    println!("[INFO] Scrittura TXT: {}", txt_path.display());
    report::generate_txt(&report, &txt_path);

    // Genera HTML per PDF
    let html_path = usb_mount.join(format!("{}.html", base_name));
    let html_content = report::generate_html(&report);
    let _ = fs::write(&html_path, html_content);
    println!("[INFO] Scrittura HTML: {}", html_path.display());

    // Genera PDF con wkhtmltopdf se disponibile
    let pdf_path = usb_mount.join(format!("{}.pdf", base_name));
    let pdf_result = std::process::Command::new("wkhtmltopdf")
        .args([
            "--page-size", "A4",
            "--margin-top", "10mm",
            "--margin-bottom", "10mm",
            "--margin-left", "10mm",
            "--margin-right", "10mm",
            "--enable-local-file-access",
            html_path.to_str().unwrap_or(""),
            pdf_path.to_str().unwrap_or(""),
        ])
        .output();

    match pdf_result {
        Ok(out) if out.status.success() => {
            println!("[OK] PDF generato: {}", pdf_path.display());
        }
        _ => {
            // Fallback: se wkhtmltopdf non c'è, copia solo HTML e TXT
            println!("[WARN] wkhtmltopdf non disponibile. Puoi aprire l'HTML nel browser per stampare PDF.");
            report.notes.push("PDF generation skipped: wkhtmltopdf not found".to_string());
        }
    }

    // Scrivi COMPLETATO.txt
    let done_path = usb_mount.join("COMPLETATO.txt");
    let done_content = format!(
        "SCAN COMPLETATO\nTimestamp: {}\nSerial: {}\nDischi rilevati: {}\nSensori rilevati: {}\nFile generati:\n  - {}\n  - {}\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        serial,
        report.storage.len(),
        report.sensors.len(),
        txt_path.file_name().unwrap_or_default().to_string_lossy(),
        pdf_path.file_name().unwrap_or_default().to_string_lossy(),
    );
    let _ = fs::write(&done_path, done_content);
    println!("[OK] File completamento: {}", done_path.display());

    // Notifica sonora: 3 beep
    println!("\x07\x07\x07");
    let _ = std::process::Command::new("sh")
        .args(["-c", "echo -e '\\a\\a\\a' > /dev/console 2>/dev/null || true"])
        .status();
    let _ = std::process::Command::new("beep").status();

    println!("========================================");
    println!("  SCAN COMPLETATO - RIMUOVI CHIAVETTA");
    println!("========================================");
}

fn find_usb_mount() -> Option<PathBuf> {
    // Cerca tra i mountpoint comuni per chiavette USB
    let candidates = [
        "/mnt/usb",
        "/mnt",
        "/media",
        "/run/media/root",
        "/run/media",
    ];

    for base in &candidates {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Verifica se writable e se contiene qualche marker (assenza di marker esistenti)
                if path.is_dir() {
                    // Prova a scrivere un test file
                    let test_file = path.join(".hwagent_test");
                    if fs::write(&test_file, "test").is_ok() {
                        let _ = fs::remove_file(&test_file);
                        return Some(path);
                    }
                }
            }
        }
    }

    // Fallback: cerca in /proc/mounts per vfat/exfat su /dev/sd*
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let dev = parts[0];
                let mnt = parts[1];
                if (dev.starts_with("/dev/sd") || dev.starts_with("/dev/sr")) && !mnt.starts_with("/boot") {
                    let mnt_path = Path::new(mnt);
                    let test_file = mnt_path.join(".hwagent_test");
                    if fs::write(&test_file, "test").is_ok() {
                        let _ = fs::remove_file(&test_file);
                        return Some(mnt_path.to_path_buf());
                    }
                }
            }
        }
    }

    None
}

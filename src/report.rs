use crate::models::SystemReport;
use std::fs;
use std::path::Path;

pub fn generate_txt(report: &SystemReport, output_path: &Path) {
    let mut text = String::new();
    text.push_str("========================================\n");
    text.push_str("   HARDWARE AGENT - RAPPORTO SERVER\n");
    text.push_str("========================================\n\n");
    text.push_str(&format!("Data scansione: {}\n", report.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    text.push_str(&format!("Serial Number:   {}\n", report.system_info.serial_number));
    text.push_str(&format!("Prodotto:        {} {}\n", report.system_info.manufacturer, report.system_info.product_name));
    text.push_str(&format!("Versione:        {}\n", report.system_info.version));
    text.push_str(&format!("Asset Tag:       {}\n", report.system_info.asset_tag));
    text.push_str(&format!("UUID:            {}\n\n", report.system_info.uuid));

    text.push_str("--- BIOS ---\n");
    text.push_str(&format!("Vendor:  {}\n", report.bios.vendor));
    text.push_str(&format!("Version: {}\n", report.bios.version));
    text.push_str(&format!("Date:    {}\n", report.bios.date));
    text.push_str(&format!("Release: {}\n\n", report.bios.release));

    text.push_str("--- CPU ---\n");
    text.push_str(&format!("{}\n", report.cpu.model_name));
    text.push_str(&format!("Socket: {} | Core: {} | Thread: {}\n", report.cpu.sockets, report.cpu.cores, report.cpu.threads));
    text.push_str(&format!("Arch:   {} | Vendor: {}\n\n", report.cpu.architecture, report.cpu.vendor_id));

    text.push_str("--- MEMORIA ---\n");
    text.push_str(&format!("Totale: {} ({} KB)\n", report.memory.total_human, report.memory.total_kb));
    for dimm in &report.memory.dimms {
        text.push_str(&format!("  {} | {} | {} MHz | {} | {} | SN:{}\n",
            dimm.locator,
            dimm.size_human,
            dimm.speed_mhz,
            dimm.memory_type,
            dimm.manufacturer,
            dimm.serial_number
        ));
    }
    text.push('\n');

    text.push_str("--- STORAGE ---\n");
    for disk in &report.storage {
        text.push_str(&format!("  {} | {} | {} | Health: {}\n",
            disk.device,
            disk.model,
            disk.capacity_human,
            disk.smart_health
        ));
        text.push_str(&format!("    Serial: {} | Firmware: {} | Temp: {}C\n",
            disk.serial,
            disk.firmware,
            disk.temperature
        ));
        text.push_str(&format!("    PowerOnHours: {} | PowerCycles: {}\n",
            disk.power_on_hours,
            disk.power_cycle_count
        ));
    }
    text.push('\n');

    text.push_str("--- RETE ---\n");
    for nic in &report.network {
        text.push_str(&format!("  {} | MAC: {} | Driver: {} | Speed: {} Mbps | {}\n",
            nic.name,
            nic.mac_address,
            nic.driver,
            nic.speed_mbps,
            nic.duplex
        ));
    }
    text.push('\n');

    text.push_str("--- PCI DEVICES ---\n");
    for dev in &report.pci_devices {
        text.push_str(&format!("  {} | {} | {} | Driver: {}\n",
            dev.slot,
            dev.vendor,
            dev.device,
            dev.kernel_driver
        ));
    }
    text.push('\n');

    text.push_str("--- SENSORI ---\n");
    for s in &report.sensors {
        text.push_str(&format!("  {}: {:.2} {} | {}\n", s.name, s.value, s.unit, s.status));
    }
    text.push('\n');

    text.push_str("--- IPMI ---\n");
    text.push_str(&format!("Disponibile: {}\n", report.ipmi.available));
    if report.ipmi.available {
        text.push_str(&format!("BMC Firmware: {}\n", report.ipmi.bmc_firmware));
        for sensor in &report.ipmi.sensors {
            text.push_str(&format!("  {}: {} | {}\n", sensor.name, sensor.value, sensor.status));
        }
        for sel in &report.ipmi.sel_entries {
            text.push_str(&format!("  SEL [{}] {} {} | {} | {} | {}\n",
                sel.id, sel.date, sel.time, sel.sensor, sel.event, sel.severity
            ));
        }
    }
    text.push('\n');

    if !report.notes.is_empty() {
        text.push_str("--- NOTE ---\n");
        for note in &report.notes {
            text.push_str(&format!("  - {}\n", note));
        }
    }

    let _ = fs::write(output_path, text);
}

pub fn generate_html(report: &SystemReport) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'/><title>Hardware Report</title>");
    html.push_str("<style>");
    html.push_str("body{font-family:Arial,Helvetica,sans-serif;margin:40px;color:#222;}");
    html.push_str("h1{border-bottom:2px solid #333;padding-bottom:8px;}");
    html.push_str("h2{color:#444;margin-top:30px;border-bottom:1px solid #ccc;padding-bottom:4px;}");
    html.push_str("table{width:100%;border-collapse:collapse;margin-top:8px;}");
    html.push_str("th,td{padding:8px 12px;text-align:left;border-bottom:1px solid #ddd;}");
    html.push_str("th{background:#f5f5f5;font-weight:600;}");
    html.push_str(".ok{color:green;}.warn{color:orange;}.critical{color:red;}");
    html.push_str(".header-box{background:#2c3e50;color:#fff;padding:20px;border-radius:6px;}");
    html.push_str(".header-box h1{margin:0;color:#fff;border:none;}");
    html.push_str(".meta{color:#bdc3c7;margin-top:6px;}");
    html.push_str("</style></head><body>");

    html.push_str("<div class='header-box'>");
    html.push_str(&format!("<h1>{} {}</h1>", html_escape(&report.system_info.manufacturer), html_escape(&report.system_info.product_name)));
    html.push_str(&format!("<div class='meta'>Serial: {} | Scan: {}</div>",
        html_escape(&report.system_info.serial_number),
        report.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    html.push_str("</div>");

    // BIOS
    html.push_str("<h2>BIOS</h2><table>");
    html.push_str(&table_row("Vendor", &report.bios.vendor));
    html.push_str(&table_row("Version", &report.bios.version));
    html.push_str(&table_row("Date", &report.bios.date));
    html.push_str("</table>");

    // CPU
    html.push_str("<h2>CPU</h2><table>");
    html.push_str(&table_row("Model", &report.cpu.model_name));
    html.push_str(&table_row("Sockets", &report.cpu.sockets.to_string()));
    html.push_str(&table_row("Cores", &report.cpu.cores.to_string()));
    html.push_str(&table_row("Threads", &report.cpu.threads.to_string()));
    html.push_str(&table_row("Architecture", &report.cpu.architecture));
    html.push_str("</table>");

    // Memory
    html.push_str("<h2>Memory</h2>");
    html.push_str(&format!("<p><strong>Total:</strong> {}</p>", report.memory.total_human));
    if !report.memory.dimms.is_empty() {
        html.push_str("<table><tr><th>Locator</th><th>Size</th><th>Speed</th><th>Type</th><th>Part Number</th><th>Serial</th></tr>");
        for d in &report.memory.dimms {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{} MHz</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&d.locator),
                html_escape(&d.size_human),
                d.speed_mhz,
                html_escape(&d.memory_type),
                html_escape(&d.part_number),
                html_escape(&d.serial_number)
            ));
        }
        html.push_str("</table>");
    }

    // Storage
    html.push_str("<h2>Storage</h2>");
    for disk in &report.storage {
        let health_class = if disk.smart_health.to_lowercase().contains("passed") { "ok" } else { "warn" };
        html.push_str(&format!("<h3>{} <span class='{}'>[{}]</span></h3>", html_escape(&disk.device), health_class, html_escape(&disk.smart_health)));
        html.push_str("<table>");
        html.push_str(&table_row("Model", &disk.model));
        html.push_str(&table_row("Serial", &disk.serial));
        html.push_str(&table_row("Firmware", &disk.firmware));
        html.push_str(&table_row("Capacity", &disk.capacity_human));
        html.push_str(&table_row("Temperature", &format!("{} C", disk.temperature)));
        html.push_str(&table_row("Power On Hours", &disk.power_on_hours.to_string()));
        html.push_str(&table_row("Power Cycles", &disk.power_cycle_count.to_string()));
        html.push_str("</table>");
    }

    // Network
    html.push_str("<h2>Network</h2><table><tr><th>Interface</th><th>MAC</th><th>Driver</th><th>Speed</th><th>Duplex</th></tr>");
    for nic in &report.network {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{} Mbps</td><td>{}</td></tr>",
            html_escape(&nic.name),
            html_escape(&nic.mac_address),
            html_escape(&nic.driver),
            nic.speed_mbps,
            html_escape(&nic.duplex)
        ));
    }
    html.push_str("</table>");

    // PCI
    html.push_str("<h2>PCI Devices</h2><table><tr><th>Slot</th><th>Class</th><th>Vendor</th><th>Device</th><th>Driver</th></tr>");
    for dev in &report.pci_devices {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&dev.slot),
            html_escape(&dev.class),
            html_escape(&dev.vendor),
            html_escape(&dev.device),
            html_escape(&dev.kernel_driver)
        ));
    }
    html.push_str("</table>");

    // Sensors
    html.push_str("<h2>Sensors</h2><table><tr><th>Name</th><th>Value</th><th>Unit</th><th>Status</th></tr>");
    for s in &report.sensors {
        html.push_str(&format!("<tr><td>{}</td><td>{:.2}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&s.name), s.value, html_escape(&s.unit), html_escape(&s.status)
        ));
    }
    html.push_str("</table>");

    // IPMI
    html.push_str("<h2>IPMI</h2>");
    if report.ipmi.available {
        html.push_str(&format!("<p><strong>BMC Firmware:</strong> {}</p>", html_escape(&report.ipmi.bmc_firmware)));
        if !report.ipmi.sensors.is_empty() {
            html.push_str("<table><tr><th>Sensor</th><th>Value</th><th>Status</th></tr>");
            for s in &report.ipmi.sensors {
                html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    html_escape(&s.name), html_escape(&s.value), html_escape(&s.status)
                ));
            }
            html.push_str("</table>");
        }
        if !report.ipmi.sel_entries.is_empty() {
            html.push_str("<h3>SEL Log</h3><table><tr><th>ID</th><th>Date</th><th>Sensor</th><th>Event</th><th>Severity</th></tr>");
            for e in &report.ipmi.sel_entries {
                let sev_class = if e.severity == "critical" { "critical" } else if e.severity == "warning" { "warn" } else { "ok" };
                html.push_str(&format!("<tr><td>{}</td><td>{} {}</td><td>{}</td><td>{}</td><td class='{}'>{}</td></tr>",
                    html_escape(&e.id), html_escape(&e.date), html_escape(&e.time),
                    html_escape(&e.sensor), html_escape(&e.event), sev_class, html_escape(&e.severity)
                ));
            }
            html.push_str("</table>");
        }
    } else {
        html.push_str("<p>IPMI not available on this system.</p>");
    }

    html.push_str("<hr/><p style='color:#888;font-size:12px;'>Generated by Hardware Agent USB</p>");
    html.push_str("</body></html>");
    html
}

fn table_row(label: &str, value: &str) -> String {
    format!("<tr><th>{}</th><td>{}</td></tr>", html_escape(label), html_escape(value))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

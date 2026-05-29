use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemReport {
    pub scan_timestamp: DateTime<Utc>,
    pub system_info: SystemInfo,
    pub bios: BiosInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub storage: Vec<StorageDevice>,
    pub network: Vec<NetworkInterface>,
    pub pci_devices: Vec<PciDevice>,
    pub sensors: Vec<SensorReading>,
    pub ipmi: IpmiInfo,
    pub notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
    pub serial_number: String,
    pub sku: String,
    pub asset_tag: String,
    pub uuid: String,
    pub family: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub date: String,
    pub release: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CpuInfo {
    pub vendor_id: String,
    pub model_name: String,
    pub cores: usize,
    pub threads: usize,
    pub sockets: usize,
    pub architecture: String,
    pub flags: Vec<String>,
    pub vulnerabilities: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub total_human: String,
    pub dimms: Vec<MemoryDimm>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MemoryDimm {
    pub locator: String,
    pub size_kb: u64,
    pub size_human: String,
    pub speed_mhz: u32,
    pub manufacturer: String,
    pub part_number: String,
    pub serial_number: String,
    pub memory_type: String,
    pub rank: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StorageDevice {
    pub device: String,
    pub model: String,
    pub serial: String,
    pub firmware: String,
    pub capacity_human: String,
    pub smart_health: String,
    pub smart_attributes: Vec<SmartAttribute>,
    pub power_on_hours: u64,
    pub power_cycle_count: u64,
    pub temperature: i32,
    pub notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub value: u8,
    pub worst: u8,
    pub threshold: u8,
    pub raw: String,
    pub flag: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub driver: String,
    pub speed_mbps: u64,
    pub duplex: String,
    pub is_virtual: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PciDevice {
    pub slot: String,
    pub class: String,
    pub vendor: String,
    pub device: String,
    pub subsystem: String,
    pub kernel_driver: String,
    pub kernel_modules: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SensorReading {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub status: String,
    pub limits: SensorLimits,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SensorLimits {
    pub low_critical: Option<f64>,
    pub low_warning: Option<f64>,
    pub high_warning: Option<f64>,
    pub high_critical: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IpmiInfo {
    pub available: bool,
    pub bmc_firmware: String,
    pub sensors: Vec<IpmiSensor>,
    pub sel_entries: Vec<SelEntry>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IpmiSensor {
    pub name: String,
    pub value: String,
    pub unit: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SelEntry {
    pub id: String,
    pub date: String,
    pub time: String,
    pub sensor: String,
    pub event: String,
    pub severity: String,
}

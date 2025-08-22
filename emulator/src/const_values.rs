use anyhow::{self, Context};
use serde::Deserialize;
use std::path::Path;

/// 主配置中保留的内存项（仅含 boot_pc）
#[derive(Deserialize, Debug)]
pub struct MemoryConfig {
    pub boot_pc: u64,
}

#[derive(Deserialize, Debug)]
pub struct InstSetConfig {
    #[serde(default)]
    pub m_ext: bool,
    #[serde(default)]
    pub a_ext: bool,
    #[serde(default)]
    pub c_ext: bool,
}

#[derive(Deserialize, Debug)]
pub struct DebugConfig {
    pub event_list_size: usize,
    #[cfg(feature = "tracer")]
    pub instruction_tracer_list_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct OthersConfig {
    pub decoder_cache_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeviceConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub base: u64,
    pub size: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// 主模拟器配置（来自 emulator/profile/config.toml）——仅保留 boot_pc、ISA 与调试等
#[derive(Deserialize, Debug)]
pub struct EmuConfig {
    pub memory: MemoryConfig,
    pub inst_set: InstSetConfig,
    pub debug: DebugConfig,
    pub others: OthersConfig,
    // 不再在主配置中包含 devices
}

impl EmuConfig {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<EmuConfig> {
        let toml_str = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取主配置文件: {:?}", &path.as_ref().as_os_str()))?;
        let config: EmuConfig = toml::from_str(&toml_str)
            .with_context(|| format!("无法解析主配置文件: {:?}", &path.as_ref().as_os_str()))?;
        anyhow::Ok(config)
    }
}

/// 从设备配置文件中读取的结构（devices/profile/device.toml）
#[derive(Deserialize, Debug)]
pub struct DeviceFileMemory {
    pub memory_base: u64,
    pub memory_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct DeviceFile {
    pub memory: DeviceFileMemory,
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
}

impl DeviceFile {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<DeviceFile> {
        let toml_str = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取设备配置文件: {:?}", &path.as_ref().as_os_str()))?;
        let profile: DeviceFile = toml::from_str(&toml_str)
            .with_context(|| format!("无法解析设备配置文件: {:?}", &path.as_ref().as_os_str()))?;
        anyhow::Ok(profile)
    }
}

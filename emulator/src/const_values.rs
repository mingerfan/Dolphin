use anyhow::{self, Context};
use serde::Deserialize;
use std::path::Path;

// /// 内存基地址
// pub const MEMORY_BASE: u64 = 0x8000_0000;

// /// 事件列表大小
// pub const EVENT_LIST_SIZE: usize = 64;

// /// Decoer LRU缓存大小
// pub const DECODER_LRU_CACHE_SIZE: usize = 1024;

// #[cfg(feature = "tracer")]
// pub const INSTRUCTION_TRACER_LIST_SIZE: usize = 64;

#[derive(Deserialize, Debug)]
pub struct MemoryConfig {
    pub memory_base: u64,
    pub memory_size: usize,
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
    pub decoder_lru_cache_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct EmuConfig {
    pub memory: MemoryConfig,
    pub inst_set: InstSetConfig,
    pub debug: DebugConfig,
    pub others: OthersConfig,
}

impl EmuConfig {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<EmuConfig> {
        let toml_str = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {:?}", &path.as_ref().as_os_str()))?;
        let config: EmuConfig = toml::from_str(&toml_str)
            .with_context(|| format!("无法解析配置文件: {:?}", &path.as_ref().as_os_str()))?;
        anyhow::Ok(config)
    }
}

//! 设备管理模块
//! 负责根据配置文件创建和管理 MMIO 设备

use std::sync::{Arc, Mutex};
use mmio_trait::MmioDevice;
use crate::const_values::DeviceConfig;
use crate::emulator::memory::Memory;

/// 设备工厂错误
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("未知设备类型: {0}")]
    UnknownDeviceType(String),
    #[error("设备创建失败: {0}")]
    CreationFailed(String),
}

/// 设备工厂
pub struct DeviceFactory;

impl DeviceFactory {
    /// 根据配置创建设备
    pub fn create_device(config: &DeviceConfig) -> Result<Arc<Mutex<dyn MmioDevice>>, DeviceError> {
        match config.device_type.as_str() {
            "uart" => {
                let uart = uart::Uart::new(config.name.clone());
                Ok(Arc::new(Mutex::new(uart)))
            }
            _ => Err(DeviceError::UnknownDeviceType(config.device_type.clone())),
        }
    }
}

/// 设备管理器
pub struct DeviceManager;

impl DeviceManager {
    /// 根据配置列表初始化所有设备
    pub fn initialize_devices(
        memory: &mut Memory,
        device_configs: &[DeviceConfig],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for config in device_configs {
            if !config.enabled {
                tracing::info!("跳过禁用的设备: {}", config.name);
                continue;
            }

            tracing::info!("初始化设备: {} (类型: {}, 地址: {:#x}, 大小: {:#x})",
                     config.name, config.device_type, config.base, config.size);

            let device = DeviceFactory::create_device(config)
                .map_err(|e| format!("创建设备 {} 失败: {}", config.name, e))?;

            memory.map_mmio(
                config.base,
                config.size,
                device,
                config.name.clone(),
            ).map_err(|e| format!("映射设备 {} 失败: {}", config.name, e))?;
        }

        Ok(())
    }
}
